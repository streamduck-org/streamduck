mod run_command;
mod key_sequence;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::{sync_channel, SyncSender};
use std::thread::{sleep, spawn};
use std::time::Duration;
use enigo::{Enigo, KeyboardControllable};
use streamduck_core::core::button::{Button, Component};
use streamduck_core::core::methods::CoreHandle;
use streamduck_core::modules::components::{ComponentDefinition, UIValue};
use streamduck_core::modules::events::SDEvent;
use streamduck_core::modules::{ModuleManager, PluginMetadata, SDModule};
use streamduck_core::util::straight_copy;
use streamduck_core::versions::{CORE, EVENTS};
use crate::key_sequence::{KeyAction, KeySequenceComponent};
use crate::run_command::RunCommandComponent;

pub fn init_module(module_manager: &Arc<ModuleManager>) {
    module_manager.add_module(Arc::new(Box::new(ActionsModule::new() )));
}


pub struct ActionsModule {
    pub key_transmitter: SyncSender<Vec<KeyAction>>,
}

impl ActionsModule {
    pub fn new() -> ActionsModule {
        let (tx, rx) = sync_channel::<Vec<KeyAction>>(0);

        spawn(move || {
            let mut enigo = Enigo::new();

            while let Ok(v) = rx.recv() {
                for action in v {
                    match action {
                        KeyAction::Press(key) => {
                            enigo.key_down(key);
                        }

                        KeyAction::Release(key) => {
                            enigo.key_up(key);
                        }

                        KeyAction::Delay(duration) => {
                            sleep(Duration::from_secs_f32(duration));
                        }

                        KeyAction::WriteText(text) => {
                            enigo.key_sequence(&text);
                        }

                        KeyAction::Click(key) => {
                            enigo.key_click(key);
                        }
                    }
                }
            }
        });

        ActionsModule {
            key_transmitter: tx
        }
    }
}

impl SDModule for ActionsModule {
    fn name(&self) -> String {
        "core/actions".to_string()
    }

    fn components(&self) -> HashMap<String, ComponentDefinition> {
        let mut map = HashMap::new();

        run_command::add_definition(&mut map);
        key_sequence::add_definition(&mut map);

        map
    }

    fn add_component(&self, _: CoreHandle, button: &mut Button, name: &str) {
        match name {
            RunCommandComponent::NAME => {
                button.insert_component(RunCommandComponent::default()).ok();
            }

            KeySequenceComponent::NAME => {
                button.insert_component(KeySequenceComponent::default()).ok();
            }

            _ => {}
        }
    }

    fn remove_component(&self, _: CoreHandle, button: &mut Button, name: &str) {
        match name {
            RunCommandComponent::NAME => {
                button.remove_component::<RunCommandComponent>();
            }

            KeySequenceComponent::NAME => {
                button.remove_component::<KeySequenceComponent>();
            }

            _ => {}
        }
    }

    fn paste_component(&self, _: CoreHandle, reference_button: &Button, new_button: &mut Button) {
        straight_copy(reference_button, new_button, RunCommandComponent::NAME);
        straight_copy(reference_button, new_button, KeySequenceComponent::NAME);
    }

    fn component_values(&self, _: CoreHandle, button: &Button, name: &str) -> Vec<UIValue> {
        match name {
            RunCommandComponent::NAME => {
                run_command::get_values(button)
            }

            KeySequenceComponent::NAME => {
                key_sequence::get_values(button)
            }

            _ => vec![],
        }
    }

    fn set_component_value(&self, _: CoreHandle, button: &mut Button, name: &str, value: Vec<UIValue>) {
        match name {
            RunCommandComponent::NAME => {
                run_command::set_values(button, value)
            }

            KeySequenceComponent::NAME => {
                key_sequence::set_values(button, value)
            }

            _ => {}
        }
    }

    fn listening_for(&self) -> Vec<String> {
        vec![
            RunCommandComponent::NAME.to_string(),
            KeySequenceComponent::NAME.to_string()
        ]
    }

    fn event(&self, _: CoreHandle, event: SDEvent) {
        match event {
            SDEvent::ButtonAction { pressed_button, .. } => {
                run_command::action(&pressed_button);
                key_sequence::action(&pressed_button, &self.key_transmitter);
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