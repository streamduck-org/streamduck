use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::process::Command;
use std::thread::spawn;
use streamduck_core::core::button::{Button, Component, parse_button_to_component, parse_unique_button_to_component};
use streamduck_core::core::UniqueButton;
use streamduck_core::modules::components::{ComponentDefinition, map_ui_values, map_ui_values_ref, UIField, UIFieldType, UIFieldValue, UIValue};
use streamduck_core::core::thread::{ButtonBackground, ButtonText, RendererComponent};
use streamduck_core::util::rendering::TextAlignment;

pub fn add_definition(map: &mut HashMap<String, ComponentDefinition>) {
    map.insert("run_command".to_string(), ComponentDefinition {
        display_name: "Run Command".to_string(),
        description: "Runs a provided command".to_string(),
        default_looks: RendererComponent {
            background: ButtonBackground::Solid((50, 50, 50, 255)),
            text: vec![
                ButtonText {
                    text: ">_".to_string(),
                    font: "default".to_string(),
                    scale: (30.0, 30.0),
                    alignment: TextAlignment::Center,
                    padding: 0,
                    offset: (0.0, 0.0),
                    color: (255, 255, 255, 255),
                    shadow: None
                }
            ],
            to_cache: true
        }
    });
}

pub fn get_values(button: &Button) -> Vec<UIValue> {
    let mut fields = vec![];

    if let Ok(component) = parse_button_to_component::<RunCommandComponent>(button){
        fields.push(
            UIValue {
                name: "use_advanced".to_string(),
                display_name: "Advanced mode".to_string(),
                ty: UIFieldType::Checkbox {
                    disabled: false
                },
                value: UIFieldValue::Checkbox(component.use_advanced)
            }
        );

        if component.use_advanced {
            fields.push(
                UIValue {
                    name: "args".to_string(),
                    display_name: "Arguments".to_string(),
                    ty: UIFieldType::Array(vec![
                        UIField {
                            name: "arg".to_string(),
                            display_name: "Argument".to_string(),
                            ty: UIFieldType::InputFieldString,
                            default_value: UIFieldValue::InputFieldString("".to_string())
                        }
                    ]),
                    value: UIFieldValue::Array({
                        let mut values = vec![];

                        for arg in &component.advanced_command {
                            let mut fields = vec![];

                            fields.push(
                                UIValue {
                                    name: "arg".to_string(),
                                    display_name: "Argument".to_string(),
                                    ty: UIFieldType::InputFieldString,
                                    value: UIFieldValue::InputFieldString(arg.to_string())
                                }
                            );

                            values.push(fields);
                        }

                        values
                    })
                }
            )
        } else {
            fields.push(
                UIValue {
                    name: "command".to_string(),
                    display_name: "Command".to_string(),
                    ty: UIFieldType::InputFieldString,
                    value: UIFieldValue::InputFieldString(component.simple_command)
                }
            );
        }
    }

    fields
}

pub fn set_values(button: &mut Button, value: Vec<UIValue>) {
    if let Ok(mut component) = parse_button_to_component::<RunCommandComponent>(button){
        let change_map = map_ui_values(value);

        if let Some(value) = change_map.get("use_advanced") {
            if let Ok(state) = value.value.try_into_bool() {
                component.use_advanced = state;
            }
        }

        if let Some(value) = change_map.get("command") {
            if let Ok(command) = value.value.try_into_string() {
                component.simple_command = command;
            }
        }

        if let Some(value) = change_map.get("args") {
            if let UIFieldValue::Array(args) = &value.value {
                let mut new_args = vec![];

                for arg in args {
                    let map = map_ui_values_ref(arg);

                    if let Some(vl) = map.get("arg") {
                        if let Ok(arg) = vl.value.try_into_string() {
                            new_args.push(arg);
                        }
                    }
                }

                component.advanced_command = new_args;
            }
        }

        button.insert_component(component).ok();
    }
}

pub fn action(button: &UniqueButton) {
    if let Ok(component) = parse_unique_button_to_component::<RunCommandComponent>(button) {
        spawn(move || {
            let execution = {
                if component.use_advanced {
                    let mut iter = component.advanced_command.iter();

                    if let Some(program) = iter.next() {
                        Some(Command::new(program)
                            .args(iter)
                            .output())
                    } else {
                        None
                    }
                } else {
                    let mut command = component.simple_command.split(" ");

                    if let Some(program) = command.next() {
                        Some(Command::new(program)
                            .args(command)
                            .output())
                    } else {
                        None
                    }
                }
            };

            if let Some(execution) = execution {
                match execution {
                    Ok(output) => {
                        log::info!("Execution of command returned: {}", output.status)
                    }

                    Err(err) => {
                        log::warn!("Execution of command failed: {}", err);
                    }
                }
            }
        });
    }
}


#[derive(Serialize, Deserialize, Default, Debug)]
pub struct RunCommandComponent {
    pub simple_command: String,
    pub advanced_command: Vec<String>,
    pub use_advanced: bool,
}

impl Component for RunCommandComponent {
    const NAME: &'static str = "run_command";
}