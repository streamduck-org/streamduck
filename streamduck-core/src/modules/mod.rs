mod folders;
/// Definitions for UI controls for components
pub mod components;
/// Definition for event enumeration
pub mod events;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::core::button::Button;
use crate::core::methods::CoreHandle;
use crate::modules::components::{ComponentDefinition, UIValue};
use crate::modules::events::SDEvent;
use crate::modules::folders::FolderModule;

use serde::{Serialize, Deserialize};

/// Manages modules
pub struct ModuleManager(RwLock<Vec<UniqueSDModule>>, RwLock<HashMap<String, Vec<String>>>);

impl ModuleManager {
    /// Creates new module manager, used in daemon for loading plugins and base modules
    pub fn new() -> Arc<ModuleManager> {
        Arc::new(ModuleManager(RwLock::default(), RwLock::default()))
    }

    /// Adds a new module to be used with core
    pub fn add_module(&self, module: UniqueSDModule) {
        self.0.write().unwrap().push(module.clone());

        let mut handle = self.1.write().unwrap();
        for component in module.listening_for() {
            if let Some(mut list) = handle.remove(&component) {
                list.push(module.name());
                handle.insert(component, list);
            } else {
                handle.insert(component, vec![module.name()]);
            }
        }
    }

    /// Attempts to get module with specified name
    pub fn get_module(&self, name: &str) -> Option<UniqueSDModule> {
        self.get_modules().get(name).cloned()
    }

    /// Returns all modules in map format
    pub fn get_modules(&self) -> HashMap<String, UniqueSDModule> {
        self.0.read().unwrap().iter().map(|x| (x.name(), x.clone())).collect()
    }

    /// Returns all modules in vector format
    pub fn get_module_list(&self) -> Vec<UniqueSDModule> {
        self.0.read().unwrap().clone()
    }

    /// Returns modules from names provided if they exist
    pub fn get_modules_from_list(&self, list: &[String]) -> Vec<UniqueSDModule> {
        let module_map = self.get_modules();

        let mut modules = vec![];

        for item in list {
            if let Some(module) = module_map.get(item) {
                modules.push(module.clone())
            }
        }

        modules
    }

    /// Retrieves modules that are listening to a specified component
    pub fn get_modules_for_component(&self, component: &str) -> Vec<UniqueSDModule> {
        let handle = self.1.read().unwrap();

        if let Some(modules) = handle.get(component) {
            self.get_modules_from_list(modules.as_slice())
        } else {
            vec![]
        }
    }

    /// Retrieves modules that are listening to specified components
    pub fn get_modules_for_components(&self, components: &[String]) -> Vec<UniqueSDModule> {
        let handle = self.1.read().unwrap();

        let mut module_names = vec![];

        for component in components {
            if let Some(modules) = handle.get(component) {
                module_names.extend(modules.clone());
            }
        }

        module_names.sort();
        module_names.dedup();

        self.get_modules_from_list(module_names.as_slice())
    }

    /// Retrieves components that module defined
    pub fn get_components_of_module(&self, module_name: &str) -> Option<HashMap<String, ComponentDefinition>> {
        let handle = self.0.read().unwrap();

        for module in handle.iter() {
            if module.name() == module_name {
                return Some(module.components())
            }
        }

        None
    }

    /// Retrieves all components that all modules define
    pub fn get_components_list_by_modules(&self) -> Vec<(String, Vec<(String, ComponentDefinition)>)> {
        let handle = self.0.read().unwrap();
        let mut result = vec![];

        for module in handle.iter() {
            let mut components: Vec<(String, ComponentDefinition)> = module.components()
                .iter()
                .map(|(name, def)| (name.clone(), def.clone()))
                .collect();

            components.sort_by(|(a, ..), (b, ..)| {
                a.cmp(b)
            });

            result.push((module.name(), components))
        }

        result.sort_by(|(a, ..), (b, ..)| {
            a.cmp(b)
        });

        result
    }
}

/// Loads built-in modules into the module manager
pub fn load_base_modules(module_manager: Arc<ModuleManager>) {
    module_manager.add_module(Arc::new(Box::new(FolderModule::default())));
}

/// Reference counted module object
pub type UniqueSDModule = Arc<Box<dyn SDModule>>;

/// Boxed module object
pub type BoxedSDModule = Box<dyn SDModule>;

/// Raw pointer to module object
pub type SDModulePointer = *mut dyn SDModule;

/// Module trait
#[allow(unused)]
pub trait SDModule: Send + Sync {
    // Module data
    /// Module name
    fn name(&self) -> String;

    // Components
    /// Definition for components that module will be providing
    fn components(&self) -> HashMap<String, ComponentDefinition>;

    /// Method for adding components onto buttons
    fn add_component(&self, button: &mut Button, name: &str);

    /// Method for letting core know what values component currently has
    fn component_values(&self, button: &Button, name: &str) -> Vec<UIValue>;

    /// Method for setting values on components
    fn set_component_value(&self, button: &mut Button, name: &str, value: UIValue);

    /// Specifies which components the module will be receiving events for
    fn listening_for(&self) -> Vec<String>;

    /// Current settings state of the plugin
    fn settings(&self) -> Vec<UIValue> { vec![] }

    /// Method for updating plugin settings from UI
    fn set_setting(&self, value: UIValue) { }

    /// Method for handling core events, add EVENTS feature to the plugin metadata to receive events
    fn event(&self, core: CoreHandle, event: SDEvent);

    /// Metadata of the module, auto-implemented for plugins from plugin metadata
    fn metadata(&self) -> PluginMetadata {
        let mut meta = PluginMetadata::default();

        meta.name = self.name();

        meta
    }
}

/// Keeps relevant information about plugins
#[derive(Serialize, Deserialize, Clone)]
pub struct PluginMetadata {
    /// Name of the plugin
    pub name: String,
    /// Author of the plugin
    pub author: String,
    /// Description of the plugin
    pub description: String,
    /// Version of the plugin
    pub version: String,
    /// Used features of the plugin, used to determine if plugin is compatible with different software versions, see [crate::versions]
    pub used_features: Vec<(String, String)>
}

impl PluginMetadata {
    /// Lets you create plugin metadata without having to bother with creating strings for each property
    pub fn from_literals(name: &str, author: &str, description: &str, version: &str, used_features: &[(&str, &str)]) -> PluginMetadata {
        PluginMetadata {
            name: name.to_string(),
            author: author.to_string(),
            description: description.to_string(),
            version: version.to_string(),
            used_features: features_to_vec(used_features)
        }
    }
}

/// Converts features slice into Vec
pub fn features_to_vec(features: &[(&str, &str)]) -> Vec<(String, String)> {
    features.iter().map(|(n, v)| (n.to_string(), v.to_string())).collect()
}

impl Default for PluginMetadata {
    fn default() -> Self {
        PluginMetadata::from_literals(
            "unspecified",
            "unspecified",
            "unspecified",
            "unspecified",
            &[]
        )
    }
}