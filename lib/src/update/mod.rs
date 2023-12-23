use std::{
    process::{Command, Output},
    sync::Mutex,
    time::Duration,
};

use crate::types::*;

pub mod default;

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
}
