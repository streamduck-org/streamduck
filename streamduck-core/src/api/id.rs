use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};

/// Plugin name is the ID
pub type PluginID = String;

/// ID of the type that belongs to a plugin
#[derive(Debug, Clone, Hash, Serialize, Deserialize, Eq, PartialEq)]
pub struct NamespacedID {
    pub id: String,
    pub plugin: PluginID
}

impl NamespacedID {
    pub fn new(id: String, plugin: PluginID) -> NamespacedID {
        Self { id, plugin }
    }
}

impl Display for NamespacedID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} [{}]", self.id, self.plugin)
    }
}