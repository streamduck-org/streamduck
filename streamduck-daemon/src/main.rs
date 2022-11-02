/// Drivers for
mod drivers;

use std::future::ready;
use std::sync::Arc;
use tracing::{Level, info};
use streamduck_core::devices::drivers::DriverManager;
use streamduck_core::events::{Event, EventDispatcher, EventInstance};
use serde::{Serialize, Deserialize};
use streamduck_core::events::listeners::{EventListener, ListensFor, SpecificListener};
use streamduck_core::events::util::cast_event;
use streamduck_core::{init_managers, type_of};
use streamduck_core::devices::buttons::ButtonPosition;
use streamduck_core::image_lib::{DynamicImage, open};
use crate::drivers::load_drivers;

/// the entry point for the streamdeck application
#[tokio::main]
async fn main() {
    // TODO: change filter level depending on flag
    tracing_subscriber::fmt()
        .compact()
        .with_target(true)
        .with_max_level(Level::TRACE)
        .init();

    info!("Starting...");

    let bundle = init_managers().await
        .expect("Failed to initialize managers");

    load_drivers(&bundle).await;

    let device_metadata = bundle.driver_manager().list_devices().await
        .into_iter()
        .find(|m| m.identifier.contains("AL10J2C00059"))
        .expect("Device not found");

    println!("Device metadata: {:?}", device_metadata);

    let device = bundle.driver_manager()
        .connect_device(&device_metadata.driver_name, &device_metadata.identifier).await
        .expect("Failed to connect");

    device.clear_screen().await;

    let image = open("13339036_medium.jpg").unwrap();

    device.add_image(123, image).await;
    device.set_button_image(ButtonPosition::from((0, 0)), 123).await;
}