use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::process::Command;
use std::thread::spawn;
use crate::core::button::{Button, Component, parse_button_to_component, parse_unique_button_to_component};
use crate::core::methods::CoreHandle;
use crate::modules::components::{ComponentDefinition, map_ui_values, map_ui_values_ref, UIField, UIFieldType, UIFieldValue, UIValue};
use crate::modules::events::SDEvent;
use crate::modules::{PluginMetadata, SDModule};
use crate::threads::rendering::{ButtonBackground, ButtonText, RendererComponent};
use crate::util::rendering::TextAlignment;
use crate::versions::{CORE, EVENTS};

pub struct ActionsModule;

impl SDModule for ActionsModule {
    fn name(&self) -> String {
        "core/actions".to_string()
    }

    fn components(&self) -> HashMap<String, ComponentDefinition> {
        let mut map = HashMap::new();

        map.insert("run_command".to_string(), ComponentDefinition {
            display_name: "Run Command".to_string(),
            description: "Runs a provided command".to_string(),
            default_looks: RendererComponent {
                background: ButtonBackground::Solid((50, 50, 50, 255)),
                text: vec![
                    ButtonText {
                        text: ">_".to_string(),
                        font: "SourceHanSans-Medium.ttf".to_string(),
                        scale: (40.0, 40.0),
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

        map
    }

    fn add_component(&self, _: CoreHandle, button: &mut Button, name: &str) {
        match name {
            RunCommandComponent::NAME => {
                button.insert_component(RunCommandComponent::default()).ok();
            }

            _ => {}
        }
    }

    fn remove_component(&self, _: CoreHandle, button: &mut Button, name: &str) {
        match name {
            RunCommandComponent::NAME => {
                button.remove_component::<RunCommandComponent>();
            }

            _ => {}
        }
    }

    fn component_values(&self, _: CoreHandle, button: &Button, name: &str) -> Vec<UIValue> {
        match name {
            RunCommandComponent::NAME => {
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

            _ => vec![],
        }
    }

    fn set_component_value(&self, _: CoreHandle, button: &mut Button, name: &str, value: Vec<UIValue>) {
        match name {
            RunCommandComponent::NAME => {
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

            _ => {}
        }
    }

    fn listening_for(&self) -> Vec<String> {
        vec![
            RunCommandComponent::NAME.to_string()
        ]
    }

    fn event(&self, _: CoreHandle, event: SDEvent) {
        match event {
            SDEvent::ButtonAction { pressed_button, .. } => {
                if let Ok(component) = parse_unique_button_to_component::<RunCommandComponent>(&pressed_button) {
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

            _ => {}
        }
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata::from_literals(
            "core/actions",
            "TheJebForge",
            "Provides components for basic actions",
            "0.1",
            &[
                CORE,
                EVENTS
            ]
        )
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
struct RunCommandComponent {
    pub simple_command: String,
    pub advanced_command: Vec<String>,
    pub use_advanced: bool,
}

impl Component for RunCommandComponent {
    const NAME: &'static str = "run_command";
}