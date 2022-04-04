use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use serde_json::{Error, Value};
use crate::core::button::Button;
use crate::core::{ButtonPanel, Panel, RawButtonPanel, UniqueButton, UniqueButtonMap};
use crate::font::get_font_names;
use crate::images::SDSerializedImage;
use crate::modules::components::{UIFieldType, UIFieldValue, UIPathValue, UIValue};

pub use rusttype;

/// Wraps button in [Arc] and [RwLock], but packed into a more convenient function
pub fn make_button_unique(button: Button) -> UniqueButton {
    Arc::new(RwLock::new(button))
}

/// Parses button panel to Value, serializing all the unique buttons in process
pub fn serialize_panel(panel: ButtonPanel) -> Result<Value, Error> {
    let panel = panel_to_raw(&panel);

    serialize_panel_raw(panel)
}

/// Serializes raw button panel to Value in more understandable function
pub fn serialize_panel_raw(panel: RawButtonPanel) -> Result<Value, Error> {
    serde_json::to_value(panel)
}

/// Parses value into a button panel
pub fn deserialize_panel(value: Value) -> Result<ButtonPanel, Error> {
    Ok(make_panel_unique(deserialize_panel_raw(value)?))
}

/// Parses value into a raw button panel
pub fn deserialize_panel_raw(value: Value) -> Result<RawButtonPanel, Error> {
    Ok(serde_json::from_value(value)?)
}

/// Converts raw button panel into button panel
pub fn make_panel_unique(raw_panel: RawButtonPanel) -> ButtonPanel {
    Arc::new(RwLock::new(
        Panel::<UniqueButtonMap> {
            display_name: raw_panel.display_name,
            data: raw_panel.data,
            buttons: raw_panel.buttons.into_iter().map(|(key, button)| (key, make_button_unique(button))).collect()
        }
    ))
}


/// Converts button panel to raw button panel
pub fn panel_to_raw(panel: &ButtonPanel) -> RawButtonPanel {
    let handle = panel.read().unwrap();
    let panel = (*handle).clone();
    drop(handle);

    RawButtonPanel {
        display_name: panel.display_name,
        data: panel.data,
        buttons: panel.buttons.into_iter().map(|(key, button)| (key, button_to_raw(&button))).collect()
    }
}

/// Converts unique button to raw button
pub fn button_to_raw(button: &UniqueButton) -> Button {
    button.read().unwrap().deref().clone()
}

/// Hashes string
pub fn hash_str(data: &String) -> String {
    let mut hasher = DefaultHasher::new();

    data.hash(&mut hasher);

    hasher.finish().to_string()
}

/// Hashes image
pub fn hash_image(data: &SDSerializedImage) -> String {
    let mut hasher = DefaultHasher::new();

    data.hash(&mut hasher);

    hasher.finish().to_string()
}

pub fn hash_value<H: Hasher>(value: &Value, state: &mut H) {
    match value {
        Value::Null => 0.hash(state),
        Value::Bool(b) => b.hash(state),
        Value::Number(n) => n.hash(state),
        Value::String(s) => s.hash(state),
        Value::Array(a) => a.iter().for_each(|x| hash_value(x, state)),
        Value::Object(o) => o.iter().for_each(|(k, v)| {
            k.hash(state);
            hash_value(v, state)
        }),
    }
}

/// Converts [UIValue] to [UIPathValue]
pub fn convert_value_to_path(value: UIValue, current_path: &str) -> UIPathValue {
    let current_path = if current_path.is_empty() { "".to_string() } else { format!("{}.", current_path) };

    match &value.value {
        UIFieldValue::Collapsable(m) => {
            let path = format!("{}{}", current_path, value.name);

            let values: Vec<UIPathValue> = m.clone().into_iter()
                .map(|x| convert_value_to_path(x, &path))
                .collect();

            UIPathValue {
                name: value.name,
                path,
                display_name: value.display_name,
                ty: value.ty,
                value: UIFieldValue::Collapsable(values)
            }
        }

        UIFieldValue::Array(a) => {
            let path = format!("{}{}", current_path, value.name);

            let values: Vec<Vec<UIPathValue>> = a.clone().into_iter()
                .enumerate()
                .map(|(index, x)| x.into_iter()
                    .map(|y| convert_value_to_path(y, &format!("{}.{}", path, index)))
                    .collect())
                .collect();

            UIPathValue {
                name: value.name,
                path,
                display_name: value.display_name,
                ty: value.ty,
                value: UIFieldValue::Array(values)
            }
        }

        _ => {
            let path = format!("{}{}", current_path, value.name);

            UIPathValue {
                name: value.name,
                path,
                display_name: value.display_name,
                ty: value.ty,
                value: value.value.into()
            }
        }
    }
}

/// Util function to navigate paths and perform changes on values
pub fn change_from_path<T: Fn(&mut UIValue) -> bool>(path: &str, ui_values: Vec<UIValue>, func: &T, keep: bool) -> (Vec<UIValue>, bool) {
    let mut path = path.split(".");

    let mut changes = vec![];
    let mut success = false;

    if let Some(path_piece) = path.next() {
        for mut value in ui_values {
            if value.name == path_piece {
                match value.value.clone() {
                    UIFieldValue::Collapsable(submenu) => {
                        let path = path.clone().collect::<Vec<&str>>().join(".");

                        let (changed_values, changed_success) = change_from_path(path.as_str(), submenu, func, keep);

                        value.value = UIFieldValue::Collapsable(changed_values);
                        success = changed_success;

                        changes.push(value);
                    }

                    UIFieldValue::Array(array) => {
                        if let Some(path_index) = path.next() {
                            if let Ok(path_index) = path_index.parse::<usize>() {
                                let mut new_array = vec![];

                                for (index, item) in array.into_iter().enumerate() {
                                    if path_index == index {
                                        let path = path.clone().collect::<Vec<&str>>().join(".");

                                        let (changed_values, changed_success) = change_from_path(path.as_str(), item, func, true);
                                        success = changed_success;
                                        new_array.push(changed_values);
                                    } else {
                                        new_array.push(item);
                                    }
                                }

                                value.value = UIFieldValue::Array(new_array);

                                changes.push(value);
                            }
                        } else {
                            success = func(&mut value);

                            changes.push(value);
                        }
                    }

                    _ => {
                        success = func(&mut value);

                        changes.push(value);
                    }
                }
            } else {
                if keep {
                    changes.push(value);
                }
            }
        }
    }

    (changes, success)
}

/// Returns function for adding an element to an array, for use with [change_from_path]
pub fn add_array_function() -> fn(&mut UIValue) -> bool {
    |x| {
        if let UIFieldType::Array(template_fields) = &x.ty {
            let mut new_item = vec![];

            for field in template_fields {
                new_item.push(UIValue {
                    name: field.name.clone(),
                    display_name: field.display_name.clone(),
                    ty: field.ty.clone(),
                    value: field.default_value.clone()
                })
            }

            if let UIFieldValue::Array(array) = &mut x.value {
                array.push(new_item);
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}

/// Returns function for removing an element from array, for use with [change_from_path]
pub fn remove_array_function(index: usize) -> Box<dyn Fn(&mut UIValue) -> bool> {
    Box::new(move |x| {
        if let UIFieldValue::Array(array) = &mut x.value {
            array.remove(index);
            true
        } else {
            false
        }
    })
}

/// Returns function for setting value to UIValue, for use with [change_from_path]
pub fn set_value_function(value: UIPathValue) -> Box<dyn Fn(&mut UIValue) -> bool> {
    let fonts = get_font_names();

    Box::new(move |x| {
        match &x.ty {
            UIFieldType::Header => false,
            UIFieldType::Label => false,
            UIFieldType::Collapsable => false,
            UIFieldType::Array(_) => false,

            UIFieldType::Choice(variants) => {
                if let Ok(variant) = value.value.try_into_string() {
                    if variants.contains(&variant) {
                        x.value = UIFieldValue::Choice(variant);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }

            UIFieldType::InputFieldFloat => {
                if let Ok(f) = value.value.try_into_f32() {
                    x.value = UIFieldValue::InputFieldFloat(f);
                    true
                } else {
                    false
                }
            }

            UIFieldType::InputFieldInteger => {
                if let Ok(i) = value.value.try_into_i32() {
                    x.value = UIFieldValue::InputFieldInteger(i);
                    true
                } else {
                    false
                }
            }

            UIFieldType::InputFieldString => {
                if let Ok(s) = value.value.try_into_string() {
                    x.value = UIFieldValue::InputFieldString(s);
                    true
                } else {
                    false
                }
            }

            UIFieldType::InputFieldFloat2 => {
                if let Ok((f1, f2)) = value.value.try_into_f32_f32() {
                    x.value = UIFieldValue::InputFieldFloat2(f1, f2);
                    true
                } else {
                    false
                }
            }

            UIFieldType::InputFieldInteger2 => {
                if let Ok((i1, i2)) = value.value.try_into_i32_i32() {
                    x.value = UIFieldValue::InputFieldInteger2(i1, i2);
                    true
                } else {
                    false
                }
            }

            UIFieldType::InputFieldUnsignedInteger => {
                if let Ok(u) = value.value.try_into_u32() {
                    x.value = UIFieldValue::InputFieldUnsignedInteger(u);
                    true
                } else {
                    false
                }
            }

            UIFieldType::ValueSliderFloat(limits) => {
                if let Ok(f) = value.value.try_into_f32() {
                    if !limits.allow_out_of_bounds {
                        x.value = UIFieldValue::ValueSliderFloat(f.clamp(limits.min_value, limits.max_value));
                    } else {
                        x.value = UIFieldValue::ValueSliderFloat(f);
                    }
                    true
                } else {
                    false
                }
            }

            UIFieldType::ValueSliderInteger(limits) => {
                if let Ok(i) = value.value.try_into_i32() {
                    if !limits.allow_out_of_bounds {
                        x.value = UIFieldValue::ValueSliderInteger(i.clamp(limits.min_value, limits.max_value));
                    } else {
                        x.value = UIFieldValue::ValueSliderInteger(i);
                    }
                    true
                } else {
                    false
                }
            }

            UIFieldType::Checkbox { .. } => {
                if let Ok(b) = value.value.try_into_bool() {
                    x.value = UIFieldValue::Checkbox(b);
                    true
                } else {
                    false
                }
            }

            UIFieldType::Color => {
                if let Ok(b) = value.value.try_into_color() {
                    x.value = UIFieldValue::Color(b.0, b.1, b.2, b.3);
                    true
                } else {
                    false
                }
            }

            UIFieldType::ImageData => {
                if let Ok(s) = value.value.try_into_string() {
                    x.value = UIFieldValue::ImageData(s);
                    true
                } else {
                    false
                }
            }

            UIFieldType::ExistingImage => {
                if let Ok(s) = value.value.try_into_string() {
                    x.value = UIFieldValue::ExistingImage(s);
                    true
                } else {
                    false
                }
            }
            UIFieldType::Font => {
                if let Ok(s) = value.value.try_into_string() {
                    if fonts.contains(&s) {
                        x.value = UIFieldValue::Font(s);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
        }
    })
}