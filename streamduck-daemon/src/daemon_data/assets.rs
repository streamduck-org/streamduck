//! Requests related to images and fonts
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use streamduck_core::font::get_font_names;
use streamduck_core::socket::{check_packet_for_data, parse_packet_to_data, send_packet, SocketData, SocketHandle, SocketPacket};
use crate::daemon_data::{DaemonListener, DaemonRequest};
use streamduck_core::async_trait;

/// Request for getting all images currently saved on device
#[derive(Serialize, Deserialize)]
pub struct ListImages {
    pub serial_number: String
}

/// Struct that keeps information about SDImage
#[derive(Serialize, Deserialize)]
pub struct SocketImage {
    pub image_blob: String,
    pub animated: bool
}

/// Response for [ListImages] request
#[derive(Serialize, Deserialize)]
pub enum ListImagesResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if successfully retrieved image list from device config
    Images(HashMap<String, SocketImage>)
}

impl SocketData for ListImages {
    const NAME: &'static str = "list_images";
}

impl SocketData for ListImagesResult {
    const NAME: &'static str = "list_images";
}

#[async_trait]
impl DaemonRequest for ListImages {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<ListImages>(packet) {
            if let Some(images) = listener.config.get_images(&request.serial_number).await {
                send_packet(handle, packet, &ListImagesResult::Images(
                    images.into_iter()
                        .map(|(id, image)| (id, SocketImage {
                            image_blob: image.as_image_blob().unwrap_or("failed".to_string()),
                            animated: image.is_animated()
                        }))
                        .collect()
                )).await.ok();
            } else {
                send_packet(handle, packet, &ListImagesResult::DeviceNotFound).await.ok();
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

#[async_trait]
impl DaemonRequest for AddImage {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<AddImage>(packet) {
            if let Some(_) = listener.core_manager.get_device(&request.serial_number).await {
                if let Some(identifier) = listener.config.add_image(&request.serial_number, request.image_data).await {
                    send_packet(handle, packet, &AddImageResult::Added(identifier)).await.ok();
                } else {
                    send_packet(handle, packet, &AddImageResult::InvalidData).await.ok();
                }
            } else {
                send_packet(handle, packet, &AddImageResult::DeviceNotFound).await.ok();
            }
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

#[async_trait]
impl DaemonRequest for RemoveImage {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<RemoveImage>(packet) {
            // Decoding image to make sure the data is correct
            if listener.config.remove_image(&request.serial_number, &request.image_identifier).await {
                send_packet(handle, packet, &RemoveImageResult::Removed).await.ok();
            } else {
                send_packet(handle, packet, &RemoveImageResult::NotFound).await.ok();
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

#[async_trait]
impl DaemonRequest for ListFonts {
    async fn process(_: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if check_packet_for_data::<ListFonts>(packet) {
            send_packet(handle, packet, &ListFonts {
                font_names: get_font_names()
            }).await.ok();
        }
    }
}