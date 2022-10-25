use std::sync::Arc;

/// Device drivers
pub mod drivers;
/// Device metadata
pub mod metadata;
/// Button structures
pub mod buttons;

/// Device interface
pub trait Device {

}

/// Device interface contained in a reference counter
pub type SharedDevice = Arc<dyn Device>;