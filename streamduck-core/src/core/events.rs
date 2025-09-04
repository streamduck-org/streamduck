use serde::{Deserialize, Serialize};
use crate::api::driver::NamespacedDeviceIdentifier;
use crate::core::Core;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreEventMessage {
    pub event: CoreEvent
}

impl From<CoreEvent> for CoreEventMessage {
    fn from(value: CoreEvent) -> Self {
        Self {
            event: value
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoreEvent {
    DiscoveredDevice(NamespacedDeviceIdentifier),
    DiscoveredDeviceLost(NamespacedDeviceIdentifier)
}

impl<'core> Core<'core> {
    pub fn send_event(&self, event: CoreEvent) {
        self.client.get().unwrap().ev_tx.send(event.into()).unwrap();
    }
}