//! Daemon config and device configs
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;
use serde::{Serialize, Deserialize};
use streamduck_core::core::RawButtonPanel;

pub const DEFAULT_POOL_RATE: u32 = 90;
pub const DEFAULT_RECONNECT_TIME: f32 = 1.0;
pub const DEFAULT_CONFIG_PATH: &'static str = "devices";
pub const DEFAULT_PLUGIN_PATH: &'static str = "plugins";

/// Struct to keep daemon settings
#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    /// Frequency of streamdeck event pooling
    pool_rate: Option<u32>,
    /// Frequency of checks for disconnected devices
    reconnect_rate: Option<f32>,
    /// Path to device configs
    device_config_path: Option<PathBuf>,
    /// Path to plugins
    plugin_path: Option<PathBuf>,

    /// Currently loaded device configs
    #[serde(skip)]
    pub loaded_configs: RwLock<HashMap<String, DeviceConfig>>
}

#[allow(dead_code)]
impl Config {
    /// Reads config and retrieves config struct
    pub fn get() -> Config {
        if let Ok(content) = fs::read_to_string("config.toml") {
            if let Ok(config) = toml::from_str(&content) {
                config
            } else {
                Default::default()
            }
        } else {
            Default::default()
        }
    }

    /// Pool rate, defaults to '90' if not set
    pub fn pool_rate(&self) -> u32 {
        self.pool_rate.unwrap_or(DEFAULT_POOL_RATE)
    }

    /// Reconnect rate, defaults to 'every second' if not set
    pub fn reconnect_rate(&self) -> f32 {
        self.reconnect_rate.unwrap_or(DEFAULT_RECONNECT_TIME)
    }

    /// Device config path, defaults to '/devices' if not set
    pub fn device_config_path(&self) -> PathBuf {
        self.device_config_path.clone().unwrap_or(PathBuf::from(DEFAULT_CONFIG_PATH))
    }

    /// Plugin folder path, defaults to '/plugins' if not set
    pub fn plugin_path(&self) -> PathBuf {
        self.plugin_path.clone().unwrap_or(PathBuf::from(DEFAULT_PLUGIN_PATH))
    }

    /// Reloads device config for specified serial
    pub fn reload_device_config(&self, serial: &str) -> Result<(), ConfigError> {
        let mut devices = self.loaded_configs.write().unwrap();

        let mut path = self.device_config_path();
        path.push(format!("{}.json", serial));

        let content = fs::read_to_string(path)?;
        let device = serde_json::from_str::<DeviceConfig>(&content)?;
        devices.insert(device.serial.clone(), device);

        Ok(())
    }

    /// Reloads all device configs
    pub fn reload_device_configs(&self) -> Result<(), ConfigError> {
        let mut devices = self.loaded_configs.write().unwrap();

        let dir = fs::read_dir(self.device_config_path())?;

        for item in dir {
            let item = item?;
            if item.path().is_file() {
                if let Some(extension) = item.path().extension() {
                    if extension == "json" {
                        let content = fs::read_to_string(item.path())?;

                        let device = serde_json::from_str::<DeviceConfig>(&content)?;

                        devices.insert(device.serial.clone(), device);
                    }
                }
            }
        }

        Ok(())
    }

    /// Saves device config for specified serial
    pub fn save_device_config(&self, serial: &str) -> Result<(), ConfigError> {
        let devices = self.loaded_configs.read().unwrap();

        if let Some(device) = devices.get(serial) {
            let mut path = self.device_config_path();
            path.push(format!("{}.json", serial));

            fs::write(path, serde_json::to_string(device).unwrap())?;
            Ok(())
        } else {
            Err(ConfigError::DeviceNotFound)
        }
    }

    /// Saves device configs for all serials
    pub fn save_device_configs(&self) -> Result<(), ConfigError> {
        let devices = self.loaded_configs.read().unwrap();

        let path = self.device_config_path();

        for (serial, device) in devices.iter() {
            let mut file_path = path.clone();
            file_path.push(format!("{}.json", serial));
            fs::write(file_path, serde_json::to_string(device).unwrap())?;
        }

        Ok(())
    }

    /// Retrieves device config for specified serial
    pub fn get_device_config(&self, serial: &str) -> Option<DeviceConfig> {
        self.loaded_configs.read().unwrap().get(serial).cloned()
    }

    /// Sets device config for specified serial
    pub fn set_device_config(&self, serial: &str, config: DeviceConfig) {
        self.loaded_configs.write().unwrap().insert(serial.to_string(), config);
    }

    /// Gets an array of all device configs
    pub fn get_all_device_configs(&self) -> Vec<DeviceConfig> {
        self.loaded_configs.read().unwrap().values().map(|x| x.clone()).collect()
    }

    /// Disables a device config, so it will not be loaded by default
    pub fn disable_device_config(&self, serial: &str) -> bool {
        let path = self.device_config_path();

        let mut initial_path = path.clone();
        initial_path.push(format!("{}.json", serial));

        let mut new_path = path.clone();
        new_path.push(format!("{}.json_disabled", serial));

        fs::rename(initial_path, new_path).is_ok()
    }

    /// Restores device config if it exists
    pub fn restore_device_config(&self, serial: &str) -> bool {
        let path = self.device_config_path();

        let mut initial_path = path.clone();
        initial_path.push(format!("{}.json_disabled", serial));

        let mut new_path = path.clone();
        new_path.push(format!("{}.json", serial));

        fs::rename(initial_path, new_path).is_ok()
    }
}

/// Error enum for various errors while loading and parsing configs
#[derive(Debug)]
pub enum ConfigError {
    IoError(std::io::Error),
    ParseError(serde_json::Error),
    DeviceNotFound
}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::IoError(err)
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(err: serde_json::Error) -> Self {
        ConfigError::ParseError(err)
    }
}

/// Device config struct
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceConfig {
    pub vid: u16,
    pub pid: u16,
    pub serial: String,
    pub brightness: u8,
    pub layout: RawButtonPanel,
}