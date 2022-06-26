//! Requests related to devices
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use streamduck_core::core::CoreHandle;
use streamduck_core::socket::{check_packet_for_data, parse_packet_to_data, send_packet, SocketData, SocketHandle, SocketPacket};
use crate::daemon_data::{DaemonListener, DaemonRequest};
use streamduck_core::streamdeck;
use streamduck_core::async_trait;

/// Request for getting device list
#[derive(Serialize, Deserialize)]
pub struct ListDevices {
    pub devices: Vec<Device>
}

impl SocketData for ListDevices {
    const NAME: &'static str = "list_devices";
}

#[async_trait]
impl DaemonRequest for ListDevices {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if check_packet_for_data::<ListDevices>(&packet) {
            let mut devices = vec![];

            // Connected devices
            for device in listener.core_manager.list_added_devices().await.values() {
                devices.push(Device {
                    device_type: DeviceType::from_pid(device.pid),
                    serial_number: device.serial.clone(),
                    managed: true,
                    online: !device.core.is_closed().await
                })
            }

            // Available devices
            for (_, pid, serial) in listener.core_manager.list_available_devices().await {
                devices.push(Device {
                    device_type: DeviceType::from_pid(pid),
                    serial_number: serial,
                    managed: false,
                    online: true
                })
            }

            send_packet(handle, &packet, &ListDevices {
                devices
            }).await.ok();
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

#[async_trait]
impl DaemonRequest for GetDevice {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if let Ok(get_request) = parse_packet_to_data::<GetDevice>(&packet) {
            let result = if let Some(device) = listener.core_manager.get_device(&get_request.serial_number).await {
                GetDeviceResult::Found(Device {
                    device_type: DeviceType::from_pid(device.pid),
                    serial_number: device.serial,
                    managed: true,
                    online: !device.core.is_closed().await
                })
            } else {
                GetDeviceResult::NotFound
            };

            send_packet(handle, &packet, &result).await.ok();
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

#[async_trait]
impl DaemonRequest for AddDevice {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if let Ok(add_request) = parse_packet_to_data::<AddDevice>(&packet) {
            if listener.core_manager.get_device(&add_request.serial_number).await.is_none() {
                for (vid, pid, serial) in listener.core_manager.list_available_devices().await {
                    if add_request.serial_number == serial {
                        listener.core_manager.add_device(vid, pid, &serial).await;
                        send_packet(handle, &packet, &AddDeviceResult::Added).await.ok();
                        return;
                    }
                }

                send_packet(handle, &packet, &AddDeviceResult::NotFound).await.ok();
            } else {
                send_packet(handle, &packet, &AddDeviceResult::AlreadyRegistered).await.ok();
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

#[async_trait]
impl DaemonRequest for RemoveDevice {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if let Ok(remove_request) = parse_packet_to_data::<RemoveDevice>(&packet) {
            if listener.core_manager.get_device(&remove_request.serial_number).await.is_some() {
                listener.core_manager.remove_device(&remove_request.serial_number).await;
                send_packet(handle, &packet, &RemoveDeviceResult::Removed).await.ok();
            } else {
                send_packet(handle, &packet, &RemoveDeviceResult::NotRegistered).await.ok();
            }
        }
    }
}

/// Request for getting device's current brightness
#[derive(Serialize, Deserialize)]
pub struct GetBrightness {
    pub serial_number: String,
}

/// Response of [GetBrightness] request
#[derive(Serialize, Deserialize)]
pub enum GetBrightnessResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if brightness was successfully set
    Brightness(u8),
}

impl SocketData for GetBrightness {
    const NAME: &'static str = "get_brightness";
}

impl SocketData for GetBrightnessResult {
    const NAME: &'static str = "get_brightness";
}

#[async_trait]
impl DaemonRequest for GetBrightness {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<GetBrightness>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number).await {
                let brightness = device.core.device_config.read().await.brightness;

                send_packet(handle, packet, &GetBrightnessResult::Brightness(brightness)).await.ok();
            } else {
                send_packet(handle, packet, &GetBrightnessResult::DeviceNotFound).await.ok();
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

#[async_trait]
impl DaemonRequest for SetBrightness {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<SetBrightness>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number).await {
                // Setting brightness
                let wrapped_core = CoreHandle::wrap(device.core);
                wrapped_core.set_brightness(request.brightness).await;

                send_packet(handle, packet, &SetBrightnessResult::Set).await.ok();
            } else {
                send_packet(handle, packet, &SetBrightnessResult::DeviceNotFound).await.ok();
            }
        }
    }
}