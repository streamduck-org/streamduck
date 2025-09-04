use std::collections::HashMap;
use crate::api::driver::RefDriver;
use crate::api::id::NamespacedID;
use crate::api::plugin::RefPlugin;

pub trait PluginIndex {
    fn all_plugins(&self) -> Vec<RefPlugin<'_>>;
    fn find_plugin(&self, id: &str) -> Option<RefPlugin<'_>>;

    fn all_drivers(&self) -> HashMap<NamespacedID, RefDriver<'_>>;
    fn all_plugin_drivers(&self, id: &str) -> Vec<RefDriver<'_>>;
    fn find_driver(&self, id: &NamespacedID) -> Option<RefDriver<'_>>;


    // Provided methods
    fn find_driver_with_str(&self, plugin: &str, id: &str) -> Option<RefDriver<'_>> {
        self.find_driver(&NamespacedID::new(id.to_string(), plugin.to_string()))
    }
}