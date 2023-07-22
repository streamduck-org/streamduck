use rmpv::Value;
use serde::{Deserialize, Serialize};

/// Config related to device
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DeviceConfig {
    /// Device config data
    pub device_data: Value
}

impl Default for DeviceConfig {
    fn default() -> Self {
        Self {
            device_data: Value::Nil,
        }
    }
}

impl DeviceConfig {
    /// Creates device config from device data
    pub fn from_device_data(data: Value) -> DeviceConfig {
        let mut default = Self::default();
        default.device_data = data;
        default
    }
}