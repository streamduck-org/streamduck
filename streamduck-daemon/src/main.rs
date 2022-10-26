use std::sync::Arc;
use tracing::{Level, info};
use streamduck_core::devices::drivers::DriverManager;
use streamduck_core::devices::images::DeviceImageMethods;

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

    let driver_manager = DriverManager::new();

    let a: Option<Arc<dyn DeviceImageMethods>> = None;
}

