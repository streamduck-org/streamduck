use std::sync::Arc;
use async_trait::async_trait;
use crate::devices::buttons::ButtonPosition;
use crate::devices::images::{DeviceImageCache, DeviceImageData};

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
pub trait Device: DeviceImageCache {
    /// Metadata associated with the device
    fn metadata(&self) -> DeviceMetadata;

    /// For the device to use to poll for button state and such
    async fn poll(&self);

    /// Clears device's screen
    async fn clear_screen(&self);

    /// Sets brightness of the screen
    async fn set_brightness(&self, brightness: u8);

    /// Sets image to device's screen on specified position
    async fn set_button_image(&self, position: ButtonPosition, image: &dyn DeviceImageData);
}

/// Device interface contained in a reference counter
pub type SharedDevice = Arc<dyn Device>;