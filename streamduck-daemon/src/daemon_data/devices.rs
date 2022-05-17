//! Requests related to devices
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use streamduck_core::core::CoreHandle;
use streamduck_core::socket::{check_packet_for_data, parse_packet_to_data, send_packet, SocketData, SocketHandle, SocketPacket};
use crate::daemon_data::{DaemonListener, DaemonRequest};
use streamduck_core::streamdeck;

/// Request for getting device list
#[derive(Serialize, Deserialize)]
pub struct ListDevices {
    pub devices: Vec<Device>
}

impl SocketData for ListDevices {
    const NAME: &'static str = "list_devices";
}

impl DaemonRequest for ListDevices {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if check_packet_for_data::<ListDevices>(&packet) {
            let mut devices = vec![];

            // Connected devices
            for device in listener.core_manager.list_added_devices().values() {
                devices.push(Device {
                    device_type: DeviceType::from_pid(device.pid),
                    serial_number: device.serial.clone(),
                    managed: true,
                    online: !device.core.is_closed()
                })
            }

            // Available devices
            for (_, pid, serial) in listener.core_manager.list_available_devices() {
                devices.push(Device {
                    device_type: DeviceType::from_pid(pid),
                    serial_number: serial,
                    managed: false,
                    online: true
                })
            }

            send_packet(handle, &packet, &ListDevices {
                devices
            }).ok();
        }
    }
}

/// Device struct
#[derive(Serialize, Deserialize)]
pub struct Device {
    /// Device type
    pub device_type: DeviceType,
    /// Serial number of the streamdeck
    pub serial_number: String,
    /// If the device was added to managed device list
    pub managed: bool,
    /// If the device is online
    pub online: bool,
}

/// Streamdeck types
#[derive(Serialize, Deserialize, Display)]
pub enum DeviceType {
    Unknown,
    Mini,
    Original,
    OriginalV2,
    XL,
    MK2
}

impl DeviceType {
    /// Gets device type from PID of the device
    pub fn from_pid(pid: u16) -> DeviceType {
        match pid {
            streamdeck::pids::ORIGINAL => DeviceType::Original,
            streamdeck::pids::ORIGINAL_V2 => DeviceType::OriginalV2,
            streamdeck::pids::MINI => DeviceType::Mini,
            streamdeck::pids::XL => DeviceType::XL,
            streamdeck::pids::MK2 => DeviceType::MK2,
            _ => DeviceType::Unknown,
        }
    }
}

/// Request for getting a device
#[derive(Serialize, Deserialize)]
pub struct GetDevice {
    pub serial_number: String
}

impl SocketData for GetDevice {
    const NAME: &'static str = "get_device";
}

/// Response of [GetDevice] request
#[derive(Serialize, Deserialize)]
pub enum GetDeviceResult {
    /// Sent when device is found
    Found(Device),

    /// Send when device wasn't found
    NotFound
}

impl SocketData for GetDeviceResult {
    const NAME: &'static str = "get_device";
}

impl DaemonRequest for GetDevice {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(get_request) = parse_packet_to_data::<GetDevice>(&packet) {
            let result = if let Some(device) = listener.core_manager.get_device(&get_request.serial_number) {
                GetDeviceResult::Found(Device {
                    device_type: DeviceType::from_pid(device.pid),
                    serial_number: device.serial,
                    managed: true,
                    online: !device.core.is_closed()
                })
            } else {
                GetDeviceResult::NotFound
            };

            send_packet(handle, &packet, &result).ok();
        }
    }
}


/// Request for adding a device
#[derive(Serialize, Deserialize)]
pub struct AddDevice {
    pub serial_number: String,
}

impl SocketData for AddDevice {
    const NAME: &'static str = "add_device";
}

/// Response of [AddDevice] request
#[derive(Serialize, Deserialize)]
pub enum AddDeviceResult {
    /// Sent if device is already added
    AlreadyRegistered,

    /// Sent if device wasn't found
    NotFound,

    /// Sent on success
    Added
}

impl SocketData for AddDeviceResult {
    const NAME: &'static str = "add_device";
}

impl DaemonRequest for AddDevice {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(add_request) = parse_packet_to_data::<AddDevice>(&packet) {
            if listener.core_manager.get_device(&add_request.serial_number).is_none() {
                for (vid, pid, serial) in listener.core_manager.list_available_devices() {
                    if add_request.serial_number == serial {
                        listener.core_manager.add_device(vid, pid, &serial);
                        send_packet(handle, &packet, &AddDeviceResult::Added).ok();
                        return;
                    }
                }

                send_packet(handle, &packet, &AddDeviceResult::NotFound).ok();
            } else {
                send_packet(handle, &packet, &AddDeviceResult::AlreadyRegistered).ok();
            }
        }
    }
}

/// Request for removing a device
#[derive(Serialize, Deserialize)]
pub struct RemoveDevice {
    pub serial_number: String,
}

impl SocketData for RemoveDevice {
    const NAME: &'static str = "remove_device";
}

/// Response of [RemoveDevice] request
#[derive(Serialize, Deserialize)]
pub enum RemoveDeviceResult {
    /// Sent if device already wasn't added
    NotRegistered,

    /// Sent on success
    Removed
}

impl SocketData for RemoveDeviceResult {
    const NAME: &'static str = "remove_device";
}

impl DaemonRequest for RemoveDevice {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(remove_request) = parse_packet_to_data::<RemoveDevice>(&packet) {
            if listener.core_manager.get_device(&remove_request.serial_number).is_some() {
                listener.core_manager.remove_device(&remove_request.serial_number);
                send_packet(handle, &packet, &RemoveDeviceResult::Removed).ok();
            } else {
                send_packet(handle, &packet, &RemoveDeviceResult::NotRegistered).ok();
            }
        }
    }
}

/// Request for setting device's brightness
#[derive(Serialize, Deserialize)]
pub struct SetBrightness {
    pub serial_number: String,
    pub brightness: u8,
}

/// Response of [SetBrightness] request
#[derive(Serialize, Deserialize)]
pub enum SetBrightnessResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if brightness was successfully set
    Set,
}

impl SocketData for SetBrightness {
    const NAME: &'static str = "set_brightness";
}

impl SocketData for SetBrightnessResult {
    const NAME: &'static str = "set_brightness";
}

impl DaemonRequest for SetBrightness {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<SetBrightness>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number) {
                // Setting brightness
                let wrapped_core = CoreHandle::wrap(device.core);
                wrapped_core.set_brightness(request.brightness);

                send_packet(handle, packet, &SetBrightnessResult::Set).ok();
            } else {
                send_packet(handle, packet, &SetBrightnessResult::DeviceNotFound).ok();
            }
        }
    }
}