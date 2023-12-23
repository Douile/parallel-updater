use std::{
    process::{Command, Output, Stdio},
    sync::Mutex,
};

use crate::types::*;

/// An Update that can be run
#[derive(Debug)]
pub struct Update {
    pub id: UpdateId,
    pub(crate) info: Mutex<Info>,
    pub state: SyncState,
    pub program: Program,
    pub output: Mutex<Option<Output>>,
}

impl Update {
    pub fn new(id: UpdateId, program: Program, info: Info) -> Self {
        Update {
            id,
            program,
            info: Mutex::new(info),
            state: SyncState::new(State::Pending),
            output: Mutex::new(None),
        }
    }

    pub fn run(&self) {
        self.state.set(State::Starting);

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

        {
            let info = match self.info.lock() {
                Ok(info) => info,
                Err(e) => {
                    eprintln!("Mutex error: {:?}", e);
                    self.state.set(State::Error);
                    return;
                }
            };

            // TODO: Capture
            if info.input {
                command.stdin(Stdio::inherit());
                command.stdout(Stdio::inherit());
                command.stderr(Stdio::inherit());
            } else {
                command.stdin(Stdio::null());
                command.stdout(Stdio::piped());
                command.stderr(Stdio::piped());
            }
        }

        let child = match command.spawn() {
            Ok(child) => child,
            Err(e) => {
                eprintln!("Error spawning child: {:?}", e);
                self.state.set(State::Error);
                return;
            }
        };

        self.state.set(State::Running);

        let output = match child.wait_with_output() {
            Ok(output) => output,
            Err(e) => {
                eprintln!("Error waiting for output: {:?}", e);
                self.state.set(State::Error);
                return;
            }
        };

        if output.status.success() {
            self.state.set(State::Success);
        } else {
            self.state
                .set(State::Failed(output.status.code().unwrap_or(0)));
        }

        *self.output.lock().unwrap() = Some(output);
    }
}
