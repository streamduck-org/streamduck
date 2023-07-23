use async_trait::async_trait;
use std::sync::{Arc, Weak};
use rmpv::Value;
use crate::config::device::DeviceConfig;
use crate::config::SharedConfig;
use crate::core::Core;
use crate::data::{NamespacedName, Options};
use crate::device::{Device, DeviceError, DriverConnection};
use crate::device::metadata::{DeviceIdentifier, DeviceMetadata, PartialIdentifier, PartialMetadata};
use crate::plugin::Plugin;

/// Driver Arc
pub type SharedDriver = Arc<Driver>;

/// Weak shared reference to a driver
pub type WeakDriver = Weak<Driver>;

/// Device driver
pub struct Driver {
    pub(crate) config: SharedConfig,

    /// Plugin that the driver originated from
    pub original_plugin: Weak<Plugin>,

    /// Name of the driver
    pub name: NamespacedName,

    /// Any options that the driver might have
    pub options: Options,

    /// Actual implementation of the driver
    pub implement: Arc<dyn DriverImpl>,
}

impl Driver {
    /// Lists all devices that can be found by this driver
    pub async fn list_devices(&self) -> Vec<DeviceIdentifier> {
        self.implement.list_devices(&self.options).await.into_iter()
            .map(|partial| partial.upgrade(&self.name))
            .collect()
    }

    /// Lists all devices that can be found by this driver
    pub async fn describe_device(&self, device: &DeviceIdentifier) -> DeviceMetadata {
        self.implement.describe_device(&self.options, device).await.upgrade(&self.name)
    }

    /// Connects to the device and returns a [SharedDevice]
    pub async fn connect_device(&self, device: &DeviceIdentifier) -> Result<Arc<Device>, DeviceError> {
        let connection = self.implement.connect_device(&self.options, device).await?;

        let config;

        if let Some(device_config) = self.config.load().get_device_config(device).await {
            config = device_config;
        } else {
            let device_config = DeviceConfig::from_device_data(
                self.implement.default_device_data(&self.options, device).await
            );

            config = device_config;
            self.config.load().write_device_config(device.clone(), config.clone()).await;
        }

        Ok(connection.upgrade(
            self.config.clone(),
            device.clone(),
            self.original_plugin.clone(),
            config.device_data
        ))
    }
}

/// Implementation of a driver
#[async_trait]
pub trait DriverImpl: Send + Sync  {
    /// Called when driver options have been changed. Updated options are given along with new data separately
    async fn options_changed(&self, options: &Options, new_data: Value);

    /// Global tick
    async fn tick(&self, options: &Options, core: Arc<Core>);

    /// Should return a list of devices that could be connected to
    async fn list_devices(&self, options: &Options) -> Vec<PartialIdentifier>;

    /// Describes input layout of the device
    async fn describe_device(&self, options: &Options, device: &DeviceIdentifier) -> PartialMetadata;

    /// Default device data that should be given to the device
    async fn default_device_data(&self, options: &Options, device: &DeviceIdentifier) -> Value;

    /// Initiates connection to the device
    async fn connect_device(&self, options: &Options, device: &DeviceIdentifier) -> Result<DriverConnection, DeviceError>;
}
