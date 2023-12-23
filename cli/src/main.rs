use std::{io::Read, process::ExitCode};

use parallel_update::{types::*, update::Update, Updater};
use parallel_update_config::config::{Config, UpdaterConfig};

fn print_update(
    update: &Update,
    state: State,
    config: &UpdaterConfig,
    indent: usize,
    print_depends: bool,
) {
    for _ in 0..indent {
        eprint!("  ");
    }

    eprint!("\x1b[1m{:?} {:?}", update.id, update.program);

    if let State::Failed(code) = state {
        eprint!(" ({})", code);
    }

    if config.output_duration {
        if let Some(ref output) = *update.output.lock().unwrap() {
            eprint!(" took {:?}", output.duration);
        } else {
            eprint!(" took ??");
        }
    }

    eprintln!("\x1b[0m");

    if print_depends {
        eprintln!("  Depends: {:?}", update.info.depends);
        eprintln!("  Conflicts: {:?}", update.info.conflicts);
    }

    if (config.output_success_logs && state == State::Success)
        || (config.output_failure_logs && matches!(state, State::Failed(_) | State::Error))
    {
        if let Some(ref output) = *update.output.lock().unwrap() {
            let mut did_print = false;

            let stdout = std::str::from_utf8(&output.output.stdout).unwrap();
            if !stdout.trim().is_empty() {
                for _ in 0..=indent {
                    eprint!("  ");
                }
                eprintln!("stdout:");

                eprintln!("{}", stdout);

                did_print = true;
            }
            let stderr = std::str::from_utf8(&output.output.stderr).unwrap();
            if !stderr.trim().is_empty() {
                for _ in 0..=indent {
                    eprint!("  ");
                }
                eprintln!("stderr:");

                eprintln!("{}", stderr);

                did_print = true;
            }
            if did_print {
                eprintln!();
            }
        }
    }
}

fn main() -> Result<ExitCode, Box<dyn std::error::Error>> {
    let mut config = std::fs::OpenOptions::new()
        .read(true)
        .open("./updates.toml")?;

    let mut config_str = String::with_capacity(config.metadata()?.len() as usize);

    config.read_to_string(&mut config_str)?;

    let config: Config = toml::from_str(&config_str)?;

    let (c, updater) = Updater::try_from_config(config)?;

    if c.debug_config {
        eprintln!("{:#?}", c);

        for update in updater.updates() {
            print_update(update, State::Pending, &c, 0, true);
        }

        return Ok(ExitCode::SUCCESS);
    }

    let start = std::time::Instant::now();
    let results = updater.run(c.threads);
    let duration = start.elapsed();

    let results: Vec<_> = results
        .into_iter()
        .map(|update| (update.state.get(), update))
        .collect();

    let successful: Vec<_> = results
        .iter()
        .filter(|(state, _)| *state == State::Success)
        .collect();
    let failed: Vec<_> = results
        .iter()
        .filter(|(state, _)| matches!(state, State::Failed(_)))
        .collect();
    let ignored: Vec<_> = results
        .iter()
        .filter(|(state, _)| *state == State::Ignored)
        .collect();

    if c.output_states {
        if !successful.is_empty() {
            eprintln!("\x1b[32;1mSuccess\x1b[0m:");
            for (state, update) in &successful {
                print_update(update, *state, &c, 1, false);
            }
            eprintln!();
        }

        if !failed.is_empty() {
            eprintln!("\x1b[31;1mFailed\x1b[0m:");
            for (state, update) in &failed {
                print_update(update, *state, &c, 1, false);
            }
            eprintln!();
        }

        if !ignored.is_empty() {
            eprintln!("\x1b[2;1mIgnored\x1b[0m:");
            for (state, update) in &ignored {
                print_update(update, *state, &c, 1, true);
            }
            eprintln!();
        }
    }

    if c.output_duration {
        eprintln!("Total time: {:?}", duration);
    }

    Ok(if failed.is_empty() && ignored.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}
