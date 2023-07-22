mod actions;
mod overlays;

pub use actions::*;
pub use overlays::*;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::config::device::DeviceConfig;
use crate::config::SharedConfig;
use crate::device::Device;

/// Shared reference to Core
pub type SharedCore = Arc<Core>;

/// The thing that manages a particular device
pub struct Core {
    config: SharedConfig,

    /// If the core is still alive
    pub active: AtomicBool,

    /// Device that's being managed by the core
    pub device: Arc<Device>,

}

impl Core {
    /// Unwraps the device and its config into a Streamduck core
    pub async fn init_core(config: SharedConfig, device: Arc<Device>, _device_config: DeviceConfig) -> SharedCore {
        Arc::new(Core {
            config,
            active: AtomicBool::new(true),
            device,
        })
    }

    /// Marks the core as dead, so it can be cleaned up by device task
    pub fn mark_as_dead(&self) {
        self.active.store(false, Ordering::Release);
    }

    /// Saves all the data belonging to the device
    pub async fn save_data(&self) {
        let device_config = DeviceConfig {
            device_data: self.device.options.data.read().await.clone(),
        };

        println!("saving device config");

        self.config.load().write_device_config(
            self.device.identifier.clone(),
            device_config
        ).await;
    }

    /// Kills the core, making it save and drop the device
    pub async fn die(&self) {
        self.active.store(false, Ordering::Release);

        self.save_data().await;
        self.device.disconnect();
    }
}