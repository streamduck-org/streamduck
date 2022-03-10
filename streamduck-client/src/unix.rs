use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::sync::{Arc, RwLock};

use serde::Serialize;
use serde::de::DeserializeOwned;

use streamduck_core::core::button::Button;
use streamduck_core::core::RawButtonPanel;
use streamduck_core::modules::components::{ComponentDefinition, UIPathValue};
use streamduck_core::modules::PluginMetadata;
use streamduck_core::versions::SOCKET_API;
use streamduck_core::socket::{parse_packet_to_data, send_no_data_packet_with_requester, send_packet_with_requester, SocketData, SocketPacket};
use streamduck_daemon::daemon_data::assets::{AddImage, AddImageResult, ListFonts, ListImages, ListImagesResult, RemoveImage, RemoveImageResult};
use streamduck_daemon::daemon_data::buttons::{AddComponent, AddComponentResult, AddComponentValue, AddComponentValueResult, ClearButton, ClearButtonResult, GetButton, GetButtonResult, GetComponentValues, GetComponentValuesResult, NewButton, NewButtonFromComponent, NewButtonFromComponentResult, NewButtonResult, RemoveComponent, RemoveComponentResult, RemoveComponentValue, RemoveComponentValueResult, SetButton, SetButtonResult, SetComponentValue, SetComponentValueResult};
use streamduck_daemon::daemon_data::config::{ExportDeviceConfig, ExportDeviceConfigResult, GetDeviceConfig, GetDeviceConfigResult, ImportDeviceConfig, ImportDeviceConfigResult, ReloadDeviceConfig, ReloadDeviceConfigResult, ReloadDeviceConfigsResult, SaveDeviceConfig, SaveDeviceConfigResult, SaveDeviceConfigsResult};
use streamduck_daemon::daemon_data::devices::{AddDevice, AddDeviceResult, Device, GetDevice, GetDeviceResult, ListDevices, RemoveDevice, RemoveDeviceResult, SetBrightness, SetBrightnessResult};
use streamduck_daemon::daemon_data::modules::{AddModuleValue, AddModuleValueResult, GetModuleValues, GetModuleValuesResult, ListComponents, ListModules, RemoveModuleValue, RemoveModuleValueResult, SetModuleValue, SetModuleValueResult};
use streamduck_daemon::daemon_data::ops::{CommitChangesToConfig, CommitChangesToConfigResult, DoButtonAction, DoButtonActionResult};
use streamduck_daemon::daemon_data::panels::{DropStackToRoot, DropStackToRootResult, ForciblyPopScreen, ForciblyPopScreenResult, GetButtonImages, GetButtonImagesResult, GetCurrentScreen, GetCurrentScreenResult, GetStack, GetStackNames, GetStackNamesResult, GetStackResult, PopScreen, PopScreenResult, PushScreen, PushScreenResult, ReplaceScreen, ReplaceScreenResult, ResetStack, ResetStackResult};
use streamduck_daemon::daemon_data::SocketAPIVersion;

use crate::{SDClient, SDClientError};

/// Definition of Unix Socket based client
pub struct UnixClient {
    connection: RwLock<BufReader<UnixStream>>
}

#[allow(dead_code)]
impl UnixClient {
    /// Initializes client using unix domain socket
    pub fn new() -> Result<Arc<Box<dyn SDClient>>, std::io::Error> {
        let client: Arc<Box<dyn SDClient>> = Arc::new(Box::new(UnixClient {
            connection: RwLock::new(BufReader::new(UnixStream::connect("/tmp/streamduck.sock")?))
        }));

        let daemon_version = client.version().expect("Failed to retrieve version");

        if daemon_version != SOCKET_API.1 {
            println!("[Warning] Version of client library doesn't match daemon API version. Client: {}, Daemon: {}", SOCKET_API.1, daemon_version);
        }

        Ok(client)
    }

    fn process_request<Req: SocketData + Serialize, Res: SocketData + DeserializeOwned>(&self, request: &Req) -> Result<Res, SDClientError> {
        let mut handle = self.connection.write().unwrap();

        send_packet_with_requester(handle.get_mut(), "", request)?;

        let mut byte_array = vec![];
        handle.read_until(0x4, &mut byte_array)?;

        let line = String::from_utf8(byte_array)?;

        let packet: SocketPacket = serde_json::from_str(line.replace("\u{0004}", "").trim())?;

        Ok(parse_packet_to_data(&packet)?)
    }

    fn process_request_without_data<Res: SocketData + DeserializeOwned>(&self) -> Result<Res, SDClientError> {
        let mut handle = self.connection.write().unwrap();

        send_no_data_packet_with_requester::<Res>(handle.get_mut(), "")?;

        let mut byte_array = vec![];
        handle.read_until(0x4, &mut byte_array)?;

        let line = String::from_utf8(byte_array)?;

        let packet: SocketPacket = serde_json::from_str(line.replace("\u{0004}", "").trim())?;

        Ok(parse_packet_to_data(&packet)?)
    }
}

impl SDClient for UnixClient {
    fn version(&self) -> Result<String, SDClientError> {
        let response: SocketAPIVersion = self.process_request_without_data()?;

        Ok(response.version)
    }

    fn device_list(&self) -> Result<Vec<Device>, SDClientError> {
        let response: ListDevices = self.process_request_without_data()?;

        Ok(response.devices)
    }

    fn get_device(&self, serial_number: &str) -> Result<GetDeviceResult, SDClientError> {
        let response: GetDeviceResult = self.process_request(&GetDevice {
            serial_number: serial_number.to_string()
        })?;

        Ok(response)
    }

    fn add_device(&self, serial_number: &str) -> Result<AddDeviceResult, SDClientError> {
        let response: AddDeviceResult = self.process_request(&AddDevice {
            serial_number: serial_number.to_string()
        })?;

        Ok(response)
    }

    fn remove_device(&self, serial_number: &str) -> Result<RemoveDeviceResult, SDClientError> {
        let response: RemoveDeviceResult = self.process_request(&RemoveDevice {
            serial_number: serial_number.to_string()
        })?;

        Ok(response)
    }

    fn reload_device_configs(&self) -> Result<ReloadDeviceConfigsResult, SDClientError> {
        let response: ReloadDeviceConfigsResult = self.process_request_without_data()?;

        Ok(response)
    }

    fn reload_device_config(&self, serial_number: &str) -> Result<ReloadDeviceConfigResult, SDClientError> {
        let response: ReloadDeviceConfigResult = self.process_request(&ReloadDeviceConfig {
            serial_number: serial_number.to_string()
        })?;

        Ok(response)
    }

    fn save_device_configs(&self) -> Result<SaveDeviceConfigsResult, SDClientError> {
        let response: SaveDeviceConfigsResult = self.process_request_without_data()?;

        Ok(response)
    }

    fn save_device_config(&self, serial_number: &str) -> Result<SaveDeviceConfigResult, SDClientError> {
        let response: SaveDeviceConfigResult = self.process_request(&SaveDeviceConfig {
            serial_number: serial_number.to_string()
        })?;

        Ok(response)
    }

    fn get_device_config(&self, serial_number: &str) -> Result<GetDeviceConfigResult, SDClientError> {
        let response: GetDeviceConfigResult = self.process_request(&GetDeviceConfig {
            serial_number: serial_number.to_string()
        })?;

        Ok(response)
    }

    fn import_device_config(&self, serial_number: &str, config: String) -> Result<ImportDeviceConfigResult, SDClientError> {
        let response: ImportDeviceConfigResult = self.process_request(&ImportDeviceConfig {
            serial_number: serial_number.to_string(),
            config
        })?;

        Ok(response)
    }

    fn export_device_config(&self, serial_number: &str) -> Result<ExportDeviceConfigResult, SDClientError> {
        let response: ExportDeviceConfigResult = self.process_request(&ExportDeviceConfig {
            serial_number: serial_number.to_string()
        })?;

        Ok(response)
    }

    fn set_brightness(&self, serial_number: &str, brightness: u8) -> Result<SetBrightnessResult, SDClientError> {
        let response: SetBrightnessResult = self.process_request(&SetBrightness {
            serial_number: serial_number.to_string(),
            brightness
        })?;

        Ok(response)
    }

    fn list_images(&self, serial_number: &str) -> Result<ListImagesResult, SDClientError> {
        let response: ListImagesResult = self.process_request(&ListImages {
            serial_number: serial_number.to_string()
        })?;

        Ok(response)
    }

    fn add_image(&self, serial_number: &str, image_data: &str) -> Result<AddImageResult, SDClientError> {
        let response: AddImageResult = self.process_request(&AddImage {
            serial_number: serial_number.to_string(),
            image_data: image_data.to_string()
        })?;

        Ok(response)
    }

    fn remove_image(&self, serial_number: &str, identifier: &str) -> Result<RemoveImageResult, SDClientError> {
        let response: RemoveImageResult = self.process_request(&RemoveImage {
            serial_number: serial_number.to_string(),
            image_identifier: identifier.to_string()
        })?;

        Ok(response)
    }

    fn list_fonts(&self) -> Result<Vec<String>, SDClientError> {
        let response: ListFonts = self.process_request_without_data()?;

        Ok(response.font_names)
    }

    fn list_modules(&self) -> Result<Vec<PluginMetadata>, SDClientError> {
        let response: ListModules = self.process_request_without_data()?;

        Ok(response.modules)
    }

    fn list_components(&self) -> Result<HashMap<String, HashMap<String, ComponentDefinition>>, SDClientError> {
        let response: ListComponents = self.process_request_without_data()?;

        Ok(response.components)
    }

    fn get_module_values(&self, module_name: &str) -> Result<GetModuleValuesResult, SDClientError> {
        let response: GetModuleValuesResult = self.process_request(&GetModuleValues {
            module_name: module_name.to_string()
        })?;

        Ok(response)
    }

    fn add_module_value(&self, module_name: &str, path: &str) -> Result<AddModuleValueResult, SDClientError> {
        let response: AddModuleValueResult = self.process_request(&AddModuleValue {
            module_name: module_name.to_string(),
            path: path.to_string()
        })?;

        Ok(response)
    }

    fn remove_module_value(&self, module_name: &str, path: &str, index: usize) -> Result<RemoveModuleValueResult, SDClientError> {
        let response: RemoveModuleValueResult = self.process_request(&RemoveModuleValue {
            module_name: module_name.to_string(),
            path: path.to_string(),
            index
        })?;

        Ok(response)
    }

    fn set_module_value(&self, module_name: &str, value: UIPathValue) -> Result<SetModuleValueResult, SDClientError> {
        let response: SetModuleValueResult = self.process_request(&SetModuleValue {
            module_name: module_name.to_string(),
            value
        })?;

        Ok(response)
    }

    fn get_stack(&self, serial_number: &str) -> Result<GetStackResult, SDClientError> {
        let response: GetStackResult = self.process_request(&GetStack {
            serial_number: serial_number.to_string()
        })?;

        Ok(response)
    }

    fn get_stack_names(&self, serial_number: &str) -> Result<GetStackNamesResult, SDClientError> {
        let response: GetStackNamesResult = self.process_request(&GetStackNames {
            serial_number: serial_number.to_string()
        })?;

        Ok(response)
    }

    fn get_current_screen(&self, serial_number: &str) -> Result<GetCurrentScreenResult, SDClientError> {
        let response: GetCurrentScreenResult = self.process_request(&GetCurrentScreen {
            serial_number: serial_number.to_string()
        })?;

        Ok(response)
    }

    fn get_button_images(&self, serial_number: &str) -> Result<GetButtonImagesResult, SDClientError> {
        let response: GetButtonImagesResult = self.process_request(&GetButtonImages {
            serial_number: serial_number.to_string()
        })?;

        Ok(response)
    }

    fn get_button(&self, serial_number: &str, key: u8) -> Result<GetButtonResult, SDClientError> {
        let response: GetButtonResult = self.process_request(&GetButton {
            serial_number: serial_number.to_string(),
            key
        })?;

        Ok(response)
    }

    fn set_button(&self, serial_number: &str, key: u8, button: Button) -> Result<SetButtonResult, SDClientError> {
        let response: SetButtonResult = self.process_request(&SetButton {
            serial_number: serial_number.to_string(),
            key,
            button
        })?;

        Ok(response)
    }

    fn clear_button(&self, serial_number: &str, key: u8) -> Result<ClearButtonResult, SDClientError> {
        let response: ClearButtonResult = self.process_request(&ClearButton {
            serial_number: serial_number.to_string(),
            key
        })?;

        Ok(response)
    }

    fn new_button(&self, serial_number: &str, key: u8) -> Result<NewButtonResult, SDClientError> {
        let response: NewButtonResult = self.process_request(&NewButton {
            serial_number: serial_number.to_string(),
            key
        })?;

        Ok(response)
    }

    fn new_button_from_component(&self, serial_number: &str, key: u8, component_name: &str) -> Result<NewButtonFromComponentResult, SDClientError> {
        let response: NewButtonFromComponentResult = self.process_request(&NewButtonFromComponent {
            serial_number: serial_number.to_string(),
            key,
            component_name: component_name.to_string()
        })?;

        Ok(response)
    }

    fn add_component(&self, serial_number: &str, key: u8, component_name: &str) -> Result<AddComponentResult, SDClientError> {
        let response: AddComponentResult = self.process_request(&AddComponent {
            serial_number: serial_number.to_string(),
            key,
            component_name: component_name.to_string()
        })?;

        Ok(response)
    }

    fn get_component_values(&self, serial_number: &str, key: u8, component_name: &str) -> Result<GetComponentValuesResult, SDClientError> {
        let response: GetComponentValuesResult = self.process_request(&GetComponentValues {
            serial_number: serial_number.to_string(),
            key,
            component_name: component_name.to_string()
        })?;

        Ok(response)
    }

    fn add_component_value(&self, serial_number: &str, key: u8, component_name: &str, path: &str) -> Result<AddComponentValueResult, SDClientError> {
        let response: AddComponentValueResult = self.process_request(&AddComponentValue {
            serial_number: serial_number.to_string(),
            key,
            component_name: component_name.to_string(),
            path: path.to_string()
        })?;

        Ok(response)
    }

    fn remove_component_value(&self, serial_number: &str, key: u8, component_name: &str, path: &str, index: usize) -> Result<RemoveComponentValueResult, SDClientError> {
        let response: RemoveComponentValueResult = self.process_request(&RemoveComponentValue {
            serial_number: serial_number.to_string(),
            key,
            component_name: component_name.to_string(),
            path: path.to_string(),
            index
        })?;

        Ok(response)
    }

    fn set_component_value(&self, serial_number: &str, key: u8, component_name: &str, value: UIPathValue) -> Result<SetComponentValueResult, SDClientError> {
        let response: SetComponentValueResult = self.process_request(&SetComponentValue {
            serial_number: serial_number.to_string(),
            key,
            component_name: component_name.to_string(),
            value
        })?;

        Ok(response)
    }

    fn remove_component(&self, serial_number: &str, key: u8, component_name: &str) -> Result<RemoveComponentResult, SDClientError> {
        let response: RemoveComponentResult = self.process_request(&RemoveComponent {
            serial_number: serial_number.to_string(),
            key,
            component_name: component_name.to_string()
        })?;

        Ok(response)
    }

    fn push_screen(&self, serial_number: &str, screen: RawButtonPanel) -> Result<PushScreenResult, SDClientError> {
        let response: PushScreenResult = self.process_request(&PushScreen {
            serial_number: serial_number.to_string(),
            screen
        })?;

        Ok(response)
    }

    fn pop_screen(&self, serial_number: &str) -> Result<PopScreenResult, SDClientError> {
        let response: PopScreenResult = self.process_request(&PopScreen {
            serial_number: serial_number.to_string()
        })?;

        Ok(response)
    }

    fn forcibly_pop_screen(&self, serial_number: &str) -> Result<ForciblyPopScreenResult, SDClientError> {
        let response: ForciblyPopScreenResult = self.process_request(&ForciblyPopScreen {
            serial_number: serial_number.to_string()
        })?;

        Ok(response)
    }

    fn replace_screen(&self, serial_number: &str, screen: RawButtonPanel) -> Result<ReplaceScreenResult, SDClientError> {
        let response: ReplaceScreenResult = self.process_request(&ReplaceScreen {
            serial_number: serial_number.to_string(),
            screen
        })?;

        Ok(response)
    }

    fn reset_stack(&self, serial_number: &str, screen: RawButtonPanel) -> Result<ResetStackResult, SDClientError> {
        let response: ResetStackResult = self.process_request(&ResetStack {
            serial_number: serial_number.to_string(),
            screen
        })?;

        Ok(response)
    }

    fn drop_stack_to_root(&self, serial_number: &str) -> Result<DropStackToRootResult, SDClientError> {
        let response: DropStackToRootResult = self.process_request(&DropStackToRoot {
            serial_number: serial_number.to_string()
        })?;

        Ok(response)
    }

    fn commit_changes(&self, serial_number: &str) -> Result<CommitChangesToConfigResult, SDClientError> {
        let response: CommitChangesToConfigResult = self.process_request(&CommitChangesToConfig {
            serial_number: serial_number.to_string()
        })?;

        Ok(response)
    }

    fn do_button_action(&self, serial_number: &str, key: u8) -> Result<DoButtonActionResult, SDClientError> {
        let response: DoButtonActionResult = self.process_request(&DoButtonAction {
            serial_number: serial_number.to_string(),
            key
        })?;

        Ok(response)
    }

    fn send_packet(&self, packet: SocketPacket) -> Result<SocketPacket, SDClientError> {
        let mut handle = self.connection.write().unwrap();
        writeln!(handle.get_mut(), "{}", serde_json::to_string(&packet)?)?;

        let mut line = String::new();
        handle.read_line(&mut line)?;

        Ok(serde_json::from_str(&line)?)
    }

    fn send_packet_without_response(&self, packet: SocketPacket) -> Result<(), SDClientError> {
        let mut handle = self.connection.write().unwrap();
        writeln!(handle.get_mut(), "{}", serde_json::to_string(&packet)?)?;
        Ok(())
    }
}