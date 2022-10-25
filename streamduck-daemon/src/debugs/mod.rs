//! Just some types so i can test things

use std::sync::Arc;
use streamduck_core::devices::drivers::{Driver, DriverError, DriverManager};
use streamduck_core::devices::metadata::{ButtonLayout, DeviceMetadata};
use streamduck_core::devices::SharedDevice;

use streamduck_core::async_trait;

pub struct TestDriver {

}

#[async_trait]
impl Driver for TestDriver {
    fn name(&self) -> String {
        "test_driver".to_string()
    }

    async fn list_devices(&self) -> Vec<DeviceMetadata> {
        vec![DeviceMetadata {
            driver_name: self.name(),
            serial_number: "some_bs".to_string(),
            has_screen: true,
            resolution: (16, 16),
            layout: ButtonLayout(vec![5, 5, 5])
        }]
    }

    async fn connect_device(&self, serial_number: &str) -> Result<SharedDevice, DriverError> {
        todo!()
    }
}

pub async fn test_driver(driver_manager: &Arc<DriverManager>) {
    driver_manager.register_driver(Arc::new(TestDriver {})).await;
}