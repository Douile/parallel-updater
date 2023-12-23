use std::io::{BufRead, BufReader, Write};
use std::process::Stdio;

use crate::types::*;
use crate::update::Update;

use super::UpdateOutput;

const SUDO_PROMPT: &str = "[sudo] password: ";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputState {
    SudoPrompt,
    ParuConfirm,
    Done,
}

// TODO: Properly handle errors
pub fn run(update: &Update, global_state: &GlobalState) {
    // Must set global stdin lock before we set state to starting
    // because schedule continues trying to schedule once we have set state to starting.
    *global_state.has_stdin_lock.lock().unwrap() = Some(update.id);

    update.state.set(State::Starting);

    let mut command = update.create_command();

    assert!(update.info.input, "Paru runner requires capturing input");
    command.env("SUDO_PROMPT", SUDO_PROMPT);

    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(e) => {
            eprintln!("Error spawning child: {:?}", e);
            update.state.set(State::Error);
            return;
        }
    };
    let start = std::time::Instant::now();

    update.state.set(State::Running);

    {
        let mut state = InputState::SudoPrompt;

        let mut stdin = std::io::stdin().lock();
        let child_stdin = child.stdin.as_mut().unwrap();
        let stdout = BufReader::new(child.stdout.as_mut().unwrap());

        let mut input = String::new();

        for line in stdout.lines() {
            let line = line.unwrap();
            println!("{}", line);

            match state {
                InputState::SudoPrompt if line.starts_with(SUDO_PROMPT) => {
                    state = InputState::ParuConfirm;

                    stdin.read_line(&mut input).unwrap();
                    child_stdin.write_all(input.as_bytes()).unwrap();
                }
                InputState::SudoPrompt | InputState::ParuConfirm
                    if line.starts_with(":: Proceed") =>
                {
                    state = InputState::Done;

                    stdin.read_line(&mut input).unwrap();
                    child_stdin.write_all(input.as_bytes()).unwrap();
                    break;
                }
                InputState::Done => break,
                _ => {}
            }
        }
    }

    // We no-longer need stdin
    *global_state.has_stdin_lock.lock().unwrap() = None;

    // Notify of a change
    global_state.should_try_scheduling.send(update.id).unwrap();

    let output = match child.wait_with_output() {
        Ok(output) => output,
        Err(e) => {
            eprintln!("Error waiting for output: {:?}", e);
            update.state.set(State::Error);
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
