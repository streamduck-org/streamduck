/// Device configs
pub mod device;

use std::collections::HashSet;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use arc_swap::ArcSwap;
use directories::ProjectDirs;
use scc::HashMap;
use serde::{Serialize, Deserialize};
use tokio::fs;
use tokio::io;
use tokio::task::block_in_place;
use tracing::{error, info, trace};
use crate::config::device::DeviceConfig;
use crate::device::DeviceIdentifier;
use crate::util::sha256_digest;

/// ArcSwapped Config instance
pub type SharedConfig = Arc<ArcSwap<Config>>;

/// Project directories of Streamduck
pub static STREAMDUCK_FOLDER: OnceLock<ProjectDirs> = OnceLock::new();

/// Name of the config file
pub const CONFIG_FILENAME: &'static str = "config.json";

/// Configuration struct for the software
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Config {
    #[serde(skip)]
    device_configs: HashMap<DeviceIdentifier, DeviceConfig>,

    /// How many ticks a second should Streamduck process, affects how often input is checked and how often plugins get ticked, ticks/second
    pub tick_rate: f32,
    /// How often Streamduck will attempt to update button images, frames/second
    pub frame_rate: f32,
    /// How often Streamduck should find new devices to use, interval in seconds
    pub device_check_frequency: f32,
    /// Interval in seconds of how often Streamduck will autosave all data
    pub autosave_interval: f32,
    /// Devices that should be autoconnected
    pub autoconnect_devices: HashSet<DeviceIdentifier>
}

impl Default for Config {
    fn default() -> Self {
        Config {
            device_configs: Default::default(),
            tick_rate: 50.0,
            frame_rate: 60.0,
            device_check_frequency: 30.0,
            autosave_interval: 60.0,
            autoconnect_devices: HashSet::new(),
        }
    }
}

impl Config {
    /// Loads or creates config file
    pub async fn load_or_create() -> SharedConfig {
        let config_path = get_streamduck_config_dir().join(CONFIG_FILENAME);

        if !fs::try_exists(&config_path).await.expect("Couldn't access config") {
            let config = Config::default();

            config.save().await.expect("Couldn't save config");

            Arc::new(
                ArcSwap::new(
                    Arc::new(
                        config
                    )
                )
            )
        } else {
            let config_str = fs::read_to_string(&config_path).await.expect("Couldn't read config");

            Arc::new(
                ArcSwap::new(
                    Arc::new(
                        block_in_place(|| {
                            serde_json::from_str(&config_str).expect("Couldn't parse config")
                        })
                    )
                )
            )
        }
    }

    /// Retrieves device config if it exists
    pub async fn get_device_config(&self, device_identifier: &DeviceIdentifier) -> Option<DeviceConfig> {
        // If it was already loaded
        if let Some(config) = self.device_configs.get_async(device_identifier).await {
            return Some(config.get().clone());
        }

        // Load if not loaded
        let name = match sha256_digest(&device_identifier) {
            Ok(n) => n,
            Err(e) => {
                error!(%e, identifier = %device_identifier, "Failed to hash identifier");
                return None;
            }
        };

        let path = get_streamduck_device_configs_dir().join(format!("{}.bin", name));

        if path.exists() {
            let contents = match fs::read(path.clone()).await {
                Ok(c) => c,
                Err(e) => {
                    error!(%e, identifier = %device_identifier, ?path, "Failed to read device config");
                    return None;
                }
            };
            let config: DeviceConfig = match block_in_place(move || { rmp_serde::from_slice(contents.as_slice()) }) {
                Ok(c) => c,
                Err(e) => {
                    error!(%e, identifier = %device_identifier, ?path, "Error trying to deserialize device config");
                    return None;
                }
            };

            self.device_configs.insert_async(device_identifier.clone(), config.clone()).await.ok();

            return Some(config);
        }

        None
    }

    /// Writes device config
    pub async fn write_device_config(&self, device_identifier: DeviceIdentifier, config: DeviceConfig) -> Option<()> {
        let name = sha256_digest(&device_identifier).ok()?;
        let configs_path = get_streamduck_device_configs_dir();

        match fs::create_dir_all(&configs_path).await {
            Ok(_) => {}
            Err(e) => {
                error!(%e, "Error happened trying to create directories");
            }
        }

        let path = configs_path.join(format!("{}.bin", name));

        self.device_configs.insert_async(device_identifier.clone(), config.clone()).await.ok();

        let config_data = match block_in_place(move || { rmp_serde::to_vec_named(&config) }) {
            Ok(data) => data,
            Err(e) => {
                error!(%e, identifier = %device_identifier, "Couldn't serialize config into MsgPack");
                return None;
            }
        };

        match fs::write(&path, config_data).await {
            Ok(_) => {
                info!(identifier = %device_identifier, ?path, "Successfully saved device config to path");
            }
            Err(e) => {
                error!(%e, identifier = %device_identifier, ?path, "Failed to write device config");
                return None;
            }
        }

        Some(())
    }

    /// Writes config to disk
    pub async fn save(&self) -> Result<(), ConfigError> {
        let config_path = get_streamduck_config_dir().join(CONFIG_FILENAME);

        fs::create_dir_all(config_path.parent().unwrap()).await?;
        Ok(
            fs::write(
                &config_path,
                block_in_place(move || serde_json::to_string_pretty(self))?
            ).await?
        )
    }
}

/// Any error that can be associated with configs
#[derive(Debug)]
pub enum ConfigError {
    /// IO Error
    IOError(io::Error),
    /// JSON parsing error
    JSONError(serde_json::Error)
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ConfigError {}

impl From<io::Error> for ConfigError {
    fn from(value: io::Error) -> Self {
        Self::IOError(value)
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(value: serde_json::Error) -> Self {
        Self::JSONError(value)
    }
}

fn get_project_dirs() -> ProjectDirs {
    ProjectDirs::from("org", "streamduck", "Streamduck")
        .expect("Couldn't retrieve OS folders")
}

/// Gets config directory path of Streamduck, the directory might not exist!
pub fn get_streamduck_config_dir() -> &'static Path {
    STREAMDUCK_FOLDER.get_or_init(get_project_dirs).config_dir()
}

/// Gets data directory path of Streamduck, the directory might not exist!
pub fn get_streamduck_data_dir() -> &'static Path {
    STREAMDUCK_FOLDER.get_or_init(get_project_dirs).data_dir()
}

/// Gets device config directory path of Streamduck, the directory might not exist!
pub fn get_streamduck_device_configs_dir() -> PathBuf {
    STREAMDUCK_FOLDER.get_or_init(get_project_dirs).data_dir().join("devices")
}