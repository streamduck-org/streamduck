//! Requests for various operations
use serde::{Deserialize, Serialize};
use streamduck_core::core::methods::{button_action, commit_changes, CoreHandle};
use streamduck_core::socket::{parse_packet_to_data, send_packet, SocketData, SocketHandle, SocketPacket};
use crate::daemon_data::{DaemonListener, DaemonRequest};

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

impl DaemonRequest for CommitChangesToConfig {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<CommitChangesToConfig>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number) {
                let wrapped_core = CoreHandle::wrap(device.core);

                commit_changes(&wrapped_core);
                send_packet(handle, packet, &CommitChangesToConfigResult::Committed).ok();
            } else {
                send_packet(handle, packet, &CommitChangesToConfigResult::DeviceNotFound).ok();
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

impl DaemonRequest for DoButtonAction {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<DoButtonAction>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number) {
                let wrapped_core = CoreHandle::wrap(device.core);

                button_action(&wrapped_core, request.key);
                send_packet(handle, packet, &DoButtonActionResult::Activated).ok();
            } else {
                send_packet(handle, packet, &DoButtonActionResult::DeviceNotFound).ok();
            }
        }
    }
}