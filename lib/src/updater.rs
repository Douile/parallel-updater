use std::{
    collections::HashSet,
    sync::{mpsc::channel, Arc, Mutex},
};

use crate::error::Result;
use crate::Update;
use crate::{error::ErrorKind, types::*};

#[derive(Debug)]
pub struct Updater {
    updates: Vec<Arc<Update>>,
}

fn validate_updates(updates: &[Update]) -> Result<()> {
    for update in updates {
        if update.id.0 >= updates.len() {
            return Err(ErrorKind::InvalidUpdater.context("Update ID is out of bounds"));
        }
        for dependency in &update.info.depends {
            if dependency.0 >= updates.len() {
                return Err(ErrorKind::InvalidUpdater.context("Dependecy ID is out of bounds"));
            }
            if *dependency == update.id {
                return Err(ErrorKind::InvalidUpdater.context("Update cannot depend on itself"));
            }
        }
        for conflict in &update.info.conflicts {
            if conflict.0 >= updates.len() {
                return Err(ErrorKind::InvalidUpdater.context("Conflict ID is out of bounds"));
            }
            if *conflict == update.id {
                return Err(ErrorKind::InvalidUpdater.context("Update cannot conflict with itself"));
            }
        }
    }

    Ok(())
}

impl Updater {
    pub fn new(updates: Vec<Update>) -> Result<Updater> {
        validate_updates(&updates)?;
        Ok(Updater {
            updates: updates.into_iter().map(Arc::new).collect(),
        })
    }

    pub fn updates(&self) -> &[Arc<Update>] {
        &self.updates
    }

    pub fn all_done(&self) -> bool {
        self.updates
            .iter()
            .all(|update| update.state.get().is_done())
    }

    pub fn done(&self) -> HashSet<UpdateId> {
        self.updates
            .iter()
            .filter(|update| update.state.get().is_done())
            .map(|update| update.id)
            .collect()
    }

    pub fn successful(&self) -> HashSet<UpdateId> {
        self.updates
            .iter()
            .filter(|update| update.state.get() == State::Success)
            .map(|update| update.id)
            .collect()
    }

    pub fn running(&self) -> HashSet<UpdateId> {
        self.updates
            .iter()
            .filter(|update| update.state.get().is_running())
            .map(|update| update.id)
            .collect()
    }

    pub fn running_count(&self) -> usize {
        self.updates
            .iter()
            .filter(|update| update.state.get().is_running())
            .count()
    }

    fn greedy_select_update(&self, global_state: &GlobalState) -> Option<UpdateId> {
        let stdin_in_use = global_state.has_stdin_lock.lock().unwrap().is_some();

        let done = self.done();
        let successful = self.successful();
        let running = self.running();

        let mut valid_pending = self.updates.iter().filter(|update| {
            // Set ignored if cannot run
            for dependecy in update.info.depends.iter() {
                let is_done = done.contains(dependecy);
                let is_success = successful.contains(dependecy);
                if is_done && !is_success {
                    update.state.set(State::Ignored);
                    return false;
                }
            }

            update.state.get() == State::Pending
                && (!stdin_in_use || !update.info.input)
                && update.info.depends.iter().all(|id| successful.contains(id))
                && update.info.conflicts.iter().all(|id| !running.contains(id))
        });

        valid_pending.next().map(|update| update.id)
    }

    pub fn run(self, threads: usize) -> Vec<Arc<Update>> {
        let (tx, rx) = channel();

        let global_state = Arc::new(GlobalState {
            should_try_scheduling: tx.clone(),
            has_stdin_lock: Mutex::new(None),
        });

        while !self.all_done() {
            for _ in self.running_count()..threads {
                let Some(next) = self.greedy_select_update(&global_state) else {
                    break;
                };

                let update = Arc::clone(&self.updates[next.0]);
                let global_state = Arc::clone(&global_state);
                let tx = tx.clone();

                std::thread::spawn(move || {
                    (update.run)(&update, &global_state);

                    // Cleanup un-closed stdin locks
                    let mut stdin_lock = global_state.has_stdin_lock.lock().unwrap();
                    if *stdin_lock == Some(update.id) {
                        *stdin_lock = None;
                    }

                    // Notify that we finished
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
