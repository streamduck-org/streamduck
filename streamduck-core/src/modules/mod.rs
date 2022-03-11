mod folders;

/// Definitions for UI controls for components
pub mod components;
/// Definition for event enumeration
pub mod events;
pub mod plugins;

use std::collections::HashMap;
use std::io::Cursor;
use std::sync::{Arc, RwLock};

use crate::core::button::{Button, parse_button_to_component};
use crate::core::methods::CoreHandle;
use crate::modules::components::{ComponentDefinition, map_ui_values, map_ui_values_ref, UIField, UIFieldType, UIFieldValue, UIPathValue, UIValue};
use crate::modules::events::SDEvent;
use crate::modules::folders::FolderModule;

use serde::{Deserialize, Serialize};
use crate::threads::rendering::{ButtonBackground, ButtonText, ButtonTextShadow, RendererComponent};
use crate::util::rendering::{resize_for_streamdeck, TextAlignment};
use crate::versions::CORE;

use strum::VariantNames;
use std::str::FromStr;
use image::DynamicImage;
use image::io::Reader;
use crate::util::{add_array_function, change_from_path, convert_value_to_path, hash_image, remove_array_function, set_value_function};

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

    pub fn get_component(&self, component_name: &str) -> Option<ComponentDefinition> {
        let handle = self.0.read().unwrap();

        for module in handle.iter() {
            if let Some(definition) = module.components().remove(component_name) {
                return Some(definition);
            }
        }

        None
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
    fn settings(&self) -> Vec<UIValue> { vec![] }

    /// Method for updating plugin settings from UI
    fn set_setting(&self, value: Vec<UIValue>) { }

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

/// Retrieves module settings in array of UIPathValue
pub fn get_module_settings(module: &UniqueSDModule) -> Vec<UIPathValue> {
    module.settings()
        .into_iter()
        .map(|x| convert_value_to_path(x, ""))
        .collect()
}

/// Adds new element into module setting's array
pub fn add_element_module_setting(module: &UniqueSDModule, path: &str) -> bool {
    let (changes, success) = change_from_path(path, module.settings(), &add_array_function(), false);

    if success {
        if !changes.is_empty() {
            module.set_setting(changes);
            true
        } else {
            false
        }
    } else {
        false
    }
}

/// Removes an element from module setting's array
pub fn remove_element_module_setting(module: &UniqueSDModule, path: &str, index: usize) -> bool {
    let (changes, success) = change_from_path(path, module.settings(), &remove_array_function(index), false);

    if success {
        if !changes.is_empty() {
            module.set_setting(changes);
            true
        } else {
            false
        }
    } else {
        false
    }
}

/// Sets value into module's setting
pub fn set_module_setting(module: &UniqueSDModule, value: UIPathValue) -> bool {
    let (changes, success) = change_from_path(&value.path, module.settings(), &set_value_function(value.clone()), false);

    if success {
        if !changes.is_empty() {
            module.set_setting(changes);
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

    fn add_component(&self, core: CoreHandle, button: &mut Button, name: &str) {
        match name {
            "renderer" => {
                button.insert_component(RendererComponent::default()).ok();
                core.core().mark_for_redraw();
            }
            _ => {}
        }
    }

    fn remove_component(&self, core: CoreHandle, button: &mut Button, name: &str) {
        match name {
            "renderer" => {
                button.remove_component::<RendererComponent>();
                core.core().mark_for_redraw();
            }
            _ => {}
        }
    }

    fn component_values(&self, _: CoreHandle, button: &Button, name: &str) -> Vec<UIValue> {
        match name {
            "renderer" => {
                if let Ok(component) = parse_button_to_component::<RendererComponent>(button) {
                    let mut fields = vec![];

                    // Choice for background type
                    fields.push(
                        UIValue {
                            name: "background_header".to_string(),
                            display_name: "- Background Parameters -".to_string(),
                            ty: UIFieldType::Header,
                            value: UIFieldValue::Header
                        }
                    );

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

                    // Text array
                    fields.push(
                        UIValue {
                            name: "text_header".to_string(),
                            display_name: "- Text Parameters -".to_string(),
                            ty: UIFieldType::Header,
                            value: UIFieldValue::Header
                        }
                    );

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

                    // Cache
                    fields.push(
                        UIValue {
                            name: "other_header".to_string(),
                            display_name: "- Other Parameters -".to_string(),
                            ty: UIFieldType::Header,
                            value: UIFieldValue::Header
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

                    // Setting background type
                    if let Some(value) = change_map.get("background") {
                        if let Ok(choice) = value.value.try_into_string() {
                            match choice.as_str() {
                                "Solid Color" => component.background = ButtonBackground::Solid((0, 0, 0, 0)),
                                "Horizontal Gradient" => component.background = ButtonBackground::HorizontalGradient((0, 0, 0, 0), (0, 0, 0, 0)),
                                "Vertical Gradient" => component.background = ButtonBackground::VerticalGradient((0, 0, 0, 0), (0, 0, 0, 0)),
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
                                        let identifier = hash_image(blob);
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
                                        handle.insert(identifier, resize_for_streamdeck(core.core.image_size, image));
                                    } else {
                                        component.background = ButtonBackground::NewImage(blob);
                                    }
                                }
                            }

                            _ => {}
                        }
                    }

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

                    if let Some(value) = change_map.get("to_cache") {
                        if let Ok(value) = value.value.try_into_bool() {
                            component.to_cache = value;
                        }
                    }

                    // Apply changes to button
                    button.insert_component(component).ok();

                    core.core().mark_for_redraw();
                }
            }

            _ => {}
        }
    }

    fn listening_for(&self) -> Vec<String> {
        vec![]
    }

    fn event(&self, _: CoreHandle, _: SDEvent) {}

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata::from_literals(
            "core",
            "TheJebForge",
            "Core of the software, provides essential components",
            "0.1",
            &[
                CORE
            ]
        )
    }
}
