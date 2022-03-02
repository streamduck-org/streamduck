use std::str::Split;
use streamduck_client::daemon::daemon_data::{GetButtonResult, GetComponentValuesResult, GetCurrentScreenResult};
use streamduck_client::util::module_component_map_to_component_map;
use streamduck_core::core::button::Button;
use streamduck_core::modules::components::{ComponentDefinition, UIFieldValue, UIValue};
use crate::helps::COMMANDS;
use crate::prompt::ClientRef;
use crate::prompt::utils::{print_table, print_table_with_strings};

pub fn prompt_help() -> String {
    let mut help = String::from("Use letters in brackets for shorten variant of the command\n\n");

    for (name, help_line) in COMMANDS {
        help += &format!("{} {}\n", name.replace("-", ""), help_line.replace("[<serial>] ", "").trim());
    }

    help
}

pub fn list_components(client: ClientRef) {
    let mut table = vec![
        vec!["Display Name"],
        vec!["Name"],
        vec!["Module"]
    ];

    let mut module_list: Vec<(String, Vec<(String, ComponentDefinition)>)> = client.list_components().expect("Failed to list components")
        .into_iter().map(|(x, map)| {
        let mut map: Vec<(String, ComponentDefinition)> = map.into_iter().collect();
        map.sort_by(|(a, _), (b, _)| a.cmp(b));
        (x, map)
    }).collect();
    module_list.sort_by(|(a, _), (b, _)| a.cmp(b));
    for (module, component_list) in &module_list {
        for (component, definition) in component_list {
            table[0].push(definition.display_name.as_str());
            table[1].push(component.as_str());
            table[2].push(module.as_str());
        }
    }

    print_table(table, "-", "|");

    println!("\nFor more information on component, enter 'component info <name>'")
}

pub fn component_info(client: ClientRef, mut args: Split<&str>) {
    if let Some(name) = args.next() {
        let component_list = client.list_components().expect("Failed to list components");

        for (module_name, component_list) in component_list {
            for (component_name, definition) in component_list {
                if name == component_name {
                    println!(
                        "- {} ({})\n{}\n- Provided by '{}' module",
                        definition.display_name,
                        component_name,
                        definition.description,
                        module_name
                    );
                    return;
                }
            }
        }

        println!("component info: Component not found");
    } else {
        println!("component info: Specify name");
    }
}

pub fn list_buttons(client: ClientRef, current_sn: &str) {
    if !current_sn.is_empty() {
        let screen = client.get_current_screen(current_sn).expect("Failed to get current screen");

        match screen {
            GetCurrentScreenResult::NoScreen => println!("button list: No screen"),
            GetCurrentScreenResult::DeviceNotFound => println!("button list: Device not found"),
            GetCurrentScreenResult::Screen(screen) => {
                let mut table = vec![
                    vec!["Index".to_string()],
                    vec!["Components".to_string()],
                ];

                let mut screen: Vec<(u8, Button)> = screen.into_iter().collect();

                screen.sort_by(|(a_key, _), (b_key, _)| a_key.cmp(b_key));

                for (key, button) in &screen {
                    table[0].push(key.to_string());

                    let mut names = button.component_names();
                    names.sort();

                    table[1].push(names.join(", ").to_string());
                }

                print_table_with_strings(table, "-", "|");

                println!("\nFor more information on button, enter 'button info <index>'")
            }
        }
    } else {
        println!("button list: No device is selected")
    }
}

pub fn button_info(client: ClientRef, current_sn: &str, mut args: Split<&str>) {
    if !current_sn.is_empty() {
        if let Some(key) = args.next() {
            if let Ok(key) = key.parse::<u8>() {
                let button = client.get_button(current_sn, key).expect("Failed to get current screen");

                match button {
                    GetButtonResult::DeviceNotFound => println!("button info: Device not found"),
                    GetButtonResult::NoButton => println!("button info: Button not found"),
                    GetButtonResult::Button(button) => {
                        println!("Components defined on the button:");

                        let component_list = module_component_map_to_component_map(client.list_components().expect("Failed to get component list"));

                        for name in button.component_names() {
                            if let Some(definition) = component_list.get(&name) {
                                println!("- {} ({})", definition.display_name, name);

                                let values = client.get_component_values(current_sn, key, &name).expect("Failed to get component values");

                                fn print_values(values: &Vec<UIValue>, tabulation: usize) {
                                    for value in values {
                                        if let UIFieldValue::Array(arr) = &value.value {
                                            println!("{s: <w$}{}: {{", value.name, w = tabulation, s = "");

                                            for (index, item) in arr.iter().enumerate() {
                                                println!("{s: <w$}[{}]: {{", index, w = tabulation + 2, s = "");

                                                print_values(item, tabulation + 4);

                                                println!("{s: <w$}}}", w = tabulation + 2, s = "");
                                            }

                                            println!("{s: <w$}}}", w = tabulation, s = "")
                                        } else {
                                            println!("{s: <w$}{}: {:?}", value.name, value.value, w = tabulation, s = "");
                                        }
                                    }
                                }

                                if let GetComponentValuesResult::Values(values) = values {
                                    print_values(&values, 0);
                                }

                                println!()
                            }
                        }
                    }
                }
            } else {
                println!("button info: Input valid key index (0-255)");
            }
        } else {
            println!("button info: Input valid key index (0-255)");
        }
    } else {
        println!("button info: No device is selected")
    }
}