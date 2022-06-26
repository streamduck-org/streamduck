//! Data types that daemon uses for core functions
pub mod devices;
pub mod config;
pub mod assets;
pub mod modules;
pub mod panels;
pub mod buttons;
pub mod ops;

use std::sync::{Arc};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use streamduck_core::versions::SOCKET_API;
use streamduck_core::core::manager::CoreManager;
use streamduck_core::socket::{check_packet_for_data, send_packet, SocketData, SocketHandle, SocketListener, SocketPacket};
use streamduck_core::modules::ModuleManager;
use streamduck_core::config::Config;
use streamduck_core::core::button::Button;
use streamduck_core::async_trait;
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

#[async_trait]
impl SocketListener for DaemonListener {
    async fn message(&self, socket: SocketHandle<'_>, packet: SocketPacket) {
        // Version
        process_for_type::<SocketAPIVersion>(self,socket, &packet).await;

        // Device management
        process_for_type::<ListDevices>(self,socket, &packet).await;
        process_for_type::<GetDevice>(self,socket, &packet).await;
        process_for_type::<AddDevice>(self,socket, &packet).await;
        process_for_type::<RemoveDevice>(self,socket, &packet).await;

        // Device configuration
        process_for_type::<ReloadDeviceConfigsResult>(self, socket, &packet).await;
        process_for_type::<ReloadDeviceConfig>(self, socket, &packet).await;
        process_for_type::<SaveDeviceConfigsResult>(self, socket, &packet).await;
        process_for_type::<SaveDeviceConfig>(self, socket, &packet).await;

        process_for_type::<GetDeviceConfig>(self, socket, &packet).await;

        process_for_type::<ImportDeviceConfig>(self, socket, &packet).await;
        process_for_type::<ExportDeviceConfig>(self, socket, &packet).await;

        process_for_type::<GetBrightness>(self, socket, &packet).await;
        process_for_type::<SetBrightness>(self, socket, &packet).await;

        process_for_type::<ListImages>(self, socket, &packet).await;
        process_for_type::<AddImage>(self, socket, &packet).await;
        process_for_type::<RemoveImage>(self, socket, &packet).await;

        process_for_type::<ListFonts>(self,socket, &packet).await;

        // Module management
        process_for_type::<ListModules>(self,socket, &packet).await;
        process_for_type::<ListComponents>(self,socket, &packet).await;

        process_for_type::<GetModuleValues>(self,socket, &packet).await;
        process_for_type::<AddModuleValue>(self,socket, &packet).await;
        process_for_type::<RemoveModuleValue>(self,socket, &packet).await;
        process_for_type::<SetModuleValue>(self,socket, &packet).await;

        // Panel management
        process_for_type::<GetStack>(self, socket, &packet).await;
        process_for_type::<GetStackNames>(self, socket, &packet).await;
        process_for_type::<GetCurrentScreen>(self, socket, &packet).await;
        process_for_type::<GetButtonImage>(self, socket, &packet).await;
        process_for_type::<GetButtonImages>(self, socket, &packet).await;

        process_for_type::<GetButton>(self, socket, &packet).await;
        process_for_type::<SetButton>(self, socket, &packet).await;
        process_for_type::<ClearButton>(self, socket, &packet).await;

        process_for_type::<ClipboardStatusResult>(self, socket, &packet).await;
        process_for_type::<CopyButton>(self, socket, &packet).await;
        process_for_type::<PasteButton>(self, socket, &packet).await;

        process_for_type::<NewButton>(self, socket, &packet).await;
        process_for_type::<NewButtonFromComponent>(self, socket, &packet).await;

        process_for_type::<AddComponent>(self, socket, &packet).await;

        process_for_type::<GetComponentValues>(self, socket, &packet).await;
        process_for_type::<AddComponentValue>(self, socket, &packet).await;
        process_for_type::<RemoveComponentValue>(self, socket, &packet).await;
        process_for_type::<SetComponentValue>(self, socket, &packet).await;

        process_for_type::<RemoveComponent>(self, socket, &packet).await;

        process_for_type::<PushScreen>(self, socket, &packet).await;
        process_for_type::<PopScreen>(self, socket, &packet).await;
        process_for_type::<ForciblyPopScreen>(self, socket, &packet).await;
        process_for_type::<ReplaceScreen>(self, socket, &packet).await;
        process_for_type::<ResetStack>(self, socket, &packet).await;
        process_for_type::<DropStackToRoot>(self, socket, &packet).await;

        process_for_type::<CommitChangesToConfig>(self, socket, &packet).await;

        process_for_type::<DoButtonAction>(self, socket, &packet).await;
    }
}

#[async_trait]
trait DaemonRequest {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket);
}

async fn process_for_type<T: DaemonRequest + SocketData>(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
    if packet.ty == T::NAME {
        T::process(listener, handle, packet).await;
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

#[async_trait]
impl DaemonRequest for SocketAPIVersion {
    async fn process(_listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if check_packet_for_data::<SocketAPIVersion>(&packet) {
            send_packet(handle, &packet, &SocketAPIVersion {
                version: SOCKET_API.1.to_string()
            }).await.ok();
        }
    }
}