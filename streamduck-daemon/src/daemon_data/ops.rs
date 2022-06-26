//! Requests for various operations
use serde::{Deserialize, Serialize};
use streamduck_core::core::CoreHandle;
use streamduck_core::socket::{parse_packet_to_data, send_packet, SocketData, SocketHandle, SocketPacket};
use crate::daemon_data::{DaemonListener, DaemonRequest};
use streamduck_core::async_trait;

/// Request for committing all changes of the stack to device config
#[derive(Serialize, Deserialize)]
pub struct CommitChangesToConfig {
    pub serial_number: String
}

/// Response of [CommitChangesToConfig] request
#[derive(Serialize, Deserialize)]
pub enum CommitChangesToConfigResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if successfully committed changes
    Committed
}

impl SocketData for CommitChangesToConfig {
    const NAME: &'static str = "commit_changes";
}

impl SocketData for CommitChangesToConfigResult {
    const NAME: &'static str = "commit_changes";
}

#[async_trait]
impl DaemonRequest for CommitChangesToConfig {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<CommitChangesToConfig>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number).await {
                let wrapped_core = CoreHandle::wrap(device.core);

                wrapped_core.commit_changes().await;
                send_packet(handle, packet, &CommitChangesToConfigResult::Committed).await.ok();
            } else {
                send_packet(handle, packet, &CommitChangesToConfigResult::DeviceNotFound).await.ok();
            }
        }
    }
}

/// Request for simulating a press on a button on current screen for a device
#[derive(Serialize, Deserialize)]
pub struct DoButtonAction {
    pub serial_number: String,
    pub key: u8,
}

/// Response of [DoButtonAction] request
#[derive(Serialize, Deserialize)]
pub enum DoButtonActionResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if successfully activated a button
    Activated
}

impl SocketData for DoButtonAction {
    const NAME: &'static str = "do_button_action";
}

impl SocketData for DoButtonActionResult {
    const NAME: &'static str = "do_button_action";
}

#[async_trait]
impl DaemonRequest for DoButtonAction {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<DoButtonAction>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number).await {
                let wrapped_core = CoreHandle::wrap(device.core);

                wrapped_core.button_action(request.key).await;
                send_packet(handle, packet, &DoButtonActionResult::Activated).await.ok();
            } else {
                send_packet(handle, packet, &DoButtonActionResult::DeviceNotFound).await.ok();
            }
        }
    }
}