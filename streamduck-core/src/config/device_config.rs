use serde::{Serialize, Deserialize};
use std::{
    time::Instant,
    path::PathBuf,
    collections::HashMap,
    sync::Arc
};
use tokio::sync::RwLock;

/// write the device configurations from a path into a HashMap
pub async fn retrieve_device_configs(_path: &PathBuf) -> Result<HashMap<String, Arc<RwLock<DeviceConfig>>>, std::io::Error> {
    todo!()
}

/// Keys for the device config.
/// This is the information that gets saved for every device.
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
