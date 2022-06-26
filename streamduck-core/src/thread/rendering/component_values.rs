use strum::VariantNames;
use std::str::FromStr;
use crate::core::button::{Button, parse_button_to_component};
use crate::core::CoreHandle;
use crate::modules::components::{map_ui_values, map_ui_values_ref, UIField, UIFieldType, UIFieldValue, UIValue};
use crate::thread::rendering::{ButtonBackground, ButtonText, ButtonTextShadow, RendererComponent};
use crate::thread::util::TextAlignment;
use crate::images::SDImage;
use crate::util::hash_str;

pub async fn get_renderer_component_values(core: &CoreHandle, button: &Button) -> Vec<UIValue> {
    if let Ok(component) = parse_button_to_component::<RendererComponent>(button) {
        let mut fields = vec![];

        fields.push(
            UIValue {
                name: "renderer".to_string(),
                display_name: "Renderer to use".to_string(),
                description: "Renderers can be added by plugins".to_string(),
                ty: UIFieldType::Choice({
                    let mut names = vec!["default".to_string()];

                    names.extend(core.core.render_manager.read_renderers().await.values()
                        .map(|x| x.name()));

                    names
                }),
                value: UIFieldValue::Choice(if component.renderer.is_empty() { "default".to_string() } else { component.renderer.clone() })
            }
        );

        if !component.renderer.is_empty() {
            if let Some(renderer) = core.core.render_manager.read_renderers().await.get(&component.renderer).cloned() {
                fields.extend(renderer.component_values(button, &component, core).await);
            }
        } else {
            // Choice for background type
            fields.push(
                UIValue {
                    name: "background_params".to_string(),
                    display_name: "Background Parameters".to_string(),
                    description: "Parameters related to background of the button".to_string(),
                    ty: UIFieldType::Collapsable,
                    value: UIFieldValue::Collapsable({
                        let mut fields = vec![];

                        fields.push(
                            UIValue {
                                name: "background".to_string(),
                                display_name: "Background Type".to_string(),
                                description: "Type of the background to use".to_string(),
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
                                        description: "Color that will be the background of the button".to_string(),
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
                                        description: "Color that will be on left side of the gradient".to_string(),
                                        ty: UIFieldType::Color,
                                        value: start_color.into()
                                    }
                                );

                                fields.push(
                                    UIValue {
                                        name: "end_color".to_string(),
                                        display_name: "Gradient End Color".to_string(),
                                        description: "Color that will be on right side of the gradient".to_string(),
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
                                        description: "Color that will be on top side of the gradient".to_string(),
                                        ty: UIFieldType::Color,
                                        value: start_color.into()
                                    }
                                );

                                fields.push(
                                    UIValue {
                                        name: "end_color".to_string(),
                                        display_name: "Gradient End Color".to_string(),
                                        description: "Color that will be on bottom side of the gradient".to_string(),
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
                                        description: "Image to use as background of the button".to_string(),
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
                                        description: "Image to use as background of the button".to_string(),
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
                    description: "Parameters related to text on the button".to_string(),
                    ty: UIFieldType::Collapsable,
                    value: UIFieldValue::Collapsable({
                        let mut fields = vec![];

                        fields.push(
                            UIValue {
                                name: "text".to_string(),
                                display_name: "Text Objects".to_string(),
                                description: "Array of text objects".to_string(),
                                ty: UIFieldType::Array(
                                    vec![
                                        UIField {
                                            name: "text".to_string(),
                                            display_name: "Text".to_string(),
                                            description: "Text that will be displayed".to_string(),
                                            ty: UIFieldType::InputFieldString,
                                            default_value: UIFieldValue::InputFieldString("".to_string())
                                        },
                                        UIField {
                                            name: "font".to_string(),
                                            display_name: "Font".to_string(),
                                            description: "Font that will be used for text rendering".to_string(),
                                            ty: UIFieldType::Font,
                                            default_value: UIFieldValue::Font("default".to_string())
                                        },
                                        UIField {
                                            name: "scale".to_string(),
                                            display_name: "Text Scale".to_string(),
                                            description: "Scale of the text".to_string(),
                                            ty: UIFieldType::InputFieldFloat2,
                                            default_value: UIFieldValue::InputFieldFloat2(15.0, 15.0)
                                        },
                                        UIField {
                                            name: "alignment".to_string(),
                                            display_name: "Alignment".to_string(),
                                            description: "To which point of the button the text will be anchored to".to_string(),
                                            ty: UIFieldType::Choice(
                                                TextAlignment::VARIANTS.iter().map(|x| x.to_string()).collect()
                                            ),
                                            default_value: UIFieldValue::Choice("Center".to_string())
                                        },
                                        UIField {
                                            name: "padding".to_string(),
                                            display_name: "Padding".to_string(),
                                            description: "Gap to have from alignment/anchor point".to_string(),
                                            ty: UIFieldType::InputFieldUnsignedInteger,
                                            default_value: UIFieldValue::InputFieldUnsignedInteger(0)
                                        },
                                        UIField {
                                            name: "offset".to_string(),
                                            display_name: "Text Offset".to_string(),
                                            description: "2D offset of the text from its alignment/anchor point".to_string(),
                                            ty: UIFieldType::InputFieldFloat2,
                                            default_value: UIFieldValue::InputFieldFloat2(0.0, 0.0)
                                        },
                                        UIField {
                                            name: "color".to_string(),
                                            display_name: "Text Color".to_string(),
                                            description: "Color that text will be displayed in".to_string(),
                                            ty: UIFieldType::Color,
                                            default_value: UIFieldValue::Color(0, 0, 0, 255)
                                        },
                                        UIField {
                                            name: "shadow_enabled".to_string(),
                                            display_name: "Text Shadow".to_string(),
                                            description: "If text shadow should be rendered or not".to_string(),
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
                                            description: "Text that will be displayed".to_string(),
                                            ty: UIFieldType::InputFieldString,
                                            value: UIFieldValue::InputFieldString(text.text.clone())
                                        });

                                        values.push(UIValue {
                                            name: "font".to_string(),
                                            display_name: "Font".to_string(),
                                            description: "Font that will be used for text rendering".to_string(),
                                            ty: UIFieldType::Font,
                                            value: UIFieldValue::Font(text.font.clone())
                                        });

                                        values.push(UIValue {
                                            name: "scale".to_string(),
                                            display_name: "Text Scale".to_string(),
                                            description: "Scale of the text".to_string(),
                                            ty: UIFieldType::InputFieldFloat2,
                                            value: UIFieldValue::InputFieldFloat2(text.scale.0, text.scale.1)
                                        });

                                        values.push(UIValue {
                                            name: "alignment".to_string(),
                                            display_name: "Alignment".to_string(),
                                            description: "To which point of the button the text will be anchored to".to_string(),
                                            ty: UIFieldType::Choice(
                                                TextAlignment::VARIANTS.iter().map(|x| x.to_string()).collect()
                                            ),
                                            value: UIFieldValue::Choice(text.alignment.to_string())
                                        });

                                        values.push(UIValue {
                                            name: "padding".to_string(),
                                            display_name: "Padding".to_string(),
                                            description: "Gap to have from alignment/anchor point".to_string(),
                                            ty: UIFieldType::InputFieldUnsignedInteger,
                                            value: UIFieldValue::InputFieldUnsignedInteger(text.padding)
                                        });

                                        values.push(UIValue {
                                            name: "offset".to_string(),
                                            display_name: "Text Offset".to_string(),
                                            description: "2D offset of the text from its alignment/anchor point".to_string(),
                                            ty: UIFieldType::InputFieldFloat2,
                                            value: UIFieldValue::InputFieldFloat2(text.offset.0, text.offset.1)
                                        });

                                        values.push(UIValue {
                                            name: "color".to_string(),
                                            display_name: "Text Color".to_string(),
                                            description: "Color that text will be displayed in".to_string(),
                                            ty: UIFieldType::Color,
                                            value: text.color.into()
                                        });

                                        if let Some(shadow) = &text.shadow {
                                            values.push(
                                                UIValue {
                                                    name: "shadow_enabled".to_string(),
                                                    display_name: "Text Shadow".to_string(),
                                                    description: "If text shadow should be rendered or not".to_string(),
                                                    ty: UIFieldType::Checkbox {
                                                        disabled: false
                                                    },
                                                    value: UIFieldValue::Checkbox(true)
                                                }
                                            );

                                            values.push(UIValue {
                                                name: "shadow_color".to_string(),
                                                display_name: "Text Shadow Color".to_string(),
                                                description: "Color of the shadow".to_string(),
                                                ty: UIFieldType::Color,
                                                value: shadow.color.into()
                                            });

                                            values.push(UIValue {
                                                name: "shadow_offset".to_string(),
                                                display_name: "Text Shadow Offset".to_string(),
                                                description: "Offset of the shadow from text".to_string(),
                                                ty: UIFieldType::InputFieldInteger2,
                                                value: UIFieldValue::InputFieldInteger2(shadow.offset.0, shadow.offset.1)
                                            });
                                        } else {
                                            values.push(
                                                UIValue {
                                                    name: "shadow_enabled".to_string(),
                                                    display_name: "Text Shadow".to_string(),
                                                    description: "If text shadow should be rendered or not".to_string(),
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

            // Ignore plugin thread menu
            fields.push(
                UIValue {
                    name: "plugin_blacklist".to_string(),
                    display_name: "Allowed plugins to render".to_string(),
                    description: "Disabled plugins will not appear on button".to_string(),
                    ty: UIFieldType::Collapsable,
                    value: UIFieldValue::Collapsable({
                        let names = core.module_manager().get_modules_for_rendering(&button.component_names()).await;

                        names.into_values()
                            .map(|x| {
                                let name = x.name();

                                UIValue {
                                    name: name.clone(),
                                    display_name: name.clone(),
                                    description: "".to_string(),
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
                    description: "If renderer should cache render result or not. Caching might use a lot of RAM, no caching will use a lot more CPU".to_string(),
                    ty: UIFieldType::Checkbox {
                        disabled: false
                    },
                    value: UIFieldValue::Checkbox(component.to_cache)
                }
            );
        }

        fields
    } else {
        vec![]
    }
}

pub async fn set_renderer_component_values(core: &CoreHandle, button: &mut Button, value: Vec<UIValue>) {
    if let Ok(mut component) = parse_button_to_component::<RendererComponent>(button) {
        let change_map = map_ui_values(value);

        if let Some(value) = change_map.get("renderer") {
            if let Ok(value) = value.value.try_into_string() {
                if value == "default" {
                    component.renderer = "".to_string();
                } else {
                    if let Some(_) = core.core.render_manager.read_renderers().await.get(&value) {
                        component.renderer = value;
                    }
                }
            }
        }

        if !component.renderer.is_empty() {
            if let Some(renderer) = core.core.render_manager.read_renderers().await.get(&component.renderer).cloned() {
                renderer.set_component_value(button, &mut component, core, change_map.values().cloned().collect()).await;
            }
        } else {
            if let Some(value) = change_map.get("background_params") {
                if let UIFieldValue::Collapsable(value) = &value.value {
                    let change_map = map_ui_values(value.clone());

                    // Setting background type
                    if let Some(value) = change_map.get("background") {
                        if let Ok(choice) = value.value.try_into_string() {
                            match choice.as_str() {
                                "Solid Color" => component.background = ButtonBackground::Solid((0, 0, 0, 255)),
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
                                    let identifier = hash_str(&blob);

                                    if let Ok(image) = SDImage::from_base64(&blob, core.core.image_size).await {
                                        component.background = ButtonBackground::ExistingImage(identifier.clone());

                                        let mut handle = core.core.image_collection.write().await;
                                        handle.insert(identifier, image);
                                    } else {
                                        component.background = ButtonBackground::NewImage("".to_string());
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
        }

        // Apply changes to button
        button.insert_component(component).ok();

        core.core.mark_for_redraw().await;
    }
}