use std::{
    collections::HashMap,
    process::{Command, Output},
    sync::Mutex,
    time::Duration,
};

use parallel_update_config::{config::UpdateConfig, primatives::UpdateKind, types::Program};

use crate::error::{ErrorKind, Result};
use crate::types::*;

pub mod default;
pub mod paru;

pub type UpdateRunner = &'static (dyn Fn(&Update, &GlobalState) + Sync);

#[derive(Debug, Clone)]
pub struct UpdateOutput {
    pub output: Output,
    pub duration: Duration,
}

/// An Update that can be run.
pub struct Update {
    pub id: UpdateId,
    pub info: Info,
    pub state: SyncState,
    pub program: Program,
    pub output: Mutex<Option<UpdateOutput>>,
    pub(crate) run: UpdateRunner,
}

impl std::fmt::Debug for Update {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Update")
            .field("id", &self.id)
            .field("info", &self.info)
            .field("state", &self.state)
            .field("program", &self.program)
            .field("output", &self.output)
            .finish()
    }
}

impl Update {
    pub fn new(id: UpdateId, program: Program, info: Info) -> Self {
        Update::new_with_runnner(id, program, info, &default::run)
    }

    pub fn new_with_runnner(
        id: UpdateId,
        program: Program,
        info: Info,
        runner: UpdateRunner,
    ) -> Self {
        Update {
            id,
            program,
            info,
            state: SyncState::new(State::Pending),
            output: Mutex::new(None),
            run: runner,
        }
    }

    fn create_command(&self) -> Command {
        let mut command = Command::new(&self.program.exe);

        if let Some(argv) = &self.program.argv {
            command.args(argv);
        }

        if !self.program.passthrough_environ {
            command.env_clear();
        }

        if let Some(environ) = &self.program.environ {
            command.envs(environ.iter().map(|(k, v)| (k.as_str(), v.as_str())));
        }

        if let Some(working_directory) = &self.program.working_directory {
            command.current_dir(working_directory);
        }

        command
    }

    pub fn try_from_config(
        config: UpdateConfig,
        name: &str,
        id_map: &HashMap<String, UpdateId>,
    ) -> Result<Update> {
        let mut conflicts = Vec::with_capacity(config.conflicts.len());
        for conflict in config.conflicts.into_iter() {
            conflicts.push(
                *id_map
                    .get(&conflict)
                    .ok_or(ErrorKind::InvalidConfig.context("Conflict doesn't exist"))?,
            );
        }

        let mut depends = Vec::with_capacity(config.depends.len());
        for depend in config.depends.into_iter() {
            depends.push(
                *id_map
                    .get(&depend)
                    .ok_or(ErrorKind::InvalidConfig.context("Dependency doesn't exist"))?,
            );
        }

        Ok(Update::new_with_runnner(
            *id_map
                .get(name)
                .ok_or(ErrorKind::InvalidConfig.context("Name doesn't exist"))?,
            config.program,
            Info {
                input: config.input,
                root: config.root,
                parallel: true,
                conflicts,
                depends,
            },
            match config.kind {
                UpdateKind::Default => &default::run,
                UpdateKind::Paru => &paru::run,
            },
        ))
    }
}
