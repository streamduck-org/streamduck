//! Data types that daemon uses for core functions
pub mod devices;
pub mod config;
pub mod assets;
pub mod modules;
pub mod panels;
pub mod buttons;
pub mod ops;

use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use streamduck_core::versions::SOCKET_API;
use streamduck_core::core::manager::CoreManager;
use streamduck_core::socket::{check_packet_for_data, send_packet, SocketData, SocketHandle, SocketListener, SocketPacket};
use streamduck_core::modules::ModuleManager;
use streamduck_core::config::Config;
use streamduck_core::core::button::Button;
use crate::daemon_data::assets::{AddImage, ListFonts, ListImages, RemoveImage};
use crate::daemon_data::buttons::{AddComponent, AddComponentValue, ClearButton, ClipboardStatusResult, CopyButton, GetButton, GetComponentValues, NewButton, NewButtonFromComponent, PasteButton, RemoveComponent, RemoveComponentValue, SetButton, SetComponentValue};
use crate::daemon_data::config::{ExportDeviceConfig, GetDeviceConfig, ImportDeviceConfig, ReloadDeviceConfig, ReloadDeviceConfigsResult, SaveDeviceConfig, SaveDeviceConfigsResult};
use crate::daemon_data::devices::{AddDevice, GetBrightness, GetDevice, ListDevices, RemoveDevice, SetBrightness};
use crate::daemon_data::modules::{AddModuleValue, GetModuleValues, ListComponents, ListModules, RemoveModuleValue, SetModuleValue};
use crate::daemon_data::ops::{CommitChangesToConfig, DoButtonAction};
use crate::daemon_data::panels::{DropStackToRoot, ForciblyPopScreen, GetButtonImage, GetButtonImages, GetCurrentScreen, GetStack, GetStackNames, PopScreen, PushScreen, ReplaceScreen, ResetStack};

/// Listener for daemon types
pub struct DaemonListener {
    pub core_manager: Arc<CoreManager>,
    pub module_manager: Arc<ModuleManager>,
    pub config: Arc<Config>,
    pub clipboard: Mutex<Option<Button>>,
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

        process_for_type::<GetDeviceConfig>(self, socket, &packet);

        process_for_type::<ImportDeviceConfig>(self, socket, &packet);
        process_for_type::<ExportDeviceConfig>(self, socket, &packet);

        process_for_type::<GetBrightness>(self, socket, &packet);
        process_for_type::<SetBrightness>(self, socket, &packet);

        process_for_type::<ListImages>(self, socket, &packet);
        process_for_type::<AddImage>(self, socket, &packet);
        process_for_type::<RemoveImage>(self, socket, &packet);

        process_for_type::<ListFonts>(self,socket, &packet);

        // Module management
        process_for_type::<ListModules>(self,socket, &packet);
        process_for_type::<ListComponents>(self,socket, &packet);

        process_for_type::<GetModuleValues>(self,socket, &packet);
        process_for_type::<AddModuleValue>(self,socket, &packet);
        process_for_type::<RemoveModuleValue>(self,socket, &packet);
        process_for_type::<SetModuleValue>(self,socket, &packet);

        // Panel management
        process_for_type::<GetStack>(self, socket, &packet);
        process_for_type::<GetStackNames>(self, socket, &packet);
        process_for_type::<GetCurrentScreen>(self, socket, &packet);
        process_for_type::<GetButtonImage>(self, socket, &packet);
        process_for_type::<GetButtonImages>(self, socket, &packet);

        process_for_type::<GetButton>(self, socket, &packet);
        process_for_type::<SetButton>(self, socket, &packet);
        process_for_type::<ClearButton>(self, socket, &packet);

        process_for_type::<ClipboardStatusResult>(self, socket, &packet);
        process_for_type::<CopyButton>(self, socket, &packet);
        process_for_type::<PasteButton>(self, socket, &packet);

        process_for_type::<NewButton>(self, socket, &packet);
        process_for_type::<NewButtonFromComponent>(self, socket, &packet);

        process_for_type::<AddComponent>(self, socket, &packet);

        process_for_type::<GetComponentValues>(self, socket, &packet);
        process_for_type::<AddComponentValue>(self, socket, &packet);
        process_for_type::<RemoveComponentValue>(self, socket, &packet);
        process_for_type::<SetComponentValue>(self, socket, &packet);

        process_for_type::<RemoveComponent>(self, socket, &packet);

        process_for_type::<PushScreen>(self, socket, &packet);
        process_for_type::<PopScreen>(self, socket, &packet);
        process_for_type::<ForciblyPopScreen>(self, socket, &packet);
        process_for_type::<ReplaceScreen>(self, socket, &packet);
        process_for_type::<ResetStack>(self, socket, &packet);
        process_for_type::<DropStackToRoot>(self, socket, &packet);

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