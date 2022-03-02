use std::str::Split;
use streamduck_client::daemon::daemon_data::{GetModuleValuesResult, SetModuleValueResult};
use streamduck_core::modules::components::{UIFieldType, UIFieldValue, UIValue};
use crate::prompt::buttons::change_from_path;
use crate::prompt::ClientRef;
use crate::prompt::utils::{print_table};

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
            let result = client.get_module_values(module_name).expect("Failed to get module values");

            match result {
                GetModuleValuesResult::ModuleNotFound => println!("module params add: Module not found"),
                GetModuleValuesResult::Values(values) => {
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
                            let result = client.set_module_value(module_name, changes).expect("Failed to set module values");

                            match result {
                                SetModuleValueResult::Set => {
                                    println!("module params add: Added new element to the array");
                                },
                                _ => {}
                            }
                        } else {
                            println!("module params add: Invalid path");
                        }
                    } else {
                        println!("module params add: No array at path");
                    }
                }
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
                    let result = client.get_module_values(module_name).expect("Failed to get module values");

                    match result {
                        GetModuleValuesResult::ModuleNotFound => println!("module params remove: Module not found"),
                        GetModuleValuesResult::Values(values) => {
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
                                    let result = client.set_module_value(module_name, changes).expect("Failed to set module values");

                                    match result {
                                        SetModuleValueResult::Set => {
                                            println!("module params remove: Removed element from the array");
                                        },
                                        _ => {}
                                    }
                                } else {
                                    println!("module params remove: Invalid path");
                                }
                            } else {
                                println!("module params remove: No array at path");
                            }
                        }
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
                            let result = client.set_module_value(module_name, changes).expect("Failed to set module values");

                            match result {
                                SetModuleValueResult::Set => {
                                    println!("module params set: Parameter set");
                                },
                                _ => {}
                            }
                        } else {
                            println!("module params set: Invalid path");
                        }
                    } else {
                        println!("module params set: No settable parameter found at path");
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

pub fn module_list_params(client: ClientRef, mut args: Split<&str>) {
    if let Some(module_name) = args.next() {
        let result = client.get_module_values(module_name).expect("Failed to get module values");

        match result {
            GetModuleValuesResult::ModuleNotFound => println!("module params list: Module not found"),
            GetModuleValuesResult::Values(values) => {
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
        println!("module params list: Specify module name");
    }

}