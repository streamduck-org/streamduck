mod actions;
mod overlays;

pub use actions::*;
pub use overlays::*;

use std::sync::Arc;
use crate::device::Device;

/// The thing that manages a particular device
pub struct Core {
    /// Device that's being managed by the core
    pub device: Arc<Device>,

}