mod folders;

/// Definitions for UI controls for components
pub mod components;
/// Definition for event enumeration
pub mod events;
pub mod plugins;

use std::collections::HashMap;
use std::hash::Hasher;
use std::io::Cursor;
use std::sync::{Arc, RwLock, RwLockReadGuard};

use crate::core::button::{Button, parse_button_to_component};
use crate::core::methods::{check_feature_list_for_feature, CoreHandle};
use crate::modules::components::{ComponentDefinition, map_ui_values, map_ui_values_ref, UIField, UIFieldType, UIFieldValue, UIPathValue, UIValue};
use crate::modules::events::SDEvent;
use crate::modules::folders::FolderModule;

use serde::{Deserialize, Serialize};
use crate::core::thread::{ButtonBackground, ButtonText, ButtonTextShadow, RendererComponent};
use crate::util::rendering::{resize_for_streamdeck, TextAlignment};
use crate::versions::{CORE, MODULE_MANAGER};

use strum::VariantNames;
use std::str::FromStr;
use image::DynamicImage;
use image::io::Reader;
use crate::core::manager::CoreManager;
use crate::core::UniqueButton;
use crate::images::SDImage;
use crate::util::{add_array_function, change_from_path, convert_value_to_path, hash_str, remove_array_function, set_value_function};

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

/// The core module, for exposing renderer component to requests and such
struct CoreModule;

impl SDModule for CoreModule {
    fn name(&self) -> String {
        "core".to_string()
    }

    fn components(&self) -> HashMap<String, ComponentDefinition> {
        let mut map = HashMap::new();

        map.insert("renderer".to_string(), ComponentDefinition {
            display_name: "Renderer".to_string(),
            description: "The only thing that makes a button render an image on streamdeck".to_string(),
            default_looks: Default::default()
        });

        map
    }

    fn add_component(&self, _: CoreHandle, button: &mut Button, name: &str) {
        match name {
            "renderer" => {
                button.insert_component(RendererComponent::default()).ok();
            }
            _ => {}
        }
    }

    fn remove_component(&self, _: CoreHandle, button: &mut Button, name: &str) {
        match name {
            "renderer" => {
                button.remove_component::<RendererComponent>();
            }
            _ => {}
        }
    }

    fn component_values(&self, core: CoreHandle, button: &Button, name: &str) -> Vec<UIValue> {
        match name {
            "renderer" => {
                if let Ok(component) = parse_button_to_component::<RendererComponent>(button) {
                    let mut fields = vec![];

                    // Choice for background type
                    fields.push(
                        UIValue {
                            name: "background_params".to_string(),
                            display_name: "Background Parameters".to_string(),
                            ty: UIFieldType::Collapsable,
                            value: UIFieldValue::Collapsable({
                                let mut fields = vec![];

                                fields.push(
                                    UIValue {
                                        name: "background".to_string(),
                                        display_name: "Background Type".to_string(),
                                        ty: UIFieldType::Choice(vec!["Solid Color".to_string(), "Horizontal Gradient".to_string(), "Vertical Gradient".to_string(), "Existing Image".to_string(), "New Image".to_string()]),
                                        value: UIFieldValue::Choice(
                                            match &component.background {
                                                ButtonBackground::Solid(_) => "Solid Color",
                                                ButtonBackground::HorizontalGradient(_, _) => "Horizontal Gradient",
                                                ButtonBackground::VerticalGradient(_, _) => "Vertical Gradient",
                                                ButtonBackground::ExistingImage(_) => "Existing Image",
                                                ButtonBackground::NewImage(_) => "New Image",
                                            }.to_string()
                                        )
                                    }
                                );

                                // Different fields depending on background type
                                match &component.background {
                                    ButtonBackground::Solid(color) => {
                                        fields.push(
                                            UIValue {
                                                name: "color".to_string(),
                                                display_name: "Background Color".to_string(),
                                                ty: UIFieldType::Color,
                                                value: color.into()
                                            }
                                        );
                                    }

                                    ButtonBackground::HorizontalGradient(start_color, end_color) => {
                                        fields.push(
                                            UIValue {
                                                name: "start_color".to_string(),
                                                display_name: "Gradient Start Color".to_string(),
                                                ty: UIFieldType::Color,
                                                value: start_color.into()
                                            }
                                        );

                                        fields.push(
                                            UIValue {
                                                name: "end_color".to_string(),
                                                display_name: "Gradient End Color".to_string(),
                                                ty: UIFieldType::Color,
                                                value: end_color.into()
                                            }
                                        );
                                    }
                                    ButtonBackground::VerticalGradient(start_color, end_color) => {
                                        fields.push(
                                            UIValue {
                                                name: "start_color".to_string(),
                                                display_name: "Gradient Start Color".to_string(),
                                                ty: UIFieldType::Color,
                                                value: start_color.into()
                                            }
                                        );

                                        fields.push(
                                            UIValue {
                                                name: "end_color".to_string(),
                                                display_name: "Gradient End Color".to_string(),
                                                ty: UIFieldType::Color,
                                                value: end_color.into()
                                            }
                                        );
                                    }
                                    ButtonBackground::ExistingImage(identifier) => {
                                        fields.push(
                                            UIValue {
                                                name: "image".to_string(),
                                                display_name: "Image".to_string(),
                                                ty: UIFieldType::ExistingImage,
                                                value: UIFieldValue::ExistingImage(identifier.to_string())
                                            }
                                        );
                                    }
                                    ButtonBackground::NewImage(blob) => {
                                        fields.push(
                                            UIValue {
                                                name: "image".to_string(),
                                                display_name: "Image".to_string(),
                                                ty: UIFieldType::ImageData,
                                                value: UIFieldValue::ImageData(blob.to_string())
                                            }
                                        );
                                    }
                                }

                                fields
                            })
                        }
                    );

                    // Text array
                    fields.push(
                        UIValue {
                            name: "text_params".to_string(),
                            display_name: "Text Parameters".to_string(),
                            ty: UIFieldType::Collapsable,
                            value: UIFieldValue::Collapsable({
                                let mut fields = vec![];

                                fields.push(
                                    UIValue {
                                        name: "text".to_string(),
                                        display_name: "Text Objects".to_string(),
                                        ty: UIFieldType::Array(
                                            vec![
                                                UIField {
                                                    name: "text".to_string(),
                                                    display_name: "Text".to_string(),
                                                    ty: UIFieldType::InputFieldString,
                                                    default_value: UIFieldValue::InputFieldString("".to_string())
                                                },
                                                UIField {
                                                    name: "font".to_string(),
                                                    display_name: "Font".to_string(),
                                                    ty: UIFieldType::Font,
                                                    default_value: UIFieldValue::Font("default".to_string())
                                                },
                                                UIField {
                                                    name: "scale".to_string(),
                                                    display_name: "Text Scale".to_string(),
                                                    ty: UIFieldType::InputFieldFloat2,
                                                    default_value: UIFieldValue::InputFieldFloat2(1.0, 1.0)
                                                },
                                                UIField {
                                                    name: "alignment".to_string(),
                                                    display_name: "Alignment".to_string(),
                                                    ty: UIFieldType::Choice(
                                                        TextAlignment::VARIANTS.iter().map(|x| x.to_string()).collect()
                                                    ),
                                                    default_value: UIFieldValue::Choice("Center".to_string())
                                                },
                                                UIField {
                                                    name: "padding".to_string(),
                                                    display_name: "Padding".to_string(),
                                                    ty: UIFieldType::InputFieldUnsignedInteger,
                                                    default_value: UIFieldValue::InputFieldUnsignedInteger(0)
                                                },
                                                UIField {
                                                    name: "offset".to_string(),
                                                    display_name: "Text Offset".to_string(),
                                                    ty: UIFieldType::InputFieldFloat2,
                                                    default_value: UIFieldValue::InputFieldFloat2(0.0, 0.0)
                                                },
                                                UIField {
                                                    name: "color".to_string(),
                                                    display_name: "Text Color".to_string(),
                                                    ty: UIFieldType::Color,
                                                    default_value: UIFieldValue::Color(0, 0, 0, 255)
                                                },
                                                UIField {
                                                    name: "shadow_enabled".to_string(),
                                                    display_name: "Text Shadow".to_string(),
                                                    ty: UIFieldType::Checkbox {
                                                        disabled: false
                                                    },
                                                    default_value: UIFieldValue::Checkbox(false)
                                                }
                                            ]
                                        ),
                                        value: UIFieldValue::Array({
                                            let mut text_objects = vec![];

                                            for text in &component.text {
                                                let mut values = vec![];

                                                values.push(UIValue {
                                                    name: "text".to_string(),
                                                    display_name: "Text".to_string(),
                                                    ty: UIFieldType::InputFieldString,
                                                    value: UIFieldValue::InputFieldString(text.text.clone())
                                                });

                                                values.push(UIValue {
                                                    name: "font".to_string(),
                                                    display_name: "Font".to_string(),
                                                    ty: UIFieldType::Font,
                                                    value: UIFieldValue::Font(text.font.clone())
                                                });

                                                values.push(UIValue {
                                                    name: "scale".to_string(),
                                                    display_name: "Text Scale".to_string(),
                                                    ty: UIFieldType::InputFieldFloat2,
                                                    value: UIFieldValue::InputFieldFloat2(text.scale.0, text.scale.1)
                                                });

                                                values.push(UIValue {
                                                    name: "alignment".to_string(),
                                                    display_name: "Alignment".to_string(),
                                                    ty: UIFieldType::Choice(
                                                        TextAlignment::VARIANTS.iter().map(|x| x.to_string()).collect()
                                                    ),
                                                    value: UIFieldValue::Choice(text.alignment.to_string())
                                                });

                                                values.push(UIValue {
                                                    name: "padding".to_string(),
                                                    display_name: "Padding".to_string(),
                                                    ty: UIFieldType::InputFieldUnsignedInteger,
                                                    value: UIFieldValue::InputFieldUnsignedInteger(text.padding)
                                                });

                                                values.push(UIValue {
                                                    name: "offset".to_string(),
                                                    display_name: "Text Offset".to_string(),
                                                    ty: UIFieldType::InputFieldFloat2,
                                                    value: UIFieldValue::InputFieldFloat2(text.offset.0, text.offset.1)
                                                });

                                                values.push(UIValue {
                                                    name: "color".to_string(),
                                                    display_name: "Text Color".to_string(),
                                                    ty: UIFieldType::Color,
                                                    value: text.color.into()
                                                });

                                                if let Some(shadow) = &text.shadow {
                                                    values.push(
                                                        UIValue {
                                                            name: "shadow_enabled".to_string(),
                                                            display_name: "Text Shadow".to_string(),
                                                            ty: UIFieldType::Checkbox {
                                                                disabled: false
                                                            },
                                                            value: UIFieldValue::Checkbox(true)
                                                        }
                                                    );

                                                    values.push(UIValue {
                                                        name: "shadow_color".to_string(),
                                                        display_name: "Text Shadow Color".to_string(),
                                                        ty: UIFieldType::Color,
                                                        value: shadow.color.into()
                                                    });

                                                    values.push(UIValue {
                                                        name: "shadow_offset".to_string(),
                                                        display_name: "Text Shadow Offset".to_string(),
                                                        ty: UIFieldType::InputFieldInteger2,
                                                        value: UIFieldValue::InputFieldInteger2(shadow.offset.0, shadow.offset.1)
                                                    });
                                                } else {
                                                    values.push(
                                                        UIValue {
                                                            name: "shadow_enabled".to_string(),
                                                            display_name: "Text Shadow".to_string(),
                                                            ty: UIFieldType::Checkbox {
                                                                disabled: false
                                                            },
                                                            value: UIFieldValue::Checkbox(false)
                                                        }
                                                    );
                                                }

                                                text_objects.push(values);
                                            }

                                            text_objects
                                        })
                                    }
                                );

                                fields
                            })
                        }
                    );

                    // Ignore plugin rendering menu
                    fields.push(
                        UIValue {
                            name: "plugin_blacklist".to_string(),
                            display_name: "Allowed plugins to render".to_string(),
                            ty: UIFieldType::Collapsable,
                            value: UIFieldValue::Collapsable({
                                let names = core.module_manager().get_modules_for_rendering(&button.component_names());

                                names.into_values()
                                    .map(|x| {
                                        let name = x.name();

                                        UIValue {
                                            name: name.clone(),
                                            display_name: name.clone(),
                                            ty: UIFieldType::Checkbox { disabled: false },
                                            value: UIFieldValue::Checkbox(!component.plugin_blacklist.contains(&name))
                                        }
                                    }).collect()
                            })
                        }
                    );

                    fields.push(
                        UIValue {
                            name: "to_cache".to_string(),
                            display_name: "Caching".to_string(),
                            ty: UIFieldType::Checkbox {
                                disabled: false
                            },
                            value: UIFieldValue::Checkbox(component.to_cache)
                        }
                    );

                    fields
                } else {
                    vec![]
                }
            }

            _ => vec![],
        }
    }

    fn set_component_value(&self, core: CoreHandle, button: &mut Button, name: &str, value: Vec<UIValue>) {
        match name {
            "renderer" => {
                if let Ok(mut component) = parse_button_to_component::<RendererComponent>(button) {
                    let change_map = map_ui_values(value);

                    if let Some(value) = change_map.get("background_params") {
                        if let UIFieldValue::Collapsable(value) = &value.value {
                            let change_map = map_ui_values(value.clone());

                            // Setting background type
                            if let Some(value) = change_map.get("background") {
                                if let Ok(choice) = value.value.try_into_string() {
                                    match choice.as_str() {
                                        "Solid Color" => component.background = ButtonBackground::Solid((0, 0, 0, 0)),
                                        "Horizontal Gradient" => component.background = ButtonBackground::HorizontalGradient((0, 0, 0, 255), (0, 0, 0, 255)),
                                        "Vertical Gradient" => component.background = ButtonBackground::VerticalGradient((0, 0, 0, 255), (0, 0, 0, 255)),
                                        "Existing Image" => component.background = ButtonBackground::ExistingImage("".to_string()),
                                        "New Image" => component.background = ButtonBackground::NewImage("".to_string()),

                                        _ => {}
                                    }
                                }
                            }

                            // Background type related parameters
                            if let Some(value) = change_map.get("color") {
                                if let ButtonBackground::Solid(_) = component.background {
                                    if let Ok(color) = (&value.value).try_into() {
                                        component.background = ButtonBackground::Solid(color);
                                    }
                                }
                            }

                            if let Some(value) = change_map.get("start_color") {
                                if let ButtonBackground::HorizontalGradient(_, end) = component.background {
                                    if let Ok(color) = (&value.value).try_into() {
                                        component.background = ButtonBackground::HorizontalGradient(color, end);
                                    }
                                }

                                if let ButtonBackground::VerticalGradient(_, end) = component.background {
                                    if let Ok(color) = (&value.value).try_into() {
                                        component.background = ButtonBackground::VerticalGradient(color, end);
                                    }
                                }
                            }

                            if let Some(value) = change_map.get("end_color") {
                                if let ButtonBackground::HorizontalGradient(start, _) = component.background {
                                    if let Ok(color) = (&value.value).try_into() {
                                        component.background = ButtonBackground::HorizontalGradient(start, color);
                                    }
                                }

                                if let ButtonBackground::VerticalGradient(start, _) = component.background {
                                    if let Ok(color) = (&value.value).try_into() {
                                        component.background = ButtonBackground::VerticalGradient(start, color);
                                    }
                                }
                            }

                            if let Some(value) = change_map.get("image") {
                                match &component.background {
                                    ButtonBackground::ExistingImage(_) => {
                                        if let Ok(identifier) = (&value.value).try_into() {
                                            component.background = ButtonBackground::ExistingImage(identifier);
                                        }
                                    }

                                    ButtonBackground::NewImage(_) => {
                                        if let Ok(blob) = (&value.value).try_into_string() {
                                            fn decode_blob(blob: &String) -> Option<(String, DynamicImage)> {
                                                let identifier = hash_str(blob);
                                                if let Ok(decoded_bytes) = base64::decode(blob) {
                                                    if let Ok(recognized_image) = Reader::new(Cursor::new(&decoded_bytes)).with_guessed_format() {
                                                        if let Ok(decoded_image) = recognized_image.decode() {
                                                            drop(decoded_bytes);
                                                            return Some((identifier, decoded_image));
                                                        }
                                                        drop(decoded_bytes);
                                                    }
                                                }

                                                None
                                            }

                                            if let Some((identifier, image)) = decode_blob(&blob) {
                                                component.background = ButtonBackground::ExistingImage(identifier.clone());

                                                let mut handle = core.core.image_collection.write().unwrap();
                                                handle.insert(identifier, SDImage::SingleImage(resize_for_streamdeck(core.core.image_size, image)));
                                            } else {
                                                component.background = ButtonBackground::NewImage(blob);
                                            }
                                        }
                                    }

                                    _ => {}
                                }
                            }
                        }
                    }

                    if let Some(value) = change_map.get("text_params") {
                        if let UIFieldValue::Collapsable(value) = &value.value {
                            let change_map = map_ui_values(value.clone());

                            if let Some(value) = change_map.get("text") {
                                if let UIFieldValue::Array(items) = &value.value {
                                    component.text = vec![];

                                    fn get_text_object(item: &Vec<UIValue>) -> Option<ButtonText> {
                                        let map = map_ui_values_ref(item);

                                        Some(ButtonText {
                                            text: (&map.get("text")?.value).try_into().ok()?,
                                            font: (&map.get("font")?.value).try_into().ok()?,
                                            scale: (&map.get("scale")?.value).try_into().ok()?,
                                            alignment: TextAlignment::from_str(&map.get("alignment")?.value.try_into_string().ok()?).ok()?,
                                            padding: (&map.get("padding")?.value).try_into().ok()?,
                                            offset: (&map.get("offset")?.value).try_into_f32_f32().ok()?,
                                            color: (&map.get("color")?.value).try_into().ok()?,
                                            shadow: if let Some(bool) = map.get("shadow_enabled")?.value.try_into_bool().ok() {
                                                let get_shadow = || {
                                                    Some(ButtonTextShadow {
                                                        offset: (&map.get("shadow_offset")?.value).try_into().ok()?,
                                                        color: (&map.get("shadow_color")?.value).try_into().ok()?
                                                    })
                                                };

                                                if bool {
                                                    get_shadow().or(Some(ButtonTextShadow {
                                                        offset: (0, 0),
                                                        color: (0, 0, 0, 0)
                                                    }))
                                                } else {
                                                    None
                                                }
                                            } else {
                                                None
                                            }
                                        })
                                    }

                                    for item in items {
                                        if let Some(object) = get_text_object(item) {
                                            component.text.push(object)
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if let Some(value) = change_map.get("plugin_blacklist") {
                        if let UIFieldValue::Collapsable(value) = &value.value {
                            let change_map = map_ui_values(value.clone());

                            for (name, value) in change_map {
                                if let UIFieldValue::Checkbox(state) = value.value {
                                    if state {
                                        component.plugin_blacklist.retain(|x| *x != name);
                                    } else {
                                        component.plugin_blacklist.push(name);
                                    }
                                }
                            }
                        }
                    }

                    if let Some(value) = change_map.get("to_cache") {
                        if let Ok(value) = value.value.try_into_bool() {
                            component.to_cache = value;
                        }
                    }

                    // Apply changes to button
                    button.insert_component(component).ok();

                    core.core.mark_for_redraw();
                }
            }

            _ => {}
        }
    }

    fn listening_for(&self) -> Vec<String> {
        vec![]
    }

    fn event(&self, _: CoreHandle, _: SDEvent) {

    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata::from_literals(
            "core",
            "TheJebForge",
            "Core of the software, provides essential components",
            "0.1",
            &[
                CORE,
                MODULE_MANAGER
            ]
        )
    }
}
