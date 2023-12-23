use std::sync::{mpsc::Sender, Condvar, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UpdateId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// Update is waiting to start
    Pending,
    /// Update is starting (configuration)
    Starting,
    /// Update is currently running
    Running,
    /// Update finished successfully
    Success,
    /// Update finished returning an error
    Failed(i32),
    /// Update encountered a rust error while running
    Error,
    /// Update wasn't able to run
    Ignored,
}

impl State {
    pub fn is_done(&self) -> bool {
        matches!(
            self,
            State::Success | State::Failed(_) | State::Error | State::Ignored
        )
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

#[derive(Debug)]
pub struct GlobalState {
    pub(crate) should_try_scheduling: Sender<UpdateId>,
    pub(crate) has_stdin_lock: Mutex<Option<UpdateId>>,
}
