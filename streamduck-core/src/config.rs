//! Core and device configs
use std::collections::HashMap;
use tokio::fs;
use dirs;
use std::ops::Deref;
use std::time::{Instant, Duration};
use std::path::PathBuf;
use std::sync::{Arc};
use image::{DynamicImage};
use serde::{Serialize, Deserialize};
use serde::de::DeserializeOwned;
use crate::core::RawButtonPanel;
use serde_json::Value;
use streamdeck::Kind;
use tokio::sync::RwLock;
use crate::ImageCollection;
use crate::images::{SDImage, SDSerializedImage};
use crate::util::{hash_image, hash_str};
use crate::thread::util::resize_for_streamdeck;

// default folder name
pub const CONFIG_FOLDER: &'static str = "streamduck";

// sub folders and files
pub const DEFAULT_FRAME_RATE: u32 = 100;
pub const DEFAULT_RECONNECT_TIME: f32 = 1.0;
pub const FONTS_FOLDER: &'static str = "fonts";
pub const DEVICE_CONFIG_FOLDER: &'static str = "devices";
pub const PLUGINS_FOLDER: &'static str = "plugins";
pub const PLUGINS_SETTINGS_FILE: &'static str = "global.json";
pub const CONFIG_FILE: &'static str = "config.toml";

/// Reference counted [DeviceConfig]
pub type UniqueDeviceConfig = Arc<RwLock<DeviceConfig>>;

/// Loads config directory (eg. $HOME/.config/streamduck) or returns the current dir
fn config_dir() -> PathBuf {
    match dirs::config_dir() {
        Some(mut dir) => {
            dir.push(CONFIG_FOLDER);
            dir
        },
        None => {
            log::warn!("config_dir not available on this system. Using executable path.");
            PathBuf::new()
        }
    }
}

/// Loads data directory (eg. $HOME/.local/share/streamduck) or returns the current dir
fn data_dir() -> PathBuf {
    match dirs::data_dir() {
        Some(mut dir) => {
            dir.push(CONFIG_FOLDER);
            dir
        },
        None => {
            log::warn!("data_dir not available on this system. Using executable path.");
            PathBuf::new()
        }
    }
}

/// Struct to keep daemon settings
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Config {
    /// Frame rate
    frame_rate: Option<u32>,
    /// Frequency of checks for disconnected devices
    reconnect_rate: Option<f32>,
    /// Path to device configs
    device_config_path: Option<PathBuf>,
    /// Path to plugins
    plugin_path: Option<PathBuf>,
    /// Path to plugin settings json
    plugin_settings_path: Option<PathBuf>,
    /// Path to fonts
    font_path: Option<PathBuf>,

    /// Config folder
    config_dir: Option<PathBuf>,
    /// Data folder
    data_dir: Option<PathBuf>,

    /// Autosave device configuration
    pub autosave: Option<bool>,

    #[serde(skip)]
    pub plugin_settings: RwLock<HashMap<String, Value>>,

    /// Currently loaded device configs
    #[serde(skip)]
    pub loaded_configs: RwLock<HashMap<String, UniqueDeviceConfig>>,

    /// Currently loaded image collections
    #[serde(skip)]
    pub loaded_images: RwLock<HashMap<String, ImageCollection>>
}

#[allow(dead_code)]
impl Config {
    /// Reads config and retrieves config struct
    pub async fn get(custom_config_path: Option<PathBuf>) -> Config {
        let config_dir = config_dir();
        let data_dir = data_dir();

        let path: PathBuf = custom_config_path.unwrap_or_else(|| {
            let mut dir = config_dir.clone();
            dir.push(CONFIG_FILE);
            dir
        });

        log::info!("Config path: {}", path.display());

        let mut config: Config = match fs::read_to_string(path).await {
            Ok(content) => {
                match toml::from_str(&content) {
                    Ok(config) => config,
                    Err(e) => {
                        log::error!("Config error: {}", e);
                        log::warn!("Using default configuration");
                        Default::default()
                    }
                }
            },
            Err(e) => {
                match e.kind() {
                    std::io::ErrorKind::NotFound => log::warn!("The config file was not found. Did you create the file yet?"),
                    _ => log::warn!("Could not access config file. Error: \"{}\".", e)
                }
                log::warn!("Using default configuration");
                Default::default()
            }
        };

        if config.data_dir == None {
            config.data_dir = Some(data_dir);
        }

        if config.config_dir == None {
            config.config_dir = Some(config_dir);
        }

        config.load_plugin_settings().await;

        log::debug!("config: {:#?}", config);
        config
    }

    /// Pool rate, defaults to [DEFAULT_FRAME_RATE] if not set
    pub fn frame_rate(&self) -> u32 {
        self.frame_rate.unwrap_or(DEFAULT_FRAME_RATE)
    }

    /// Reconnect rate, defaults to [DEFAULT_RECONNECT_TIME] if not set
    pub fn reconnect_rate(&self) -> f32 {
        self.reconnect_rate.unwrap_or(DEFAULT_RECONNECT_TIME)
    }

    pub fn autosave(&self) -> bool {
        self.autosave.unwrap_or(true)
    }


    /// Device config path, defaults to [data_dir]/[DEVICE_CONFIG_FOLDER] or [DEVICE_CONFIG_FOLDER] if not set
    pub fn device_config_path(&self) -> PathBuf {
        self.device_config_path.clone().unwrap_or_else(|| {
                let mut dir = self.data_dir().clone();
                dir.push(DEVICE_CONFIG_FOLDER);
                dir
            }
        )
    }

    /// Plugin folder path, defaults to [config_dir]/[PLUGINS_FOLDER] or [PLUGINS_FOLDER] if not set
    pub fn plugin_path(&self) -> PathBuf {
        self.plugin_path.clone().unwrap_or_else(|| {
                let mut dir = self.config_dir().clone();
                dir.push(PLUGINS_FOLDER);
                dir
            }
        )
    }

    /// Fonts folder path, defaults to [config_dir]/[FONTS_FOLDER] or [FONTS_FOLDER] if not set
    pub fn font_path(&self) -> PathBuf {
        self.font_path.clone().unwrap_or_else(|| {
                let mut dir = self.config_dir().clone();
                dir.push(FONTS_FOLDER);
                dir
            }
        )
    }

    /// Plugin settings file path, defaults to [data_dir]/[PLUGINS_SETTINGS_FILE] or [PLUGINS_SETTINGS_FILE] if not set
    pub fn plugin_settings_path(&self) -> PathBuf {
        self.plugin_settings_path.clone().unwrap_or_else(|| {
                let mut dir = self.data_dir().clone();
                dir.push(PLUGINS_SETTINGS_FILE);
                dir
        })
    }

    /// Data path, defaults to [dirs::data_dir()] if not set
    pub fn data_dir(&self) -> &PathBuf {
        &self.data_dir.as_ref().expect("data_dir not available")
    }

    /// Config path, defaults to [dirs::config_dir()] if not set
    pub fn config_dir(&self) -> &PathBuf {
        &self.config_dir.as_ref().expect("config_dir not available")
    }

    /// Loads plugin settings from file
    pub async fn load_plugin_settings(&self) {
        if let Ok(settings) = fs::read_to_string(self.plugin_settings_path()).await {
            let mut lock = self.plugin_settings.write().await;

            match serde_json::from_str(&settings) {
                Ok(vals) => *lock = vals,
                Err(err) => log::error!("Failed to parse plugin settings: {:?}", err),
            }
        }
    }

    /// Retrieves plugin settings if it exists
    pub async fn get_plugin_settings<T: PluginConfig + DeserializeOwned>(&self) -> Option<T> {
        let lock = self.plugin_settings.read().await;
        Some(serde_json::from_value(lock.get(T::NAME)?.clone()).ok()?)
    }

    /// Sets plugin settings
    pub async fn set_plugin_settings<T: PluginConfig + Serialize>(&self, value: T) {
        let mut lock = self.plugin_settings.write().await;
        lock.insert(T::NAME.to_string(), serde_json::to_value(value).unwrap());
        drop(lock);

        self.write_plugin_settings().await;
    }

    /// Writes plugin settings to file
    pub async fn write_plugin_settings(&self) {
        let lock = self.plugin_settings.read().await;
        if let Err(err) = fs::write(self.plugin_settings_path(), serde_json::to_string(lock.deref()).unwrap()).await {
            log::error!("Failed to write plugin settings: {:?}", err);
        }
    }

    /// Reloads device config for specified serial
    pub async fn reload_device_config(&self, serial: &str) -> Result<(), ConfigError> {
        // Clearing image collection to make sure it's fresh for reload
        self.get_image_collection(serial).await.write().await.clear();

        let mut devices = self.loaded_configs.write().await;

        let mut path = self.device_config_path();
        path.push(format!("{}.json", serial));

        let content = fs::read_to_string(path).await?;
        let device = serde_json::from_str::<DeviceConfig>(&content)?;


        if let Some(device_config) = devices.get(serial) {
            *device_config.write().await = device;
        } else {
            devices.insert(serial.to_string(), Arc::new(RwLock::new(device)));
        }

        self.update_collection(devices.get(serial).unwrap()).await;

        Ok(())
    }

    /// Reloads all device configs
    pub async fn reload_device_configs(&self) -> Result<(), ConfigError> {
        let mut devices = self.loaded_configs.write().await;

        let mut dir = fs::read_dir(self.device_config_path()).await?;

        while let Some(item) = dir.next_entry().await? {
            if item.path().is_file() {
                if let Some(extension) = item.path().extension() {
                    if extension == "json" {
                        let content = fs::read_to_string(item.path()).await?;

                        let device = serde_json::from_str::<DeviceConfig>(&content)?;
                        let serial = device.serial.to_string();

                        // Clearing image collection so it's fresh for reload
                        self.get_image_collection(&device.serial).await.write().await.clear();
                        if let Some(device_config) = devices.get(&serial) {
                            *device_config.write().await = device;
                        } else {
                            devices.insert(serial.to_string(), Arc::new(RwLock::new(device)));
                        }

                        self.update_collection(devices.get(&serial).unwrap()).await;
                    }
                }
            }
        }

        Ok(())
    }

    /// Saves device config for specified serial
    pub async fn save_device_config(&self, serial: &str) -> Result<(), ConfigError> {
        let devices = self.loaded_configs.read().await;

        if let Some(device) = devices.get(serial).cloned() {
            self.update_collection(&device).await;
            let path = self.device_config_path();
            fs::create_dir_all(&path).await.ok();
            self.write_to_filesystem(device).await?;

            Ok(())
        } else {
            Err(ConfigError::DeviceNotFound)
        }
    }

    /// Saves device configs for all serials
    pub async fn save_device_configs(&self) -> Result<(), ConfigError> {
        let devices = self.loaded_configs.read().await;

        let path = self.device_config_path();
        fs::create_dir_all(&path).await.ok();

        for (_, device) in devices.iter() {
            let device = device.clone();
            self.update_collection(&device).await;
            self.write_to_filesystem(device).await?
        }

        Ok(())
    }

    pub async fn write_to_filesystem(&self, device: UniqueDeviceConfig) -> Result<(), ConfigError> {
        let mut path = self.device_config_path();
        let device_conf = device.read().await;
        path.push(format!("{}.json", device_conf.serial));
        fs::write(path, serde_json::to_string(device_conf.deref()).unwrap()).await?;

        drop(device_conf);
        let mut device_conf = device.write().await;
        device_conf.mark_clean();

        Ok(())
    }

    /// Retrieves device config for specified serial
    pub async fn get_device_config(&self, serial: &str) -> Option<UniqueDeviceConfig> {
        self.loaded_configs.read().await.get(serial).cloned()
    }

    /// Sets device config for specified serial
    pub async fn set_device_config(&self, serial: &str, config: DeviceConfig) {
        let mut handle = self.loaded_configs.write().await;

        if let Some(device_config) = handle.get(serial) {
            *device_config.write().await = config;
        } else {
            handle.insert(serial.to_string(), Arc::new(RwLock::new(config)));
        }
    }

    /// Gets an array of all device configs
    pub async fn get_all_device_configs(&self) -> Vec<UniqueDeviceConfig> {
        self.loaded_configs.read().await.values().map(|x| x.clone()).collect()
    }

    /// Disables a device config, so it will not be loaded by default
    pub async fn disable_device_config(&self, serial: &str) -> bool {
        let path = self.device_config_path();

        let mut initial_path = path.clone();
        initial_path.push(format!("{}.json", serial));

        let mut new_path = path.clone();
        new_path.push(format!("{}.json_disabled", serial));

        fs::rename(initial_path, new_path).await.is_ok()
    }

    /// Restores device config if it exists
    pub async fn restore_device_config(&self, serial: &str) -> bool {
        let path = self.device_config_path();

        let mut initial_path = path.clone();
        initial_path.push(format!("{}.json_disabled", serial));

        let mut new_path = path.clone();
        new_path.push(format!("{}.json", serial));

        fs::rename(initial_path, new_path).await.is_ok()
    }

    /// Adds base64 image to device config image collection
    pub async fn add_image(&self, serial: &str, image: String) -> Option<String> {
        if let Some(config) = self.get_device_config(serial).await {
            let mut config_handle = config.write().await;
            let identifier = hash_str(&image);

            if let Ok(image) = SDImage::from_base64(&image, config_handle.kind().image_size()).await {
                config_handle.images.insert(identifier.clone(), image.into());
                drop(config_handle);

                self.update_collection(&config).await;
                Some(identifier)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Encodes image to base64 and adds it to device config image collection
    pub async fn add_image_encode(&self, serial: &str, image: DynamicImage) -> Option<String> {
        if let Some(config) = self.get_device_config(serial).await {
            let mut config_handle = config.write().await;
            let serialized_image = SDImage::SingleImage(resize_for_streamdeck(config_handle.kind().image_size(), image)).into();
            let identifier = hash_image(&serialized_image);
            config_handle.images.insert(identifier.clone(), serialized_image);
            drop(config_handle);

            self.update_collection(&config).await;
            return Some(identifier);
        }

        None
    }

    /// Gets images from device config
    pub async fn get_images(&self, serial: &str) -> Option<HashMap<String, SDSerializedImage>> {
        if let Some(config) = self.get_device_config(serial).await {
            let config_handle = config.read().await;
            Some(config_handle.images.clone())
        } else {
            None
        }
    }

    /// Removes image from device config
    pub async fn remove_image(&self, serial: &str, identifier: &str) -> bool {
        if let Some(config) = self.get_device_config(serial).await {
            let mut config_handle = config.write().await;
            config_handle.images.remove(identifier);
            drop(config_handle);

            self.remove_from_collection(serial, identifier).await;
            true
        } else {
            false
        }
    }

    /// Syncs images with core
    pub async fn sync_images(&self, serial: &str) {
        if let Some(config) = self.get_device_config(serial).await {
            self.update_collection(&config).await;
        }
    }

    /// Retrieves image collection for device if device exists
    pub async fn get_image_collection(&self, serial: &str) -> ImageCollection {
        let mut handle = self.loaded_images.write().await;

        if let Some(collection) = handle.get(serial) {
            collection.clone()
        } else {
            let collection: ImageCollection = Default::default();
            handle.insert(serial.to_string(), collection.clone());
            collection
        }
    }

    /// For making sure image collections strictly follow device config
    async fn update_collection(&self, device_config: &UniqueDeviceConfig) {
        let mut device_config = device_config.write().await;
        let mut handle = self.loaded_images.write().await;

        if let Some(collection) = handle.get_mut(&device_config.serial) {
            let mut collection_handle = collection.write().await;

            // Adding missing images from device config
            for (key, image) in &device_config.images {
                if !collection_handle.contains_key(key) {
                    if let Ok(image) = image.try_into() {
                        collection_handle.insert(key.to_string(), image);
                    }
                }
            }

            // Adding any images in collection to device config
            for (key, image) in collection_handle.iter() {
                if !device_config.images.contains_key(key) {
                    device_config.images.insert(key.to_string(), image.into());
                }
            }
        }
    }

    /// For removing images from image collections
    async fn remove_from_collection(&self, serial: &str, identifier: &str) {
        let mut handle = self.loaded_images.write().await;

        if let Some(collection) = handle.get_mut(serial) {
            let mut collection_handle = collection.write().await;
            collection_handle.remove(identifier);
        }
    }
}

/// Plugin Config trait for serialization and deserialization methods
pub trait PluginConfig {
    const NAME: &'static str;
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
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DeviceConfig {
    pub vid: u16,
    pub pid: u16,
    pub serial: String,
    pub brightness: u8,
    pub layout: RawButtonPanel,
    pub images: HashMap<String, SDSerializedImage>,
    pub plugin_data: HashMap<String, Value>,
    #[serde(skip)]
    pub commit_time: Option<Instant>,
    #[serde(skip)]
    pub dirty_state: bool
}

impl DeviceConfig {
    /// Gets kind of the device
    pub fn kind(&self) -> Kind {
        match self.pid {
            streamdeck::pids::ORIGINAL_V2 => Kind::OriginalV2,
            streamdeck::pids::MINI => Kind::Mini,
            streamdeck::pids::MK2 => Kind::Mk2,
            streamdeck::pids::XL => Kind::Xl,

            _ => Kind::Original,
        }
    }

    /// check if there are config changes
    pub fn is_dirty(&self) -> bool {
        self.dirty_state
    }

    /// remove dirty state
    pub fn mark_clean(&mut self) {
        self.dirty_state = false
    }

    /// duration from now to the last commit
    pub fn commit_duration(&self) -> Duration {
        Instant::now().duration_since(self.commit_time.unwrap_or(Instant::now()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn config_sys_config_dir() {
        // check if config dir gets created
        let config = Config::get(None).await;
        assert_ne!(config.config_dir, None)
    }

    #[tokio::test]
    async fn config_sys_data_dir() {
        // check if data dir gets created
        let config = Config::get(None).await;
        assert_ne!(config.data_dir, None)
    }

    #[tokio::test]
    async fn config_mark_clean() {
        // simulate a changed config
        let mut device_conf = DeviceConfig {
            vid: Default::default(),
            pid: Default::default(),
            serial: String::from("TestSerial1"),
            brightness: Default::default(),
            layout: Default::default(),
            images: Default::default(),
            plugin_data: Default::default(),
            commit_time: Default::default(),
            dirty_state: true
        };
        assert_eq!(device_conf.dirty_state, true);
        device_conf.mark_clean();
        assert_eq!(device_conf.dirty_state, false);
    }

    #[tokio::test]
    async fn config_filesystem_writing() { 
        let config = Config::get(None).await;
        // simulate a changed config
        let device_conf = DeviceConfig {
            vid: Default::default(),
            pid: Default::default(),
            serial: String::from("TestSerial1"),
            brightness: Default::default(),
            layout: Default::default(),
            images: Default::default(),
            plugin_data: Default::default(),
            commit_time: Default::default(),
            dirty_state: true
        };
        let serial = device_conf.serial.clone();

        // get the path
        let mut path = config.device_config_path();
        fs::create_dir_all(&path).await.ok();
        path.push(format!("{}.json", serial));

        // delete device data if it exists (clean start)
        if path.exists() {
            std::fs::remove_file(&path).unwrap();
        }

        // is the device config dirty?
        assert_eq!(device_conf.dirty_state, true);
        let device_conf = Arc::new(RwLock::new(device_conf));

        // write to the filesystem
        config.write_to_filesystem(device_conf).await.unwrap();

        // does the path exist?
        assert_eq!(path.exists(), true);

        // clean up
        std::fs::remove_file(&path).unwrap();

        // TODO: check if dirty_state is updated

    }
}
