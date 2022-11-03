use tracing::{
    warn,
    error,
    info,
};
use tokio::fs;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use toml;

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

/// Keys for the global config.
/// For more description of the keys usage see: <https://docs.streamduck.org/daemon/configuration.html>.
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct GlobalConfig {
    /// Path to device config folder
    pub device_config_path: Option<PathBuf>,
    /// Path to plugins folder
    pub plugin_path: Option<PathBuf>,
    /// Path to plugin settings json
    pub plugin_settings_path: Option<PathBuf>,
    /// Path to fonts
    pub font_path: Option<PathBuf>,

    /// Autosave device configuration
    pub autosave: Option<bool>,

    /// If plugin compatibility checks should be performed
    pub plugin_compatibility_checks: Option<bool>,
}
