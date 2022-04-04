//! Crate responsible for managing streamdeck devices, rendering, managing configuration and pretty much everything

/// Utility code for rendering and conversions
pub mod util;
/// Core object and button definitions
pub mod core;
/// Font related code
pub mod font;
/// Module definition and built-in modules
pub mod modules;
/// Everything related to image processing
pub mod images;

pub mod versions;
pub mod config;
pub mod socket;
pub mod thread;

pub use streamdeck;
pub use hidapi;
pub use palette;
pub use image;

use std::sync::Arc;
use hidapi::HidApi;
use streamdeck::pids;
use streamdeck::StreamDeck;
use crate::config::{Config, UniqueDeviceConfig};
use crate::core::{KeyHandler, SDCore};
use crate::modules::ModuleManager;
use thread::ImageCollection;
use crate::thread::rendering::custom::RenderingManager;

/// Attempts to connect to any detected streamdeck
pub fn connect_any(module_manager: Arc<ModuleManager>, render_manager: Arc<RenderingManager>, config: Arc<Config>, device_config: UniqueDeviceConfig, image_collection: ImageCollection, hid: &HidApi, pool_rate: u32) -> Result<(Arc<SDCore>, KeyHandler), Error> {
    Ok(SDCore::new(module_manager, render_manager, config, device_config, image_collection, attempt_connection_to_any(hid)?, pool_rate))
}

fn attempt_connection_to_any(hid: &HidApi) -> Result<StreamDeck, Error> {
    let mut decks = find_decks(hid);

    if let Some((vid, pid, serial)) = decks.pop() {
        match StreamDeck::connect_with_hid(&hid, vid, pid, serial) {
            Ok(streamdeck) => Ok(streamdeck),
            Err(err) => Err(Error::StreamDeckError(err))
        }
    } else {
        Err(Error::DeviceNotFound)
    }
}

/// Attempts to connect to specified device as a streamdeck
pub fn connect(module_manager: Arc<ModuleManager>, render_manager: Arc<RenderingManager>, config: Arc<Config>, device_config: UniqueDeviceConfig, image_collection: ImageCollection, hid: &HidApi, vid: u16, pid: u16, serial: &str, pool_rate: u32) -> Result<(Arc<SDCore>, KeyHandler), Error> {
    Ok(SDCore::new(module_manager, render_manager, config, device_config, image_collection, attempt_connection(hid, vid, pid, serial)?, pool_rate))
}

fn attempt_connection(hid: &HidApi, vid: u16, pid: u16, serial: &str) -> Result<StreamDeck, Error> {
    match StreamDeck::connect_with_hid(&hid, vid, pid, Some(serial.to_string())) {
        Ok(streamdeck) => Ok(streamdeck),
        Err(err) => Err(Error::StreamDeckError(err))
    }
}

/// Retrieves a list of found streamdeck devices
pub fn find_decks(hid: &HidApi) -> Vec<(u16, u16, Option<String>)> {
    let devices = hid
        .device_list()
        .filter(|item| check_if_streamdeck(item.product_id()));

    devices.map(
        |d| (
            d.vendor_id(),
            d.product_id(),
            d.serial_number().map(|f| f.to_string())
        ))
        .collect()
}

/// Checks if PID of the device matches streamdeck
pub fn check_if_streamdeck(product_id: u16) -> bool {
    match product_id {
        pids::MINI | pids::ORIGINAL | pids::ORIGINAL_V2 | pids::XL => true,
        _ => false,
    }
}

/// Error type for streamdeck connections
#[derive(Debug)]
pub enum Error {
    /// Couldn't find the device while establishing connection to any streamdeck
    DeviceNotFound,
    /// Any error of the under laying streamdeck library
    StreamDeckError(streamdeck::Error)
}
