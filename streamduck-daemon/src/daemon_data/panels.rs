//! Requests related to panels
use std::collections::HashMap;
use std::io::Cursor;
use serde::{Deserialize, Serialize};
use streamduck_core::core::methods::{CoreHandle, get_button_image, get_button_images, get_current_screen, get_root_screen, get_stack, pop_screen, push_screen, replace_screen, reset_stack};
use streamduck_core::core::RawButtonPanel;
use streamduck_core::image::ImageOutputFormat;
use streamduck_core::socket::{parse_packet_to_data, send_packet, SocketData, SocketHandle, SocketPacket};
use streamduck_core::util::{make_panel_unique, panel_to_raw};
use crate::daemon_data::{DaemonListener, DaemonRequest};

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

impl DaemonRequest for GetStackNames {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<GetStackNames>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number) {
                let wrapped_core = CoreHandle::wrap(device.core);

                let mut raw_stack = vec![];

                for stack_item in get_stack(&wrapped_core) {
                    let raw_item = panel_to_raw(&stack_item);
                    raw_stack.push(raw_item.display_name);
                }

                send_packet(handle, packet, &GetStackNamesResult::Stack(raw_stack)).ok();
            } else {
                send_packet(handle, packet, &GetStackNamesResult::DeviceNotFound).ok();
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

impl DaemonRequest for GetCurrentScreen {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<GetCurrentScreen>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number) {
                let wrapped_core = CoreHandle::wrap(device.core);

                if let Some(screen) = get_current_screen(&wrapped_core) {
                    send_packet(handle, packet, &GetCurrentScreenResult::Screen(panel_to_raw(&screen))).ok();
                } else {
                    send_packet(handle, packet, &GetCurrentScreenResult::NoScreen).ok();
                }
            } else {
                send_packet(handle, packet, &GetCurrentScreenResult::DeviceNotFound).ok();
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

impl DaemonRequest for GetButtonImages {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<GetButtonImages>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number) {
                let wrapped_core = CoreHandle::wrap(device.core);

                if let Some(images) = get_button_images(&wrapped_core) {
                    let images = images.into_iter()
                        .map(|(key, image)| {
                            let mut buffer: Vec<u8> = vec![];
                            image.write_to(&mut Cursor::new(&mut buffer), ImageOutputFormat::Png).ok();
                            (key, base64::encode(buffer))
                        })
                        .collect();

                    send_packet(handle, packet, &GetButtonImagesResult::Images(images)).ok();
                    return;
                }
            }

            send_packet(handle, packet, &GetButtonImagesResult::DeviceNotFound).ok();
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

impl DaemonRequest for GetButtonImage {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<GetButtonImage>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number) {
                let wrapped_core = CoreHandle::wrap(device.core);

                if let Some(image) = get_button_image(&wrapped_core, request.key) {
                    let mut buffer: Vec<u8> = vec![];
                    image.write_to(&mut Cursor::new(&mut buffer), ImageOutputFormat::Png).ok();

                    send_packet(handle, packet, &GetButtonImageResult::Image(base64::encode(buffer))).ok();
                } else {
                    send_packet(handle, packet, &GetButtonImageResult::NoButton).ok();
                }
            } else {
                send_packet(handle, packet, &GetButtonImageResult::DeviceNotFound).ok();
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

impl DaemonRequest for ResetStack {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<ResetStack>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number) {
                let wrapped_core = CoreHandle::wrap(device.core);

                reset_stack(&wrapped_core, make_panel_unique(request.screen));
                send_packet(handle, packet, &ResetStackResult::Reset).ok();
            } else {
                send_packet(handle, packet, &ResetStackResult::DeviceNotFound).ok();
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

impl DaemonRequest for DropStackToRoot {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<DropStackToRoot>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number) {
                let wrapped_core = CoreHandle::wrap(device.core);

                let first_screen = get_root_screen(&wrapped_core);
                reset_stack(&wrapped_core, first_screen);
                send_packet(handle, packet, &DropStackToRootResult::Dropped).ok();
            } else {
                send_packet(handle, packet, &DropStackToRootResult::DeviceNotFound).ok();
            }
        }
    }
}