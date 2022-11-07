use crate::config::{
    global_config::{
        retrieve_global_config,
        GlobalConfig
    },
    device_config::retrieve_device_configs
};
use tokio::{
    sync::RwLock,
};
use std::{
    path::PathBuf,
    sync::Arc,
};
use tracing::{
    warn,
    debug,
};

/// Device Configuration specifc data
pub mod device_config;
/// Global Configuration specifc data
pub mod global_config;

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

impl ConfigManager {
    /// create a new ConfigManager
    pub async fn new(path: Option<PathBuf>) -> ConfigManager {
        debug!("New ConfigManager created");
        let path = path.unwrap_or_else(|| {
            let mut dir = config_dir();
            dir.push(GLOBAL_CONFIG_FILE);
            dir
        });
        let global_config = retrieve_global_config(&path).await;
        debug!("{:#?}", global_config);
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
    pub fn global_configuration(&self) -> &Arc<RwLock<GlobalConfig>> {
        &self.global_config
    }

    /// returns the path from the global configuration file
    pub fn global_configuration_path(&self) -> &PathBuf {
        &self.global_config_path
    }

    /// return the path to the fonts folder
    pub fn fonts_path() {}

    /// return the path for the plugins folder
    pub fn plugins_path() {}

}
