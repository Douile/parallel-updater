use std::process::Stdio;

use crate::types::*;
use crate::update::Update;

pub fn run(update: &Update) {
    update.state.set(State::Starting);

    let mut command = update.create_command();
    {
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
    }

    let child = match command.spawn() {
        Ok(child) => child,
        Err(e) => {
            eprintln!("Error spawning child: {:?}", e);
            update.state.set(State::Error);
            return;
        }
    };

    update.state.set(State::Running);

    let output = match child.wait_with_output() {
        Ok(output) => output,
        Err(e) => {
            eprintln!("Error waiting for output: {:?}", e);
            update.state.set(State::Error);
            return;
        }
    };

    if output.status.success() {
        update.state.set(State::Success);
    } else {
        update
            .state
            .set(State::Failed(output.status.code().unwrap_or(0)));
    }

    *update.output.lock().unwrap() = Some(output);
}
