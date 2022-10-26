use std::sync::Arc;
use std::time::Instant;
use serde::Deserialize;

use crate::tests::lib::{start_benchmark, DataPoint};

use async_trait::async_trait;

use crate::devices::drivers::{Driver, DriverError, DriverManager};
use crate::devices::metadata::{ButtonLayout, DeviceMetadata};
use crate::devices::SharedDevice;

pub struct TestDriver {}

#[async_trait]
impl Driver for TestDriver {
    fn name(&self) -> String {
        "test_driver".to_string()
    }

    async fn list_devices(&self) -> Vec<DeviceMetadata> {
        vec![DeviceMetadata {
            driver_name: self.name(),
            serial_number: "test_serial".to_string(),
            has_screen: true,
            resolution: (16, 16),
            layout: ButtonLayout(vec![5, 5, 5])
        }]
    }

    async fn connect_device(&self, _: &str) -> Result<SharedDevice, DriverError> {
        todo!()
    }
}

#[tokio::test]
async fn test_driver_device_list() {
    let bench = start_benchmark(Some(DataPoint::DriverDeviceList));

    let driver_manager = DriverManager::new();

    driver_manager.register_driver(Arc::new(TestDriver {})).await;

    let list = driver_manager.list_devices().await;

    bench.stop();

    assert_eq!(
        list
            .iter()
            .position(|x| x.serial_number == "test_serial")
            .is_some(),
        true
    )
}