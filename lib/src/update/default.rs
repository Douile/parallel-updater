use std::process::Stdio;

use crate::types::*;
use crate::update::Update;
use crate::error::ErrorKind::{CommandOutput, CommandSpawn};

use super::UpdateOutput;

pub fn run(update: &Update, global_state: &GlobalState) {
    if update.info.input {
        // Must set global stdin lock before we set state to starting
        // because schedule continues trying to schedule once we have set state to starting.
        *global_state.has_stdin_lock.lock().unwrap() = Some(update.id);
    }

    update.state.set(State::Starting);

    let mut command = update.create_command();

    // TODO: Capture
    if update.info.input {
        command.stdin(Stdio::inherit());
        command.stdout(Stdio::inherit());
        command.stderr(Stdio::inherit());
    } else {
        command.stdin(Stdio::null());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
    }

    let child = match command.spawn() {
        Ok(child) => child,
        Err(e) => {
            eprintln!("Error spawning child: {:?}", e);
            update.state.set(State::Error(CommandSpawn));
            return;
        }
    };
    let start = std::time::Instant::now();

    update.state.set(State::Running);

    let output = match child.wait_with_output() {
        Ok(output) => output,
        Err(e) => {
            eprintln!("Error waiting for output: {:?}", e);
            update.state.set(State::Error(CommandOutput));
            return;
        }
    };
    let duration = start.elapsed();

    if output.status.success() {
        update.state.set(State::Success);
    } else {
        update
            .state
            .set(State::Failed(output.status.code().unwrap_or(0)));
    }

    *update.output.lock().unwrap() = Some(UpdateOutput { output, duration });
}
