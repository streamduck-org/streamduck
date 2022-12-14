#![warn(missing_docs)]
//! Core of the Streamduck daemon, defines pretty much everything there is about the project

use std::sync::Arc;

pub use async_trait::async_trait;
pub use image as image_lib;

use crate::bundle::{ManagerBundle, ManagerInitError};
use crate::events::EventDispatcher;

/// Manager bundle definition
pub mod bundle;
/// Everything related to file writes and reads
pub mod config;
/// Everything related to devices
pub mod devices;
/// Event system of the core
pub mod events;
/// Everything related to images
pub mod images;
/// Localization types
pub mod localization;
/// Assortment of various useful macros
mod macros;
/// Dynamic parameters
pub mod parameters;

#[cfg(test)]
mod tests;

/// Initialize all managers
pub async fn init_managers() -> Result<Arc<ManagerBundle>, ManagerInitError> {
    let bundle = ManagerBundle::new().await?;

    Ok(bundle)
}
