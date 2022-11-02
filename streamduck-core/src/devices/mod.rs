use std::sync::Arc;
use async_trait::async_trait;
use image::DynamicImage;
use crate::devices::buttons::ButtonPosition;

use crate::devices::metadata::DeviceMetadata;

/// Device drivers
pub mod drivers;
/// Device metadata
pub mod metadata;
/// Button structures
pub mod buttons;

/// Device interface
#[async_trait]
pub trait Device {
    /// Metadata associated with the device
    fn metadata(&self) -> DeviceMetadata;

    /// For the device to use to poll for button state and such
    async fn poll(&self);

    /// Resets device to initial state
    async fn reset(&self);

    /// Clears device's screen
    async fn clear_screen(&self);

    /// Sets brightness of the screen
    async fn set_brightness(&self, brightness: u8);

    /// Checks if image with specified key exists
    async fn contains_image(&self, key: u128) -> bool;

    /// Adds image to the device's cache
    async fn add_image(&self, key: u128, image: DynamicImage);

    /// Removes image from the device's cache
    async fn remove_image(&self, key: u128) -> bool;

    /// Sets image under specified key to device's screen on specified position
    async fn set_button_image(&self, position: ButtonPosition, key: u128);
}

/// Device interface contained in a reference counter
pub type SharedDevice = Arc<dyn Device>;