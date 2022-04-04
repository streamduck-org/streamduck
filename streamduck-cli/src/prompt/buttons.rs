use std::str::Split;
use streamduck_client::daemon::daemon_data::buttons::{AddComponentResult, AddComponentValueResult, ClearButtonResult, GetComponentValuesResult, NewButtonFromComponentResult, NewButtonResult, RemoveComponentResult, RemoveComponentValueResult, SetComponentValueResult};
use streamduck_core::modules::components::{map_ui_path_values, UIFieldType, UIFieldValue, UIPathValue};
use crate::prompt::ClientRef;
use crate::prompt::utils::parse_string_to_value;

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

pub fn button_component_params_add(client: ClientRef, current_sn: &str, mut args: Split<&str>) {
    if let Some(key) = args.next() {
        if let Ok(key) = key.parse::<u8>() {
            if let Some(component) = args.next() {
                if let Some(path) = args.next() {
                    let result = client.add_component_value(current_sn, key, component, path).expect("Failed to add element to component value");

                    match result {
                        AddComponentValueResult::DeviceNotFound => println!("button component params add: Device not found"),
                        AddComponentValueResult::FailedToAdd => println!("button component params add: No array at path"),
                        AddComponentValueResult::Added => {
                            client.commit_changes(current_sn).expect("Failed to commit changes");
                            println!("button component params add: Added new element to the array")
                        },
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
                            let result = client.remove_component_value(current_sn, key, component, path, element_index).expect("Failed to remove element from component value");

                            match result {
                                RemoveComponentValueResult::DeviceNotFound => println!("button component params remove: Device not found"),
                                RemoveComponentValueResult::FailedToRemove => println!("button component params remove: No array at path"),
                                RemoveComponentValueResult::Removed => {
                                    client.commit_changes(current_sn).expect("Failed to commit changes");
                                    println!("button component params remove: Removed element from the array")
                                },
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
                            let values_map = map_ui_path_values(&values);

                            if let Some(mut value) = values_map.get(path).cloned() {
                                let inputted_value = args.collect::<Vec<&str>>().join(" ");

                                let field_value = parse_string_to_value(&inputted_value, &value.ty);

                                if let Some(field_value) = field_value {
                                    value.value = field_value;
                                    let result = client.set_component_value(current_sn, key, component, value).expect("Failed to set component value");

                                    match result {
                                        SetComponentValueResult::DeviceNotFound => println!("button component params set: Device not found"),
                                        SetComponentValueResult::FailedToSet => println!("button component params set: Failed to set value"),
                                        SetComponentValueResult::Set => {
                                            client.commit_changes(current_sn).expect("Failed to commit changes");
                                            println!("button component params set: Parameter set")
                                        },
                                    }
                                } else {
                                    println!("button components params set: Invalid value")
                                }
                            } else {
                                println!("button components params set: Invalid path");
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
                            let values_map = map_ui_path_values(&values);

                            if let Some(mut value) = values_map.get(path).cloned() {
                                if let UIFieldType::ImageData = &value.ty {
                                    let file_path = args.collect::<Vec<&str>>().join(" ");

                                    if let Ok(data) = std::fs::read(&file_path) {
                                        value.value = UIFieldValue::ImageData(base64::encode(&data));

                                        let result = client.set_component_value(current_sn, key, component, value).expect("Failed to set component value");

                                        match result {
                                            SetComponentValueResult::DeviceNotFound => println!("button component params upload: Device not found"),
                                            SetComponentValueResult::FailedToSet => println!("button component params upload: Failed to upload image"),
                                            SetComponentValueResult::Set => {
                                                client.commit_changes(current_sn).expect("Failed to commit changes");
                                                println!("button component params upload: Uploaded image")
                                            },
                                        }
                                    } else {
                                        println!("button component params upload: Failed to read file");
                                    }
                                } else {
                                    println!("button components params upload: No image data component value at path");
                                }
                            } else {
                                println!("button components params upload: Invalid path");
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
                        fn list_fields(items: Vec<UIPathValue>, tabs_count: usize) {
                            let tabs = format!("{: <w$}", "", w = tabs_count);

                            for item in items {
                                // Name
                                println!("{}{}", tabs, item.display_name);

                                println!("{}Description: {}", tabs, item.description);

                                if let UIFieldValue::Header | UIFieldValue::Collapsable(_) = &item.value {} else {
                                    println!("{}Path: {}", tabs, item.path)
                                }

                                // Value
                                match item.value {
                                    UIFieldValue::Header => {
                                        println!("{}Type: Header", tabs);
                                    }

                                    UIFieldValue::Label(str) => {
                                        println!("{}Type: Label", tabs);
                                        println!("{}Value: {}", tabs, str);
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

                                        list_fields(submenu, tabs_count + 2);
                                    }

                                    UIFieldValue::Array(array) => {
                                        println!("{}Type: Array", tabs);
                                        println!("{}Items:", tabs);

                                        for array_item in array.into_iter() {
                                            list_fields(array_item, tabs_count + 2);
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
                                    UIFieldValue::Font(font) => {
                                        println!("{}Type: Font", tabs);
                                        println!("{}Font Name: {}", tabs, font);
                                    }
                                }

                                println!();
                            }
                        }

                        list_fields(values, 0);
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