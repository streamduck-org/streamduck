use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use async_trait::async_trait;
use hidapi::HidError;
use image::{DynamicImage, ImageError};
use crate::devices::buttons::ButtonPosition;

use crate::devices::metadata::DeviceMetadata;
use crate::EventDispatcher;

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

    /// For the device to use to poll for button state and such, dispatcher is provided that device should use
    async fn poll(&self, dispatcher: &Arc<EventDispatcher>) -> Result<(), DeviceError>;

    /// Resets device to initial state
    async fn reset(&self) -> Result<(), DeviceError>;

    /// Clears device's screen
    async fn clear_screen(&self) -> Result<(), DeviceError>;

    /// Sets brightness of the screen
    async fn set_brightness(&self, brightness: u8) -> Result<(), DeviceError>;

    /// Checks if image with specified key exists
    async fn contains_image(&self, key: u128) -> bool;

    /// Adds image to the device's cache
    async fn add_image(&self, key: u128, image: DynamicImage) -> Result<(), DeviceError>;

    /// Removes image from the device's cache
    async fn remove_image(&self, key: u128) -> bool;

    /// Clears image from button on specified position
    async fn clear_button_image(&self, position: ButtonPosition) -> Result<(), DeviceError>;
    
    /// Sets image under specified key to device's screen on specified position
    async fn set_button_image(&self, position: ButtonPosition, key: u128) -> Result<(), DeviceError>;
}

/// Device interface contained in a reference counter
pub type SharedDevice = Arc<dyn Device + Send + Sync>;

/// Different errors for devices to report
#[derive(Debug)]
pub enum DeviceError {
    /// Connection to the device is lost
    DeviceDisconnected,
    /// Image is missing from cache
    ImageMissing,
    /// Device is being used in wrong ways
    InvalidUse(String),
    /// Error related to HidApi
    HidError(HidError),
    /// Image error
    ImageError(ImageError),
    /// Any other error
    Other(Box<dyn Error>)
}

impl From<HidError> for DeviceError {
    fn from(e: HidError) -> Self {
        Self::HidError(e)
    }
}

impl From<ImageError> for DeviceError {
    fn from(e: ImageError) -> Self {
        Self::ImageError(e)
    }
}

impl Display for DeviceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for DeviceError {}