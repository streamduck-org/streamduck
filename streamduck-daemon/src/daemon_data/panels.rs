//! Requests related to panels
use std::collections::HashMap;
use std::io::Cursor;
use serde::{Deserialize, Serialize};
use streamduck_core::core::{CoreHandle, RawButtonPanel};
use streamduck_core::image::ImageOutputFormat;
use streamduck_core::socket::{parse_packet_to_data, send_packet, SocketData, SocketHandle, SocketPacket};
use streamduck_core::util::{make_panel_unique, panel_to_raw};
use crate::daemon_data::{DaemonListener, DaemonRequest};
use streamduck_core::async_trait;

/// Request for getting current stack on a device
#[derive(Serialize, Deserialize)]
pub struct GetStack {
    pub serial_number: String
}

/// Response of [GetStack] request
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

#[async_trait]
impl DaemonRequest for GetStack {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<GetStack>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number).await {
                let wrapped_core = CoreHandle::wrap(device.core);

                let mut raw_stack = vec![];

                for stack_item in wrapped_core.get_stack().await {
                    let raw_item = panel_to_raw(&stack_item).await;
                    raw_stack.push(raw_item);
                }

                send_packet(handle, packet, &GetStackResult::Stack(raw_stack)).await.ok();
            } else {
                send_packet(handle, packet, &GetStackResult::DeviceNotFound).await.ok();
            }
        }
    }
}

/// Request for getting current stack names on a device, similar to GetStack, but only provides names of
#[derive(Serialize, Deserialize)]
pub struct GetStackNames {
    pub serial_number: String
}

/// Response of [GetStackNames] request
#[derive(Serialize, Deserialize)]
pub enum GetStackNamesResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if successfully got stack
    Stack(Vec<String>)
}

impl SocketData for GetStackNames {
    const NAME: &'static str = "get_stack_names";
}

impl SocketData for GetStackNamesResult {
    const NAME: &'static str = "get_stack_names";
}

#[async_trait]
impl DaemonRequest for GetStackNames {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<GetStackNames>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number).await {
                let wrapped_core = CoreHandle::wrap(device.core);

                let mut raw_stack = vec![];

                for stack_item in wrapped_core.get_stack().await {
                    let raw_item = panel_to_raw(&stack_item).await;
                    raw_stack.push(raw_item.display_name);
                }

                send_packet(handle, packet, &GetStackNamesResult::Stack(raw_stack)).await.ok();
            } else {
                send_packet(handle, packet, &GetStackNamesResult::DeviceNotFound).await.ok();
            }
        }
    }
}

/// Request for getting current screen on a device
#[derive(Serialize, Deserialize)]
pub struct GetCurrentScreen {
    pub serial_number: String
}

/// Response of [GetCurrentScreen] request
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

#[async_trait]
impl DaemonRequest for GetCurrentScreen {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<GetCurrentScreen>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number).await {
                let wrapped_core = CoreHandle::wrap(device.core);

                if let Some(screen) = wrapped_core.get_current_screen().await {
                    send_packet(handle, packet, &GetCurrentScreenResult::Screen(panel_to_raw(&screen).await)).await.ok();
                } else {
                    send_packet(handle, packet, &GetCurrentScreenResult::NoScreen).await.ok();
                }
            } else {
                send_packet(handle, packet, &GetCurrentScreenResult::DeviceNotFound).await.ok();
            }
        }
    }
}



/// Request for getting current button images on a device
#[derive(Serialize, Deserialize)]
pub struct GetButtonImages {
    pub serial_number: String
}

/// Response of [GetButtonImages] request
#[derive(Serialize, Deserialize)]
pub enum GetButtonImagesResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if successfully generated images
    Images(HashMap<u8, String>)
}

impl SocketData for GetButtonImages {
    const NAME: &'static str = "get_button_images";
}

impl SocketData for GetButtonImagesResult {
    const NAME: &'static str = "get_button_images";
}

#[async_trait]
impl DaemonRequest for GetButtonImages {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<GetButtonImages>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number).await {
                let wrapped_core = CoreHandle::wrap(device.core);

                if let Some(images) = wrapped_core.get_button_images().await {
                    let images = images.into_iter()
                        .map(|(key, image)| {
                            let mut buffer: Vec<u8> = vec![];
                            image.write_to(&mut Cursor::new(&mut buffer), ImageOutputFormat::Png).ok();
                            (key, base64::encode(buffer))
                        })
                        .collect();

                    send_packet(handle, packet, &GetButtonImagesResult::Images(images)).await.ok();
                    return;
                }
            }

            send_packet(handle, packet, &GetButtonImagesResult::DeviceNotFound).await.ok();
        }
    }
}


/// Request for getting current button image on a device
#[derive(Serialize, Deserialize)]
pub struct GetButtonImage {
    pub serial_number: String,
    pub key: u8,
}

/// Response of [GetButtonImage] request
#[derive(Serialize, Deserialize)]
pub enum GetButtonImageResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if there's no button
    NoButton,

    /// Sent if successfully generated image
    Image(String)
}

impl SocketData for GetButtonImage {
    const NAME: &'static str = "get_button_image";
}

impl SocketData for GetButtonImageResult {
    const NAME: &'static str = "get_button_image";
}

#[async_trait]
impl DaemonRequest for GetButtonImage {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<GetButtonImage>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number).await {
                let wrapped_core = CoreHandle::wrap(device.core);

                if let Some(image) = wrapped_core.get_button_image(request.key).await {
                    let mut buffer: Vec<u8> = vec![];
                    image.write_to(&mut Cursor::new(&mut buffer), ImageOutputFormat::Png).ok();

                    send_packet(handle, packet, &GetButtonImageResult::Image(base64::encode(buffer))).await.ok();
                } else {
                    send_packet(handle, packet, &GetButtonImageResult::NoButton).await.ok();
                }
            } else {
                send_packet(handle, packet, &GetButtonImageResult::DeviceNotFound).await.ok();
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

/// Response of [PushScreen] request
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

#[async_trait]
impl DaemonRequest for PushScreen {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<PushScreen>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number).await {
                let wrapped_core = CoreHandle::wrap(device.core);

                wrapped_core.push_screen(make_panel_unique(request.screen)).await;
                send_packet(handle, packet, &PushScreenResult::Pushed).await.ok();
            } else {
                send_packet(handle, packet, &PushScreenResult::DeviceNotFound).await.ok();
            }
        }
    }
}

/// Request for popping top-most screen on a device
#[derive(Serialize, Deserialize)]
pub struct PopScreen {
    pub serial_number: String
}

/// Response of [PopScreen] request
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

#[async_trait]
impl DaemonRequest for PopScreen {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<PopScreen>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number).await {
                let count = {
                    let stack = device.core.current_stack.lock().await;
                    stack.len()
                };

                let wrapped_core = CoreHandle::wrap(device.core);

                if count > 1 {
                    wrapped_core.pop_screen().await;
                    send_packet(handle, packet, &PopScreenResult::Popped).await.ok();
                } else {
                    send_packet(handle, packet, &PopScreenResult::OnlyOneRemaining).await.ok();
                }
            } else {
                send_packet(handle, packet, &PopScreenResult::DeviceNotFound).await.ok();
            }
        }
    }
}

/// Request for popping top-most screen on a device, even if it's the only one remaining
#[derive(Serialize, Deserialize)]
pub struct ForciblyPopScreen {
    pub serial_number: String
}

/// Response of [ForciblyPopScreen] request
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

#[async_trait]
impl DaemonRequest for ForciblyPopScreen {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<ForciblyPopScreen>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number).await {
                let wrapped_core = CoreHandle::wrap(device.core);

                wrapped_core.pop_screen().await;
                send_packet(handle, packet, &ForciblyPopScreenResult::Popped).await.ok();
            } else {
                send_packet(handle, packet, &ForciblyPopScreenResult::DeviceNotFound).await.ok();
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

/// Response of [ReplaceScreen] request
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

#[async_trait]
impl DaemonRequest for ReplaceScreen {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<ReplaceScreen>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number).await {
                let wrapped_core = CoreHandle::wrap(device.core);

                wrapped_core.replace_screen(make_panel_unique(request.screen)).await;
                send_packet(handle, packet, &ReplaceScreenResult::Replaced).await.ok();
            } else {
                send_packet(handle, packet, &ReplaceScreenResult::DeviceNotFound).await.ok();
            }
        }
    }
}

/// Request for resetting stack with provided screen
#[derive(Serialize, Deserialize)]
pub struct ResetStack {
    pub serial_number: String,
    pub screen: RawButtonPanel
}

/// Response of [ResetStack] request
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

#[async_trait]
impl DaemonRequest for ResetStack {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<ResetStack>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number).await {
                let wrapped_core = CoreHandle::wrap(device.core);

                wrapped_core.reset_stack(make_panel_unique(request.screen)).await;
                send_packet(handle, packet, &ResetStackResult::Reset).await.ok();
            } else {
                send_packet(handle, packet, &ResetStackResult::DeviceNotFound).await.ok();
            }
        }
    }
}

/// Request for going to root screen
#[derive(Serialize, Deserialize)]
pub struct DropStackToRoot {
    pub serial_number: String
}

/// Response of [DropStackToRoot] request
#[derive(Serialize, Deserialize)]
pub enum DropStackToRootResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if successfully dropped to root
    Dropped
}

impl SocketData for DropStackToRoot {
    const NAME: &'static str = "drop_stack_to_root";
}

impl SocketData for DropStackToRootResult {
    const NAME: &'static str = "drop_stack_to_root";
}

#[async_trait]
impl DaemonRequest for DropStackToRoot {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<DropStackToRoot>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number).await {
                let wrapped_core = CoreHandle::wrap(device.core);

                let first_screen = wrapped_core.get_root_screen().await;
                wrapped_core.reset_stack(first_screen).await;
                send_packet(handle, packet, &DropStackToRootResult::Dropped).await.ok();
            } else {
                send_packet(handle, packet, &DropStackToRootResult::DeviceNotFound).await.ok();
            }
        }
    }
}