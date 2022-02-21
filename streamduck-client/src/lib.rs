use std::collections::HashMap;
use std::io::Error;
use std::string::FromUtf8Error;
use streamduck_core::core::button::Button;
use streamduck_core::core::RawButtonPanel;
use streamduck_daemon::socket::daemon_data::{AddComponentResult, AddDeviceResult, ClearButtonResult, CommitChangesToConfigResult, Device, DoButtonActionResult, ExportDeviceConfigResult, ForciblyPopScreenResult, GetButtonResult, GetComponentValuesResult, GetCurrentScreenResult, GetDeviceConfigResult, GetDeviceResult, GetModuleValuesResult, GetStackResult, ImportDeviceConfigResult, NewButtonFromComponentResult, NewButtonResult, PopScreenResult, PushScreenResult, ReloadDeviceConfigResult, ReloadDeviceConfigsResult, RemoveComponentResult, RemoveDeviceResult, ReplaceScreenResult, ResetStackResult, SaveDeviceConfigResult, SaveDeviceConfigsResult, SetBrightnessResult, SetButtonResult, SetComponentValueResult, SetModuleValueResult};

pub use streamduck_daemon as daemon;
use streamduck_core::modules::components::{ComponentDefinition, UIValue};
use streamduck_core::modules::PluginMetadata;
use streamduck_daemon::socket::{SocketError, SocketPacket};

#[cfg(target_family = "unix")]
pub mod unix;


pub mod util;

/// Trait that defines a client, implementations of clients must be separate due to conditional compiling
pub trait SDClient {
    // Version
    /// Retrieves version of the daemon socket API
    fn version(&self) -> Result<String, SDClientError>;

    // Device management
    /// Device list
    fn device_list(&self) -> Result<Vec<Device>, SDClientError>;
    /// Adds device to managed list
    fn get_device(&self, serial_number: &str) -> Result<GetDeviceResult, SDClientError>;
    /// Adds device to managed list
    fn add_device(&self, serial_number: &str) -> Result<AddDeviceResult, SDClientError>;
    /// Removes device from managed list
    fn remove_device(&self, serial_number: &str) -> Result<RemoveDeviceResult, SDClientError>;

    // Device configuration
    /// Reloads all device configs, all changes will be lost executing this
    fn reload_device_configs(&self) -> Result<ReloadDeviceConfigsResult, SDClientError>;
    /// Reloads device config for specific device, all changes will be lost executing this
    fn reload_device_config(&self, serial_number: &str) -> Result<ReloadDeviceConfigResult, SDClientError>;
    /// Saves all device configs
    fn save_device_configs(&self) -> Result<SaveDeviceConfigsResult, SDClientError>;
    /// Saves device config for specific device
    fn save_device_config(&self, serial_number: &str) -> Result<SaveDeviceConfigResult, SDClientError>;

    /// Gets device config for a device
    fn get_device_config(&self, serial_number: &str) -> Result<GetDeviceConfigResult, SDClientError>;

    /// Imports device config from string
    fn import_device_config(&self, serial_number: &str, config: String) -> Result<ImportDeviceConfigResult, SDClientError>;
    /// Exports device config into string
    fn export_device_config(&self, serial_number: &str) -> Result<ExportDeviceConfigResult, SDClientError>;


    /// Sets device brightness, usually 0-100, but different for each device
    fn set_brightness(&self, serial_number: &str, brightness: u8) -> Result<SetBrightnessResult, SDClientError>;

    // Module management
    /// Lists all modules loaded by daemon
    fn list_modules(&self) -> Result<Vec<PluginMetadata>, SDClientError>;
    /// Lists all components that were introduced by modules
    fn list_components(&self) -> Result<HashMap<String, HashMap<String, ComponentDefinition>>, SDClientError>;

    /// Gets module settings
    fn get_module_values(&self, module_name: &str) -> Result<GetModuleValuesResult, SDClientError>;
    /// Sets module settings
    fn set_module_value(&self, module_name: &str, value: Vec<UIValue>) -> Result<SetModuleValueResult, SDClientError>;

    // Panel management
    /// Gets stack of a device
    fn get_stack(&self, serial_number: &str) -> Result<GetStackResult, SDClientError>;
    /// Gets current screen of a device
    fn get_current_screen(&self, serial_number: &str) -> Result<GetCurrentScreenResult, SDClientError>;

    /// Gets a button from current screen of a device
    fn get_button(&self, serial_number: &str, key: u8) -> Result<GetButtonResult, SDClientError>;
    /// Sets a button on current screen of a device
    fn set_button(&self, serial_number: &str, key: u8, button: Button) -> Result<SetButtonResult, SDClientError>;
    /// Clears a button from current screen of a device
    fn clear_button(&self, serial_number: &str, key: u8) -> Result<ClearButtonResult, SDClientError>;

    /// Creates a new empty button on current screen of a device
    fn new_button(&self, serial_number: &str, key: u8) -> Result<NewButtonResult, SDClientError>;
    /// Creates a button from component on current screen of a device
    fn new_button_from_component(&self, serial_number: &str, key: u8, component_name: &str) -> Result<NewButtonFromComponentResult, SDClientError>;

    /// Adds component on a button
    fn add_component(&self, serial_number: &str, key: u8, component_name: &str) -> Result<AddComponentResult, SDClientError>;
    /// Gets component values on a button
    fn get_component_values(&self, serial_number: &str, key: u8, component_name: &str) -> Result<GetComponentValuesResult, SDClientError>;
    /// Sets component value on a button
    fn set_component_values(&self, serial_number: &str, key: u8, component_name: &str, value: Vec<UIValue>) -> Result<SetComponentValueResult, SDClientError>;
    /// Removes component from a button
    fn remove_component(&self, serial_number: &str, key: u8, component_name: &str) -> Result<RemoveComponentResult, SDClientError>;

    /// Pushes a new screen on a device
    fn push_screen(&self, serial_number: &str, screen: RawButtonPanel) -> Result<PushScreenResult, SDClientError>;
    /// Pops a screen from a device
    fn pop_screen(&self, serial_number: &str) -> Result<PopScreenResult, SDClientError>;
    /// Pops a screen from a device, even if it's only one remaining
    fn forcibly_pop_screen(&self, serial_number: &str) -> Result<ForciblyPopScreenResult, SDClientError>;
    /// Replaces current screen on a device
    fn replace_screen(&self, serial_number: &str, screen: RawButtonPanel) -> Result<ReplaceScreenResult, SDClientError>;
    /// Resets stack and sets provided screen as root screen
    fn reset_stack(&self, serial_number: &str, screen: RawButtonPanel) -> Result<ResetStackResult, SDClientError>;

    /// Commits all changes to stack to device config, should be called after each change/sequence of changes, otherwise all changes will be lost on reconnect
    fn commit_changes(&self, serial_number: &str) -> Result<CommitChangesToConfigResult, SDClientError>;

    /// Simulate a press on a button on current screen for a device
    fn do_button_action(&self, serial_number: &str, key: u8) -> Result<DoButtonActionResult, SDClientError>;

    /// Sends a custom packet to daemon and returns response, for use with plugins that utilize socket functionality
    fn send_packet(&self, packet: SocketPacket) -> Result<SocketPacket, SDClientError>;
    /// Sends a custom packet to daemon and returns response, for use with plugins that utilize socket functionality
    fn send_packet_without_response(&self, packet: SocketPacket) -> Result<(), SDClientError>;
}

/// Errors that could happen with the client
#[derive(Debug)]
pub enum SDClientError {
    WriteError(std::io::Error),
    SerializeError(serde_json::Error),
    SocketError(streamduck_daemon::socket::SocketError),
    UTF8Error(std::string::FromUtf8Error),
    Custom(String)
}

impl From<std::io::Error> for SDClientError {
    fn from(err: Error) -> Self {
        SDClientError::WriteError(err)
    }
}

impl From<serde_json::Error> for SDClientError {
    fn from(err: serde_json::Error) -> Self {
        SDClientError::SerializeError(err)
    }
}

impl From<streamduck_daemon::socket::SocketError> for SDClientError {
    fn from(err: SocketError) -> Self {
        SDClientError::SocketError(err)
    }
}

impl From<std::string::FromUtf8Error> for SDClientError {
    fn from(err: FromUtf8Error) -> Self {
        SDClientError::UTF8Error(err)
    }
}