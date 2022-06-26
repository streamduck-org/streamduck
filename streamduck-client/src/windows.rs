use std::collections::HashMap;
use std::io::BufReader;
use std::ops::DerefMut;
use std::sync::{Arc, RwLock, RwLockWriteGuard};
use named_pipe::PipeClient;
use rand::distributions::Alphanumeric;
use rand::Rng;
use streamduck_core::core::button::Button;
use streamduck_core::core::RawButtonPanel;
use streamduck_core::modules::components::{ComponentDefinition, UIPathValue};
use streamduck_core::modules::events::SDGlobalEvent;
use streamduck_core::modules::PluginMetadata;
use streamduck_core::socket::{ SocketPacket};
use streamduck_core::versions::SOCKET_API;
use streamduck_daemon::daemon_data::assets::{AddImage, AddImageResult, ListFonts, ListImages, ListImagesResult, RemoveImage, RemoveImageResult};
use streamduck_daemon::daemon_data::buttons::{AddComponent, AddComponentResult, AddComponentValue, AddComponentValueResult, ClearButton, ClearButtonResult, ClipboardStatusResult, CopyButton, CopyButtonResult, GetButton, GetButtonResult, GetComponentValues, GetComponentValuesResult, NewButton, NewButtonFromComponent, NewButtonFromComponentResult, NewButtonResult, PasteButton, PasteButtonResult, RemoveComponent, RemoveComponentResult, RemoveComponentValue, RemoveComponentValueResult, SetButton, SetButtonResult, SetComponentValue, SetComponentValueResult};
use streamduck_daemon::daemon_data::config::{ExportDeviceConfig, ExportDeviceConfigResult, GetDeviceConfig, GetDeviceConfigResult, ImportDeviceConfig, ImportDeviceConfigResult, ReloadDeviceConfig, ReloadDeviceConfigResult, ReloadDeviceConfigsResult, SaveDeviceConfig, SaveDeviceConfigResult, SaveDeviceConfigsResult};
use streamduck_daemon::daemon_data::devices::{AddDevice, AddDeviceResult, Device, GetDevice, GetDeviceResult, ListDevices, RemoveDevice, RemoveDeviceResult, SetBrightness, SetBrightnessResult};
use streamduck_daemon::daemon_data::modules::{AddModuleValue, AddModuleValueResult, GetModuleValues, GetModuleValuesResult, ListComponents, ListModules, RemoveModuleValue, RemoveModuleValueResult, SetModuleValue, SetModuleValueResult};
use streamduck_daemon::daemon_data::ops::{CommitChangesToConfig, CommitChangesToConfigResult, DoButtonAction, DoButtonActionResult};
use streamduck_daemon::daemon_data::panels::{DropStackToRoot, DropStackToRootResult, ForciblyPopScreen, ForciblyPopScreenResult, GetButtonImages, GetButtonImagesResult, GetCurrentScreen, GetCurrentScreenResult, GetStack, GetStackNames, GetStackNamesResult, GetStackResult, PopScreen, PopScreenResult, PushScreen, PushScreenResult, ReplaceScreen, ReplaceScreenResult, ResetStack, ResetStackResult};
use streamduck_daemon::daemon_data::SocketAPIVersion;
use streamduck_daemon::WINDOWS_PIPE_NAME;
use std::io::Write;
use crate::{SDClientError, SDSyncEventClient, SDSyncRequestClient};
use crate::util::{process_request_no_buffer, process_request_without_data_no_buffer, read_response, read_socket};

/// Windows Named Pipe based Streamduck event client
pub struct WinEventClient {
    connection: RwLock<BufReader<PipeClient>>
}

impl WinEventClient {
    pub fn new() -> Result<Arc<dyn SDSyncEventClient>, std::io::Error> {
        let client: Arc<dyn SDSyncEventClient> = Arc::new(WinEventClient {
            connection: RwLock::new(BufReader::new(PipeClient::connect(WINDOWS_PIPE_NAME)?))
        });

        Ok(client)
    }

    fn get_handle(&self) -> RwLockWriteGuard<BufReader<PipeClient>> {
        self.connection.write().unwrap()
    }
}

impl SDSyncEventClient for WinEventClient {
    fn get_event(&self) -> Result<SDGlobalEvent, SDClientError> {
        loop {
            let packet = read_socket(self.get_handle().deref_mut())?;

            if packet.ty == "event" {
                if let Some(data) = packet.data {
                    return Ok(serde_json::from_value(data)?);
                }
            }
        }
    }
}

/// Windows Named Pipe based Streamduck request client
pub struct WinRequestClient {
    connection: RwLock<BufReader<PipeClient>>
}

impl WinRequestClient {
    pub fn new() -> Result<Arc<dyn SDSyncRequestClient>, std::io::Error> {
        let client: Arc<dyn SDSyncRequestClient> = Arc::new(WinRequestClient {
            connection: RwLock::new(BufReader::new(PipeClient::connect(WINDOWS_PIPE_NAME)?))
        });

        let daemon_version = client.version().expect("Failed to retrieve version");

        if daemon_version != SOCKET_API.1 {
            println!("[Warning] Version of client library doesn't match daemon API version. Client: {}, Daemon: {}", SOCKET_API.1, daemon_version);
        }

        Ok(client)
    }

    fn get_handle(&self) -> RwLockWriteGuard<BufReader<PipeClient>> {
        self.connection.write().unwrap()
    }
}

impl SDSyncRequestClient for WinRequestClient {
    fn version(&self) -> Result<String, SDClientError> {
        let response: SocketAPIVersion = process_request_without_data_no_buffer(self.get_handle().deref_mut())?;
        Ok(response.version)
    }

    fn device_list(&self) -> Result<Vec<Device>, SDClientError> {
        let response: ListDevices = process_request_without_data_no_buffer::<ListDevices, PipeClient>(self.get_handle().deref_mut())?;
        Ok(response.devices)
    }

    fn get_device(&self, serial_number: &str) -> Result<GetDeviceResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &GetDevice {
            serial_number: serial_number.to_string()
        })?)
    }

    fn add_device(&self, serial_number: &str) -> Result<AddDeviceResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &AddDevice {
            serial_number: serial_number.to_string()
        })?)
    }

    fn remove_device(&self, serial_number: &str) -> Result<RemoveDeviceResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &RemoveDevice {
            serial_number: serial_number.to_string()
        })?)
    }

    fn reload_device_configs(&self) -> Result<ReloadDeviceConfigsResult, SDClientError> {
        Ok(process_request_without_data_no_buffer(self.get_handle().deref_mut())?)
    }

    fn reload_device_config(&self, serial_number: &str) -> Result<ReloadDeviceConfigResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &ReloadDeviceConfig {
            serial_number: serial_number.to_string()
        })?)
    }

    fn save_device_configs(&self) -> Result<SaveDeviceConfigsResult, SDClientError> {
        Ok(process_request_without_data_no_buffer(self.get_handle().deref_mut())?)
    }

    fn save_device_config(&self, serial_number: &str) -> Result<SaveDeviceConfigResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &SaveDeviceConfig {
            serial_number: serial_number.to_string()
        })?)
    }

    fn get_device_config(&self, serial_number: &str) -> Result<GetDeviceConfigResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &GetDeviceConfig {
            serial_number: serial_number.to_string()
        })?)
    }

    fn import_device_config(&self, serial_number: &str, config: String) -> Result<ImportDeviceConfigResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &ImportDeviceConfig {
            serial_number: serial_number.to_string(),
            config
        })?)
    }

    fn export_device_config(&self, serial_number: &str) -> Result<ExportDeviceConfigResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &ExportDeviceConfig {
            serial_number: serial_number.to_string()
        })?)
    }

    fn set_brightness(&self, serial_number: &str, brightness: u8) -> Result<SetBrightnessResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &SetBrightness {
            serial_number: serial_number.to_string(),
            brightness
        })?)
    }

    fn list_images(&self, serial_number: &str) -> Result<ListImagesResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &ListImages {
            serial_number: serial_number.to_string()
        })?)
    }

    fn add_image(&self, serial_number: &str, image_data: &str) -> Result<AddImageResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &AddImage {
            serial_number: serial_number.to_string(),
            image_data: image_data.to_string()
        })?)
    }

    fn remove_image(&self, serial_number: &str, identifier: &str) -> Result<RemoveImageResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &RemoveImage {
            serial_number: serial_number.to_string(),
            image_identifier: identifier.to_string()
        })?)
    }

    fn list_fonts(&self) -> Result<Vec<String>, SDClientError> {
        let response: ListFonts = process_request_without_data_no_buffer(self.get_handle().deref_mut())?;
        Ok(response.font_names)
    }

    fn list_modules(&self) -> Result<Vec<PluginMetadata>, SDClientError> {
        let response: ListModules = process_request_without_data_no_buffer(self.get_handle().deref_mut())?;
        Ok(response.modules)
    }

    fn list_components(&self) -> Result<HashMap<String, HashMap<String, ComponentDefinition>>, SDClientError> {
        let response: ListComponents = process_request_without_data_no_buffer(self.get_handle().deref_mut())?;
        Ok(response.components)
    }

    fn get_module_values(&self, module_name: &str) -> Result<GetModuleValuesResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &GetModuleValues {
            module_name: module_name.to_string()
        })?)
    }

    fn add_module_value(&self, module_name: &str, path: &str) -> Result<AddModuleValueResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &AddModuleValue {
            module_name: module_name.to_string(),
            path: path.to_string()
        })?)
    }

    fn remove_module_value(&self, module_name: &str, path: &str, index: usize) -> Result<RemoveModuleValueResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &RemoveModuleValue {
            module_name: module_name.to_string(),
            path: path.to_string(),
            index
        })?)
    }

    fn set_module_value(&self, module_name: &str, value: UIPathValue) -> Result<SetModuleValueResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &SetModuleValue {
            module_name: module_name.to_string(),
            value
        })?)
    }

    fn get_stack(&self, serial_number: &str) -> Result<GetStackResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &GetStack {
            serial_number: serial_number.to_string()
        })?)
    }

    fn get_stack_names(&self, serial_number: &str) -> Result<GetStackNamesResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &GetStackNames {
            serial_number: serial_number.to_string()
        })?)
    }

    fn get_current_screen(&self, serial_number: &str) -> Result<GetCurrentScreenResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &GetCurrentScreen {
            serial_number: serial_number.to_string()
        })?)
    }

    fn get_button_images(&self, serial_number: &str) -> Result<GetButtonImagesResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &GetButtonImages {
            serial_number: serial_number.to_string()
        })?)
    }

    fn get_button(&self, serial_number: &str, key: u8) -> Result<GetButtonResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &GetButton {
            serial_number: serial_number.to_string(),
            key
        })?)
    }

    fn set_button(&self, serial_number: &str, key: u8, button: Button) -> Result<SetButtonResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &SetButton {
            serial_number: serial_number.to_string(),
            key,
            button
        })?)
    }

    fn clear_button(&self, serial_number: &str, key: u8) -> Result<ClearButtonResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &ClearButton {
            serial_number: serial_number.to_string(),
            key
        })?)
    }

    fn clipboard_status(&self) -> Result<ClipboardStatusResult, SDClientError> {
        Ok(process_request_without_data_no_buffer(self.get_handle().deref_mut())?)
    }

    fn copy_button(&self, serial_number: &str, key: u8) -> Result<CopyButtonResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &CopyButton {
            serial_number: serial_number.to_string(),
            key
        })?)
    }

    fn paste_button(&self, serial_number: &str, key: u8) -> Result<PasteButtonResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &PasteButton {
            serial_number: serial_number.to_string(),
            key
        })?)
    }

    fn new_button(&self, serial_number: &str, key: u8) -> Result<NewButtonResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &NewButton {
            serial_number: serial_number.to_string(),
            key
        })?)
    }

    fn new_button_from_component(&self, serial_number: &str, key: u8, component_name: &str) -> Result<NewButtonFromComponentResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &NewButtonFromComponent {
            serial_number: serial_number.to_string(),
            key,
            component_name: component_name.to_string()
        })?)
    }

    fn add_component(&self, serial_number: &str, key: u8, component_name: &str) -> Result<AddComponentResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &AddComponent {
            serial_number: serial_number.to_string(),
            key,
            component_name: component_name.to_string()
        })?)
    }

    fn get_component_values(&self, serial_number: &str, key: u8, component_name: &str) -> Result<GetComponentValuesResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &GetComponentValues {
            serial_number: serial_number.to_string(),
            key,
            component_name: component_name.to_string()
        })?)
    }

    fn add_component_value(&self, serial_number: &str, key: u8, component_name: &str, path: &str) -> Result<AddComponentValueResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &AddComponentValue {
            serial_number: serial_number.to_string(),
            key,
            component_name: component_name.to_string(),
            path: path.to_string()
        })?)
    }

    fn remove_component_value(&self, serial_number: &str, key: u8, component_name: &str, path: &str, index: usize) -> Result<RemoveComponentValueResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &RemoveComponentValue {
            serial_number: serial_number.to_string(),
            key,
            component_name: component_name.to_string(),
            path: path.to_string(),
            index
        })?)
    }

    fn set_component_value(&self, serial_number: &str, key: u8, component_name: &str, value: UIPathValue) -> Result<SetComponentValueResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &SetComponentValue {
            serial_number: serial_number.to_string(),
            key,
            component_name: component_name.to_string(),
            value
        })?)
    }

    fn remove_component(&self, serial_number: &str, key: u8, component_name: &str) -> Result<RemoveComponentResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &RemoveComponent {
            serial_number: serial_number.to_string(),
            key,
            component_name: component_name.to_string()
        })?)
    }

    fn push_screen(&self, serial_number: &str, screen: RawButtonPanel) -> Result<PushScreenResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &PushScreen {
            serial_number: serial_number.to_string(),
            screen
        })?)
    }

    fn pop_screen(&self, serial_number: &str) -> Result<PopScreenResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &PopScreen {
            serial_number: serial_number.to_string()
        })?)
    }

    fn forcibly_pop_screen(&self, serial_number: &str) -> Result<ForciblyPopScreenResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &ForciblyPopScreen {
            serial_number: serial_number.to_string()
        })?)
    }

    fn replace_screen(&self, serial_number: &str, screen: RawButtonPanel) -> Result<ReplaceScreenResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &ReplaceScreen {
            serial_number: serial_number.to_string(),
            screen
        })?)
    }

    fn reset_stack(&self, serial_number: &str, screen: RawButtonPanel) -> Result<ResetStackResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &ResetStack {
            serial_number: serial_number.to_string(),
            screen
        })?)
    }

    fn drop_stack_to_root(&self, serial_number: &str) -> Result<DropStackToRootResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &DropStackToRoot {
            serial_number: serial_number.to_string()
        })?)
    }

    fn commit_changes(&self, serial_number: &str) -> Result<CommitChangesToConfigResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &CommitChangesToConfig {
            serial_number: serial_number.to_string()
        })?)
    }

    fn do_button_action(&self, serial_number: &str, key: u8) -> Result<DoButtonActionResult, SDClientError> {
        Ok(process_request_no_buffer(self.get_handle().deref_mut(), &DoButtonAction {
            serial_number: serial_number.to_string(),
            key
        })?)
    }

    fn send_packet(&self, mut packet: SocketPacket) -> Result<SocketPacket, SDClientError> {
        let id = rand::thread_rng().sample_iter(&Alphanumeric).take(20).map(char::from).collect::<String>();
        packet.requester = Some(id.clone());

        let mut handle = self.get_handle();
        write!(handle.get_mut(), "{}\u{0004}", serde_json::to_string(&packet)?)?;
        read_response(handle.deref_mut(), &id, None)
    }

    fn send_packet_without_response(&self, packet: SocketPacket) -> Result<(), SDClientError> {
        let mut handle = self.get_handle();
        Ok(write!(handle.get_mut(), "{}\u{0004}", serde_json::to_string(&packet)?)?)
    }
}