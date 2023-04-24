#![warn(missing_docs)]

//! Main functionality of the project

use std::collections::HashMap;
use std::sync::Arc;
use crate::core::Core;
use crate::device::DeviceIdentifier;
use crate::plugin::Plugin;

/// Core that would manage the device
pub mod core;

/// Device types
pub mod device;

/// Various data structures
pub mod data;

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
    plugins: HashMap<String, Plugin>,
    cores: HashMap<DeviceIdentifier, Arc<Core>>,

}
