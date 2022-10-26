use std::sync::Arc;
use async_trait::async_trait;

use crate::devices::metadata::DeviceMetadata;

/// Device drivers
pub mod drivers;
/// Device metadata
pub mod metadata;
/// Button structures
pub mod buttons;
/// Image device types
pub mod images;

/// Device interface
#[async_trait]
pub trait Device {
    /// Metadata associated with the device
    fn metadata(&self) -> DeviceMetadata;

    /// For the device to use to poll for button state and such
    async fn poll(&self);
}

/// Device interface contained in a reference counter
pub type SharedDevice = Arc<dyn Device>;