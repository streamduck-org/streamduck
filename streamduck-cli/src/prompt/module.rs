use std::str::Split;
use streamduck_client::daemon::daemon_data::modules::{AddModuleValueResult, GetModuleValuesResult, RemoveModuleValueResult, SetModuleValueResult};
use streamduck_core::modules::components::{map_ui_path_values, UIFieldType, UIFieldValue, UIPathValue};
use crate::prompt::ClientRef;
use crate::prompt::images::show_image;
use crate::prompt::utils::{parse_string_to_value, print_table};

pub fn list_modules(client: ClientRef) {
    let mut table = vec![
        vec!["Name"],
        vec!["Version"],
        vec!["Description"]
    ];

    let module_list = client.list_modules().expect("Failed to list modules");

    for module in &module_list {
        table[0].push(module.name.as_str());
        table[1].push(module.version.as_str());
        table[2].push(module.description.as_str());
    }

    print_table(table, "-", "|");

    println!("\nFor more information on module, enter 'module info <name>'")
}

pub fn module_info(client: ClientRef, mut args: Split<&str>) {
    if let Some(name) = args.next() {
        let module_list = client.list_modules().expect("Failed to list modules");

        for module in module_list {
            if name == module.name {
                println!(
                    "- {} v{} by {}\n{}\n- Using features: {}",
                    module.name,
                    module.version,
                    module.author,
                    module.description,
                    {
                        let mut names = vec![];

                        for (name, _) in &module.used_features {
                            names.push(name.as_str())
                        }

                        names.join(", ")
                    }
                );
                return;
            }
        }

        println!("module info: Module not found");
    } else {
        println!("module info: Specify name");
    }
}

pub fn module_params_add(client: ClientRef, mut args: Split<&str>) {
    if let Some(module_name) = args.next() {
        if let Some(path) = args.next() {
            let result = client.add_module_value(module_name, path).expect("Failed to add element to module setting");

            match result {
                AddModuleValueResult::ModuleNotFound => println!("module params add: Module not found"),
                AddModuleValueResult::FailedToAdd => println!("module params add: No array at path"),
                AddModuleValueResult::Added => println!("module params add: Added new element to the array"),
            }
        } else {
            println!("module params add: Specify parameter path");
        }
    } else {
        println!("module params add: Specify component");
    }
}

pub fn module_params_remove(client: ClientRef, mut args: Split<&str>) {
    if let Some(module_name) = args.next() {
        if let Some(path) = args.next() {
            if let Some(element_index) = args.next() {
                if let Ok(element_index) = element_index.parse::<usize>() {
                    let result = client.remove_module_value(module_name, path, element_index).expect("Failed to remove element from module setting");

                    match result {
                        RemoveModuleValueResult::ModuleNotFound => println!("module params remove: Module not found"),
                        RemoveModuleValueResult::FailedToRemove => println!("module params add: No array at path"),
                        RemoveModuleValueResult::Removed => println!("module params remove: Removed element from the array"),
                    }
                } else {
                    println!("module params remove: Input valid array index");
                }
            } else {
                println!("module params remove: Input valid array index");
            }
        } else {
            println!("module params remove: Specify parameter path");
        }
    } else {
        println!("module params remove: Specify component");
    }
}

pub fn module_params_set(client: ClientRef, mut args: Split<&str>) {
    if let Some(module_name) = args.next() {
        if let Some(path) = args.next() {
            let result = client.get_module_values(module_name).expect("Failed to get module values");

            match result {
                GetModuleValuesResult::ModuleNotFound => println!("module params set: Module not found"),
                GetModuleValuesResult::Values(values) => {
                    let values_map = map_ui_path_values(&values);

                    if let Some(mut value) = values_map.get(path).cloned() {
                        let inputted_value = args.collect::<Vec<&str>>().join(" ");

                        let field_value = parse_string_to_value(&inputted_value, &value.ty);

                        if let Some(field_value) = field_value {
                            value.value = field_value;
                            let result = client.set_module_value(module_name, value).expect("Failed to set module value");

                            match result {
                                SetModuleValueResult::FailedToSet => println!("module params set: Failed to set value"),
                                SetModuleValueResult::ModuleNotFound => println!("module params set: Module not found"),
                                SetModuleValueResult::Set => println!("module params set: Parameter set"),
                            }
                        } else {
                            println!("module params set: Invalid value")
                        }
                    } else {
                        println!("module params set: Invalid path");
                    }
                }
            }
        } else {
            println!("module params set: Specify parameter path");
        }
    } else {
        println!("module params set: Specify module name");
    }

}

pub fn module_params_upload(client: ClientRef, mut args: Split<&str>) {
    if let Some(module_name) = args.next() {
        if let Some(path) = args.next() {
            let result = client.get_module_values(module_name).expect("Failed to get module values");

            match result {
                GetModuleValuesResult::ModuleNotFound => println!("module params upload: Module not found"),
                GetModuleValuesResult::Values(values) => {
                    let values_map = map_ui_path_values(&values);

                    if let Some(mut value) = values_map.get(path).cloned() {
                        if let UIFieldType::ImageData = &value.ty {
                            let file_path = args.collect::<Vec<&str>>().join(" ");

                            if let Ok(data) = std::fs::read(&file_path) {
                                value.value = UIFieldValue::ImageData(base64::encode(&data));
                                let result = client.set_module_value(module_name, value).expect("Failed to set module value");

                                match result {
                                    SetModuleValueResult::FailedToSet => println!("module params upload: Failed to set value"),
                                    SetModuleValueResult::ModuleNotFound => println!("module params upload: Module not found"),
                                    SetModuleValueResult::Set => println!("module params upload: Uploaded image"),
                                }
                            } else {
                                println!("module params upload: Failed to read file");
                            }
                        } else {
                            println!("module params upload: Invalid value")
                        }
                    } else {
                        println!("module params upload: Invalid path");
                    }
                }
            }
        } else {
            println!("module params upload: Specify parameter path");
        }
    } else {
        println!("module params upload: Specify module name");
    }

}

pub fn module_list_params(client: ClientRef, mut args: Split<&str>) {
    if let Some(module_name) = args.next() {
        let result = client.get_module_values(module_name).expect("Failed to get module values");

        match result {
            GetModuleValuesResult::ModuleNotFound => println!("module params list: Module not found"),
            GetModuleValuesResult::Values(values) => {
                fn list_fields(items: Vec<UIPathValue>, path: &str, tabs_count: usize) {
                    let tabs = format!("{: <w$}", "", w = tabs_count);

                    for item in items {
                        // Name
                        println!("{}{}", tabs, item.display_name);
                        println!("{}Description: {}", tabs, item.description);

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

                                list_fields(submenu, &item_path, tabs_count + 2);
                            }

                            UIFieldValue::Array(array) => {
                                println!("{}Type: Array", tabs);
                                println!("{}Items:", tabs);

                                for (index, array_item) in array.into_iter().enumerate() {
                                    list_fields(array_item, &format!("{}.{}", item_path, index), tabs_count + 2);
                                    println!();
                                }
                            }

                            UIFieldValue::ImageData(img) => {
                                println!("{}Type: ImageData", tabs);
                                show_image(img, 40);
                            }

                            UIFieldValue::ExistingImage(identifier) => {
                                println!("{}Type: ExistingImage", tabs);
                                println!("{}Identifier: {}", tabs, identifier);
                            }

                            UIFieldValue::Font(font) => {
                                println!("{}Type: Font", tabs);
                                println!("{}Font Name: {}", tabs, font);
                            }

                            UIFieldValue::Button => {
                                println!("{}Type: Button", tabs);
                            }

                            UIFieldValue::ImagePreview(img) => {
                                println!("{}Type: ImagePreview", tabs);
                                show_image(img, 40);
                            }
                        }

                        println!();
                    }
                }

                list_fields(values, "", 0);
            }
        }
    } else {
        println!("module params list: Specify module name");
    }

}