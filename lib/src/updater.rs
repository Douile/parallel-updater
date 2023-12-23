use std::{
    collections::HashSet,
    sync::{mpsc::channel, Arc},
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
        let stdin_in_use = self
            .updates
            .iter()
            .any(|update| update.state.get() == State::Running && update.info.input);

        let done = self.done();
        let running = self.running();

        let mut valid_pending = self.updates.iter().filter(|update| {
            let info = &update.info;
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
                    (update.run)(&update);
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
