use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
use tokio::fs;
use toml;
use tracing::{error, info, warn};

/// Returns the GlobalConfig from the path.
/// Defaults to default if not available.
pub async fn retrieve_global_config(path: &PathBuf) -> GlobalConfig {
    let config = match fs::read_to_string(&path).await {
        Ok(content) => match toml::from_str(&content) {
            Ok(config) => config,
            Err(e) => {
                error!("Configuration error in {}:\n{}", &path.display(), e);
                info!("Using standard configuration");
                Default::default()
            }
        },
        Err(e) => {
            match e.kind() {
                std::io::ErrorKind::NotFound => info!(
                    "The configuration file ({}) was not created yet.",
                    &path.display()
                ),
                _ => warn!("Access to the configuration file failed: \"{}\".", e),
            }
            info!("Using standard configuration");
            Default::default()
        }
    };
    config
}

/// # Keys for the global config.
/// For more description of the keys usage see: <https://docs.streamduck.org/daemon/configuration.html>.
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct GlobalConfig {
    /// Path to device config folder
    pub device_config_path: Option<PathBuf>,
    /// If the [GlobalConfig] and [crate::config::device_config::DeviceConfig] should be saved automatically
    pub autosave: Option<bool>,
    /// The time a commit was last made.
    #[serde(skip)]
    commit_time: Option<Instant>,
    /// If the config got changed and now needs to be saved.
    #[serde(skip)]
    dirty_state: bool,
}

impl GlobalConfig {
    /// Mark config as dirty.
    pub(crate) fn mark_dirty(&mut self) {
        self.commit_time = Some(Instant::now());
        self.dirty_state = true;
    }

    /// Mark config as clean.
    pub(crate) fn mark_clean(&mut self) {
        self.dirty_state = false;
    }

    /// See if config is dirty
    pub fn is_dirty(&self) -> bool {
        self.dirty_state
    }

    /// The time since a commit was last made
    pub fn last_commit(&self) -> Option<Duration> {
        match self.commit_time {
            Some(commit_time) => Some(Instant::now().duration_since(commit_time)),
            None => None,
        }
    }
}
