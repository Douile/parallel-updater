use std::{io::Read, process::ExitCode};

use parallel_update::{config::Config, types::*};

fn main() -> Result<ExitCode, Box<dyn std::error::Error>> {
    let mut config = std::fs::OpenOptions::new()
        .read(true)
        .open("./updates.toml")?;

    let mut config_str = String::with_capacity(config.metadata()?.len() as usize);

    config.read_to_string(&mut config_str)?;

    let config: Config = toml::from_str(&config_str)?;

    eprintln!("{:#?}", config);

    let (c, updater) = config.updater()?;

    eprintln!("{:#?}", updater);

    let start = std::time::Instant::now();
    let result = updater.run(6);
    let duration = start.elapsed();

    let mut exitcode = ExitCode::SUCCESS;

    eprintln!("\x1b[32;1mSuccess\x1b[0m:");
    for update in &result {
        if update.state.get() == State::Success {
            eprintln!("\t\x1b[1m{:?} {:?}\x1b[0m", update.id, update.program);

            if c.output_success {
                if let Some(ref output) = *update.output.lock().unwrap() {
                    eprintln!("\t{:?}", output.duration);
                    eprintln!("{}", std::str::from_utf8(&output.output.stdout).unwrap());
                    eprintln!("{}", std::str::from_utf8(&output.output.stderr).unwrap());
                    eprintln!();
                }
            }
        }
    }

    eprintln!();
    eprintln!("\x1b[31;1mFailed\x1b[0m:");
    for update in &result {
        if let State::Failed(failed) = update.state.get() {
            exitcode = ExitCode::FAILURE;
            eprintln!(
                "\t\x1b[1m{:?} {:?} ({})\x1b[0m",
                update.id, update.program, failed
            );

            if let Some(ref output) = *update.output.lock().unwrap() {
                eprintln!("\t{:?}", output.duration);
                eprintln!("{}", std::str::from_utf8(&output.output.stdout).unwrap());
                eprintln!("{}", std::str::from_utf8(&output.output.stderr).unwrap());
                eprintln!();
            }
        }
        if update.state.get() == State::Error {
            eprintln!("\t\x1b[1m{:?} {:?}\x1b[0m", update.id, update.program);
        }
    }

    if c.output_duration {
        eprintln!("Total time: {:?}", duration);
    }

    Ok(exitcode)
}
