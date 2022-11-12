use crate::config::{
    global_config::{
        retrieve_global_config,
        GlobalConfig
    },
    device_config::{
        retrieve_device_configs,
        DeviceConfig
    },
};
use tokio::{
    sync::RwLock,
    io::AsyncWriteExt
};
use std::{
    path::PathBuf,
    sync::Arc,
    time::Duration
};
use tracing::{
    warn,
    debug,
    error,
    instrument,
    trace
};

/// Device Configuration specifc data
pub mod device_config;
/// Global Configuration specifc data
pub mod global_config;

/// # General
/// The ConfigManager handles the file reading and writing.
pub struct ConfigManager {
    global_config_path: PathBuf,
    global_config: Arc<RwLock<GlobalConfig>>,
    // device_configs: HashMap<String, Arc<RwLock<DeviceConfig>>>
}

/// Errors used in the ConfigManager
#[derive(Debug)]
pub enum ConfigError {
    /// A [Toml parsing error][toml::ser::Error]
    TomlParse(toml::ser::Error),
    /// An [IoError][std::io::Error]
    IoError(std::io::Error)
}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::IoError(err)
    }
}

impl From<toml::ser::Error> for ConfigError {
    fn from(err: toml::ser::Error) -> Self {
        ConfigError::TomlParse(err)
    }
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

/// Handles the autosaving loop for the [ConfigManager].
/// Iterates through [ConfigManager::autosave()] in the specified interval.
#[instrument(name = "autosaver", skip_all)]
pub(crate) async fn autosave_loop(config_manager: Arc<ConfigManager>, interval: Duration) {
    trace!("started");
    loop {
        config_manager.autosave_task().await;
        tokio::time::sleep(interval).await;
    }
}

impl ConfigManager {
    /// create a new ConfigManager
    pub async fn new(path: Option<PathBuf>) -> Result<Arc<ConfigManager>, ConfigError> {
        trace!("ConfigManager created");
        // create config_dir
        tokio::fs::create_dir_all(config_dir()).await?;
        // retrieve the global configuration
        let path = path.unwrap_or_else(|| {
            let mut dir = config_dir();
            dir.push(GLOBAL_CONFIG_FILE);
            dir
        });
        let global_config = retrieve_global_config(&path).await;
        debug!(?path, ?global_config);
        // create the manager
        let manager = Arc::new(ConfigManager {
            global_config_path: path,
            global_config: Arc::new(RwLock::new(global_config))
        });
        // start autosave loop if needed
        if manager.global_configuration().await.autosave.unwrap_or(true) {
            tokio::spawn(
                autosave_loop(manager.clone(), Duration::from_secs(10))
            );
        }
        Ok(manager)
    }

    /// checks if the config got changed and saves it if the last change was over 30 secs ago.
    pub async fn autosave_task(&self) {
        // TODO: implement device config
        let config = self.global_configuration().await;
        if let Some(last_commit) = config.last_commit() {
            if config.is_dirty() && last_commit > Duration::from_secs(30) {
                drop(config);
                match self.write_global_configuration().await {
                    Err(e) => error!("Writing failed: {:?}", e),
                    Ok(_) => trace!("Saved")
                }
                self.mut_global_configuration().await.mark_clean();
            }
        }
    }

    /// Returns the path from the global configuration file
    pub fn global_configuration_path(&self) -> &PathBuf {
        &self.global_config_path
    }

    /// Returns the path to a specifc [DeviceConfig]
    pub async fn device_config_path(&self, identifier: String) -> PathBuf {
        let mut dir = self.device_config_folder().await.clone();
        dir.push(format!("{}.json", identifier));
        dir
    }

    /// Returns the path to the folder where all [DeviceConfig]s are stored.
    pub async fn device_config_folder(&self) -> PathBuf {
        let global_config = self.global_config.read().await;
        global_config.device_config_path.clone().unwrap_or_else(|| {
            let mut dir = data_dir().clone();
            dir.push(DEVICE_CONFIG_FOLDER);
            dir
        })
    }

    /// Returns a readable lock of the [GlobalConfig].
    pub async fn global_configuration(&self) -> tokio::sync::RwLockReadGuard<GlobalConfig> {
        trace!("GlobalConfig lock aquired");
        self.global_config.read().await
    }

    /// Return a writeable lock of the [GlobalConfig].
    pub async fn mut_global_configuration(&self) -> tokio::sync::RwLockWriteGuard<GlobalConfig> {
        trace!("GlobalConfig mutable lock aquired");
        let mut lock = self.global_config.write().await;
        lock.mark_dirty();
        lock
    }

    /// Write the [GlobalConfig] to the filesystem
    pub(crate) async fn write_global_configuration(&self) -> Result<(), ConfigError> {
        let path = self.global_configuration_path().clone();

        // retrieve the configuration and write it to the file
        let conf = self.global_configuration().await;
        let mut file = tokio::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .await?;

        file.write(toml::to_string(&*conf)?.as_bytes()).await?;

        drop(conf);

        // mark the config clean
        let mut conf = self.mut_global_configuration().await;
        conf.mark_clean();
        Ok(())
    }


    /// return the path to the fonts folder
    pub fn fonts_path() {}

    /// return the path for the plugins folder
    pub fn plugins_path() {}

}
