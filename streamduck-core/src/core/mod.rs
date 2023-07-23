mod actions;
mod overlays;

pub use actions::*;
pub use overlays::*;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::time::Duration;
use rmpv::Value;
use tracing::{error, info, trace};
use crate::config::device::DeviceConfig;
use crate::config::SharedConfig;
use crate::device::Device;
use crate::event::dispatcher::{EventDispatcher, SharedEventDispatcher};
use crate::event::Event;

/// Shared reference to Core
pub type SharedCore = Arc<Core>;

static TRACK_COUNT: AtomicU16 = AtomicU16::new(0);

/// Event that the core will see after dying
pub const DEATH_EVENT_NAME: &str = "death";

/// The thing that manages a particular device
pub struct Core {
    config: SharedConfig,

    /// If the core is still alive
    pub active: AtomicBool,

    /// Device that's being managed by the core
    pub device: Arc<Device>,

    /// Core event dispatcher
    pub dispatcher: SharedEventDispatcher,
}

impl Core {
    /// Unwraps the device and its config into a Streamduck core
    pub async fn init_core(config: SharedConfig, device: Arc<Device>, _device_config: DeviceConfig) -> SharedCore {
        let dispatcher = EventDispatcher::new();

        let subscriber = dispatcher.subscribe().await;

        let core = Arc::new(Core {
            config,
            active: AtomicBool::new(true),
            device,
            dispatcher,
        });

        let core_copy = core.clone();
        tokio::task::spawn(async move {
            while core_copy.active.load(Ordering::Acquire) {
                let event = subscriber.get().await;

                let iden = &core_copy.device.identifier;
                trace!(%iden, ?event)
            }

            trace!("listener dead");
        });

        core
    }

    /// Polls the device, core dies if any error is experienced
    pub async fn poll_device(self: &Arc<Core>) {
        if self.device.busy.load(Ordering::Acquire) {
            return;
        }

        self.device.busy.store(true, Ordering::Release);

        if let Err(e) = self.device.poll(self.clone()).await {
            error!(?e, "Error while polling the device");
            self.mark_as_dead();
        }
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

        info!(iden = %self.device.identifier, "Saving device data");

        self.config.load().write_device_config(
            self.device.identifier.clone(),
            device_config
        ).await;
    }

    /// Kills the core, making it save and drop the device
    pub async fn die(&self) {
        self.active.store(false, Ordering::Release);

        self.save_data().await;
        self.device.disconnect().await;

        self.dispatcher.send(Event {
            name: DEATH_EVENT_NAME.to_string(),
            payload: Value::Nil,
        }).await;
    }

    /// Gets global config
    pub fn get_config(&self) -> SharedConfig {
        self.config.clone()
    }
}