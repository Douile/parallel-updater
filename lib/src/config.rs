use std::collections::HashMap;

use crate::{
    error::{ErrorKind, Result},
    types::{Info, Program, UpdateId},
    Update, Updater,
};

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum UpdateKind {
    #[default]
    Default,
    Paru,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UpdateConfig {
    /// The kind of the update
    #[cfg_attr(feature = "serde", serde(default))]
    pub kind: UpdateKind,
    /// Whether the update requires exclusive input
    #[cfg_attr(feature = "serde", serde(default))]
    pub input: bool,
    /// Whether the update uses a program that gives root (e.g. sudo)
    #[cfg_attr(feature = "serde", serde(default))]
    pub root: bool,
    /// Updates that cannot run at the same time (order doesn't matter)
    #[cfg_attr(feature = "serde", serde(default))]
    pub conflicts: Vec<String>,
    /// Updates that must run before
    #[cfg_attr(feature = "serde", serde(default))]
    pub depends: Vec<String>,
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub program: Program,
}

#[cfg(feature = "serde")]
const fn default_true() -> bool {
    true
}

#[cfg(feature = "serde")]
const fn default_two() -> usize {
    2
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UpdaterConfig {
    /// Output how long each update took
    #[cfg_attr(feature = "serde", serde(default))]
    pub output_duration: bool,
    /// Output stdout/stderr for successful updates
    #[cfg_attr(feature = "serde", serde(default))]
    pub output_success_logs: bool,
    /// Output stdout/stderr for failed updates
    #[cfg_attr(feature = "serde", serde(default = "default_true"))]
    pub output_failure_logs: bool,
    /// Output update states
    #[cfg_attr(feature = "serde", serde(default = "default_true"))]
    pub output_states: bool,
    /// Number of updates to run at once
    #[cfg_attr(feature = "serde", serde(default = "default_two"))]
    pub threads: usize,
    /// Debug config
    #[cfg_attr(feature = "serde", serde(default))]
    pub debug_config: bool,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Config {
    pub updater: UpdaterConfig,
    /// Update configuration
    pub updates: HashMap<String, UpdateConfig>,
}

impl UpdateConfig {
    pub fn try_into_update(self, name: &str, id_map: &HashMap<String, UpdateId>) -> Result<Update> {
        let mut conflicts = Vec::with_capacity(self.conflicts.len());
        for conflict in self.conflicts.into_iter() {
            conflicts.push(
                *id_map
                    .get(&conflict)
                    .ok_or(ErrorKind::InvalidConfig.context("Conflict doesn't exist"))?,
            );
        }

        let mut depends = Vec::with_capacity(self.depends.len());
        for depend in self.depends.into_iter() {
            depends.push(
                *id_map
                    .get(&depend)
                    .ok_or(ErrorKind::InvalidConfig.context("Dependency doesn't exist"))?,
            );
        }

        Ok(Update::new_with_runnner(
            *id_map
                .get(name)
                .ok_or(ErrorKind::InvalidConfig.context("Name doesn't exist"))?,
            self.program,
            Info {
                input: self.input,
                root: self.root,
                parallel: true,
                conflicts,
                depends,
            },
            match self.kind {
                UpdateKind::Default => &crate::update::default::run,
                UpdateKind::Paru => &crate::update::paru::run,
            },
        ))
    }
}

impl Config {
    pub fn updater(self) -> Result<(UpdaterConfig, Updater)> {
        let mut id_map = HashMap::new();

        let update_configs: Vec<_> = self
            .updates
            .into_iter()
            .enumerate()
            .map(|(i, (name, config))| {
                id_map.insert(name.clone(), UpdateId(i));
                (name, config)
            })
            .collect();

        let mut updates = Vec::with_capacity(update_configs.len());

        for (name, update) in update_configs {
            updates.push(update.try_into_update(&name, &id_map)?)
        }

        Ok((self.updater, Updater::new(updates)?))
    }
}
