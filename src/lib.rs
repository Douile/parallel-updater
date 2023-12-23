use std::{
    collections::HashSet,
    process::{Command, Output, Stdio},
    sync::{mpsc::channel, Arc, Condvar, Mutex},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UpdateId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Pending,
    Starting,
    Running,
    Success,
    Failed(i32),
    Error,
}

impl State {
    pub fn is_done(&self) -> bool {
        matches!(self, State::Success | State::Failed(_) | State::Error)
    }
    pub fn is_running(&self) -> bool {
        matches!(self, State::Starting | State::Running)
    }
}

/// Represents relevent info about a program.
#[derive(Debug, Clone, PartialEq)]
pub struct Info {
    /// Whether the update requires exclusive input
    pub input: bool,
    /// Whether the update uses a program that gives root (e.g. sudo)
    pub root: bool,
    /// Can run in parallel
    pub parallel: bool,
    /// Updates that cannot run at the same time (order doesn't matter)
    pub conflicts: Vec<UpdateId>,
    /// Updates that must run before
    pub depends: Vec<UpdateId>,
}

/// Represents details needed to start a program.
#[derive(Debug, Clone)]
pub struct Program {
    /// Path to executable of program
    pub exe: String,
    /// Optional arguments for the program
    pub argv: Option<Vec<String>>,
    /// Optional extra environment variables for the program
    pub environ: Option<Vec<(String, String)>>,
    /// 
    pub working_directory: Option<String>,
    pub passthrough_environ: bool,
}

impl Program {
    /// Create new Program (builder)
    pub fn new(exe: impl Into<String>) -> Self {
        Program {
            exe: exe.into(),
            argv: None,
            environ: None,
            working_directory: None,
            passthrough_environ: true,
        }
    }

    pub fn argv(mut self, argv: impl Into<Vec<String>>) -> Self {
        self.argv = Some(argv.into());

        self
    }

    pub fn argv_str(self, argv: &[&str]) -> Self {
        self.argv(argv.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    }

    pub fn environ(mut self, environ: Vec<(String, String)>) -> Self {
        self.environ = Some(environ);

        self
    }

    pub fn working_directory(mut self, working_directory: String) -> Self {
        self.working_directory = Some(working_directory);

        self
    }

    pub fn passthrough_environ(mut self, passthough_environ: bool) -> Self {
        self.passthrough_environ = passthough_environ;

        self
    }
}

/// Thread-safe update state
#[derive(Debug)]
pub struct SyncState {
    mutex: Mutex<State>,
    condvar: Condvar,
}

impl SyncState {
    fn new(state: State) -> SyncState {
        SyncState {
            mutex: Mutex::new(state),
            condvar: Condvar::new(),
        }
    }

    fn set(&self, state: State) {
        let mut store = self.mutex.lock().unwrap();
        *store = state;
        self.condvar.notify_one();
    }

    pub fn get(&self) -> State {
        *self.mutex.lock().unwrap()
    }

    fn wait_until_not(&self, state: State) {
        {
            if *self.mutex.lock().unwrap() != state {
                return;
            }
        }
        let _guard = self
            .condvar
            .wait_while(self.mutex.lock().unwrap(), |s| *s == state)
            .unwrap();
    }
}

/// An Update that can be run
#[derive(Debug)]
pub struct Update {
    pub id: UpdateId,
    info: Mutex<Info>,
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

#[derive(Debug)]
pub struct Updater {
    updates: Vec<Arc<Update>>,
}

impl Updater {
    pub fn new(updates: Vec<Update>) -> Updater {
        Updater {
            updates: updates.into_iter().map(Arc::new).collect(),
        }
    }

    fn all_done(&self) -> bool {
        self.updates
            .iter()
            .all(|update| update.state.get().is_done())
    }

    fn done(&self) -> HashSet<UpdateId> {
        self.updates
            .iter()
            .filter(|update| update.state.get().is_done())
            .map(|update| update.id)
            .collect()
    }

    fn running(&self) -> HashSet<UpdateId> {
        self.updates
            .iter()
            .filter(|update| update.state.get().is_running())
            .map(|update| update.id)
            .collect()
    }

    fn running_count(&self) -> usize {
        self.updates
            .iter()
            .filter(|update| update.state.get().is_running())
            .count()
    }

    fn greedy_select_update(&self) -> Option<UpdateId> {
        let stdin_in_use = self.updates.iter().any(|update| {
            update.state.get() == State::Running && update.info.lock().unwrap().input
        });

        let done = self.done();
        let running = self.running();

        let mut valid_pending = self.updates.iter().filter(|update| {
            let info = update.info.lock().unwrap();
            update.state.get() == State::Pending
                && (!stdin_in_use || !info.input)
                && info.depends.iter().all(|id| done.contains(id))
                && info.conflicts.iter().all(|id| !running.contains(id))
        });

        valid_pending.next().map(|update| update.id)
    }

    pub fn run(self, threads: usize) -> Vec<Arc<Update>> {
        let (tx, rx) = channel();

        while !self.all_done() {
            for _ in self.running_count()..threads {
                let Some(next) = self.greedy_select_update() else {
                    break;
                };

                let update = Arc::clone(&self.updates[next.0]);
                let tx = tx.clone();

                std::thread::spawn(move || {
                    update.run();
                    tx.send(update.id)
                });

                self.updates[next.0].state.wait_until_not(State::Pending);
            }

            rx.recv().unwrap();
            while rx.try_recv().is_ok() {}
        }

        self.updates
    }
}
