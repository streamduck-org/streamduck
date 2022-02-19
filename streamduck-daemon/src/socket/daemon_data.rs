//! Data types that daemon uses for core functions
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use streamduck_core::versions::SOCKET_API;
use crate::core_manager::CoreManager;
use crate::socket::{check_packet_for_data, parse_packet_to_data, send_packet, SocketData, SocketHandle, SocketListener, SocketPacket};
use strum_macros::Display;
use streamduck_core::core::button::Button;
use streamduck_core::core::methods::{button_action, clear_button, CoreHandle, get_button, get_current_screen, get_stack, load_panels, pop_screen, push_screen, replace_screen, save_panels, set_brightness, set_button};
use streamduck_core::core::RawButtonPanel;
use streamduck_core::modules::{ModuleManager, PluginMetadata};
use streamduck_core::modules::components::{ComponentDefinition, UIValue};
use streamduck_core::util::{button_to_raw, make_button_unique, make_panel_unique, panel_to_raw};
use crate::config::{Config, ConfigError};

/// Listener for daemon types
pub struct DaemonListener {
    pub core_manager: Arc<CoreManager>,
    pub module_manager: Arc<ModuleManager>,
    pub config: Arc<Config>,
}

impl SocketListener for DaemonListener {
    fn message(&self, socket: SocketHandle, packet: SocketPacket) {
        // Version
        process_for_type::<SocketAPIVersion>(self,socket, &packet);

        // Device management
        process_for_type::<ListDevices>(self,socket, &packet);
        process_for_type::<GetDevice>(self,socket, &packet);
        process_for_type::<AddDevice>(self,socket, &packet);
        process_for_type::<RemoveDevice>(self,socket, &packet);

        // Device configuration
        process_for_type::<ReloadDeviceConfigsResult>(self, socket, &packet);
        process_for_type::<ReloadDeviceConfig>(self, socket, &packet);
        process_for_type::<SaveDeviceConfigsResult>(self, socket, &packet);
        process_for_type::<SaveDeviceConfig>(self, socket, &packet);

        process_for_type::<SetBrightness>(self, socket, &packet);

        // Module management
        process_for_type::<ListModules>(self,socket, &packet);
        process_for_type::<ListComponents>(self,socket, &packet);

        process_for_type::<GetModuleValues>(self,socket, &packet);
        process_for_type::<SetModuleValue>(self,socket, &packet);

        // Panel management
        process_for_type::<GetStack>(self, socket, &packet);
        process_for_type::<GetCurrentScreen>(self, socket, &packet);

        process_for_type::<GetButton>(self, socket, &packet);
        process_for_type::<SetButton>(self, socket, &packet);
        process_for_type::<ClearButton>(self, socket, &packet);

        process_for_type::<NewButton>(self, socket, &packet);
        process_for_type::<NewButtonFromComponent>(self, socket, &packet);

        process_for_type::<AddComponent>(self, socket, &packet);
        process_for_type::<GetComponentValues>(self, socket, &packet);
        process_for_type::<SetComponentValue>(self, socket, &packet);
        process_for_type::<RemoveComponent>(self, socket, &packet);

        process_for_type::<PushScreen>(self, socket, &packet);
        process_for_type::<PopScreen>(self, socket, &packet);
        process_for_type::<ForciblyPopScreen>(self, socket, &packet);
        process_for_type::<ReplaceScreen>(self, socket, &packet);
        process_for_type::<ResetStack>(self, socket, &packet);

        process_for_type::<CommitChangesToConfig>(self, socket, &packet);

        process_for_type::<DoButtonAction>(self, socket, &packet);
    }
}

trait DaemonRequest {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket);
}

fn process_for_type<T: DaemonRequest + SocketData>(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
    if packet.ty == T::NAME {
        T::process(listener, handle, packet);
    }
}

// Version

/// Request for socket API version
#[derive(Serialize, Deserialize)]
pub struct SocketAPIVersion {
    pub version: String
}

impl SocketData for SocketAPIVersion {
    const NAME: &'static str = "socket_version";
}

impl DaemonRequest for SocketAPIVersion {
    fn process(_listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if check_packet_for_data::<SocketAPIVersion>(&packet) {
            send_packet(handle, &packet, &SocketAPIVersion {
                version: SOCKET_API.1.to_string()
            }).ok();
        }
    }
}

// Device management

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
    XL
}

impl DeviceType {
    /// Gets device type from PID of the device
    pub fn from_pid(pid: u16) -> DeviceType {
        match pid {
            streamduck_core::streamdeck::pids::ORIGINAL => DeviceType::Original,
            streamduck_core::streamdeck::pids::ORIGINAL_V2 => DeviceType::OriginalV2,
            streamduck_core::streamdeck::pids::MINI => DeviceType::Mini,
            streamduck_core::streamdeck::pids::XL => DeviceType::XL,
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

/// Response of GetDevice request
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

/// Response of AddDevice request
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

/// Response of RemoveDevice request
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

// Device configuration
/// Request for reloading all device configs
#[derive(Serialize, Deserialize)]
pub enum ReloadDeviceConfigsResult {
    /// Sent if error happened while reloading configs
    ConfigError,

    /// Sent if successfully reloaded configs
    Reloaded,
}

impl SocketData for ReloadDeviceConfigsResult {
    const NAME: &'static str = "reload_device_configs";
}

impl DaemonRequest for ReloadDeviceConfigsResult {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if check_packet_for_data::<ReloadDeviceConfigsResult>(packet) {
            match listener.config.reload_device_configs() {
                Ok(_) => {
                    for (_, device) in listener.core_manager.list_added_devices() {
                        if !device.core.is_closed() {
                            device.core.mark_for_redraw();
                        }
                    }

                    send_packet(handle, packet, &ReloadDeviceConfigsResult::Reloaded).ok();
                },
                Err(err) => {
                    log::error!("Error encountered while reloading configs: {:?}", err);
                    send_packet(handle, packet, &ReloadDeviceConfigsResult::ConfigError).ok();
                }
            };
        }
    }
}

/// Request for reloading device config for specific device
#[derive(Serialize, Deserialize)]
pub struct ReloadDeviceConfig {
    pub serial_number: String
}

/// Response of ReloadDeviceConfig request
#[derive(Serialize, Deserialize)]
pub enum ReloadDeviceConfigResult {
    /// Sent if error happened while reloading configs
    ConfigError,

    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if successfully reloaded configs
    Reloaded,
}

impl SocketData for ReloadDeviceConfig {
    const NAME: &'static str = "reload_device_config";
}

impl SocketData for ReloadDeviceConfigResult {
    const NAME: &'static str = "reload_device_config";
}

impl DaemonRequest for ReloadDeviceConfig {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<ReloadDeviceConfig>(packet) {
            match listener.config.reload_device_config(&request.serial_number) {
                Ok(_) => {
                    if let Some(device) = listener.core_manager.get_device(&request.serial_number) {
                        if !device.core.is_closed() {
                            device.core.mark_for_redraw();
                        }
                    }

                    send_packet(handle, packet, &ReloadDeviceConfigResult::Reloaded).ok();
                },
                Err(err) => {
                    if let ConfigError::DeviceNotFound = err {
                        send_packet(handle, packet, &ReloadDeviceConfigResult::DeviceNotFound).ok();
                    } else {
                        log::error!("Error encountered while reloading config for {}: {:?}", request.serial_number, err);
                        send_packet(handle, packet, &ReloadDeviceConfigResult::ConfigError).ok();
                    }
                }
            }
        }
    }
}

/// Request for saving all device configs
#[derive(Serialize, Deserialize)]
pub enum SaveDeviceConfigsResult {
    /// Sent if error happened while saving configs
    ConfigError,

    /// Sent if successfully saved all configs
    Saved,
}

impl SocketData for SaveDeviceConfigsResult {
    const NAME: &'static str = "save_device_configs";
}

impl DaemonRequest for SaveDeviceConfigsResult {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if check_packet_for_data::<SaveDeviceConfigsResult>(packet) {
            match listener.config.save_device_configs() {
                Ok(_) => {
                    send_packet(handle, packet, &SaveDeviceConfigsResult::Saved).ok();
                },
                Err(err) => {
                    log::error!("Error encountered while saving configs: {:?}", err);
                    send_packet(handle, packet, &SaveDeviceConfigsResult::ConfigError).ok();
                }
            };
        }
    }
}

/// Request for saving device config for specific device
#[derive(Serialize, Deserialize)]
pub struct SaveDeviceConfig {
    pub serial_number: String,
}

/// Response of SaveDeviceConfig request
#[derive(Serialize, Deserialize)]
pub enum SaveDeviceConfigResult {
    /// Sent if error happened while saving config
    ConfigError,

    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if successfully saved
    Saved,
}

impl SocketData for SaveDeviceConfig {
    const NAME: &'static str = "save_device_config";
}

impl SocketData for SaveDeviceConfigResult {
    const NAME: &'static str = "save_device_config";
}

impl DaemonRequest for SaveDeviceConfig {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<SaveDeviceConfig>(packet) {
            match listener.config.save_device_config(&request.serial_number) {
                Ok(_) => {
                    send_packet(handle, packet, &SaveDeviceConfigResult::Saved).ok();
                },
                Err(err) => {
                    if let ConfigError::DeviceNotFound = err {
                        send_packet(handle, packet, &SaveDeviceConfigResult::DeviceNotFound).ok();
                    } else {
                        log::error!("Error encountered while saving config for {}: {:?}", request.serial_number, err);
                        send_packet(handle, packet, &SaveDeviceConfigResult::ConfigError).ok();
                    }
                }
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

/// Response of SetBrightness request
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
                set_brightness(&wrapped_core, request.brightness);

                // Updating current device config
                if let Some(mut config) = listener.config.get_device_config(&request.serial_number) {
                    config.brightness = request.brightness;

                    listener.config.set_device_config(&request.serial_number, config);
                }

                send_packet(handle, packet, &SetBrightnessResult::Set).ok();
            } else {
                send_packet(handle, packet, &SetBrightnessResult::DeviceNotFound).ok();
            }
        }
    }
}

// Module management
/// Request for getting all loaded modules
#[derive(Serialize, Deserialize)]
pub struct ListModules {
    pub modules: Vec<PluginMetadata>
}

impl SocketData for ListModules {
    const NAME: &'static str = "list_modules";
}

impl DaemonRequest for ListModules {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if check_packet_for_data::<ListModules>(&packet) {
            let modules = listener.module_manager.get_module_list()
                .iter()
                .map(|m| m.metadata())
                .collect::<Vec<PluginMetadata>>();

            send_packet(handle, &packet, &ListModules {
                modules
            }).ok();
        }
    }
}

/// Request for getting all components defined by all modules
#[derive(Serialize, Deserialize)]
pub struct ListComponents {
    /// Hashmap of module name to component map
    pub components: HashMap<String, HashMap<String, ComponentDefinition>>
}

impl SocketData for ListComponents {
    const NAME: &'static str = "list_components";
}

impl DaemonRequest for ListComponents {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if check_packet_for_data::<ListComponents>(&packet) {
            let components = listener.module_manager.get_components_list_by_modules()
                .into_iter()
                .map(|(n, c)| (n, c.into_iter().collect()))
                .collect();

            send_packet(handle, packet, &ListComponents {
                components
            }).ok();
        }
    }
}

/// Request for adding components onto buttons
#[derive(Serialize, Deserialize)]
pub struct GetModuleValues {
    pub module_name: String,
}

/// Response of AddComponent request
#[derive(Serialize, Deserialize)]
pub enum GetModuleValuesResult {
    /// Sent if module wasn't found
    ModuleNotFound,

    /// Sent if module values were successfully retrieved
    Values(Vec<UIValue>),
}

impl SocketData for GetModuleValues {
    const NAME: &'static str = "get_module_values";
}

impl SocketData for GetModuleValuesResult {
    const NAME: &'static str = "get_module_values";
}

impl DaemonRequest for GetModuleValues {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<GetModuleValues>(packet) {
            for module in listener.module_manager.get_module_list() {
                if module.name() == request.module_name {
                    let values = module.settings();
                    send_packet(handle, packet, &GetModuleValuesResult::Values(values)).ok();
                    return;
                }
            }

            send_packet(handle, packet, &GetModuleValuesResult::ModuleNotFound).ok();
        }
    }
}

/// Request for adding components onto buttons
#[derive(Serialize, Deserialize)]
pub struct SetModuleValue {
    pub module_name: String,
    pub value: Vec<UIValue>,
}

/// Response of AddComponent request
#[derive(Serialize, Deserialize)]
pub enum SetModuleValueResult {
    /// Sent if module wasn't found
    ModuleNotFound,

    /// Sent if module value was successfully set
    Set
}

impl SocketData for SetModuleValue {
    const NAME: &'static str = "set_module_value";
}

impl SocketData for SetModuleValueResult {
    const NAME: &'static str = "set_module_value";
}

impl DaemonRequest for SetModuleValue {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<SetModuleValue>(packet) {
            for module in listener.module_manager.get_module_list() {
                if module.name() == request.module_name {
                    module.set_setting(request.value);
                    send_packet(handle, packet, &SetModuleValueResult::Set).ok();
                    return;
                }
            }

            send_packet(handle, packet, &SetModuleValueResult::ModuleNotFound).ok();
        }
    }
}

// Panel management
/// Request for getting current stack on a device
#[derive(Serialize, Deserialize)]
pub struct GetStack {
    pub serial_number: String
}

/// Response of GetStack request
#[derive(Serialize, Deserialize)]
pub enum GetStackResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if successfully got stack
    Stack(Vec<RawButtonPanel>)
}

impl SocketData for GetStack {
    const NAME: &'static str = "get_stack";
}

impl SocketData for GetStackResult {
    const NAME: &'static str = "get_stack";
}

impl DaemonRequest for GetStack {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<GetStack>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number) {
                let wrapped_core = CoreHandle::wrap(device.core);

                let mut raw_stack = vec![];

                for stack_item in get_stack(&wrapped_core) {
                    let raw_item = panel_to_raw(&stack_item);
                    raw_stack.push(raw_item);
                }

                send_packet(handle, packet, &GetStackResult::Stack(raw_stack)).ok();
            } else {
                send_packet(handle, packet, &GetStackResult::DeviceNotFound).ok();
            }
        }
    }
}

/// Request for getting current screen on a device
#[derive(Serialize, Deserialize)]
pub struct GetCurrentScreen {
    pub serial_number: String
}

/// Response of GetCurrentScreen request
#[derive(Serialize, Deserialize)]
pub enum GetCurrentScreenResult {
    /// Sent if there's no screen
    NoScreen,

    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if successfully got the stack
    Screen(RawButtonPanel)
}

impl SocketData for GetCurrentScreen {
    const NAME: &'static str = "get_current_screen";
}

impl SocketData for GetCurrentScreenResult {
    const NAME: &'static str = "get_current_screen";
}

impl DaemonRequest for GetCurrentScreen {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<GetCurrentScreen>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number) {
                let wrapped_core = CoreHandle::wrap(device.core);

                if let Some(screen) = get_current_screen(&wrapped_core) {
                    send_packet(handle, packet, &GetCurrentScreenResult::Screen(panel_to_raw(&screen))).unwrap();
                } else {
                    send_packet(handle, packet, &GetCurrentScreenResult::NoScreen).ok();
                }
            } else {
                send_packet(handle, packet, &GetCurrentScreenResult::DeviceNotFound).ok();
            }
        }
    }
}

/// Request for getting a button from current screen on a device
#[derive(Serialize, Deserialize)]
pub struct GetButton {
    pub serial_number: String,
    pub key: u8
}

/// Response of GetButton request
#[derive(Serialize, Deserialize)]
pub enum GetButtonResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if there's no button there
    NoButton,

    /// Sent if successfully got the button
    Button(Button)
}

impl SocketData for GetButton {
    const NAME: &'static str = "get_button";
}

impl SocketData for GetButtonResult {
    const NAME: &'static str = "get_button";
}

impl DaemonRequest for GetButton {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<GetButton>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number) {
                let wrapped_core = CoreHandle::wrap(device.core);

                if let Some(button) = get_button(&wrapped_core, request.key) {
                    send_packet(handle, packet, &GetButtonResult::Button(button_to_raw(&button))).ok();
                } else {
                    send_packet(handle, packet, &GetButtonResult::NoButton).ok();
                }
            } else {
                send_packet(handle, packet, &GetButtonResult::DeviceNotFound).ok();
            }
        }
    }
}

/// Request for setting a button on current screen on a device
#[derive(Serialize, Deserialize)]
pub struct SetButton {
    pub serial_number: String,
    pub key: u8,
    pub button: Button
}

/// Response of SetButton request
#[derive(Serialize, Deserialize)]
pub enum SetButtonResult {
    /// Sent if there's no screen to set to
    NoScreen,

    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if successfully set the button
    Set
}

impl SocketData for SetButton {
    const NAME: &'static str = "set_button";
}

impl SocketData for SetButtonResult {
    const NAME: &'static str = "set_button";
}

impl DaemonRequest for SetButton {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<SetButton>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number) {
                let wrapped_core = CoreHandle::wrap(device.core);

                if set_button(&wrapped_core, request.key, make_button_unique(request.button)) {
                    send_packet(handle, packet, &SetButtonResult::Set).ok();
                } else {
                    send_packet(handle, packet, &SetButtonResult::NoScreen).ok();
                }
            } else {
                send_packet(handle, packet, &SetButtonResult::DeviceNotFound).ok();
            }
        }
    }
}

/// Request for clearing a button on current screen on a device
#[derive(Serialize, Deserialize)]
pub struct ClearButton {
    pub serial_number: String,
    pub key: u8,
}

/// Response of ClearButton request
#[derive(Serialize, Deserialize)]
pub enum ClearButtonResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if there's no screen, or there's no button to clear
    FailedToClear,

    /// Sent if successfully set the button
    Cleared
}

impl SocketData for ClearButton {
    const NAME: &'static str = "clear_button";
}

impl SocketData for ClearButtonResult {
    const NAME: &'static str = "clear_button";
}

impl DaemonRequest for ClearButton {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<ClearButton>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number) {
                let wrapped_core = CoreHandle::wrap(device.core);

                if clear_button(&wrapped_core, request.key) {
                    send_packet(handle, packet, &ClearButtonResult::Cleared).ok();
                } else {
                    send_packet(handle, packet, &ClearButtonResult::FailedToClear).ok();
                }
            } else {
                send_packet(handle, packet, &ClearButtonResult::DeviceNotFound).ok();
            }
        }
    }
}

/// Request for adding a new empty button
#[derive(Serialize, Deserialize)]
pub struct NewButton {
    pub serial_number: String,
    pub key: u8,
}

/// Response of NewButton request
#[derive(Serialize, Deserialize)]
pub enum NewButtonResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if button failed to be created on specified spot
    FailedToCreate,

    /// Sent if button was successfully created
    Created,
}

impl SocketData for NewButton {
    const NAME: &'static str = "new_button";
}

impl SocketData for NewButtonResult {
    const NAME: &'static str = "new_button";
}

impl DaemonRequest for NewButton {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<NewButton>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number) {
                let wrapped_core = CoreHandle::wrap(device.core);

                if set_button(&wrapped_core, request.key, make_button_unique(Button::new())) {
                    send_packet(handle, packet, &NewButtonResult::Created).ok();
                } else {
                    send_packet(handle, packet, &NewButtonResult::FailedToCreate).ok();
                }
            } else {
                send_packet(handle, packet, &NewButtonResult::DeviceNotFound).ok();
            }
        }
    }
}

/// Request for adding a new empty button
#[derive(Serialize, Deserialize)]
pub struct NewButtonFromComponent {
    pub serial_number: String,
    pub key: u8,
    pub component_name: String,
}

/// Response of NewButton request
#[derive(Serialize, Deserialize)]
pub enum NewButtonFromComponentResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if component wasn't found
    ComponentNotFound,

    /// Sent if button failed to be created on specified spot
    FailedToCreate,

    /// Sent if button was successfully created
    Created,
}

impl SocketData for NewButtonFromComponent {
    const NAME: &'static str = "new_button_from_component";
}

impl SocketData for NewButtonFromComponentResult {
    const NAME: &'static str = "new_button_from_component";
}

impl DaemonRequest for NewButtonFromComponent {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<NewButtonFromComponent>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number) {
                let wrapped_core = CoreHandle::wrap(device.core);

                let map = listener.module_manager.get_components_list_by_modules();

                for (module_name, component_list) in map {
                    for (component_name, definition) in component_list {
                        if request.component_name == component_name {
                            let module = listener.module_manager.get_module(&module_name).unwrap();

                            let mut button = Button::new();
                            button.insert_component(definition.default_looks).ok();

                            module.add_component(wrapped_core.clone_for(&module), &mut button, &component_name);

                            if set_button(&wrapped_core, request.key, make_button_unique(button)) {
                                send_packet(handle, packet, &NewButtonFromComponentResult::Created).ok();
                            } else {
                                send_packet(handle, packet, &NewButtonFromComponentResult::FailedToCreate).ok();
                            }

                            return;
                        }
                    }
                }

                send_packet(handle, packet, &NewButtonFromComponentResult::ComponentNotFound).ok();
            } else {
                send_packet(handle, packet, &NewButtonFromComponentResult::DeviceNotFound).ok();
            }
        }
    }
}

// Components
/// Request for adding components onto buttons
#[derive(Serialize, Deserialize)]
pub struct AddComponent {
    pub serial_number: String,
    pub key: u8,
    pub component_name: String,
}

/// Response of AddComponent request
#[derive(Serialize, Deserialize)]
pub enum AddComponentResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if there's no screen
    NoScreen,

    /// Sent if button wasn't found
    NoButton,

    /// Sent if component wasn't found
    ComponentNotFound,

    /// Sent if component already exists on a button
    AlreadyExists,

    /// Sent if component was successfully added
    Added,
}

impl SocketData for AddComponent {
    const NAME: &'static str = "add_component";
}

impl SocketData for AddComponentResult {
    const NAME: &'static str = "add_component";
}

impl DaemonRequest for AddComponent {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<AddComponent>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number) {
                let wrapped_core = CoreHandle::wrap(device.core);

                if let Some(screen) = get_current_screen(&wrapped_core) {
                    if let Some(button) = screen.get(&request.key) {
                        let mut button_handle = button.write().unwrap();

                        if button_handle.component_names().contains(&request.component_name) {
                            send_packet(handle, packet, &AddComponentResult::AlreadyExists).ok();
                        } else {
                            let components = listener.module_manager.get_components_list_by_modules();

                            for (module, component_list) in components {
                                for (component, _) in component_list {
                                    if component == request.component_name {
                                        let module = listener.module_manager.get_module(&module).unwrap();
                                        module.add_component(wrapped_core.clone_for(&module), button_handle.deref_mut(), &component);
                                        send_packet(handle, packet, &AddComponentResult::Added).ok();
                                        return;
                                    }
                                }
                            }

                            send_packet(handle, packet, &AddComponentResult::ComponentNotFound).ok();
                        }
                    } else {
                        send_packet(handle, packet, &AddComponentResult::NoButton).ok();
                    }
                } else {
                    send_packet(handle, packet, &AddComponentResult::NoScreen).ok();
                }
            } else {
                send_packet(handle, packet, &AddComponentResult::DeviceNotFound).ok();
            }
        }
    }
}

/// Request for adding components onto buttons
#[derive(Serialize, Deserialize)]
pub struct GetComponentValues {
    pub serial_number: String,
    pub key: u8,
    pub component_name: String,
}

/// Response of AddComponent request
#[derive(Serialize, Deserialize)]
pub enum GetComponentValuesResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if there's no screen
    NoScreen,

    /// Sent if button wasn't found
    NoButton,

    /// Sent if component wasn't found
    ComponentNotFound,

    /// Sent if component values were successfully retrieved
    Values(Vec<UIValue>),
}

impl SocketData for GetComponentValues {
    const NAME: &'static str = "get_component_values";
}

impl SocketData for GetComponentValuesResult {
    const NAME: &'static str = "get_component_values";
}

impl DaemonRequest for GetComponentValues {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<GetComponentValues>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number) {
                let wrapped_core = CoreHandle::wrap(device.core);

                if let Some(screen) = get_current_screen(&wrapped_core) {
                    if let Some(button) = screen.get(&request.key) {
                        let button_handle = button.read().unwrap();

                        if button_handle.component_names().contains(&request.component_name) {
                            let components = listener.module_manager.get_components_list_by_modules();

                            for (module, component_list) in components {
                                for (component, _) in component_list {
                                    if component == request.component_name {
                                        let module = listener.module_manager.get_module(&module).unwrap();
                                        let values = module.component_values(wrapped_core.clone_for(&module), button_handle.deref(), &component);
                                        send_packet(handle, packet, &GetComponentValuesResult::Values(values)).ok();
                                        return;
                                    }
                                }
                            }
                        }

                        send_packet(handle, packet, &GetComponentValuesResult::ComponentNotFound).ok();
                    } else {
                        send_packet(handle, packet, &GetComponentValuesResult::NoButton).ok();
                    }
                } else {
                    send_packet(handle, packet, &GetComponentValuesResult::NoScreen).ok();
                }
            } else {
                send_packet(handle, packet, &GetComponentValuesResult::DeviceNotFound).ok();
            }
        }
    }
}

/// Request for adding components onto buttons
#[derive(Serialize, Deserialize)]
pub struct SetComponentValue {
    pub serial_number: String,
    pub key: u8,
    pub component_name: String,
    pub value: Vec<UIValue>,
}

/// Response of AddComponent request
#[derive(Serialize, Deserialize)]
pub enum SetComponentValueResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if there's no screen
    NoScreen,

    /// Sent if button wasn't found
    NoButton,

    /// Sent if component wasn't found
    ComponentNotFound,

    /// Sent if component value was successfully set
    Set,
}

impl SocketData for SetComponentValue {
    const NAME: &'static str = "set_component_value";
}

impl SocketData for SetComponentValueResult {
    const NAME: &'static str = "set_component_value";
}

impl DaemonRequest for SetComponentValue {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<SetComponentValue>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number) {
                let wrapped_core = CoreHandle::wrap(device.core);

                if let Some(screen) = get_current_screen(&wrapped_core) {
                    if let Some(button) = screen.get(&request.key) {
                        let mut button_handle = button.write().unwrap();

                        if button_handle.component_names().contains(&request.component_name) {
                            let components = listener.module_manager.get_components_list_by_modules();

                            for (module, component_list) in components {
                                for (component, _) in component_list {
                                    if component == request.component_name {
                                        let module = listener.module_manager.get_module(&module).unwrap();
                                        module.set_component_value(wrapped_core.clone_for(&module), button_handle.deref_mut(), &component, request.value);
                                        send_packet(handle, packet, &SetComponentValueResult::Set).ok();
                                        return;
                                    }
                                }
                            }
                        }

                        send_packet(handle, packet, &SetComponentValueResult::ComponentNotFound).ok();
                    } else {
                        send_packet(handle, packet, &SetComponentValueResult::NoButton).ok();
                    }
                } else {
                    send_packet(handle, packet, &SetComponentValueResult::NoScreen).ok();
                }
            } else {
                send_packet(handle, packet, &SetComponentValueResult::DeviceNotFound).ok();
            }
        }
    }
}

/// Request for adding components onto buttons
#[derive(Serialize, Deserialize)]
pub struct RemoveComponent {
    pub serial_number: String,
    pub key: u8,
    pub component_name: String,
}

/// Response of AddComponent request
#[derive(Serialize, Deserialize)]
pub enum RemoveComponentResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if there's no screen
    NoScreen,

    /// Sent if button wasn't found
    NoButton,

    /// Sent if component wasn't found
    ComponentNotFound,

    /// Sent if component value was successfully set
    Removed,
}

impl SocketData for RemoveComponent {
    const NAME: &'static str = "remove_component";
}

impl SocketData for RemoveComponentResult {
    const NAME: &'static str = "remove_component";
}

impl DaemonRequest for RemoveComponent {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<RemoveComponent>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number) {
                let wrapped_core = CoreHandle::wrap(device.core);

                if let Some(screen) = get_current_screen(&wrapped_core) {
                    if let Some(button) = screen.get(&request.key) {
                        let mut button_handle = button.write().unwrap();

                        if button_handle.component_names().contains(&request.component_name) {
                            let components = listener.module_manager.get_components_list_by_modules();

                            for (module, component_list) in components {
                                for (component, _) in component_list {
                                    if component == request.component_name {
                                        let module = listener.module_manager.get_module(&module).unwrap();
                                        module.remove_component(wrapped_core.clone_for(&module), button_handle.deref_mut(), &component);
                                        send_packet(handle, packet, &RemoveComponentResult::Removed).ok();
                                        return;
                                    }
                                }
                            }

                            send_packet(handle, packet, &RemoveComponentResult::ComponentNotFound).ok();
                        } else {
                            send_packet(handle, packet, &RemoveComponentResult::ComponentNotFound).ok();
                        }
                    } else {
                        send_packet(handle, packet, &RemoveComponentResult::NoButton).ok();
                    }
                } else {
                    send_packet(handle, packet, &RemoveComponentResult::NoScreen).ok();
                }
            } else {
                send_packet(handle, packet, &RemoveComponentResult::DeviceNotFound).ok();
            }
        }
    }
}

/// Request for pushing a new screen on a device
#[derive(Serialize, Deserialize)]
pub struct PushScreen {
    pub serial_number: String,
    pub screen: RawButtonPanel
}

/// Response of PushScreen request
#[derive(Serialize, Deserialize)]
pub enum PushScreenResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if successfully pushed a screen
    Pushed
}

impl SocketData for PushScreen {
    const NAME: &'static str = "push_screen";
}

impl SocketData for PushScreenResult {
    const NAME: &'static str = "push_screen";
}

impl DaemonRequest for PushScreen {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<PushScreen>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number) {
                let wrapped_core = CoreHandle::wrap(device.core);

                push_screen(&wrapped_core, make_panel_unique(request.screen));
                send_packet(handle, packet, &PushScreenResult::Pushed).ok();
            } else {
                send_packet(handle, packet, &PushScreenResult::DeviceNotFound).ok();
            }
        }
    }
}

/// Request for popping top-most screen on a device
#[derive(Serialize, Deserialize)]
pub struct PopScreen {
    pub serial_number: String
}

/// Response of PopScreen request
#[derive(Serialize, Deserialize)]
pub enum PopScreenResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if current screen is the only one remaining
    OnlyOneRemaining,

    /// Sent if successfully popped a screen
    Popped
}

impl SocketData for PopScreen {
    const NAME: &'static str = "pop_screen";
}

impl SocketData for PopScreenResult {
    const NAME: &'static str = "pop_screen";
}

impl DaemonRequest for PopScreen {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<PopScreen>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number) {
                let count = {
                    let stack = device.core.current_stack.lock().unwrap();
                    stack.len()
                };

                let wrapped_core = CoreHandle::wrap(device.core);

                if count > 1 {
                    pop_screen(&wrapped_core);
                    send_packet(handle, packet, &PopScreenResult::Popped).ok();
                } else {
                    send_packet(handle, packet, &PopScreenResult::OnlyOneRemaining).ok();
                }
            } else {
                send_packet(handle, packet, &PopScreenResult::DeviceNotFound).ok();
            }
        }
    }
}

/// Request for popping top-most screen on a device, even if it's the only one remaining
#[derive(Serialize, Deserialize)]
pub struct ForciblyPopScreen {
    pub serial_number: String
}

/// Response of PopScreen request
#[derive(Serialize, Deserialize)]
pub enum ForciblyPopScreenResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if successfully popped a screen
    Popped
}

impl SocketData for ForciblyPopScreen {
    const NAME: &'static str = "force_pop_screen";
}

impl SocketData for ForciblyPopScreenResult {
    const NAME: &'static str = "force_pop_screen";
}

impl DaemonRequest for ForciblyPopScreen {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<ForciblyPopScreen>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number) {
                let wrapped_core = CoreHandle::wrap(device.core);

                pop_screen(&wrapped_core);
                send_packet(handle, packet, &ForciblyPopScreenResult::Popped).ok();
            } else {
                send_packet(handle, packet, &ForciblyPopScreenResult::DeviceNotFound).ok();
            }
        }
    }
}

/// Request for replacing a screen on a device
#[derive(Serialize, Deserialize)]
pub struct ReplaceScreen {
    pub serial_number: String,
    pub screen: RawButtonPanel
}

/// Response of ReplaceScreen request
#[derive(Serialize, Deserialize)]
pub enum ReplaceScreenResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if successfully replaced the screen
    Replaced
}

impl SocketData for ReplaceScreen {
    const NAME: &'static str = "replace_screen";
}

impl SocketData for ReplaceScreenResult {
    const NAME: &'static str = "replace_screen";
}

impl DaemonRequest for ReplaceScreen {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<ReplaceScreen>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number) {
                let wrapped_core = CoreHandle::wrap(device.core);

                replace_screen(&wrapped_core, make_panel_unique(request.screen));
                send_packet(handle, packet, &ReplaceScreenResult::Replaced).ok();
            } else {
                send_packet(handle, packet, &ReplaceScreenResult::DeviceNotFound).ok();
            }
        }
    }
}

/// Request for replacing a screen on a device
#[derive(Serialize, Deserialize)]
pub struct ResetStack {
    pub serial_number: String,
    pub screen: RawButtonPanel
}

/// Response of ReplaceScreen request
#[derive(Serialize, Deserialize)]
pub enum ResetStackResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if successfully reset stack with a screen
    Reset
}

impl SocketData for ResetStack {
    const NAME: &'static str = "reset_stack";
}

impl SocketData for ResetStackResult {
    const NAME: &'static str = "reset_stack";
}

impl DaemonRequest for ResetStack {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<ResetStack>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number) {
                let wrapped_core = CoreHandle::wrap(device.core);

                load_panels(&wrapped_core, make_panel_unique(request.screen));
                send_packet(handle, packet, &ResetStackResult::Reset).ok();
            } else {
                send_packet(handle, packet, &ResetStackResult::DeviceNotFound).ok();
            }
        }
    }
}

/// Request for committing all changes of the stack to device config
#[derive(Serialize, Deserialize)]
pub struct CommitChangesToConfig {
    pub serial_number: String
}

/// Response of CommitChangesToConfig request
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

                let stack = save_panels(&wrapped_core);

                if let Some(mut config) = listener.config.get_device_config(&request.serial_number) {
                    config.layout = panel_to_raw(&stack);
                    listener.config.set_device_config(&request.serial_number, config);

                    send_packet(handle, packet, &CommitChangesToConfigResult::Committed).ok();
                } else {
                    send_packet(handle, packet, &CommitChangesToConfigResult::DeviceNotFound).ok();
                }
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

/// Response of DoButtonAction request
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