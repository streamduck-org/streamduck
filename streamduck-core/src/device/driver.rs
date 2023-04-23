use async_trait::async_trait;
use std::sync::Arc;
use crate::data::Options;
use crate::device::Device;
use crate::device::metadata::{DeviceIdentifier, DeviceMetadata};

/// Device driver
pub struct Driver {
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
    /// Should return a list of devices that could be connected to
    async fn list_devices(&self, options: &Options) -> Vec<DeviceIdentifier>;

    /// Describes input layout of the device
    async fn describe_device(&self, options: &Options, device: &DeviceIdentifier) -> DeviceMetadata;

    /// Initiates connection to the device
    async fn connect_device(&self, options: &Options, device: &DeviceIdentifier) -> Arc<Device>;
}
