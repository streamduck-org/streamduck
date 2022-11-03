#![warn(missing_docs)]
use tokio::{
    sync::RwLock,
    fs
};
use toml;
use std::{
    path::PathBuf,
    sync::Arc,
    collections::HashMap,
    time::Instant
};
use tracing::{warn, trace, error, info};
use serde::{Serialize, Deserialize};

/// # General
/// The ConfigManager handles the file reading and writing for:
/// - device configuration
/// - global configuration
/// - fonts
/// - plugins
///
/// This also includes efficient automatic writing of:
/// - device configuration
/// - global configuration
///
/// # Plugin usage 
/// Plugins should avoid creating a custom ConfigManager (or using at least file writes and reads) as it would create multiple auto-saving
/// tasks and could result in unknown behavior.
pub struct ConfigManager {
    global_config_path: PathBuf,
    global_config: Arc<RwLock<GlobalConfig>>,
    // device_configs: HashMap<String, Arc<RwLock<DeviceConfig>>>
}

/// Keys for the global config
/// For more description of the keys usage see: https://docs.streamduck.org/daemon/configuration.html
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct GlobalConfig {
    /// Path to device config folder
    device_config_path: Option<PathBuf>,
    /// Path to plugins folder
    plugin_path: Option<PathBuf>,
    /// Path to plugin settings json
    plugin_settings_path: Option<PathBuf>,
    /// Path to fonts
    font_path: Option<PathBuf>,

    /// Autosave device configuration
    autosave: Option<bool>,

    /// If plugin compatibility checks should be performed
    plugin_compatibility_checks: Option<bool>,
}

/// Keys for the device config
/// This is the information that gets saved for every device
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct DeviceConfig {
    // TODO: identifier is going to change
    /// Vendor ID
    pub identifier: String,
    #[serde(skip)]
    /// Last time the config was committed
    pub commit_time: Option<Instant>,
    #[serde(skip)]
    /// If config is dirty
    pub dirty_state: bool
    // TODO: add other data
}

/// folder name for the other subdirs
pub const CONFIG_FOLDER: &'static str = "streamduck";
/// folder name for the fonts
pub const FONTS_FOLDER: &'static str = "fonts";
/// folder name for the plugins
pub const PLUGINS_FOLDER: &'static str = "plugins";
/// folder name for the device configurations
pub const DEVICE_CONFIG_FOLDER: &'static str = "devices";
/// file name for the global config
pub const GLOBAL_CONFIG_FILE: &'static str = "config.toml";

/// Returns config directory (path: [`dirs::config_dir()`]/[`CONFIG_FOLDER`]) or returns the current dir. 
/// Data is allowed to be changed by a user.
pub fn config_dir() -> PathBuf {
    match dirs::config_dir() {
        Some(mut dir) => {
            dir.push(CONFIG_FOLDER);
            dir
        },
        None => {
            warn!("Configuration Directory not available on this system. Using executable path.");
            PathBuf::new()
        }
    }
}

/// Returns data directory (path: [`dirs::data_dir()`]/[`CONFIG_FOLDER`]) or returns the current dir.
/// Data is not intended to be changed by a user.
pub fn data_dir() -> PathBuf {
    match dirs::data_dir() {
        Some(mut dir) => {
            dir.push(CONFIG_FOLDER);
            dir
        },
        None => {
            warn!("Data directory not available on this system. Using executable path.");
            PathBuf::new()
        }
    }
}

/// Returns the GlobalConfig from the path.
/// Defaults to default if not available.
pub async fn retrieve_global_config(path: &PathBuf) -> GlobalConfig {
    let config = match fs::read_to_string(&path).await {
        Ok(content) => {
            match toml::from_str(&content) {
                Ok(config) => config,
                Err(e) => {
                    error!("Configuration error in {}:\n{}", &path.display(), e);
                    info!("Using standard configuration");
                    Default::default()
                }
            }
        },
        Err(e) => {
            match e.kind() {
                std::io::ErrorKind::NotFound => info!("The configuration file ({}) was not created yet.", &path.display()),
                _ => warn!("Access to the configuration file failed: \"{}\".", e)
            }
            info!("Using standard configuration");
            Default::default()
        }
    };
    config
}

/// write the device configurations from a path into a HashMap
pub async fn retrieve_device_configs(_path: &PathBuf) -> Result<HashMap<String, Arc<RwLock<DeviceConfig>>>, std::io::Error> {
    todo!()
}

impl ConfigManager {
    /// create a new ConfigManager
    pub async fn new(path: Option<PathBuf>) -> ConfigManager {
        let path = path.unwrap_or_else(|| {
            let mut dir = config_dir();
            dir.push(GLOBAL_CONFIG_FILE);
            dir
        });
        let global_config = retrieve_global_config(&path).await;
        trace!("Loaded GlobalConfig: {:?}", global_config);
        ConfigManager {
            global_config_path: path,
            global_config: Arc::new(RwLock::new(global_config))
        }
    }

    /// returns the path to a device config
    pub async fn device_config_path(&self, identifier: String) -> PathBuf {
        // TODO: maybe sanitize file names
        let mut dir = self.device_config_folder().await.clone();
        dir.push(format!("{}.json", identifier));
        dir
    }

    /// returns the path to the device configs folder
    pub async fn device_config_folder(&self) -> PathBuf {
        let global_config = self.global_config.read().await;
        global_config.device_config_path.clone().unwrap_or_else(|| {
            let mut dir = data_dir().clone();
            dir.push(DEVICE_CONFIG_FOLDER);
            dir
        })
    }

    /// returns the device from the HashMap of device configurations
    pub fn device_config(&self, _identifier: String) {
    }

    /// return the path to the fonts folder
    pub fn fonts_path() {}

    /// return the path for the plugins folder
    pub fn plugins_path() {}

}
