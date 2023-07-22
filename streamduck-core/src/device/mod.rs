use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::{Arc, Weak};
use async_trait::async_trait;
use image::DynamicImage;
use rmpv::Value;
use tokio::sync::RwLock;
use crate::data::Options;

/// Types related to input from devices
pub mod input;
/// Device metadata
pub mod metadata;
/// Device drivers
pub mod driver;

pub use driver::{Driver, DriverImpl};
pub use metadata::{DeviceIdentifier, DeviceMetadata};
use crate::config::SharedConfig;
use crate::plugin::{Plugin, WeakPlugin};
use crate::ui::UISchema;

/// Device Arc
pub type SharedDevice = Arc<Device>;

/// Connected device
pub struct Device {
    pub(crate) config: SharedConfig,

    /// Plugin that the device originated from
    pub original_plugin: Weak<Plugin>,

    /// Device identifier
    pub identifier: DeviceIdentifier,

    /// Options that the device might have
    pub options: Options,

    /// Actual implementation of the device
    pub implement: Arc<dyn DeviceImpl>,
}

/// Device that was just connected to by the driver
pub struct DriverConnection {
    /// UI Schema that the device should have
    pub ui: UISchema,
    /// Implementation for the device
    pub implement: Arc<dyn DeviceImpl>,
}

impl DriverConnection {
    /// Upgrades connection to device struct
    pub(crate) fn upgrade(self, config: SharedConfig, identifier: DeviceIdentifier, plugin: WeakPlugin, device_data: Value) -> SharedDevice {
        Arc::new(Device {
            config,
            original_plugin: plugin,
            identifier,
            options: Options {
                data: RwLock::new(device_data),
                ui: self.ui,
            },
            implement: self.implement,
        })
    }
}

impl Device {
    /// Asks the device instance to disconnect
    pub async fn disconnect(&self) {
        self.implement.disconnect().await;
    }
}

/// Implemenation of a device
#[async_trait]
pub trait DeviceImpl: Send + Sync + ImageOps {
    /// Called when device options have been changed. Updated options are given along with new data separately
    async fn options_changed(&self, options: &Options, new_data: Value);

    /// Polls the device in case the device needs to check the state, read from the device and so on
    async fn poll_device(&self, options: &Options) -> Result<(), DeviceError>;

    /// Called when device is being forcefully disconnected
    async fn disconnect(&self);
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
#[derive(Debug)]
pub enum DeviceError {
    /// Device has been disconnected, should be used if that's the case, otherwise Streamduck may take the error more seriously
    LostConnection,

    /// Returned if Streamduck couldn't find the driver for this device anymore
    DriverNoLongerExists,

    /// Any other error that could had happened with the device, will be alerted to the user
    DeviceError(Box<dyn Error>)
}

impl Display for DeviceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for DeviceError {}