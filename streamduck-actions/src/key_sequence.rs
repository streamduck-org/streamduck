use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::mpsc::SyncSender;
use enigo::Key;
use streamduck_core::core::button::{Button, Component, parse_button_to_component, parse_unique_button_to_component};
use streamduck_core::core::UniqueButton;
use streamduck_core::modules::components::{ComponentDefinition, map_ui_values, map_ui_values_ref, UIField, UIFieldType, UIFieldValue, UIValue};
use streamduck_core::thread::rendering::{ButtonBackground, ButtonText, RendererComponentBuilder};
use streamduck_core::thread::util::TextAlignment;

pub fn add_definition(map: &mut HashMap<String, ComponentDefinition>) {
    map.insert("key_sequence".to_string(), ComponentDefinition {
        display_name: "Key Sequence".to_string(),
        description: "Performs a sequence of keystrokes defined by component's parameters".to_string(),
        default_looks: RendererComponentBuilder::new()
            .background(ButtonBackground::Solid((50, 50, 50, 255)))
            .add_text(ButtonText {
                text: "Aa".to_string(),
                font: "default".to_string(),
                scale: (30.0, 30.0),
                alignment: TextAlignment::Center,
                padding: 0,
                offset: (0.0, 0.0),
                color: (255, 255, 255, 255),
                shadow: None
            })
            .build()
    });
}

pub fn key_variants() -> Vec<String> {
    let keys = vec![
        "Alt",
        "Backspace",
        "Caps Lock",
        "Control",
        "Delete",
        "Down Arrow",
        "End",
        "Escape",
        "F1",
        "F10",
        "F11",
        "F12",
        "F2",
        "F3",
        "F4",
        "F5",
        "F6",
        "F7",
        "F8",
        "F9",
        "Home",
        "Left Arrow",
        "Meta",
        "Page Down",
        "Page Up",
        "Return",
        "Right Arrow",
        "Shift",
        "Space",
        "Tab",
        "Up Arrow",
        "Char"
    ];

    keys.into_iter()
        .map(|x| x.to_string())
        .collect()
}

pub fn to_key(key_variant: String, other_key: char) -> Option<Key> {
    match key_variant.as_str() {
        "Alt" => Some(Key::Alt),
        "Backspace" => Some(Key::Backspace),
        "Caps Lock" => Some(Key::CapsLock),
        "Control" => Some(Key::Control),
        "Delete" => Some(Key::Delete),
        "Down Arrow" => Some(Key::DownArrow),
        "End" => Some(Key::End),
        "Escape" => Some(Key::End),
        "F1" => Some(Key::F1),
        "F10" => Some(Key::F10),
        "F11" => Some(Key::F11),
        "F12" => Some(Key::F12),
        "F2" => Some(Key::F2),
        "F3" => Some(Key::F3),
        "F4" => Some(Key::F4),
        "F5" => Some(Key::F5),
        "F6" => Some(Key::F6),
        "F7" => Some(Key::F7),
        "F8" => Some(Key::F8),
        "F9" => Some(Key::F9),
        "Home" => Some(Key::Home),
        "Left Arrow" => Some(Key::LeftArrow),
        "Meta" => Some(Key::Meta),
        "Page Down" => Some(Key::PageDown),
        "Page Up" => Some(Key::PageUp),
        "Return" => Some(Key::Return),
        "Right Arrow" => Some(Key::RightArrow),
        "Shift" => Some(Key::Shift),
        "Space" => Some(Key::Space),
        "Tab" => Some(Key::Tab),
        "Up Arrow" => Some(Key::UpArrow),
        "Char" => Some(Key::Layout(other_key)),

        _ => None,
    }
}

pub fn to_key_variant(key: Key) -> (String, Option<char>) {
    match key {
        Key::Alt => ("Alt".to_string(), None),
        Key::Backspace => ("Backspace".to_string(), None),
        Key::CapsLock => ("Caps Lock".to_string(), None),
        Key::Control => ("Control".to_string(), None),
        Key::Delete => ("Delete".to_string(), None),
        Key::DownArrow => ("Down Arrow".to_string(), None),
        Key::End => ("End".to_string(), None),
        Key::Escape => ("Escape".to_string(), None),
        Key::F1 => ("F1".to_string(), None),
        Key::F10 => ("F10".to_string(), None),
        Key::F11 => ("F11".to_string(), None),
        Key::F12 => ("F12".to_string(), None),
        Key::F2 => ("F2".to_string(), None),
        Key::F3 => ("F3".to_string(), None),
        Key::F4 => ("F4".to_string(), None),
        Key::F5 => ("F5".to_string(), None),
        Key::F6 => ("F6".to_string(), None),
        Key::F7 => ("F7".to_string(), None),
        Key::F8 => ("F8".to_string(), None),
        Key::F9 => ("F9".to_string(), None),
        Key::Home => ("Home".to_string(), None),
        Key::LeftArrow => ("Left Arrow".to_string(), None),
        Key::Meta => ("Meta".to_string(), None),
        Key::PageDown => ("Page Down".to_string(), None),
        Key::PageUp => ("Page Up".to_string(), None),
        Key::Return => ("Return".to_string(), None),
        Key::RightArrow => ("Right Arrow".to_string(), None),
        Key::Shift => ("Shift".to_string(), None),
        Key::Space => ("Space".to_string(), None),
        Key::Tab => ("Tab".to_string(), None),
        Key::UpArrow => ("Up Arrow".to_string(), None),
        Key::Layout(c) => ("Char".to_string(), Some(c)),

        _ => ("".to_string(), None)
    }
}

pub fn get_values(button: &Button) -> Vec<UIValue> {
    let mut fields = vec![];

    let action_types = vec!["Click".to_string(), "Press".to_string(), "Release".to_string(), "Delay".to_string(), "Write Text".to_string()];

    if let Ok(component) = parse_button_to_component::<KeySequenceComponent>(button) {
        fields.push(
            UIValue {
                name: "actions".to_string(),
                display_name: "Key Actions".to_string(),
                description: "Sequence of actions to perform on button press".to_string(),
                ty: UIFieldType::Array(
                    vec![
                        UIField {
                            name: "type".to_string(),
                            display_name: "Action Type".to_string(),
                            description: "Type of action to perform".to_string(),
                            ty: UIFieldType::Choice(action_types.clone()),
                            default_value: UIFieldValue::Choice("Click".to_string())
                        },
                        UIField {
                            name: "key".to_string(),
                            display_name: "Key".to_string(),
                            description: "Key to use".to_string(),
                            ty: UIFieldType::Choice(key_variants()),
                            default_value: UIFieldValue::Choice("Alt".to_string())
                        }
                    ]
                ),
                value: UIFieldValue::Array({
                    let mut values = vec![];

                    for key_action in component.key_actions {
                        let mut fields = vec![];

                        fields.push(
                            UIValue {
                                name: "type".to_string(),
                                display_name: "Action Type".to_string(),
                                description: "Type of action to perform".to_string(),
                                ty: UIFieldType::Choice(action_types.clone()),
                                value: UIFieldValue::Choice(
                                    match &key_action {
                                        KeyAction::Press(_) => "Press".to_string(),
                                        KeyAction::Release(_) => "Release".to_string(),
                                        KeyAction::Click(_) => "Click".to_string(),
                                        KeyAction::Delay(_) => "Delay".to_string(),
                                        KeyAction::WriteText(_) => "Write Text".to_string(),

                                    }
                                )
                            }
                        );

                        match key_action {
                            KeyAction::Press(key) | KeyAction::Release(key) | KeyAction::Click(key) => {
                                let (choice, char) = to_key_variant(key);

                                fields.push(
                                    UIValue {
                                        name: "key".to_string(),
                                        display_name: "Key".to_string(),
                                        description: "Key to use".to_string(),
                                        ty: UIFieldType::Choice(key_variants()),
                                        value: UIFieldValue::Choice(choice)
                                    }
                                );

                                if let Some(char) = char {
                                    fields.push(
                                        UIValue {
                                            name: "char".to_string(),
                                            display_name: "Character".to_string(),
                                            description: "Character to use".to_string(),
                                            ty: UIFieldType::InputFieldString,
                                            value: UIFieldValue::InputFieldString(char.to_string())
                                        }
                                    );
                                }
                            }

                            KeyAction::Delay(delay) => {
                                fields.push(
                                    UIValue {
                                        name: "delay".to_string(),
                                        display_name: "Delay".to_string(),
                                        description: "Amount of time to wait".to_string(),
                                        ty: UIFieldType::InputFieldFloat,
                                        value: UIFieldValue::InputFieldFloat(delay)
                                    }
                                );
                            }

                            KeyAction::WriteText(text) => {
                                fields.push(
                                    UIValue {
                                        name: "text".to_string(),
                                        display_name: "Text".to_string(),
                                        description: "Text to write, non-ascii text might lag the libxdo".to_string(),
                                        ty: UIFieldType::InputFieldString,
                                        value: UIFieldValue::InputFieldString(text)
                                    }
                                );
                            }
                        }

                        values.push(fields);
                    }

                    values
                })
            }
        );
    }

    fields
}

pub fn set_values(button: &mut Button, value: Vec<UIValue>) {
    if let Ok(mut component) = parse_button_to_component::<KeySequenceComponent>(button) {
        let change_map = map_ui_values(value);

        if let Some(value) = change_map.get("actions") {
            if let UIFieldValue::Array(items) = &value.value {
                let mut actions = vec![];

                for item in items {
                    let map = map_ui_values_ref(item);

                    if let Some(ty) = map.get("type") {
                        if let Ok(choice) = ty.value.try_into_string() {

                            let figure_out_key = || {
                                if let Some(key) = map.get("key") {
                                    if let Ok(key) = key.value.try_into_string() {
                                        if key == "Char" {
                                            if let Some(char) = map.get("char") {
                                                if let Ok(char) = char.value.try_into_string() {
                                                    if let Some(char) = char.chars().next() {
                                                        if let Some(key) = to_key(key, char) {
                                                            return Some(key);
                                                        }
                                                    }
                                                }
                                            } else {
                                                if let Some(key) = to_key(key, ' ') {
                                                    return Some(key);
                                                }
                                            }
                                        } else {
                                            if let Some(key) = to_key(key, ' ') {
                                                return Some(key);
                                            }
                                        }
                                    }
                                } else {
                                    return Some(Key::Alt)
                                }

                                None
                            };


                            match choice.as_str() {
                                "Press" => {
                                    if let Some(key) = figure_out_key() {
                                        actions.push(KeyAction::Press(key));
                                    }
                                }

                                "Release" => {
                                    if let Some(key) = figure_out_key() {
                                        actions.push(KeyAction::Release(key));
                                    }
                                }

                                "Click" => {
                                    if let Some(key) = figure_out_key() {
                                        actions.push(KeyAction::Click(key));
                                    }
                                }

                                "Delay" => {
                                    if let Some(delay) = map.get("delay") {
                                        if let Ok(delay) = delay.value.try_into_f32() {
                                            actions.push(KeyAction::Delay(delay));
                                        }
                                    } else {
                                        actions.push(KeyAction::Delay(0.0));
                                    }
                                }

                                "Write Text" => {
                                    if let Some(text) = map.get("text") {
                                        if let Ok(text) = text.value.try_into_string() {
                                            actions.push(KeyAction::WriteText(text));
                                        }
                                    } else {
                                        actions.push(KeyAction::WriteText("".to_string()));
                                    }
                                }

                                _ => {}
                            }
                        }
                    }
                }

                component.key_actions = actions;
            }
        }

        button.insert_component(component).ok();
    }
}

pub async fn action(button: &UniqueButton, transmitter: &SyncSender<Vec<KeyAction>>) {
    if let Ok(component) = parse_unique_button_to_component::<KeySequenceComponent>(button).await {
        transmitter.send(component.key_actions).ok();
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct KeySequenceComponent {
    pub key_actions: Vec<KeyAction>
}

#[derive(Serialize, Deserialize, Debug)]
pub enum KeyAction {
    Click(Key),
    Press(Key),
    Release(Key),
    Delay(f32),
    WriteText(String),
}

impl Component for KeySequenceComponent {
    const NAME: &'static str = "key_sequence";
}