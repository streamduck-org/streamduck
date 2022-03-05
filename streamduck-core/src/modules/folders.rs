use std::collections::HashMap;
use std::sync::RwLock;
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::{Serialize, Deserialize};
use crate::core::button::{Button, Component, parse_button_to_component, parse_unique_button_to_component};
use crate::core::{ButtonPanel, RawButtonPanel};
use crate::core::methods::{CoreHandle, get_stack, pop_screen, push_screen};
use crate::modules::components::{ComponentDefinition, map_ui_values, UIFieldType, UIFieldValue, UIValue};
use crate::modules::events::SDEvent;
use crate::modules::{PluginMetadata, SDModule};
use crate::threads::rendering::{ButtonBackground, ButtonText, RendererComponent};
use crate::util::{button_to_raw, make_panel_unique};
use crate::util::rendering::TextAlignment;
use crate::versions::{CORE, CORE_METHODS, EVENTS, MODULE_MANAGER};

const MODULE_NAME: &str = "core/folder";

#[derive(Default, Debug)]
pub struct FolderModule {
    folder_stack: RwLock<Vec<String>>,
    folder_references: RwLock<HashMap<String, ButtonPanel>>
}

impl SDModule for FolderModule {
    fn name(&self) -> String {
        MODULE_NAME.to_string()
    }

    fn components(&self) -> HashMap<String, ComponentDefinition> {
        let mut map = HashMap::new();

        map.insert(FolderComponent::NAME.to_string(), ComponentDefinition {
            display_name: "Folder".to_string(),
            description: "Opens folder".to_string(),
            default_looks: RendererComponent {
                background: ButtonBackground::Solid((0, 50, 200, 255)),
                text: vec![
                    ButtonText {
                        text: "Folder".to_string(),
                        font: "default".to_string(),
                        scale: (22.0, 22.0),
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

        map.insert(FolderLinkComponent::NAME.to_string(), ComponentDefinition {
            display_name: "Folder Link".to_string(),
            description: "Opens existing folders in symlink style".to_string(),
            default_looks: RendererComponent {
                background: ButtonBackground::Solid((0, 50, 200, 255)),
                text: vec![
                    ButtonText {
                        text: "⇲".to_string(),
                        font: "default".to_string(),
                        scale: (32.0, 32.0),
                        alignment: TextAlignment::BottomRight,
                        padding: 7,
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
                        font: "default".to_string(),
                        scale: (22.0, 22.0),
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

    fn add_component(&self, core: CoreHandle, button: &mut Button, name: &str) {
        match name {
            FolderComponent::NAME => {
                let folder_id = self.new_folder(&core);

                button.insert_component(
                    FolderComponent {
                        id: folder_id,
                        name: "Folder".to_string()
                    }
                ).ok();
            }

            FolderLinkComponent::NAME => {
                button.insert_component(
                    FolderLinkComponent {
                        id: "".to_string()
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

    fn remove_component(&self, core: CoreHandle, button: &mut Button, name: &str) {
        match name {
            FolderComponent::NAME => {
                if let Ok(component) = parse_button_to_component::<FolderComponent>(button) {
                    self.delete_folder_recursively(&core, &component.id);
                }

                button.remove_component::<FolderComponent>();
            }

            FolderLinkComponent::NAME => {
                button.remove_component::<FolderLinkComponent>();
            }

            FolderUpComponent::NAME => {
                button.remove_component::<FolderUpComponent>();
            }

            _ => {}
        }
    }

    fn component_values(&self, core: CoreHandle, button: &Button, component: &str) -> Vec<UIValue> {
        match component {
            FolderComponent::NAME => {
                if let Ok(component) = parse_button_to_component::<FolderComponent>(button) {
                    return vec![
                        UIValue {
                            name: "id".to_string(),
                            display_name: "ID".to_string(),
                            ty: UIFieldType::Label,
                            value: UIFieldValue::Label(component.id)
                        },
                        UIValue {
                            name: "name".to_string(),
                            display_name: "Folder Name".to_string(),
                            ty: UIFieldType::InputFieldString,
                            value: UIFieldValue::InputFieldString(component.name)
                        }
                    ];
                }
            }

            FolderLinkComponent::NAME => {
                if let Ok(component) = parse_button_to_component::<FolderLinkComponent>(button) {
                    let choices = self.list_folders(&core)
                        .into_iter()
                        .map(|(id, panel)| format!("{} ({})", panel.display_name, id))
                        .collect::<Vec<String>>();

                    let choice = if let Some(panel) = self.get_folder(&core, &component.id) {
                        format!("{} ({})", panel.display_name, component.id)
                    } else {
                        "".to_string()
                    };

                    return vec![
                        UIValue {
                            name: "id".to_string(),
                            display_name: "ID".to_string(),
                            ty: UIFieldType::Choice(choices),
                            value: UIFieldValue::Choice(choice)
                        }
                    ];
                }
            }

            _ => {}
        }

        vec![]
    }

    fn set_component_value(&self, core: CoreHandle, button: &mut Button, component: &str, values: Vec<UIValue>) {
        match component {
            FolderComponent::NAME => {
                if let Ok(mut component) = parse_button_to_component::<FolderComponent>(button) {
                    let change_map = map_ui_values(values);

                    if let Some(value) = change_map.get("name") {
                        if let Ok(str) = value.value.try_into_string() {
                            component.name = str;

                            if let Some(mut folder) = self.get_folder(&core, &component.id) {
                                folder.display_name = component.name.clone();
                                self.update_folder(&core, component.id.clone(), folder);
                            }

                            let handle = self.folder_references.read().unwrap();
                            if let Some(folder) = handle.get(&component.id).cloned() {
                                let mut folder_handle = folder.write().unwrap();
                                folder_handle.display_name = component.name.clone()
                            }
                        }
                    }

                    button.insert_component(component).ok();
                }
            }

            FolderLinkComponent::NAME => {
                if let Ok(mut component) = parse_button_to_component::<FolderLinkComponent>(button) {
                    let choices = self.list_folders(&core)
                        .into_iter()
                        .map(|(id, panel)| format!("{} ({})", panel.display_name, id))
                        .collect::<Vec<String>>();

                    let change_map = map_ui_values(values);

                    if let Some(value) = change_map.get("id") {
                        if let Ok(str) = value.value.try_into_string() {
                            if choices.contains(&str) {
                                let split = str.split(&['(', ')'][..]).collect::<Vec<&str>>();
                                component.id = split[1].to_string();
                            }
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
            FolderComponent::NAME.to_string(),
            FolderLinkComponent::NAME.to_string(),
            FolderUpComponent::NAME.to_string()
        ]
    }

    fn event(&self, core: CoreHandle, event: SDEvent) {
        match event {
            SDEvent::ButtonAdded { key, added_button, .. } => {
                let mut stack = self.folder_stack.read().unwrap().clone();

                if let Some(id) = stack.pop() {
                    if let Some(mut contents) = self.get_folder(&core, &id) {
                        contents.buttons.insert(key, button_to_raw(&added_button));
                        self.update_folder(&core, id, contents);
                    }
                }
            }

            SDEvent::ButtonUpdated { key, new_button, .. } => {
                let mut stack = self.folder_stack.read().unwrap().clone();

                if let Some(id) = stack.pop() {
                    if let Some(mut contents) = self.get_folder(&core, &id) {
                        contents.buttons.insert(key, button_to_raw(&new_button));
                        self.update_folder(&core, id, contents);
                    }
                }
            }

            SDEvent::ButtonDeleted { key, deleted_button, .. } => {
                let mut stack = self.folder_stack.read().unwrap().clone();

                if let Some(id) = stack.pop() {
                    if let Some(mut contents) = self.get_folder(&core, &id) {
                        contents.buttons.remove(&key);
                        self.update_folder(&core, id, contents);
                    }
                }

                if let Ok(component) = parse_unique_button_to_component::<FolderComponent>(&deleted_button) {
                    self.delete_folder_recursively(&core, &component.id);
                }
            }

            SDEvent::ButtonAction { pressed_button, .. } => {
                if let Ok(_) = parse_unique_button_to_component::<FolderUpComponent>(&pressed_button) {
                    if get_stack(&core).len() > 1 {
                        pop_screen(&core);

                        self.folder_stack.write().unwrap().pop();
                    }
                } else if let Ok(folder) = parse_unique_button_to_component::<FolderComponent>(&pressed_button) {
                    let mut folder_ref_handle = self.folder_references.write().unwrap();

                    if let Some(panel) = folder_ref_handle.get(&folder.id).cloned() {
                        push_screen(&core, panel);
                        self.folder_stack.write().unwrap().push(folder.id.clone());
                    } else {
                        if let Some(mut contents) = self.get_folder(&core, &folder.id) {
                            contents.display_name = folder.name;
                            let panel = make_panel_unique(contents);
                            push_screen(&core, panel.clone());
                            self.folder_stack.write().unwrap().push(folder.id.clone());
                            folder_ref_handle.insert(folder.id, panel);
                        }
                    }


                } else if let Ok(folder_link) = parse_unique_button_to_component::<FolderLinkComponent>(&pressed_button) {
                    let mut folder_ref_handle = self.folder_references.write().unwrap();

                    if let Some(panel) = folder_ref_handle.get(&folder_link.id).cloned() {
                        push_screen(&core, panel);
                        self.folder_stack.write().unwrap().push(folder_link.id.clone());
                    } else {
                        if let Some(contents) = self.get_folder(&core, &folder_link.id) {
                            let panel = make_panel_unique(contents);
                            push_screen(&core, panel.clone());
                            self.folder_stack.write().unwrap().push(folder_link.id.clone());
                            folder_ref_handle.insert(folder_link.id, panel);
                        }
                    }
                }
            }

            SDEvent::PanelPushed { .. } => {
                self.folder_stack.write().unwrap().push("unknown".to_string())
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

type FolderMap = HashMap<String, RawButtonPanel>;

impl FolderModule {
    /// Creates a new folder in plugin data
    fn new_folder(&self, core: &CoreHandle) -> String {
        let core = core.core();
        let mut config_handle = core.device_config.write().unwrap();

        let mut folders = if let Some(folders) = config_handle.plugin_data.get("folders") {
            if let Ok(folders) = serde_json::from_value::<FolderMap>(folders.clone()) {
                folders
            } else {
                Default::default()
            }
        } else {
            Default::default()
        };

        loop {
            let str = rand::thread_rng().sample_iter(&Alphanumeric).take(16).map(char::from).collect::<String>();
            if !folders.contains_key(&str) {
                folders.insert(str.clone(), RawButtonPanel {
                    display_name: "Folder".to_string(),
                    data: Default::default(),
                    buttons: Default::default()
                });
                config_handle.plugin_data.insert("folders".to_string(), serde_json::to_value(folders).unwrap());
                return str;
            }
        }
    }

    /// Lists folders in plugin data
    fn list_folders(&self, core: &CoreHandle) -> FolderMap {
        let core = core.core();
        let config_handle = core.device_config.read().unwrap();

        if let Some(folders) = config_handle.plugin_data.get("folders") {
            if let Ok(folders) = serde_json::from_value::<FolderMap>(folders.clone()) {
                folders
            } else {
                Default::default()
            }
        } else {
            Default::default()
        }
    }

    /// Gets folder contents from plugin data
    fn get_folder(&self, core: &CoreHandle, folder_id: &str) -> Option<RawButtonPanel> {
        let core = core.core();
        let config_handle = core.device_config.read().unwrap();

        if let Some(folders) = config_handle.plugin_data.get("folders") {
            if let Ok(mut folders) = serde_json::from_value::<FolderMap>(folders.clone()) {
                folders.remove(folder_id)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Sets folder in plugin data
    fn update_folder(&self, core: &CoreHandle, folder_id: String, folder_content: RawButtonPanel) {
        let core = core.core();
        let mut config_handle = core.device_config.write().unwrap();

        let mut folders = if let Some(folders) = config_handle.plugin_data.get("folders") {
            if let Ok(folders) = serde_json::from_value::<FolderMap>(folders.clone()) {
                folders
            } else {
                Default::default()
            }
        } else {
            Default::default()
        };

        folders.insert(folder_id.clone(), folder_content);
        config_handle.plugin_data.insert("folders".to_string(), serde_json::to_value(folders).unwrap());
    }

    /// Deletes folder from plugin data
    fn delete_folder(&self, core: &CoreHandle, folder_id: &str) {
        let core = core.core();
        let mut config_handle = core.device_config.write().unwrap();

        let mut folders = if let Some(folders) = config_handle.plugin_data.get("folders") {
            if let Ok(folders) = serde_json::from_value::<FolderMap>(folders.clone()) {
                folders
            } else {
                Default::default()
            }
        } else {
            Default::default()
        };

        folders.remove(folder_id);
        config_handle.plugin_data.insert("folders".to_string(), serde_json::to_value(folders).unwrap());
    }

    /// Deletes folder with all folders that are linked from the folder recursively
    fn delete_folder_recursively(&self, core: &CoreHandle, folder_id: &str) {
        if let Some(folder) = self.get_folder(core, folder_id) {
            for (_, button) in folder.buttons {
                if let Ok(folder) = parse_button_to_component::<FolderComponent>(&button) {
                    self.delete_folder_recursively(core, &folder.id);
                }
            }

            self.delete_folder(core, folder_id);
        }
    }
}


#[derive(Serialize, Deserialize, Default)]
pub struct FolderComponent {
    #[serde(default)]
    pub id: String,
    pub name: String,
}

impl Component for FolderComponent {
    const NAME: &'static str = "folder";
}

#[derive(Serialize, Deserialize, Default)]
pub struct FolderLinkComponent {
    #[serde(default)]
    pub id: String
}

impl Component for FolderLinkComponent {
    const NAME: &'static str = "folder_link";
}

#[derive(Serialize, Deserialize, Default)]
pub struct FolderUpComponent {}

impl Component for FolderUpComponent {
    const NAME: &'static str = "folder_up";
}