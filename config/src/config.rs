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
#[cfg_attr(feature = "clap", derive(clap::Args))]
pub struct UpdaterConfig {
    /// Output how long each update took
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "clap", arg(long))]
    pub output_duration: bool,
    /// Output stdout/stderr for successful updates
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "clap", arg(long))]
    pub output_success_logs: bool,
    /// Output stdout/stderr for failed updates
    #[cfg_attr(feature = "serde", serde(default = "default_true"))]
    #[cfg_attr(feature = "clap", arg(long, default_value = "true"))]
    pub output_failure_logs: bool,
    /// Output update states
    #[cfg_attr(feature = "serde", serde(default = "default_true"))]
    #[cfg_attr(feature = "clap", arg(long, default_value = "true"))]
    pub output_states: bool,
    /// Number of updates to run at once
    #[cfg_attr(feature = "serde", serde(default = "default_two"))]
    #[cfg_attr(feature = "clap", arg(short, long, default_value = "2"))]
    pub threads: usize,
    /// Debug config
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "clap", arg(long))]
    pub debug_config: bool,
}

macro_rules! override_if_not_default {
    ($self: ident, $other: ident, $default: ident, $member: ident) => {
        if $other.$member != $default.$member {
            $self.$member = $other.$member;
        }
    };
}

impl UpdaterConfig {
    /// Merge this UpdaterConfig with another, the other taking precedence when its values are not the default
    pub fn merge(&mut self, other: &UpdaterConfig) {
        let default = Self::default();

        override_if_not_default!(self, other, default, output_duration);
        override_if_not_default!(self, other, default, output_success_logs);
        override_if_not_default!(self, other, default, output_failure_logs);
        override_if_not_default!(self, other, default, output_states);
        // FIXME: This doesn't work well for numbers
        override_if_not_default!(self, other, default, threads);
        override_if_not_default!(self, other, default, debug_config);
    }

    pub const fn default() -> UpdaterConfig {
        UpdaterConfig {
            output_duration: false,
            output_success_logs: false,
            output_failure_logs: true,
            output_states: true,
            threads: 2,
            debug_config: false,
        }
    }
}

impl Default for UpdaterConfig {
    fn default() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Config {
    pub updater: UpdaterConfig,
    /// Update configuration
    pub updates: HashMap<String, UpdateConfig>,
}
