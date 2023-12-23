use std::sync::{Condvar, Mutex};

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
    pub(crate) fn new(state: State) -> SyncState {
        SyncState {
            mutex: Mutex::new(state),
            condvar: Condvar::new(),
        }
    }

    pub(crate) fn set(&self, state: State) {
        let mut store = self.mutex.lock().unwrap();
        *store = state;
        self.condvar.notify_one();
    }

    pub fn get(&self) -> State {
        *self.mutex.lock().unwrap()
    }

    pub(crate) fn wait_until_not(&self, state: State) {
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
