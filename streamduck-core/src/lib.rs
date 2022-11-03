#![warn(missing_docs)]
//! Core of the Streamduck daemon, defines pretty much everything there is about the project

use std::sync::Arc;

pub use async_trait::async_trait;
pub use image as image_lib;

use crate::bundle::ManagerBundle;
use crate::devices::drivers::DriverManager;
use crate::events::EventDispatcher;

/// Everything related to devices
pub mod devices;
/// Everything related to images
pub mod images;
/// Event system of the core
pub mod events;
/// Assortment of various useful macros
pub mod macros;
/// Manager bundle definition
pub mod bundle;
/// Everything related to file writes and reads
pub mod config;

#[cfg(test)]
mod tests;

/// Initialize all managers
pub async fn init_managers() -> Option<Arc<ManagerBundle>> {
    let bundle = ManagerBundle::new();

    // Global event dispatcher
    bundle.global_dispatcher.set(EventDispatcher::new()).ok()?;


    // Driver manager
    bundle.driver_manager.set(DriverManager::new()).ok()?;


    Some(bundle)
}
