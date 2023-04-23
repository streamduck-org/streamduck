use std::error::Error;
use std::sync::Arc;
use async_trait::async_trait;
use image::DynamicImage;
use rmpv::Value;
use crate::data::Options;

/// Types related to input from devices
pub mod input;

mod metadata;
mod driver;

pub use driver::{Driver, DriverImpl};
pub use metadata::{DeviceIdentifier, DeviceMetadata};

/// Connected device
pub struct Device {
    /// Device identifier
    pub identifier: DeviceIdentifier,

    /// Options that the device might have
    pub options: Options,

    /// Actual implementation of the device
    pub implement: Box<dyn DeviceImpl>,
}

/// Implemenation of a device
#[async_trait]
pub trait DeviceImpl: ImageOps {
    /// Polls the device in case the device needs to check the state, read from the device and so on
    async fn poll_device(&self, options: &Options) -> Result<(), DeviceError>;
}

/// Image operations associated with the devices
#[async_trait]
pub trait ImageOps {
    /// Checks if device cache still has the image under the hash key
    async fn hash_exists(&self, options: &Options, key: u128) -> Result<bool, DeviceError>;

    /// Should set image under the hash key onto specified input
    async fn set_image(&self, options: &Options, input: u16, key: u128) -> Result<bool, DeviceError>;

    /// Should clear image of specified input
    async fn clear_image(&self, options: &Options, input: u16) -> Result<(), DeviceError>;

    /// Should convert and save the image to the device
    async fn upload_image(&self, options: &Options, key: u128, image: DynamicImage) -> Result<(), DeviceError>;
}

/// Issues that the device may encounter
pub enum DeviceError {
    /// Device has been disconnected, should be used if that's the case, otherwise Streamduck may take the error more seriously
    LostConnection,

    /// Any other error that could had happened with the device, will be alerted to the user
    DeviceError(Box<dyn Error>)
}