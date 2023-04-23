#![warn(missing_docs)]

//! Main functionality of the project

/// Core that would manage the device
pub mod core;

/// Dynamic data definition
pub mod data;

/// Device types
pub mod device;

/// Event system
pub mod event;

/// Structures for UI schema
pub mod ui;

/// Trigger condition definitions
pub mod trigger;

/// Various utility functions
pub mod util;

/// Plugin definitions
pub mod plugin;


/// Instance of the core of the software, is what loads the plugins and makes things tick
pub struct Streamduck {

}