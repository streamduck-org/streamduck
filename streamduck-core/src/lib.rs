#![warn(missing_docs)]
//! Core of the Streamduck daemon, defines pretty much everything there is about the project

use std::sync::Arc;

pub use async_trait::async_trait;
use hidapi::HidError;
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
pub async fn init_managers() -> Result<Arc<ManagerBundle>, ManagerInitError> {
    let bundle = ManagerBundle::new().await;

    // Global event dispatcher
    bundle.global_dispatcher.set(EventDispatcher::new())
        .map_err(|_| ManagerInitError::FailedOnceCell)?;


    // Driver manager
    bundle.driver_manager.set(DriverManager::new()?)
        .map_err(|_| ManagerInitError::FailedOnceCell)?;


    Ok(bundle)
}

/// Error that might be returned by the managers as they init
#[derive(Debug)]
pub enum ManagerInitError {
    /// Failed to set value to OnceCell
    FailedOnceCell,

    /// Error that happened while initializing HidApi
    HidError(HidError)
}

impl From<HidError> for ManagerInitError {
    fn from(e: HidError) -> Self {
        Self::HidError(e)
    }
}