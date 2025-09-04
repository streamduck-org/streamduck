use std::env::current_dir;
use std::fs;
use std::path::PathBuf;
use serde_with::DurationSeconds;
use serde_with::serde_as;
use arc_swap::ArcSwap;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, LazyLock};
use std::time::Duration;
use directories::ProjectDirs;
use log::info;

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub client_srv_addr: String,
    pub tick_rate: u16,
    #[serde_as(as = "DurationSeconds")]
    pub device_discovery_interval: Duration
}

impl Default for Config {
    fn default() -> Self {
        Self {
            client_srv_addr: "127.0.0.1:27434".to_string(),
            tick_rate: 60,
            device_discovery_interval: Duration::from_secs(60),
        }
    }
}

pub static CONFIG: LazyLock<ArcSwap<Config>> = LazyLock::new(|| ArcSwap::default());

pub static PROJECT_DIRS: LazyLock<Option<ProjectDirs>> = LazyLock::new(||
    ProjectDirs::from("org", "streamduck", "streamduck_daemon")
);

pub fn config_folder_path() -> PathBuf {
    PROJECT_DIRS.as_ref().map(|p| p.config_dir().to_path_buf())
        .unwrap_or_else(|| current_dir().unwrap())
}

pub const CONFIG_FILENAME: &str = "config.json";

pub fn load_config() -> anyhow::Result<()> {
    let folder = config_folder_path();
    let file_path = folder.join(CONFIG_FILENAME);

    if !file_path.exists() {
        info!("Config file wasn't found, creating...");
        save_config()?;
        return Ok(())
    }

    let text = fs::read_to_string(file_path)?;
    let config = serde_json::from_str::<Config>(&text)?;
    CONFIG.swap(Arc::new(config));

    info!("Loaded config from disk");

    Ok(())
}

pub fn save_config() -> anyhow::Result<()> {
    let folder = config_folder_path();

    fs::create_dir_all(&folder)?;

    let file_path = folder.join(CONFIG_FILENAME);

    fs::write(file_path, serde_json::to_string(CONFIG.load().as_ref())?)?;

    info!("Saved config to disk");

    Ok(())
}