use std::collections::HashMap;
use std::io::BufReader;
use std::ops::DerefMut;
use std::os::unix::net::UnixStream;
use std::sync::{Arc, RwLock, RwLockWriteGuard};
use rand::distributions::Alphanumeric;
use rand::Rng;

use streamduck_core::core::button::Button;
use streamduck_core::core::RawButtonPanel;
use streamduck_core::modules::components::{ComponentDefinition, UIPathValue};
use streamduck_core::modules::events::SDGlobalEvent;
use streamduck_core::modules::PluginMetadata;
use streamduck_core::versions::SOCKET_API;
use streamduck_core::socket::{send_packet_as_is, SocketPacket};
use streamduck_daemon::daemon_data::assets::{AddImage, AddImageResult, ListFonts, ListImages, ListImagesResult, RemoveImage, RemoveImageResult};
use streamduck_daemon::daemon_data::buttons::{AddComponent, AddComponentResult, AddComponentValue, AddComponentValueResult, ClearButton, ClearButtonResult, ClipboardStatusResult, CopyButton, CopyButtonResult, GetButton, GetButtonResult, GetComponentValues, GetComponentValuesResult, NewButton, NewButtonFromComponent, NewButtonFromComponentResult, NewButtonResult, PasteButton, PasteButtonResult, RemoveComponent, RemoveComponentResult, RemoveComponentValue, RemoveComponentValueResult, SetButton, SetButtonResult, SetComponentValue, SetComponentValueResult};
use streamduck_daemon::daemon_data::config::{ExportDeviceConfig, ExportDeviceConfigResult, GetDeviceConfig, GetDeviceConfigResult, ImportDeviceConfig, ImportDeviceConfigResult, ReloadDeviceConfig, ReloadDeviceConfigResult, ReloadDeviceConfigsResult, SaveDeviceConfig, SaveDeviceConfigResult, SaveDeviceConfigsResult};
use streamduck_daemon::daemon_data::devices::{AddDevice, AddDeviceResult, Device, GetDevice, GetDeviceResult, ListDevices, RemoveDevice, RemoveDeviceResult, SetBrightness, SetBrightnessResult};
use streamduck_daemon::daemon_data::modules::{AddModuleValue, AddModuleValueResult, GetModuleValues, GetModuleValuesResult, ListComponents, ListModules, RemoveModuleValue, RemoveModuleValueResult, SetModuleValue, SetModuleValueResult};
use streamduck_daemon::daemon_data::ops::{CommitChangesToConfig, CommitChangesToConfigResult, DoButtonAction, DoButtonActionResult};
use streamduck_daemon::daemon_data::panels::{DropStackToRoot, DropStackToRootResult, ForciblyPopScreen, ForciblyPopScreenResult, GetButtonImages, GetButtonImagesResult, GetCurrentScreen, GetCurrentScreenResult, GetStack, GetStackNames, GetStackNamesResult, GetStackResult, PopScreen, PopScreenResult, PushScreen, PushScreenResult, ReplaceScreen, ReplaceScreenResult, ResetStack, ResetStackResult};
use streamduck_daemon::daemon_data::SocketAPIVersion;
use streamduck_daemon::UNIX_SOCKET_PATH;

use crate::{SDSyncRequestClient, SDClientError, SDSyncEventClient, SDSyncClient, SDSyncUpcastRequestClient, SDSyncUpcastEventClient};
use crate::util::{process_request, process_request_without_data, read_response, read_socket};

/// Unix Socket based Streamduck client
pub struct UnixClient {
    connection: RwLock<BufReader<UnixStream>>,
    event_buffer: RwLock<Vec<SDGlobalEvent>>
}

#[allow(dead_code)]
impl UnixClient {
    fn make_client() -> Result<UnixClient, std::io::Error> {
        let client = UnixClient {
            connection: RwLock::new(BufReader::new(UnixStream::connect(UNIX_SOCKET_PATH)?)),
            event_buffer: Default::default()
        };

        let daemon_version = client.version().expect("Failed to retrieve version");

        if daemon_version != SOCKET_API.1 {
            println!("[Warning] Version of client library doesn't match daemon API version. Client: {}, Daemon: {}", SOCKET_API.1, daemon_version);
        }

        Ok(client)
    }

    /// Initializes client using unix domain socket
    pub fn new() -> Result<Arc<dyn SDSyncClient>, std::io::Error> {
        Ok(Arc::new(UnixClient::make_client()?))
    }

    fn get_handle(&self) -> RwLockWriteGuard<BufReader<UnixStream>> {
        self.connection.write().unwrap()
    }
}

impl SDSyncRequestClient for UnixClient {
    fn version(&self) -> Result<String, SDClientError> {
        let response: SocketAPIVersion = process_request_without_data(self.get_handle().deref_mut(), Some(self.event_buffer.write().unwrap()))?;

        Ok(response.version)
    }

    fn device_list(&self) -> Result<Vec<Device>, SDClientError> {
        let response: ListDevices = process_request_without_data(self.get_handle().deref_mut(), Some(self.event_buffer.write().unwrap()))?;

        Ok(response.devices)
    }

    fn get_device(&self, serial_number: &str) -> Result<GetDeviceResult, SDClientError> {
        let response: GetDeviceResult = process_request(self.get_handle().deref_mut(), &GetDevice {
            serial_number: serial_number.to_string()
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn add_device(&self, serial_number: &str) -> Result<AddDeviceResult, SDClientError> {
        let response: AddDeviceResult = process_request(self.get_handle().deref_mut(), &AddDevice {
            serial_number: serial_number.to_string()
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn remove_device(&self, serial_number: &str) -> Result<RemoveDeviceResult, SDClientError> {
        let response: RemoveDeviceResult = process_request(self.get_handle().deref_mut(), &RemoveDevice {
            serial_number: serial_number.to_string()
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn reload_device_configs(&self) -> Result<ReloadDeviceConfigsResult, SDClientError> {
        let response: ReloadDeviceConfigsResult = process_request_without_data(self.get_handle().deref_mut(), Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn reload_device_config(&self, serial_number: &str) -> Result<ReloadDeviceConfigResult, SDClientError> {
        let response: ReloadDeviceConfigResult = process_request(self.get_handle().deref_mut(), &ReloadDeviceConfig {
            serial_number: serial_number.to_string()
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn save_device_configs(&self) -> Result<SaveDeviceConfigsResult, SDClientError> {
        let response: SaveDeviceConfigsResult = process_request_without_data(self.get_handle().deref_mut(), Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn save_device_config(&self, serial_number: &str) -> Result<SaveDeviceConfigResult, SDClientError> {
        let response: SaveDeviceConfigResult = process_request(self.get_handle().deref_mut(), &SaveDeviceConfig {
            serial_number: serial_number.to_string()
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn get_device_config(&self, serial_number: &str) -> Result<GetDeviceConfigResult, SDClientError> {
        let response: GetDeviceConfigResult = process_request(self.get_handle().deref_mut(), &GetDeviceConfig {
            serial_number: serial_number.to_string()
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn import_device_config(&self, serial_number: &str, config: String) -> Result<ImportDeviceConfigResult, SDClientError> {
        let response: ImportDeviceConfigResult = process_request(self.get_handle().deref_mut(), &ImportDeviceConfig {
            serial_number: serial_number.to_string(),
            config
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn export_device_config(&self, serial_number: &str) -> Result<ExportDeviceConfigResult, SDClientError> {
        let response: ExportDeviceConfigResult = process_request(self.get_handle().deref_mut(), &ExportDeviceConfig {
            serial_number: serial_number.to_string()
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn set_brightness(&self, serial_number: &str, brightness: u8) -> Result<SetBrightnessResult, SDClientError> {
        let response: SetBrightnessResult = process_request(self.get_handle().deref_mut(), &SetBrightness {
            serial_number: serial_number.to_string(),
            brightness
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn list_images(&self, serial_number: &str) -> Result<ListImagesResult, SDClientError> {
        let response: ListImagesResult = process_request(self.get_handle().deref_mut(), &ListImages {
            serial_number: serial_number.to_string()
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn add_image(&self, serial_number: &str, image_data: &str) -> Result<AddImageResult, SDClientError> {
        let response: AddImageResult = process_request(self.get_handle().deref_mut(), &AddImage {
            serial_number: serial_number.to_string(),
            image_data: image_data.to_string()
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn remove_image(&self, serial_number: &str, identifier: &str) -> Result<RemoveImageResult, SDClientError> {
        let response: RemoveImageResult = process_request(self.get_handle().deref_mut(), &RemoveImage {
            serial_number: serial_number.to_string(),
            image_identifier: identifier.to_string()
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn list_fonts(&self) -> Result<Vec<String>, SDClientError> {
        let response: ListFonts = process_request_without_data(self.get_handle().deref_mut(), Some(self.event_buffer.write().unwrap()))?;

        Ok(response.font_names)
    }

    fn list_modules(&self) -> Result<Vec<PluginMetadata>, SDClientError> {
        let response: ListModules = process_request_without_data(self.get_handle().deref_mut(), Some(self.event_buffer.write().unwrap()))?;

        Ok(response.modules)
    }

    fn list_components(&self) -> Result<HashMap<String, HashMap<String, ComponentDefinition>>, SDClientError> {
        let response: ListComponents = process_request_without_data(self.get_handle().deref_mut(), Some(self.event_buffer.write().unwrap()))?;

        Ok(response.components)
    }

    fn get_module_values(&self, module_name: &str) -> Result<GetModuleValuesResult, SDClientError> {
        let response: GetModuleValuesResult = process_request(self.get_handle().deref_mut(), &GetModuleValues {
            module_name: module_name.to_string()
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn add_module_value(&self, module_name: &str, path: &str) -> Result<AddModuleValueResult, SDClientError> {
        let response: AddModuleValueResult = process_request(self.get_handle().deref_mut(), &AddModuleValue {
            module_name: module_name.to_string(),
            path: path.to_string()
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn remove_module_value(&self, module_name: &str, path: &str, index: usize) -> Result<RemoveModuleValueResult, SDClientError> {
        let response: RemoveModuleValueResult = process_request(self.get_handle().deref_mut(), &RemoveModuleValue {
            module_name: module_name.to_string(),
            path: path.to_string(),
            index
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn set_module_value(&self, module_name: &str, value: UIPathValue) -> Result<SetModuleValueResult, SDClientError> {
        let response: SetModuleValueResult = process_request(self.get_handle().deref_mut(), &SetModuleValue {
            module_name: module_name.to_string(),
            value
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn get_stack(&self, serial_number: &str) -> Result<GetStackResult, SDClientError> {
        let response: GetStackResult = process_request(self.get_handle().deref_mut(), &GetStack {
            serial_number: serial_number.to_string()
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn get_stack_names(&self, serial_number: &str) -> Result<GetStackNamesResult, SDClientError> {
        let response: GetStackNamesResult = process_request(self.get_handle().deref_mut(), &GetStackNames {
            serial_number: serial_number.to_string()
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn get_current_screen(&self, serial_number: &str) -> Result<GetCurrentScreenResult, SDClientError> {
        let response: GetCurrentScreenResult = process_request(self.get_handle().deref_mut(), &GetCurrentScreen {
            serial_number: serial_number.to_string()
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn get_button_images(&self, serial_number: &str) -> Result<GetButtonImagesResult, SDClientError> {
        let response: GetButtonImagesResult = process_request(self.get_handle().deref_mut(), &GetButtonImages {
            serial_number: serial_number.to_string()
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn get_button(&self, serial_number: &str, key: u8) -> Result<GetButtonResult, SDClientError> {
        let response: GetButtonResult = process_request(self.get_handle().deref_mut(), &GetButton {
            serial_number: serial_number.to_string(),
            key
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn set_button(&self, serial_number: &str, key: u8, button: Button) -> Result<SetButtonResult, SDClientError> {
        let response: SetButtonResult = process_request(self.get_handle().deref_mut(), &SetButton {
            serial_number: serial_number.to_string(),
            key,
            button
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn clear_button(&self, serial_number: &str, key: u8) -> Result<ClearButtonResult, SDClientError> {
        let response: ClearButtonResult = process_request(self.get_handle().deref_mut(), &ClearButton {
            serial_number: serial_number.to_string(),
            key
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn clipboard_status(&self) -> Result<ClipboardStatusResult, SDClientError> {
        let response: ClipboardStatusResult = process_request_without_data(self.get_handle().deref_mut(), Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn copy_button(&self, serial_number: &str, key: u8) -> Result<CopyButtonResult, SDClientError> {
        let response: CopyButtonResult = process_request(self.get_handle().deref_mut(), &CopyButton {
            serial_number: serial_number.to_string(),
            key
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn paste_button(&self, serial_number: &str, key: u8) -> Result<PasteButtonResult, SDClientError> {
        let response: PasteButtonResult = process_request(self.get_handle().deref_mut(), &PasteButton {
            serial_number: serial_number.to_string(),
            key
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn new_button(&self, serial_number: &str, key: u8) -> Result<NewButtonResult, SDClientError> {
        let response: NewButtonResult = process_request(self.get_handle().deref_mut(), &NewButton {
            serial_number: serial_number.to_string(),
            key
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn new_button_from_component(&self, serial_number: &str, key: u8, component_name: &str) -> Result<NewButtonFromComponentResult, SDClientError> {
        let response: NewButtonFromComponentResult = process_request(self.get_handle().deref_mut(), &NewButtonFromComponent {
            serial_number: serial_number.to_string(),
            key,
            component_name: component_name.to_string()
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn add_component(&self, serial_number: &str, key: u8, component_name: &str) -> Result<AddComponentResult, SDClientError> {
        let response: AddComponentResult = process_request(self.get_handle().deref_mut(), &AddComponent {
            serial_number: serial_number.to_string(),
            key,
            component_name: component_name.to_string()
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn get_component_values(&self, serial_number: &str, key: u8, component_name: &str) -> Result<GetComponentValuesResult, SDClientError> {
        let response: GetComponentValuesResult = process_request(self.get_handle().deref_mut(), &GetComponentValues {
            serial_number: serial_number.to_string(),
            key,
            component_name: component_name.to_string()
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn add_component_value(&self, serial_number: &str, key: u8, component_name: &str, path: &str) -> Result<AddComponentValueResult, SDClientError> {
        let response: AddComponentValueResult = process_request(self.get_handle().deref_mut(), &AddComponentValue {
            serial_number: serial_number.to_string(),
            key,
            component_name: component_name.to_string(),
            path: path.to_string()
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn remove_component_value(&self, serial_number: &str, key: u8, component_name: &str, path: &str, index: usize) -> Result<RemoveComponentValueResult, SDClientError> {
        let response: RemoveComponentValueResult = process_request(self.get_handle().deref_mut(), &RemoveComponentValue {
            serial_number: serial_number.to_string(),
            key,
            component_name: component_name.to_string(),
            path: path.to_string(),
            index
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn set_component_value(&self, serial_number: &str, key: u8, component_name: &str, value: UIPathValue) -> Result<SetComponentValueResult, SDClientError> {
        let response: SetComponentValueResult = process_request(self.get_handle().deref_mut(), &SetComponentValue {
            serial_number: serial_number.to_string(),
            key,
            component_name: component_name.to_string(),
            value
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn remove_component(&self, serial_number: &str, key: u8, component_name: &str) -> Result<RemoveComponentResult, SDClientError> {
        let response: RemoveComponentResult = process_request(self.get_handle().deref_mut(), &RemoveComponent {
            serial_number: serial_number.to_string(),
            key,
            component_name: component_name.to_string()
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn push_screen(&self, serial_number: &str, screen: RawButtonPanel) -> Result<PushScreenResult, SDClientError> {
        let response: PushScreenResult = process_request(self.get_handle().deref_mut(), &PushScreen {
            serial_number: serial_number.to_string(),
            screen
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn pop_screen(&self, serial_number: &str) -> Result<PopScreenResult, SDClientError> {
        let response: PopScreenResult = process_request(self.get_handle().deref_mut(), &PopScreen {
            serial_number: serial_number.to_string()
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn forcibly_pop_screen(&self, serial_number: &str) -> Result<ForciblyPopScreenResult, SDClientError> {
        let response: ForciblyPopScreenResult = process_request(self.get_handle().deref_mut(), &ForciblyPopScreen {
            serial_number: serial_number.to_string()
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn replace_screen(&self, serial_number: &str, screen: RawButtonPanel) -> Result<ReplaceScreenResult, SDClientError> {
        let response: ReplaceScreenResult = process_request(self.get_handle().deref_mut(), &ReplaceScreen {
            serial_number: serial_number.to_string(),
            screen
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn reset_stack(&self, serial_number: &str, screen: RawButtonPanel) -> Result<ResetStackResult, SDClientError> {
        let response: ResetStackResult = process_request(self.get_handle().deref_mut(), &ResetStack {
            serial_number: serial_number.to_string(),
            screen
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn drop_stack_to_root(&self, serial_number: &str) -> Result<DropStackToRootResult, SDClientError> {
        let response: DropStackToRootResult = process_request(self.get_handle().deref_mut(), &DropStackToRoot {
            serial_number: serial_number.to_string()
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn commit_changes(&self, serial_number: &str) -> Result<CommitChangesToConfigResult, SDClientError> {
        let response: CommitChangesToConfigResult = process_request(self.get_handle().deref_mut(), &CommitChangesToConfig {
            serial_number: serial_number.to_string()
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn do_button_action(&self, serial_number: &str, key: u8) -> Result<DoButtonActionResult, SDClientError> {
        let response: DoButtonActionResult = process_request(self.get_handle().deref_mut(), &DoButtonAction {
            serial_number: serial_number.to_string(),
            key
        }, Some(self.event_buffer.write().unwrap()))?;

        Ok(response)
    }

    fn send_packet(&self, mut packet: SocketPacket) -> Result<SocketPacket, SDClientError> {
        let id = rand::thread_rng().sample_iter(&Alphanumeric).take(20).map(char::from).collect::<String>();
        packet.requester = Some(id.clone());

        let mut handle = self.connection.write().unwrap();
        send_packet_as_is(handle.get_mut(), packet)?;

        read_response(handle.deref_mut(), &id, Some(self.event_buffer.write().unwrap()))
    }

    fn send_packet_without_response(&self, packet: SocketPacket) -> Result<(), SDClientError> {
        let mut handle = self.connection.write().unwrap();
        send_packet_as_is(handle.get_mut(), packet)?;
        Ok(())
    }
}

impl SDSyncEventClient for UnixClient {
    fn get_event(&self) -> Result<SDGlobalEvent, SDClientError> {
        let buffer = self.event_buffer.write().unwrap();

        if let Some(event) = buffer.pop() {
            return Ok(event);
        }

        drop(buffer);


        loop {
            let packet = read_socket(self.get_handle().deref_mut())?;

            if let Some(data) = packet.data {
                return Ok(serde_json::from_value(data)?);
            }
        }
    }
}


impl SDSyncUpcastRequestClient for UnixClient {
    fn as_request(self: Arc<Self>) -> Arc<dyn SDSyncRequestClient> {
        self
    }
}

impl SDSyncUpcastEventClient for UnixClient {
    fn as_event(self: Arc<Self>) -> Arc<dyn SDSyncEventClient> {
        self
    }
}

impl SDSyncClient for UnixClient {}