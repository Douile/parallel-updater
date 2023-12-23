use std::collections::HashMap;

use crate::{
    primatives::UpdateKind,
    types::Program,
    util::{default_true, default_two},
};

/// Config for a specfic updater
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

/// Config for the overall parallel updater
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
