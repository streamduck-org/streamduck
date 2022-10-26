use std::future::Future;
use async_trait::async_trait;
use image::DynamicImage;

/// Device should be able to manage its images
#[async_trait]
pub trait DeviceImageMethods {
    /// Gets image saved on the device
    async fn get_image(&self, key: &str) -> Option<&dyn DeviceImage>;

    /// Adds image to the device
    async fn add_image(&self, key: &str, image: DynamicImage);
}

/// Trait that device format images should implement
pub trait DeviceImage {}