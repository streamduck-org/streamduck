use std::sync::Arc;
use streamduck_core::bundle::ManagerBundle;
use crate::drivers::streamdeck::StreamDeckDriver;

/// Stream Deck implementation
mod streamdeck;

pub async fn load_drivers(bundle: &Arc<ManagerBundle>) {
    let driver_manager = bundle.driver_manager();

    driver_manager.register_driver(
        StreamDeckDriver::new()
    ).await;
}