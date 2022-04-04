mod folders;

/// Definitions for UI controls for components
pub mod components;
/// Definition for event enumeration
pub mod events;
pub mod plugins;
pub mod core_module;

use std::collections::HashMap;
use std::hash::Hasher;
use std::sync::{Arc, RwLock, RwLockReadGuard};

use crate::core::button::{Button};
use crate::core::methods::{check_feature_list_for_feature, CoreHandle};
use crate::modules::components::{ComponentDefinition, UIPathValue, UIValue};
use crate::modules::events::SDEvent;
use crate::modules::folders::FolderModule;

use serde::{Deserialize, Serialize};

use image::DynamicImage;
use crate::core::manager::CoreManager;
use crate::core::UniqueButton;
use crate::modules::core_module::CoreModule;
use crate::util::{add_array_function, change_from_path, convert_value_to_path, remove_array_function, set_value_function};

/// Manages modules
#[derive(Default)]
pub struct ModuleManager {
    // Using a bunch of various maps to move performance cost to adding module, so getting info is as costless as possible

    module_map: RwLock<HashMap<String, UniqueSDModule>>,
    module_component_map: RwLock<HashMap<String, HashMap<String, ComponentDefinition>>>,
    component_map: RwLock<HashMap<String, (ComponentDefinition, UniqueSDModule)>>,
    component_listener_map: RwLock<HashMap<String, Vec<UniqueSDModule>>>,

    /// Separate list of modules that can render things
    rendering_modules: RwLock<HashMap<String, HashMap<String, UniqueSDModule>>>,
}

impl ModuleManager {
    /// Creates new module manager, used in daemon for loading plugins and base modules
    pub fn new() -> Arc<ModuleManager> {
        Arc::new(ModuleManager::default())
    }

    /// Adds a new module to be used with core
    pub fn add_module(&self, module: UniqueSDModule) {
        let module_name = module.name();

        // Adding to module map
        let mut module_map = self.module_map.write().unwrap();
        module_map.insert(module_name.clone(), module.clone());
        drop(module_map);

        // Adding to module component map
        let mut module_component_map = self.module_component_map.write().unwrap();
        for (component, definition) in module.components() {
            if let Some(component_map) = module_component_map.get_mut(&module_name) {
                component_map.insert(component, definition);
            } else {
                module_component_map.insert(module_name.clone(), {
                    let mut map = HashMap::new();
                    map.insert(component, definition);
                    map
                });
            }
        }
        drop(module_component_map);

        // Adding to component to module map
        let mut component_map = self.component_map.write().unwrap();
        for (component, definition) in module.components() {
            component_map.insert(component, (definition, module.clone()));
        }
        drop(component_map);

        // Adding to component listener map
        let mut component_listener_map = self.component_listener_map.write().unwrap();
        for listens_for in module.listening_for() {
            if let Some(array) = component_listener_map.get_mut(&listens_for) {
                array.push(module.clone());
            } else {
                component_listener_map.insert(listens_for, vec![module.clone()]);
            }
        }
        drop(component_listener_map);

        // Adding rendering modules to rendering map
        let mut rendering_modules = self.rendering_modules.write().unwrap();
        if check_feature_list_for_feature(&module.metadata().used_features, "rendering") {
            for component in module.listening_for() {
                if let Some(map) = rendering_modules.get_mut(&component) {
                    map.insert(module.name(), module.clone());
                } else {
                    rendering_modules.insert(component, {
                        let mut map = HashMap::new();

                        map.insert(module.name(), module.clone());

                        map
                    });
                }
            }
        }
        drop(rendering_modules);
    }

    /// Attempts to get module with specified name
    pub fn get_module(&self, name: &str) -> Option<UniqueSDModule> {
        self.get_modules().get(name).cloned()
    }

    /// Returns all modules in map format
    pub fn get_modules(&self) -> HashMap<String, UniqueSDModule> {
        self.module_map.read().unwrap().clone()
    }

    /// Returns all modules in vector format
    pub fn get_module_list(&self) -> Vec<UniqueSDModule> {
        self.module_map.read().unwrap().values().cloned().collect()
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
        let handle = self.component_listener_map.read().unwrap();

        if let Some(modules) = handle.get(component) {
            modules.clone()
        } else {
            vec![]
        }
    }

    /// Retrieves modules that have added specified components
    pub fn get_modules_for_declared_components(&self, components: &[String]) -> Vec<UniqueSDModule> {
        let handle = self.component_map.read().unwrap();

        let mut shared_modules = vec![];

        for component in components {
            if let Some((_, module)) = handle.get(component) {
                shared_modules.push(module.clone());
            }
        }

        shared_modules.sort_by(|a, b| a.name().cmp(&b.name()));
        shared_modules.dedup_by(|a, b| a.name() == b.name());

        shared_modules
    }

    /// Retrieves modules that are listening to specified components
    pub fn get_modules_for_components(&self, components: &[String]) -> Vec<UniqueSDModule> {
        let handle = self.component_listener_map.read().unwrap();

        let mut shared_modules = vec![];

        for component in components {
            if let Some(modules) = handle.get(component) {
                shared_modules.extend(modules.clone());
            }
        }

        shared_modules.sort_by(|a, b| a.name().cmp(&b.name()));
        shared_modules.dedup_by(|a, b| a.name() == b.name());

        shared_modules
    }

    /// Retrieves components that module defined
    pub fn get_components_of_module(&self, module_name: &str) -> Option<HashMap<String, ComponentDefinition>> {
        let handle = self.module_map.read().unwrap();

        if let Some(module) = handle.get(module_name) {
            Some(module.components())
        } else {
            None
        }
    }

    /// Retrieves all components that all modules define
    pub fn get_components(&self) -> HashMap<String, (ComponentDefinition, UniqueSDModule)> {
        self.component_map.read().unwrap().clone()
    }

    /// Retrieves all components that all modules define, but in module to component map format
    pub fn get_module_component_map(&self) -> HashMap<String, HashMap<String, ComponentDefinition>> {
        self.module_component_map.read().unwrap().clone()
    }

    /// Retrieves all modules that can render things
    pub fn get_rendering_module_map(&self) -> HashMap<String, HashMap<String, UniqueSDModule>> {
        self.rendering_modules.read().unwrap().clone()
    }

    /// Retrieves all modules that should be able to render according to list of component names
    pub fn get_modules_for_rendering(&self, names: &Vec<String>) -> HashMap<String, UniqueSDModule> {
        let rendering_map = self.rendering_modules.read().unwrap();

        let mut map = HashMap::new();

        for name in names {
            if let Some(modules) = rendering_map.get(name) {
                map.extend(modules.clone())
            }
        }

        map
    }


    /// Retrieves component if it exists
    pub fn get_component(&self, component_name: &str) -> Option<(ComponentDefinition, UniqueSDModule)> {
        self.component_map.read().unwrap().get(component_name).cloned()
    }

    /// Returns module map read lock
    pub fn read_module_map(&self) -> RwLockReadGuard<HashMap<String, UniqueSDModule>> {
        self.module_map.read().unwrap()
    }

    /// Returns component map read lock
    pub fn read_component_map(&self) -> RwLockReadGuard<HashMap<String, (ComponentDefinition, UniqueSDModule)>> {
        self.component_map.read().unwrap()
    }

    /// Returns module component map read lock
    pub fn read_module_component_map(&self) -> RwLockReadGuard<HashMap<String, HashMap<String, ComponentDefinition>>> {
        self.module_component_map.read().unwrap()
    }

    /// Returns component listener map read lock
    pub fn read_component_listener_map(&self) -> RwLockReadGuard<HashMap<String, Vec<UniqueSDModule>>> {
        self.component_listener_map.read().unwrap()
    }

    /// Returns rendering modules map read lock
    pub fn read_rendering_modules_map(&self) -> RwLockReadGuard<HashMap<String, HashMap<String, UniqueSDModule>>> {
        self.rendering_modules.read().unwrap()
    }
}

/// Loads built-in modules into the module manager
pub fn load_base_modules(module_manager: Arc<ModuleManager>) {
    module_manager.add_module(Arc::new(Box::new(CoreModule)));
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
    fn add_component(&self, core: CoreHandle, button: &mut Button, name: &str);

    /// Method for removing components from buttons
    fn remove_component(&self, core: CoreHandle, button: &mut Button, name: &str);

    /// Method for handling pasting components of plugin, can be used for any additional handling
    fn paste_component(&self, core: CoreHandle, reference_button: &Button, new_button: &mut Button);

    /// Method for letting core know what values component currently has
    fn component_values(&self, core: CoreHandle, button: &Button, name: &str) -> Vec<UIValue>;

    /// Method for setting values on components
    fn set_component_value(&self, core: CoreHandle, button: &mut Button, name: &str, value: Vec<UIValue>);

    /// Specifies which components the module will be receiving events for
    fn listening_for(&self) -> Vec<String>;

    /// Current settings state of the plugin
    fn settings(&self, core_manager: Arc<CoreManager>) -> Vec<UIValue> { vec![] }

    /// Method for updating plugin settings from UI
    fn set_setting(&self, core_manager: Arc<CoreManager>, value: Vec<UIValue>) { }

    /// Method for handling core events, add EVENTS feature to the plugin metadata to receive events
    fn event(&self, core: CoreHandle, event: SDEvent) {}

    /// Method renderer will run for rendering additional information on a button if RENDERING feature was specified
    fn render(&self, core: CoreHandle, button: &UniqueButton, frame: &mut DynamicImage) {}

    /// Method for telling renderer if anything changed
    ///
    /// Changing state of the hash in anyway will cause renderer to either rerender, or use previous cache.
    /// This method will also called very frequently, so keep code in here fast
    fn render_hash(&self, core: CoreHandle, button: &UniqueButton, hash: &mut Box<dyn Hasher>) {}

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

/// Retrieves module settings in array of UIPathValue
pub fn get_module_settings(core_manager: Arc<CoreManager>, module: &UniqueSDModule) -> Vec<UIPathValue> {
    module.settings(core_manager)
        .into_iter()
        .map(|x| convert_value_to_path(x, ""))
        .collect()
}

/// Adds new element into module setting's array
pub fn add_element_module_setting(core_manager: Arc<CoreManager>, module: &UniqueSDModule, path: &str) -> bool {
    let (changes, success) = change_from_path(path, module.settings(core_manager.clone()), &add_array_function(), false);

    if success {
        if !changes.is_empty() {
            module.set_setting(core_manager, changes);
            true
        } else {
            false
        }
    } else {
        false
    }
}

/// Removes an element from module setting's array
pub fn remove_element_module_setting(core_manager: Arc<CoreManager>, module: &UniqueSDModule, path: &str, index: usize) -> bool {
    let (changes, success) = change_from_path(path, module.settings(core_manager.clone()), &remove_array_function(index), false);

    if success {
        if !changes.is_empty() {
            module.set_setting(core_manager, changes);
            true
        } else {
            false
        }
    } else {
        false
    }
}

/// Sets value into module's setting
pub fn set_module_setting(core_manager: Arc<CoreManager>, module: &UniqueSDModule, value: UIPathValue) -> bool {
    let (changes, success) = change_from_path(&value.path, module.settings(core_manager.clone()), &set_value_function(value.clone()), false);

    if success {
        if !changes.is_empty() {
            module.set_setting(core_manager, changes);
            true
        } else {
            false
        }
    } else {
        false
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