use async_trait::async_trait;
use std::sync::{Arc, Weak};
use rmpv::Value;
use crate::data::Options;
use crate::device::Device;
use crate::device::metadata::{DeviceIdentifier, DeviceMetadata};
use crate::plugin::Plugin;

/// Device driver
pub struct Driver {
    /// Plugin that the driver originated from
    pub original_plugin: Weak<Plugin>,

    /// Name of the driver
    pub name: String,

    /// Any options that the driver might have
    pub options: Options,

    /// Actual implementation of the driver
    pub implement: Box<dyn DriverImpl>,
}

/// Implementation of a driver
#[async_trait]
pub trait DriverImpl {
    /// Called when driver options have been changed. Updated options are given along with new data separately
    async fn options_changed(&self, options: &Options, new_data: Value);

    /// Should return a list of devices that could be connected to
    async fn list_devices(&self, options: &Options) -> Vec<DeviceIdentifier>;

    /// Describes input layout of the device
    async fn describe_device(&self, options: &Options, device: &DeviceIdentifier) -> DeviceMetadata;

    /// Initiates connection to the device
    async fn connect_device(&self, options: &Options, device: &DeviceIdentifier) -> Arc<Device>;
}
