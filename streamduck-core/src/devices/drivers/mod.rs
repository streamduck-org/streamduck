use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use async_trait::async_trait;
use futures::future::join_all;
use hidapi::{HidApi, HidError};
use tokio::sync::RwLock;

use crate::devices::metadata::DeviceMetadata;
use crate::devices::SharedDevice;
use tracing::instrument;

/// Driver interface
#[async_trait]
pub trait Driver: Send + Sync {
    /// Name of the driver
    fn name(&self) -> String;

    /// List devices that can be detected by this driver
    async fn list_devices(&self, hidapi: &HidApi) -> Vec<DeviceMetadata>;

    /// Connect to the specified device
    async fn connect_device(&self, hidapi: &HidApi, identifier: String) -> Result<SharedDevice, DriverError>;
}

/// All possible errors with device drivers
#[derive(Debug)]
pub enum DriverError {
    /// Device wasn't found with the driver
    DeviceNotFound,
    /// Failed to connect to the device
    FailedToConnect(String),
    /// If specified driver wasn't found
    NoSuchDriver,
    /// Identifier failed to be parsed
    InvalidIdentifier,
    /// Any other error
    Other(Box<dyn Error>),
}

impl Display for DriverError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for DriverError {}

/// Driver interface contained in a reference counter
pub type SharedDriver = Arc<dyn Driver>;



/// Driver manager
pub struct DriverManager {
    /// Drivers that were registered in the manager
    drivers: RwLock<HashMap<String, SharedDriver>>,
    /// Most drivers will probably use HidApi anyway, there can only be one instance of it
    hidapi: HidApi,
}

impl DriverManager {
    /// Creates a new instance of the driver manager
    pub fn new() -> Result<Arc<DriverManager>, HidError> {
        Ok(Arc::new(DriverManager {
            drivers: Default::default(),
            hidapi: HidApi::new()?
        }))
    }

    /// Registers a new driver
    pub async fn register_driver(&self, driver: SharedDriver) {
        let mut lock = self.drivers.write().await;
        lock.insert(driver.name(), driver);
    }

    /// Gets list of registered drivers
    pub async fn get_drivers(&self) -> Vec<SharedDriver> {
        self.drivers.read().await.values().cloned().collect()
    }

    /// Lists all found devices by registered drivers
    pub async fn list_devices(&self) -> Vec<DeviceMetadata> {
        let lists = self.get_drivers().await
            .into_iter()
            .map(|x| async move { x.list_devices(&self.hidapi).await });

        join_all(lists).await.into_iter()
            .flatten()
            .collect()
    }

    /// Connects to a device using specified driver
    pub async fn connect_device(&self, driver_name: &str, identifier: &str) -> Result<SharedDevice, DriverError> {
        let lock = self.drivers.read().await;

        if let Some(driver) = lock.get(driver_name).cloned() {
            drop(lock); // Who knows what the driver might do

            driver.connect_device(&self.hidapi, identifier.to_string()).await
        } else {
            Err(DriverError::NoSuchDriver)
        }
    }
}

