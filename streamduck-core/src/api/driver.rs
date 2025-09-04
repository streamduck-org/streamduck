use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use crate::api::device::BoxDevice;
use crate::api::id::NamespacedID;

/// Serves as unique identifier for a specific device,
/// first value is preferably serial number,
/// second value is description/type of the device
/// third value is for optional identifiers
#[derive(Debug, Clone, Hash, Serialize, Deserialize, Eq, PartialEq)]
pub struct DeviceIdentifier(String, String, Value);

impl DeviceIdentifier {
    pub fn namespaced(self, id: NamespacedID) -> NamespacedDeviceIdentifier {
        NamespacedDeviceIdentifier {
            id: self,
            driver: id
        }
    }
}

impl Display for DeviceIdentifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.0, self.1)
    }
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize, Eq, PartialEq)]
pub struct DriverspacedDeviceIdentifier {
    pub id: DeviceIdentifier,
    pub driver: String
}

impl DriverspacedDeviceIdentifier {
    pub fn namespaced(self, plugin: String) -> NamespacedDeviceIdentifier {
        NamespacedDeviceIdentifier {
            id: self.id,
            driver: NamespacedID::new(self.driver, plugin)
        }
    }
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize, Eq, PartialEq)]
pub struct NamespacedDeviceIdentifier {
    pub id: DeviceIdentifier,
    pub driver: NamespacedID
}

impl NamespacedDeviceIdentifier {
    pub fn new(id: DeviceIdentifier, driver: NamespacedID) -> NamespacedDeviceIdentifier {
        Self {
            id, driver
        }
    }
}

impl Display for NamespacedDeviceIdentifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.id, self.driver)
    }
}

#[derive(Debug, Error)]
pub enum DeviceListError {
    #[error(transparent)]
    LuaError(#[from] mlua::Error),
    #[error(transparent)]
    OtherError(#[from] anyhow::Error)
}

#[derive(Debug, Error)]
pub enum DeviceConnectionError {
    #[error("driver capable to connect to the device wasn't found")]
    DriverNotFound,
    #[error("device wasn't found while trying to connect")]
    DeviceNotFound,
    #[error(transparent)]
    LuaError(#[from] mlua::Error),
    #[error(transparent)]
    OtherError(#[from] anyhow::Error)
}

pub trait Driver {
    fn name(&self) -> String;
    fn list_devices(&self) -> Result<Vec<DeviceIdentifier>, DeviceListError>;
    fn connect_device(&self, id: &DeviceIdentifier) -> Result<BoxDevice, DeviceConnectionError>;
}

pub type BoxDriver = Box<dyn Driver>;
pub type RefDriver<'a> = &'a dyn Driver;