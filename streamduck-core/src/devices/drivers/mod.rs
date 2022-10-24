use std::sync::Arc;
use crate::devices::metadata::DeviceMetadata;
use crate::devices::SharedDevice;

use async_trait::async_trait;

/// Driver interface
#[async_trait]
pub trait Driver {
    /// List devices that can be detected by this driver
    async fn list_devices(&self) -> Vec<DeviceMetadata>;

    /// Connect to the specified device
    async fn connect_device(&self, serial_number: &str) -> SharedDevice;
}

/// Driver interface contained in a reference counter
pub type SharedDriver = Arc<dyn Driver>;