/// Builder for all plugin related structs
pub mod builder;

use std::collections::HashMap;
use std::sync::{Arc, OnceLock, Weak};
use std::sync::atomic::AtomicBool;
use crate::config::SharedConfig;
use crate::core::{SharedAction, SharedOverlay};
use crate::data::NamespacedName;
use crate::device::driver::SharedDriver;

/// Plugin Arc
pub type SharedPlugin = Arc<Plugin>;

/// Weak shared reference to a plugin
pub type WeakPlugin = Weak<Plugin>;

/// Structure that contains different things that the plugin defined
pub struct Plugin {
    pub(crate) config: SharedConfig,
    /// Name of the plugin
    pub name: String,
    /// Flag if plugin is still busy processing a tick
    pub busy: AtomicBool,
    /// Overlays introduced by the plugin, ensured to be set by the plugin loaders
    pub overlays: OnceLock<HashMap<String, SharedOverlay>>,
    /// Actions introduced by the plugin, ensured to be set by the plugin loaders
    pub actions: OnceLock<HashMap<String, SharedAction>>,
    /// Drivers introduced by the plugin, ensured to be set by the plugin loaders
    pub drivers: OnceLock<HashMap<String, SharedDriver>>,
    /// Plugin holder
    pub holder: Arc<dyn PluginHolder>
}

impl Plugin {
    /// Creates a new name based on the plugin
    pub fn new_name(&self, name: &str) -> NamespacedName {
        NamespacedName::from_plugin(self, name)
    }

    /// Gets overlays introduced by the plugin
    pub fn overlays(&self) -> &HashMap<String, SharedOverlay> {
        self.overlays.get().unwrap()
    }

    /// Gets overlays introduced by the plugin
    pub fn actions(&self) -> &HashMap<String, SharedAction> {
        self.actions.get().unwrap()
    }

    /// Gets overlays introduced by the plugin
    pub fn drivers(&self) -> &HashMap<String, SharedDriver> {
        self.drivers.get().unwrap()
    }
}

/// Trait that plugin loaders should implement to hold any important memory
pub trait PluginHolder: Send + Sync {}