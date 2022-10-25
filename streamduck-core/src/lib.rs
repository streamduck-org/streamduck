#![warn(missing_docs)]
//! Core of the Streamduck daemon, defines pretty much everything there is about the project

/// Everything related to devices
pub mod devices;
/// Everything related to images
pub mod images;

pub use async_trait::async_trait;