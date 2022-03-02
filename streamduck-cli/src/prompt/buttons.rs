use std::str::Split;
use streamduck_client::daemon::daemon_data::{AddComponentResult, ClearButtonResult, GetComponentValuesResult, NewButtonFromComponentResult, NewButtonResult, RemoveComponentResult, SetComponentValueResult};
use streamduck_core::modules::components::{UIFieldType, UIFieldValue, UIValue};
use crate::prompt::ClientRef;

pub fn button_new(client: ClientRef, current_sn: &str, mut args: Split<&str>) {
    if !current_sn.is_empty() {
        if let Some(key) = args.next() {
            if let Ok(key) = key.parse::<u8>() {
                let result = client.new_button(current_sn, key).expect("Failed to create a button");

                match result {
                    NewButtonResult::DeviceNotFound => println!("button new: Device not found"),
                    NewButtonResult::FailedToCreate => println!("button new: Failed to create button"),
                    NewButtonResult::Created => {
                        client.commit_changes(current_sn).expect("Failed to commit changes");
                        println!("button new: Created button");
                    },
                }
            } else {
                println!("button new: Input valid key index (0-255)");
            }
        } else {
            println!("button new: Input valid key index (0-255)");
        }
    } else {
        println!("button new: No device is selected")
    }
}

pub fn button_from(client: ClientRef, current_sn: &str, mut args: Split<&str>) {
    if !current_sn.is_empty() {
        if let Some(key) = args.next() {
            if let Ok(key) = key.parse::<u8>() {
                if let Some(component) = args.next() {
                    let result = client.new_button_from_component(current_sn, key, component).expect("Failed to create a button");

                    match result {
                        NewButtonFromComponentResult::DeviceNotFound => println!("button from: Device not found"),
                        NewButtonFromComponentResult::FailedToCreate => println!("button from: Failed to create button"),
                        NewButtonFromComponentResult::ComponentNotFound => println!("button from: Component not found"),
                        NewButtonFromComponentResult::Created => {
                            client.commit_changes(current_sn).expect("Failed to commit changes");
                            println!("button from: Created button");
                        },
                    }
                } else {
                    println!("button from: Specify component name");
                }
            } else {
                println!("button from: Input valid key index (0-255)");
            }
        } else {
            println!("button from: Input valid key index (0-255)");
        }
    } else {
        println!("button from: No device is selected")
    }
}

pub fn button_remove(client: ClientRef, current_sn: &str, mut args: Split<&str>) {
    if !current_sn.is_empty() {
        if let Some(key) = args.next() {
            if let Ok(key) = key.parse::<u8>() {
                let result = client.clear_button(current_sn, key).expect("Failed to clear a button");

                match result {
                    ClearButtonResult::DeviceNotFound => println!("button remove: Device not found"),
                    ClearButtonResult::FailedToClear => println!("button remove: Failed to remove button"),
                    ClearButtonResult::Cleared => {
                        client.commit_changes(current_sn).expect("Failed to commit changes");
                        println!("button remove: Cleared the button");
                    },
                }

            } else {
                println!("button remove: Input valid key index (0-255)");
            }
        } else {
            println!("button remove: Input valid key index (0-255)");
        }
    } else {
        println!("button remove: No device is selected")
    }
}

pub fn button_component(client: ClientRef, current_sn: &str, mut args: Split<&str>) {
    if !current_sn.is_empty() {
        if let Some(command) = args.next() {
            match command {
                "add" | "a" => button_add_component(client, current_sn, args),
                "remove" | "r" => button_remove_component(client, current_sn, args),
                "params" | "p" => button_component_params(client, current_sn, args),

                _ => println!("button component: Unknown command"),
            }
        } else {
            println!("button component: Unknown command");
        }
    } else {
        println!("button component: No device is selected")
    }
}

pub fn button_add_component(client: ClientRef, current_sn: &str, mut args: Split<&str>) {
    if let Some(key) = args.next() {
        if let Ok(key) = key.parse::<u8>() {
            if let Some(component) = args.next() {
                let result = client.add_component(current_sn, key, component).expect("Failed to add component");

                match result {
                    AddComponentResult::DeviceNotFound => println!("button component add: Device not found"),
                    AddComponentResult::FailedToAdd => println!("button component add: Failed to add"),
                    AddComponentResult::Added => {
                        client.commit_changes(current_sn).expect("Failed to commit changes");
                        println!("button component add: Added");
                    }
                }
            } else {
                println!("button component add: Specify component name");
            }
        } else {
            println!("button component add: Input valid key index (0-255)");
        }
    } else {
        println!("button component add: Input valid key index (0-255)");
    }
}

pub fn button_remove_component(client: ClientRef, current_sn: &str, mut args: Split<&str>) {
    if let Some(key) = args.next() {
        if let Ok(key) = key.parse::<u8>() {
            if let Some(component) = args.next() {
                let result = client.remove_component(current_sn, key, component).expect("Failed to remove component");

                match result {
                    RemoveComponentResult::DeviceNotFound => println!("button component remove: Device not found"),
                    RemoveComponentResult::FailedToRemove => println!("button component remove: Failed to remove"),
                    RemoveComponentResult::Removed => {
                        client.commit_changes(current_sn).expect("Failed to commit changes");
                        println!("button component remove: Removed");
                    }
                }
            } else {
                println!("button component remove: Specify component name");
            }
        } else {
            println!("button component remove: Input valid key index (0-255)");
        }
    } else {
        println!("button component remove: Input valid key index (0-255)");
    }
}

pub fn button_component_params(client: ClientRef, current_sn: &str, mut args: Split<&str>) {
    if let Some(command) = args.next() {
        match command {
            "add" | "a" => button_component_params_add(client, current_sn, args),
            "remove" | "r" => button_component_params_remove(client, current_sn, args),
            "set" | "s" => button_component_params_set(client, current_sn, args),
            "list" | "l" => button_component_list_params(client, current_sn, args),
            "upload" | "u" => button_component_params_upload(client, current_sn, args),
            _ => println!("button component params: Unknown command"),
        }
    } else {
        println!("button component params: Unknown command");
    }
}

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

pub fn button_component_params_add(client: ClientRef, current_sn: &str, mut args: Split<&str>) {
    if let Some(key) = args.next() {
        if let Ok(key) = key.parse::<u8>() {
            if let Some(component) = args.next() {
                if let Some(path) = args.next() {
                    let result = client.get_component_values(current_sn, key, component).expect("Failed to get component values");

                    match result {
                        GetComponentValuesResult::DeviceNotFound => println!("button component params add: Device not found"),
                        GetComponentValuesResult::FailedToGet => println!("button component params add: Failed to get values"),
                        GetComponentValuesResult::Values(values) => {
                            let (changes, success) = change_from_path(path, values, &|x| {
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
                            }, false);

                            if success {
                                if !changes.is_empty() {
                                    let result = client.set_component_values(current_sn, key, component, changes).expect("Failed to set component values");

                                    match result {
                                        SetComponentValueResult::Set => {
                                            client.commit_changes(current_sn).expect("Failed to commit changes");
                                            println!("button component params add: Added new element to the array");
                                        },
                                        _ => {}
                                    }
                                } else {
                                    println!("button component params add: Invalid path");
                                }
                            } else {
                                println!("button component params add: No array at path");
                            }
                        }
                    }
                } else {
                    println!("button component params add: Specify parameter path");
                }
            } else {
                println!("button component params add: Specify component");
            }
        } else {
            println!("button component params add: Input valid key index (0-255)");
        }
    } else {
        println!("button component params add: Input valid key index (0-255)");
    }
}

pub fn button_component_params_remove(client: ClientRef, current_sn: &str, mut args: Split<&str>) {
    if let Some(key) = args.next() {
        if let Ok(key) = key.parse::<u8>() {
            if let Some(component) = args.next() {
                if let Some(path) = args.next() {
                    if let Some(element_index) = args.next() {
                        if let Ok(element_index) = element_index.parse::<usize>() {
                            let result = client.get_component_values(current_sn, key, component).expect("Failed to get component values");

                            match result {
                                GetComponentValuesResult::DeviceNotFound => println!("button component params remove: Device not found"),
                                GetComponentValuesResult::FailedToGet => println!("button component params remove: Failed to get values"),
                                GetComponentValuesResult::Values(values) => {
                                    let (changes, success) = change_from_path(path, values, &|x| {
                                        if let UIFieldValue::Array(array) = &mut x.value {
                                            array.remove(element_index);
                                            true
                                        } else {
                                            false
                                        }
                                    }, false);

                                    if success {
                                        if !changes.is_empty() {
                                            let result = client.set_component_values(current_sn, key, component, changes).expect("Failed to set component values");

                                            match result {
                                                SetComponentValueResult::Set => {
                                                    client.commit_changes(current_sn).expect("Failed to commit changes");
                                                    println!("button component params remove: Removed element from the array");
                                                },
                                                _ => {}
                                            }
                                        } else {
                                            println!("button component params remove: Invalid path");
                                        }
                                    } else {
                                        println!("button component params remove: No array at path");
                                    }
                                }
                            }
                        } else {
                            println!("button component params remove: Input valid array index");
                        }
                    } else {
                        println!("button component params remove: Input valid array index");
                    }
                } else {
                    println!("button component params remove: Specify parameter path");
                }
            } else {
                println!("button component params remove: Specify component");
            }
        } else {
            println!("button component params remove: Input valid key index (0-255)");
        }
    } else {
        println!("button component params remove: Input valid key index (0-255)");
    }
}

pub fn button_component_params_set(client: ClientRef, current_sn: &str, mut args: Split<&str>) {
    if let Some(key) = args.next() {
        if let Ok(key) = key.parse::<u8>() {
            if let Some(component) = args.next() {
                if let Some(path) = args.next() {
                    let result = client.get_component_values(current_sn, key, component).expect("Failed to get component values");

                    match result {
                        GetComponentValuesResult::DeviceNotFound => println!("button component params set: Device not found"),
                        GetComponentValuesResult::FailedToGet => println!("button component params set: Failed to get values"),
                        GetComponentValuesResult::Values(values) => {
                            let value = args.collect::<Vec<&str>>().join(" ");

                            let (changes, success) = change_from_path(path, values, &|x| {
                                match &x.ty {
                                    UIFieldType::Header => false,
                                    UIFieldType::Collapsable(_) => false,
                                    UIFieldType::Array(_) => false,

                                    UIFieldType::Choice(variants) => {
                                        if variants.contains(&value) {
                                            x.value = UIFieldValue::Choice(value.clone());
                                            true
                                        } else {
                                            false
                                        }
                                    }

                                    UIFieldType::InputFieldFloat => {
                                        if let Ok(f) = value.parse::<f32>() {
                                            x.value = UIFieldValue::InputFieldFloat(f);
                                            true
                                        } else {
                                            false
                                        }
                                    }

                                    UIFieldType::InputFieldInteger => {
                                        if let Ok(i) = value.parse::<i32>() {
                                            x.value = UIFieldValue::InputFieldInteger(i);
                                            true
                                        } else {
                                            false
                                        }
                                    }

                                    UIFieldType::InputFieldString => {
                                        x.value = UIFieldValue::InputFieldString(value.clone());
                                        true
                                    }

                                    UIFieldType::InputFieldFloat2 => {
                                        let mut floats = value.split(",");

                                        if let Some(float1) = floats.next() {
                                            if let Ok(float1) = float1.parse::<f32>() {
                                                if let Some(float2) = floats.next() {
                                                    if let Ok(float2) = float2.parse::<f32>() {
                                                        x.value = UIFieldValue::InputFieldFloat2(float1, float2);
                                                        return true;
                                                    }
                                                }
                                            }
                                        }

                                        false
                                    }

                                    UIFieldType::InputFieldInteger2 => {
                                        let mut ints = value.split(",");

                                        if let Some(int1) = ints.next() {
                                            if let Ok(int1) = int1.parse::<i32>() {
                                                if let Some(int2) = ints.next() {
                                                    if let Ok(int2) = int2.parse::<i32>() {
                                                        x.value = UIFieldValue::InputFieldInteger2(int1, int2);
                                                        return true;
                                                    }
                                                }
                                            }
                                        }

                                        false
                                    }

                                    UIFieldType::InputFieldUnsignedInteger => {
                                        if let Ok(u) = value.parse::<u32>() {
                                            x.value = UIFieldValue::InputFieldUnsignedInteger(u);
                                            true
                                        } else {
                                            false
                                        }
                                    }

                                    UIFieldType::ValueSliderFloat(limits) => {
                                        if let Ok(f) = value.parse::<f32>() {
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
                                        if let Ok(i) = value.parse::<i32>() {
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
                                        if let Ok(b) = value.parse::<bool>() {
                                            x.value = UIFieldValue::Checkbox(b);
                                            true
                                        } else {
                                            false
                                        }
                                    }

                                    UIFieldType::Color => {
                                        let mut ints = value.split(",");

                                        if let Some(c1) = ints.next() {
                                            if let Ok(c1) = c1.parse::<u8>() {
                                                if let Some(c2) = ints.next() {
                                                    if let Ok(c2) = c2.parse::<u8>() {
                                                        if let Some(c3) = ints.next() {
                                                            if let Ok(c3) = c3.parse::<u8>() {
                                                                if let Some(c4) = ints.next() {
                                                                    if let Ok(c4) = c4.parse::<u8>() {
                                                                        x.value = UIFieldValue::Color(c1, c2, c3, c4);
                                                                        return true;
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                        false
                                    }

                                    UIFieldType::ImageData => {
                                        x.value = UIFieldValue::ImageData(value.clone());
                                        true
                                    }

                                    UIFieldType::ExistingImage => {
                                        x.value = UIFieldValue::ExistingImage(value.clone());
                                        true
                                    }
                                }
                            }, false);

                            if success {
                                if !changes.is_empty() {
                                    let result = client.set_component_values(current_sn, key, component, changes).expect("Failed to set component values");

                                    match result {
                                        SetComponentValueResult::Set => {
                                            client.commit_changes(current_sn).expect("Failed to commit changes");
                                            println!("button component params set: Parameter set");
                                        },
                                        _ => {}
                                    }
                                } else {
                                    println!("button component params set: Invalid path");
                                }
                            } else {
                                println!("button component params set: No settable parameter found at path");
                            }
                        }
                    }
                } else {
                    println!("button component params set: Specify parameter path");
                }
            } else {
                println!("button component params set: Specify component");
            }
        } else {
            println!("button component params set: Input valid key index (0-255)");
        }
    } else {
        println!("button component params set: Input valid key index (0-255)");
    }
}

pub fn button_component_params_upload(client: ClientRef, current_sn: &str, mut args: Split<&str>) {
    if let Some(key) = args.next() {
        if let Ok(key) = key.parse::<u8>() {
            if let Some(component) = args.next() {
                if let Some(path) = args.next() {
                    let result = client.get_component_values(current_sn, key, component).expect("Failed to get component values");

                    match result {
                        GetComponentValuesResult::DeviceNotFound => println!("button component params upload: Device not found"),
                        GetComponentValuesResult::FailedToGet => println!("button component params upload: Failed to get values"),
                        GetComponentValuesResult::Values(values) => {
                            let file_path = args.collect::<Vec<&str>>().join(" ");

                            if let Ok(data) = std::fs::read(&file_path) {
                                let (changes, success) = change_from_path(path, values, &|x| {
                                    if let UIFieldType::ImageData = &x.ty {
                                        x.value = UIFieldValue::ImageData(base64::encode(&data));
                                        true
                                    } else {
                                        false
                                    }
                                }, false);

                                if success {
                                    if !changes.is_empty() {
                                        let result = client.set_component_values(current_sn, key, component, changes).expect("Failed to set component values");

                                        match result {
                                            SetComponentValueResult::Set => {
                                                client.commit_changes(current_sn).expect("Failed to commit changes");
                                                println!("button component params upload: Uploaded file to parameter");
                                            },
                                            _ => {}
                                        }
                                    } else {
                                        println!("button component params upload: Invalid path");
                                    }
                                } else {
                                    println!("button component params upload: No image data at path");
                                }
                            } else {
                                println!("button component params upload: Failed to read file");
                            }
                        }
                    }
                } else {
                    println!("button component params upload: Specify parameter path");
                }
            } else {
                println!("button component params upload: Specify component");
            }
        } else {
            println!("button component params upload: Input valid key index (0-255)");
        }
    } else {
        println!("button component params upload: Input valid key index (0-255)");
    }
}

pub fn button_component_list_params(client: ClientRef, current_sn: &str, mut args: Split<&str>) {
    if let Some(key) = args.next() {
        if let Ok(key) = key.parse::<u8>() {
            if let Some(component) = args.next() {
                let result = client.get_component_values(current_sn, key, component).expect("Failed to get component values");

                match result {
                    GetComponentValuesResult::DeviceNotFound => println!("button component params list: Device not found"),
                    GetComponentValuesResult::FailedToGet => println!("button component params list: Failed to get values"),
                    GetComponentValuesResult::Values(values) => {
                        fn list_fields(items: Vec<UIValue>, path: &str, tabs_count: usize) {
                            let tabs = format!("{: <w$}", "", w = tabs_count);

                            for item in items {
                                // Name
                                println!("{}{}", tabs, item.display_name);

                                // Path
                                let item_path = format!("{}{}", if path.is_empty() { "".to_string() } else { format!("{}.", path) }, item.name);

                                if let UIFieldValue::Header | UIFieldValue::Collapsable(_) = &item.value {} else {
                                    println!("{}Path: {}", tabs, item_path)
                                }

                                // Value
                                match item.value {
                                    UIFieldValue::Header => {
                                        println!("{}Type: Header", tabs);
                                    }

                                    UIFieldValue::InputFieldFloat(f) | UIFieldValue::ValueSliderFloat(f) => {
                                        println!("{}Type: Float", tabs);
                                        println!("{}Value: {}", tabs, f);
                                    }

                                    UIFieldValue::InputFieldInteger(i) | UIFieldValue::ValueSliderInteger(i) => {
                                        println!("{}Type: Integer", tabs);
                                        println!("{}Value: {}", tabs, i);
                                    }

                                    UIFieldValue::InputFieldString(s) => {
                                        println!("{}Type: String", tabs);
                                        println!("{}Value: {}", tabs, s);
                                    }

                                    UIFieldValue::InputFieldFloat2(f1, f2) => {
                                        println!("{}Type: Float2", tabs);
                                        println!("{}Value: {},{}", tabs, f1, f2);
                                    }

                                    UIFieldValue::InputFieldInteger2(i1, i2) => {
                                        println!("{}Type: Integer2", tabs);
                                        println!("{}Value: {},{}", tabs, i1, i2);
                                    }

                                    UIFieldValue::InputFieldUnsignedInteger(u) => {
                                        println!("{}Type: Positive Integer", tabs);
                                        println!("{}Value: {}", tabs, u);
                                    }

                                    UIFieldValue::Choice(s) => {
                                        println!("{}Type: Choice", tabs);
                                        println!("{}Value: {}", tabs, s);

                                        if let UIFieldType::Choice(variants) = &item.ty {
                                            println!("{}Options: {}", tabs, variants.join(","));
                                        }
                                    }

                                    UIFieldValue::Checkbox(b) => {
                                        println!("{}Type: Boolean", tabs);
                                        println!("{}Value: {}", tabs, b);
                                    }

                                    UIFieldValue::Color(c1, c2, c3, c4) => {
                                        println!("{}Type: Color", tabs);
                                        println!("{}Value: {},{},{},{}", tabs, c1, c2, c3, c4);
                                    }


                                    UIFieldValue::Collapsable(submenu) => {
                                        println!("{}Type: Submenu", tabs);
                                        println!("{}Items:", tabs);

                                        list_fields(submenu, &item_path, tabs_count + 2);
                                    }

                                    UIFieldValue::Array(array) => {
                                        println!("{}Type: Submenu", tabs);
                                        println!("{}Items:", tabs);

                                        for (index, array_item) in array.into_iter().enumerate() {
                                            list_fields(array_item, &format!("{}.{}", item_path, index), tabs_count + 2);
                                            println!();
                                        }
                                    }
                                    UIFieldValue::ImageData(data) => {
                                        println!("{}Type: ImageData", tabs);
                                        println!("{}Data: {}", tabs, data);
                                    }
                                    UIFieldValue::ExistingImage(identifier) => {
                                        println!("{}Type: ExistingImage", tabs);
                                        println!("{}Identifier: {}", tabs, identifier);
                                    }
                                }

                                println!();
                            }
                        }

                        list_fields(values, "", 0);
                    }
                }
            } else {
                println!("button component params list: Specify component name");
            }
        } else {
            println!("button component params list: Input valid key index (0-255)");
        }
    } else {
        println!("button component params list: Input valid key index (0-255)");
    }
}