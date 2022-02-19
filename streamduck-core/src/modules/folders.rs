use std::collections::HashMap;
use std::sync::RwLock;
use serde::{Serialize, Deserialize};
use crate::core::button::{Button, Component, parse_button_to_component, parse_unique_button_to_component};
use crate::core::{RawButtonPanel, UniqueButton};
use crate::core::methods::{CoreHandle, get_stack, pop_screen, push_screen};
use crate::modules::components::{ComponentDefinition, UIValue};
use crate::modules::events::SDEvent;
use crate::modules::{PluginMetadata, SDModule};
use crate::threads::rendering::{ButtonBackground, ButtonText, RendererComponent};
use crate::util::{button_to_raw, make_panel_unique};
use crate::util::rendering::TextAlignment;
use crate::versions::{CORE, CORE_METHODS, EVENTS, MODULE_MANAGER};

const MODULE_NAME: &str = "core/folder";

#[derive(Default, Debug)]
pub struct FolderModule {
    pub folder_stack: RwLock<Vec<(u8, UniqueButton)>>
}

impl SDModule for FolderModule {
    fn name(&self) -> String {
        MODULE_NAME.to_string()
    }

    fn components(&self) -> HashMap<String, ComponentDefinition> {
        let mut map = HashMap::new();

        map.insert(FolderComponent::NAME.to_string(), ComponentDefinition {
            display_name: "Folder".to_string(),
            description: "Enables Folder functionality on the button".to_string(),
            default_looks: RendererComponent {
                background: ButtonBackground::Solid((0, 50, 200, 255)),
                text: vec![
                    ButtonText {
                        text: "Folder".to_string(),
                        font: "SourceHanSans-Medium.ttf".to_string(),
                        scale: (25.0, 25.0),
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

        map.insert(FolderUpComponent::NAME.to_string(), ComponentDefinition {
            display_name: "Folder Up".to_string(),
            description: "Back button for folders".to_string(),
            default_looks: RendererComponent {
                background: ButtonBackground::Solid((50, 50, 50, 255)),
                text: vec![
                    ButtonText {
                        text: "Back".to_string(),
                        font: "SourceHanSans-Medium.ttf".to_string(),
                        scale: (25.0, 25.0),
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
            FolderComponent::NAME => {
                button.insert_component(
                    FolderComponent {
                        buttons: Default::default()
                    }
                ).ok();
            }

            FolderUpComponent::NAME => {
                button.insert_component(
                    FolderUpComponent {}
                ).ok();
            }

            _ => {}
        }
    }

    fn remove_component(&self, _: CoreHandle, button: &mut Button, name: &str) {
        match name {
            FolderComponent::NAME => {
                button.remove_component::<FolderComponent>();
            }

            FolderUpComponent::NAME => {
                button.remove_component::<FolderUpComponent>();
            }

            _ => {}
        }
    }

    fn component_values(&self, _: CoreHandle, _: &Button, _: &str) -> Vec<UIValue> {
        // There's no values on folder components
        vec![]
    }

    fn set_component_value(&self, _: CoreHandle, _: &mut Button, _: &str, _: Vec<UIValue>) {
        // There's no values
    }

    fn listening_for(&self) -> Vec<String> {
        vec![
            FolderComponent::NAME.to_string(),
            FolderUpComponent::NAME.to_string()
        ]
    }

    fn event(&self, core: CoreHandle, event: SDEvent) {
        match event {
            SDEvent::ButtonAdded { key, added_button, .. } => {
                self.update_folder(self.folder_stack.read().unwrap().clone(), key, added_button);
            }

            SDEvent::ButtonUpdated { key, new_button, .. } => {
                self.update_folder(self.folder_stack.read().unwrap().clone(), key, new_button);
            }

            SDEvent::ButtonDeleted { key, .. } => {
                let mut stack = self.folder_stack.read().unwrap().clone();

                if let Some((button_key, button)) = stack.pop() {
                    let mut button_handle = button.write().unwrap();

                    if let Ok(mut folder_component) = parse_button_to_component::<FolderComponent>(&button_handle) {
                        folder_component.buttons.remove(&key);
                        button_handle.insert_component(folder_component).ok();
                        drop(button_handle);

                        self.update_folder(stack, button_key, button);
                    }
                }
            }

            SDEvent::ButtonAction { key, pressed_button, .. } => {
                if let Ok(_) = parse_unique_button_to_component::<FolderUpComponent>(&pressed_button) {
                    if get_stack(&core).len() > 1 {
                        pop_screen(&core);

                        self.folder_stack.write().unwrap().pop();
                    }
                } else if let Ok(folder) = parse_unique_button_to_component::<FolderComponent>(&pressed_button) {
                    push_screen(&core, make_panel_unique(folder.buttons));
                    self.folder_stack.write().unwrap().push((key, pressed_button));
                }
            }

            SDEvent::PanelPopped { .. } => {
                self.folder_stack.write().unwrap().pop();
            }

            SDEvent::StackReset { .. } => {
                self.folder_stack.write().unwrap().clear();
            }

            _ => {}
        }
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata::from_literals(
            MODULE_NAME,
            "TheJebForge",
            "Folder module, provides folder components",
            "0.1",
            &[
                CORE,
                CORE_METHODS,
                MODULE_MANAGER,
                EVENTS
            ]
        )
    }
}

impl FolderModule {
    fn update_folder(&self, mut current_stack: Vec<(u8, UniqueButton)>, current_key: u8, current_button: UniqueButton) {
        if let Some((key, button)) = current_stack.pop() {
            let current_raw_button = button_to_raw(&current_button);
            let mut button_handle = button.write().unwrap();

            if let Ok(mut folder_data) = parse_button_to_component::<FolderComponent>(&button_handle) {
                folder_data.buttons.insert(current_key, current_raw_button);
                button_handle.insert_component(folder_data).ok();
                drop(button_handle);

                self.update_folder(current_stack, key, button);
            }
        }
    }
}


#[derive(Serialize, Deserialize, Default)]
pub struct FolderComponent {
    #[serde(default)]
    pub buttons: RawButtonPanel
}

impl Component for FolderComponent {
    const NAME: &'static str = "folder";
}

#[derive(Serialize, Deserialize, Default)]
pub struct FolderUpComponent {}

impl Component for FolderUpComponent {
    const NAME: &'static str = "folder_up";
}