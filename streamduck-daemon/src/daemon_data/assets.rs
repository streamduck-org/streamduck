//! Requests related to images and fonts
use std::collections::HashMap;
use std::io::Cursor;
use serde::{Deserialize, Serialize};
use streamduck_core::font::get_font_names;
use streamduck_core::image::io::Reader;
use streamduck_core::socket::{check_packet_for_data, parse_packet_to_data, send_packet, SocketData, SocketHandle, SocketPacket};
use streamduck_core::util::rendering::resize_for_streamdeck;
use crate::daemon_data::{DaemonListener, DaemonRequest};

/// Request for getting all images currently saved on device
#[derive(Serialize, Deserialize)]
pub struct ListImages {
    pub serial_number: String
}

/// Response for [ListImages] request
#[derive(Serialize, Deserialize)]
pub enum ListImagesResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if successfully retrieved image list from device config
    Images(HashMap<String, String>)
}

impl SocketData for ListImages {
    const NAME: &'static str = "list_images";
}

impl SocketData for ListImagesResult {
    const NAME: &'static str = "list_images";
}

impl DaemonRequest for ListImages {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<ListImages>(packet) {
            if let Some(images) = listener.config.get_images(&request.serial_number) {
                send_packet(handle, packet, &ListImagesResult::Images(images)).ok();
            } else {
                send_packet(handle, packet, &ListImagesResult::DeviceNotFound).ok();
            }
        }
    }
}

/// Request for adding a new image into image collection
#[derive(Serialize, Deserialize)]
pub struct AddImage {
    pub serial_number: String,
    pub image_data: String,
}

/// Response for [AddImage] request
#[derive(Serialize, Deserialize)]
pub enum AddImageResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if image data is invalid
    InvalidData,

    /// Sent if successfully added image, contains identifier for the image
    Added(String)
}

impl SocketData for AddImage {
    const NAME: &'static str = "add_image";
}

impl SocketData for AddImageResult {
    const NAME: &'static str = "add_image";
}

impl DaemonRequest for AddImage {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<AddImage>(packet) {
            // Decoding image to make sure the data is correct
            if let Ok(byte_array) = base64::decode(request.image_data) {
                if let Ok(recognized_image) = Reader::new(Cursor::new(&byte_array)).with_guessed_format() {
                    if let Ok(decoded_image) = recognized_image.decode() {
                        if let Some(device) = listener.core_manager.get_device(&request.serial_number) {
                            let decoded_image = resize_for_streamdeck(device.core.image_size, decoded_image);



                            if let Some(identifier) = listener.config.add_image_encode(&request.serial_number, decoded_image) {
                                send_packet(handle, packet, &AddImageResult::Added(identifier)).ok();
                            } else {
                                send_packet(handle, packet, &AddImageResult::DeviceNotFound).ok();
                            }
                        } else {
                            send_packet(handle, packet, &AddImageResult::DeviceNotFound).ok();
                        }

                        drop(byte_array);

                        return;
                    }
                }

                drop(byte_array);
            }

            send_packet(handle, packet, &AddImageResult::InvalidData).ok();
        }
    }
}

/// Request for removing an image from image collection
#[derive(Serialize, Deserialize)]
pub struct RemoveImage {
    pub serial_number: String,
    pub image_identifier: String,
}

/// Response for [RemoveImage] request
#[derive(Serialize, Deserialize)]
pub enum RemoveImageResult {
    /// Sent if image wasn't found
    NotFound,

    /// Sent if successfully removed image
    Removed
}

impl SocketData for RemoveImage {
    const NAME: &'static str = "remove_image";
}

impl SocketData for RemoveImageResult {
    const NAME: &'static str = "remove_image";
}

impl DaemonRequest for RemoveImage {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<RemoveImage>(packet) {
            // Decoding image to make sure the data is correct
            if listener.config.remove_image(&request.serial_number, &request.image_identifier) {
                send_packet(handle, packet, &RemoveImageResult::Removed).ok();
            } else {
                send_packet(handle, packet, &RemoveImageResult::NotFound).ok();
            }
        }
    }
}

/// Request for getting fonts loaded by daemon
#[derive(Serialize, Deserialize)]
pub struct ListFonts {
    pub font_names: Vec<String>
}

impl SocketData for ListFonts {
    const NAME: &'static str = "list_fonts";
}

impl DaemonRequest for ListFonts {
    fn process(_: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if check_packet_for_data::<ListFonts>(packet) {
            send_packet(handle, packet, &ListFonts {
                font_names: get_font_names()
            }).ok();
        }
    }
}