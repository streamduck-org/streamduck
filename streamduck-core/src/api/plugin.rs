use std::path::Path;
use crate::api::driver::Driver;
use crate::api::id::PluginID;
use crate::util::id_set::IdSet;

/// It's a plugin, plugin is expected to initialize and declare everything as it's being loaded
pub trait Plugin {
    fn name(&self) -> String;
    /// Is ran when plugins are done loading
    fn post_init(&self);
    fn drivers(&self) -> Vec<&dyn Driver>;
    
    fn tick(&self);
}

pub type BoxPlugin = Box<dyn Plugin>;
pub type RefPlugin<'a> = &'a dyn Plugin;

/// The thing that brings plugins into the application by whatever means it can
pub trait PluginSource {
    fn name(&self) -> &'static str;

    fn load_plugins(&self, id_set: &mut IdSet<PluginID>, plugin_path: impl AsRef<Path>) -> Vec<BoxPlugin>;
}