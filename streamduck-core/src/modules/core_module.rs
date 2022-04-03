//! Core module

use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;
use image::DynamicImage;
use image::io::Reader;
use std::str::FromStr;
use strum::VariantNames;
use serde::{Serialize, Deserialize};
use crate::config::PluginConfig;
use crate::core::button::{Button, parse_button_to_component};
use crate::core::manager::CoreManager;
use crate::core::methods::{check_feature_list_for_feature, CoreHandle};
use crate::core::thread::{ButtonBackground, ButtonText, ButtonTextShadow, RendererComponent, RendererSettings};
use crate::images::SDImage;
use crate::modules::components::{ComponentDefinition, map_ui_values, map_ui_values_ref, UIField, UIFieldType, UIFieldValue, UIValue};
use crate::modules::{PluginMetadata, SDModule};
use crate::util::hash_str;
use crate::util::rendering::{resize_for_streamdeck, TextAlignment};
use crate::versions::{CORE, MODULE_MANAGER};

/// The core module, for exposing renderer component to requests and such
pub struct CoreModule;

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

    fn settings(&self, core_manager: Arc<CoreManager>) -> Vec<UIValue> {
        let settings: CoreSettings = core_manager.config.get_plugin_settings().unwrap_or_default();

        let mut fields = vec![];

        fields.push(
            UIValue {
                name: "rendering".to_string(),
                display_name: "Rendering Settings".to_string(),
                ty: UIFieldType::Collapsable,
                value: UIFieldValue::Collapsable({
                    let mut fields = vec![];

                    fields.push(
                        UIValue {
                            name: "plugin_blacklist".to_string(),
                            display_name: "Allowed plugins to render".to_string(),
                            ty: UIFieldType::Collapsable,
                            value: UIFieldValue::Collapsable({
                                core_manager.module_manager.get_modules()
                                    .into_values()
                                    .filter_map(|x| if check_feature_list_for_feature(&x.metadata().used_features, "rendering") {
                                        let name = x.name();
                                        Some(UIValue {
                                            name: name.clone(),
                                            display_name: name.clone(),
                                            ty: UIFieldType::Checkbox { disabled: false },
                                            value: UIFieldValue::Checkbox(!settings.renderer.plugin_blacklist.contains(&name))
                                        })
                                    } else { None })
                                    .collect()
                            })
                        }
                    );

                    fields
                })
            }
        );

        fields
    }

    fn set_setting(&self, core_manager: Arc<CoreManager>, value: Vec<UIValue>) {
        let mut settings: CoreSettings = core_manager.config.get_plugin_settings().unwrap_or_default();

        let change_map = map_ui_values(value);

        if let Some(value) = change_map.get("rendering") {
            if let UIFieldValue::Collapsable(value) = &value.value {
                let change_map = map_ui_values(value.clone());

                if let Some(value) = change_map.get("plugin_blacklist") {
                    if let UIFieldValue::Collapsable(value) = &value.value {
                        let change_map = map_ui_values(value.clone());

                        for (name, value) in change_map {
                            if let UIFieldValue::Checkbox(state) = value.value {
                                if state {
                                    settings.renderer.plugin_blacklist.retain(|x| *x != name);
                                } else {
                                    settings.renderer.plugin_blacklist.push(name);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Calling redraw for all devices
        for device in core_manager.list_added_devices().into_values() {
            device.core.mark_for_redraw();
        }

        core_manager.config.set_plugin_settings(settings);
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

#[derive(Serialize, Deserialize, Default)]
pub struct CoreSettings {
    pub renderer: RendererSettings
}

impl PluginConfig for CoreSettings {
    const NAME: &'static str = "core";
}

