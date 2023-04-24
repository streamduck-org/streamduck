use crate::data::NamespacedName;

/// Structure that contains different things that the plugin defined
pub struct Plugin {
    /// Name of the plugin
    pub(crate) name: String
}

impl Plugin {
    /// Creates a new name based on the plugin
    pub fn new_name(&self, name: &str) -> NamespacedName {
        NamespacedName::from_plugin(self, name)
    }
}