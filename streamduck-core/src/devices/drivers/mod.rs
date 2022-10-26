use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use futures::future::join_all;
use tokio::sync::RwLock;

use crate::devices::metadata::DeviceMetadata;
use crate::devices::SharedDevice;

/// Driver interface
#[async_trait]
pub trait Driver: Send + Sync {
    /// Name of the driver
    fn name(&self) -> String;

    /// List devices that can be detected by this driver
    async fn list_devices(&self) -> Vec<DeviceMetadata>;

    /// Connect to the specified device
    async fn connect_device(&self, serial_number: &str) -> Result<SharedDevice, DriverError>;
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
    /// Any other error
    Other(String),
}

/// Driver interface contained in a reference counter
pub type SharedDriver = Arc<dyn Driver>;



/// Driver manager
pub struct DriverManager {
    /// Drivers that were registered in the manager
    drivers: RwLock<HashMap<String, SharedDriver>>
}

impl DriverManager {
    /// Creates a new instance of the driver manager
    pub fn new() -> Arc<DriverManager> {
        Arc::new(DriverManager {
            drivers: Default::default()
        })
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
            .map(|x| async move { x.list_devices().await });

        join_all(lists).await.into_iter()
            .flatten()
            .collect()
    }

    /// Connects to a device using specified driver
    pub async fn connect_device(&self, driver_name: &str, serial_number: &str) -> Result<SharedDevice, DriverError> {
        let lock = self.drivers.read().await;

        if let Some(driver) = lock.get(driver_name).cloned() {
            drop(lock); // Who knows what the driver might do

            driver.connect_device(serial_number).await
        } else {
            Err(DriverError::NoSuchDriver)
        }
    }
}

