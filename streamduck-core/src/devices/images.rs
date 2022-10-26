use async_trait::async_trait;
use image::DynamicImage;

/// Device should be able to manage its image cache
#[async_trait]
pub trait DeviceImageCache {
    /// Gets image saved on the device
    async fn get_image(&self, key: u128) -> Option<&dyn DeviceImageData>;

    /// Adds image to the device
    async fn add_image(&self, key: u128, image: DynamicImage);

    /// Removes image from the device
    async fn remove_image(&self, key: u128) -> bool;
}

/// Trait that device format images should implement
pub trait DeviceImageData {}