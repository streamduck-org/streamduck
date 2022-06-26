use std::collections::{HashMap, HashSet};
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::{Deserialize, Serialize};
use async_recursion::async_recursion;
use tokio::sync::RwLock;
use crate::core::button::{Button, Component, parse_button_to_component, parse_unique_button_to_component};
use crate::core::{ButtonPanel, CoreHandle, RawButtonPanel};
use crate::modules::components::{ComponentDefinition, map_ui_values, UIFieldType, UIFieldValue, UIValue};
use crate::modules::events::SDCoreEvent;
use crate::modules::{PluginMetadata, SDModule};
use crate::thread::rendering::{ButtonBackground, ButtonText, RendererComponentBuilder};
use crate::util::{button_to_raw, make_panel_unique, straight_copy};
use crate::thread::util::TextAlignment;
use crate::versions::{CORE, CORE_METHODS, CORE_EVENTS, MODULE_MANAGER};

const MODULE_NAME: &str = "core/folder";

#[derive(Debug)]
pub struct FolderModule {
    folder_references: RwLock<HashMap<String, ButtonPanel>>,
}

impl Default for FolderModule {
    fn default() -> Self {
        Self {
            folder_references: Default::default(),
        }
    }
}

#[async_trait]
impl SDModule for FolderModule {
    fn name(&self) -> String {
        MODULE_NAME.to_string()
    }

    fn components(&self) -> HashMap<String, ComponentDefinition> {
        let mut map = HashMap::new();

        map.insert(FolderComponent::NAME.to_string(), ComponentDefinition {
            display_name: "Folder".to_string(),
            description: "Opens folder".to_string(),
            default_looks: RendererComponentBuilder::new()
                .background(ButtonBackground::Solid((0, 50, 200, 255)))
                .add_text(ButtonText {
                    text: "Folder".to_string(),
                    font: "default".to_string(),
                    scale: (22.0, 22.0),
                    alignment: TextAlignment::Center,
                    padding: 0,
                    offset: (0.0, 0.0),
                    color: (255, 255, 255, 255),
                    shadow: None
                })
                .build()
        });

        map.insert(FolderLinkComponent::NAME.to_string(), ComponentDefinition {
            display_name: "Folder Link".to_string(),
            description: "Opens existing folders in symlink style".to_string(),
            default_looks: RendererComponentBuilder::new()
                .background(ButtonBackground::Solid((0, 50, 200, 255)))
                .add_text(ButtonText {
                                text: "⇲".to_string(),
                                font: "default".to_string(),
                                scale: (32.0, 32.0),
                                alignment: TextAlignment::BottomRight,
                                padding: 7,
                                offset: (0.0, 0.0),
                                color: (255, 255, 255, 255),
                                shadow: None
                })
                .build()
        });

        map.insert(FolderUpComponent::NAME.to_string(), ComponentDefinition {
            display_name: "Folder Up".to_string(),
            description: "Back button for folders".to_string(),
            default_looks: RendererComponentBuilder::new()
                .background(ButtonBackground::Solid((50, 50, 50, 255)))
                .add_text(ButtonText {
                    text: "Back".to_string(),
                    font: "default".to_string(),
                    scale: (22.0, 22.0),
                    alignment: TextAlignment::Center,
                    padding: 0,
                    offset: (0.0, 0.0),
                    color: (255, 255, 255, 255),
                    shadow: None
                })
                .build()
        });

        map
    }

    async fn add_component(&self, core: CoreHandle, button: &mut Button, name: &str) {
        match name {
            FolderComponent::NAME => {
                if !button.contains(FolderLinkComponent::NAME) {
                    let folder_id = self.new_folder(&core).await;

                    button.insert_component(
                        FolderComponent {
                            id: folder_id,
                            name: "Folder".to_string()
                        }
                    ).ok();
                }
            }

            FolderLinkComponent::NAME => {
                if !button.contains(FolderComponent::NAME) {
                    button.insert_component(
                        FolderLinkComponent {
                            id: "".to_string()
                        }
                    ).ok();
                }
            }

            FolderUpComponent::NAME => {
                button.insert_component(
                    FolderUpComponent {}
                ).ok();
            }

            _ => {}
        }
    }

    async fn remove_component(&self, core: CoreHandle, button: &mut Button, name: &str) {
        match name {
            FolderComponent::NAME => {
                button.remove_component::<FolderComponent>();
                self.clean_unused_folders(&core).await;
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

    async fn paste_component(&self, core: CoreHandle, reference_button: &Button, new_button: &mut Button) {
        straight_copy(reference_button, new_button, FolderLinkComponent::NAME);
        straight_copy(reference_button, new_button, FolderUpComponent::NAME);

        if let Ok(component) = parse_button_to_component::<FolderComponent>(reference_button) {
            let new_id = if let Some(id) = self.copy_folder_recursively(&core, &component.id).await {
                id
            } else {
                self.new_folder(&core).await
            };

            new_button.insert_component(FolderComponent {
                id: new_id,
                name: component.name
            }).ok();
        }
    }

    async fn component_values(&self, core: CoreHandle, button: &Button, component: &str) -> Vec<UIValue> {
        match component {
            FolderComponent::NAME => {
                if let Ok(component) = parse_button_to_component::<FolderComponent>(button) {
                    return vec![
                        UIValue {
                            name: "id".to_string(),
                            display_name: "ID".to_string(),
                            description: "ID of the folder".to_string(),
                            ty: UIFieldType::Label,
                            value: UIFieldValue::Label(component.id)
                        },
                        UIValue {
                            name: "name".to_string(),
                            display_name: "Folder Name".to_string(),
                            description: "Name that will appear in breadcrumbs of the stack".to_string(),
                            ty: UIFieldType::InputFieldString,
                            value: UIFieldValue::InputFieldString(component.name)
                        }
                    ];
                }
            }

            FolderLinkComponent::NAME => {
                if let Ok(component) = parse_button_to_component::<FolderLinkComponent>(button) {
                    let choices = self.list_folders(&core).await
                        .into_iter()
                        .map(|(id, panel)| format!("{} ({})", panel.display_name, id))
                        .collect::<Vec<String>>();

                    let choice = if let Some(panel) = self.get_folder(&core, &component.id).await {
                        format!("{} ({})", panel.display_name, component.id)
                    } else {
                        "".to_string()
                    };

                    return vec![
                        UIValue {
                            name: "id".to_string(),
                            display_name: "ID".to_string(),
                            description: "Folder to link to".to_string(),
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

    async fn set_component_value(&self, core: CoreHandle, button: &mut Button, component: &str, values: Vec<UIValue>) {
        match component {
            FolderComponent::NAME => {
                if let Ok(mut component) = parse_button_to_component::<FolderComponent>(button) {
                    let change_map = map_ui_values(values);

                    if let Some(value) = change_map.get("name") {
                        if let Ok(str) = value.value.try_into_string() {
                            component.name = str;

                            if let Some(mut folder) = self.get_folder(&core, &component.id).await {
                                folder.display_name = component.name.clone();
                                self.update_folder(&core, &component.id, folder).await;
                            }

                            let handle = self.folder_references.read().await;
                            if let Some(folder) = handle.get(&component.id).cloned() {
                                let mut folder_handle = folder.write().await;
                                folder_handle.display_name = component.name.clone()
                            }
                        }
                    }

                    button.insert_component(component).ok();
                }
            }

            FolderLinkComponent::NAME => {
                if let Ok(mut component) = parse_button_to_component::<FolderLinkComponent>(button) {
                    let choices = self.list_folders(&core).await
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

    async fn event(&self, core: CoreHandle, event: SDCoreEvent) {
        match event {
            SDCoreEvent::ButtonAdded { key, added_button, panel } |
            SDCoreEvent::ButtonUpdated { key, new_button: added_button, panel, .. } => {
                let panel = panel.read().await;

                if let Ok(stack_data) = serde_json::from_value::<FolderStackData>(panel.data.clone()) {
                    if let Some(mut contents) = self.get_folder(&core, &stack_data.folder_id).await {
                        contents.buttons.insert(key, button_to_raw(&added_button).await);
                        self.update_folder(&core, &stack_data.folder_id, contents).await;
                    }
                }
            }

            SDCoreEvent::ButtonDeleted { key, panel, .. } => {
                let panel = panel.read().await;

                if let Ok(stack_data) = serde_json::from_value::<FolderStackData>(panel.data.clone()) {
                    if let Some(mut contents) = self.get_folder(&core, &stack_data.folder_id).await {
                        contents.buttons.remove(&key);
                        self.update_folder(&core, &stack_data.folder_id, contents).await;
                    }
                }

                self.clean_unused_folders(&core).await;
            }

            SDCoreEvent::ButtonAction { pressed_button, .. } => {
                if let Ok(_) = parse_unique_button_to_component::<FolderUpComponent>(&pressed_button).await {
                    if core.current_stack().await.len() > 1 {
                        core.pop_screen().await;
                    }
                } else if let Ok(folder) = parse_unique_button_to_component::<FolderComponent>(&pressed_button).await {
                    let mut folder_ref_handle = self.folder_references.write().await;

                    if let Some(panel) = folder_ref_handle.get(&folder.id).cloned() {
                        core.push_screen(panel).await;
                    } else {
                        if let Some(mut contents) = self.get_folder(&core, &folder.id).await {
                            contents.display_name = folder.name;
                            contents.data = serde_json::to_value(FolderStackData {
                                folder_id: folder.id.to_string()
                            }).unwrap();

                            let panel = make_panel_unique(contents);
                            core.push_screen(panel.clone()).await;
                            folder_ref_handle.insert(folder.id, panel);
                        }
                    }


                } else if let Ok(folder_link) = parse_unique_button_to_component::<FolderLinkComponent>(&pressed_button).await {
                    let mut folder_ref_handle = self.folder_references.write().await;

                    if let Some(panel) = folder_ref_handle.get(&folder_link.id).cloned() {
                        core.push_screen(panel).await;
                    } else {
                        if let Some(mut contents) = self.get_folder(&core, &folder_link.id).await {
                            contents.data = serde_json::to_value(FolderStackData {
                                folder_id: folder_link.id.to_string()
                            }).unwrap();

                            let panel = make_panel_unique(contents);
                            core.push_screen(panel.clone()).await;
                            folder_ref_handle.insert(folder_link.id, panel);
                        }
                    }
                }
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
                CORE_EVENTS
            ]
        )
    }
}

type FolderMap = HashMap<String, RawButtonPanel>;

impl FolderModule {
    /// Generates a random name for folder
    fn random_name(&self) -> String {
        rand::thread_rng().sample_iter(&Alphanumeric).take(16).map(char::from).collect::<String>()
    }

    /// Generates a random name for folder and ensures it's not used anywhere
    async fn random_unique_name(&self, core: &CoreHandle) -> String {
        let folder_list = self.list_folders(core).await;

        let mut name = self.random_name();
        while folder_list.get(&name).is_some() {
            name = self.random_name();
        }

        name
    }

    /// Creates a new folder in plugin data
    async fn new_folder(&self, core: &CoreHandle) -> String {
        let folder_id = self.random_unique_name(&core).await;

        self.update_folders_data(core, |f| {
            f.insert(folder_id.clone(), RawButtonPanel {
                display_name: "Folder".to_string(),
                data: Default::default(),
                buttons: Default::default()
            });
        }).await;

        folder_id
    }

    /// Lists folders in plugin data
    async fn list_folders(&self, core: &CoreHandle) -> FolderMap {
        let core = core.core();
        let config_handle = core.device_config.read().await;

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
    async fn get_folder(&self, core: &CoreHandle, folder_id: &str) -> Option<RawButtonPanel> {
        let core = core.core();
        let config_handle = core.device_config.read().await;

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

    /// Updates folders data
    async fn update_folders_data<F: Fn(&mut FolderMap) -> ()>(&self, core: &CoreHandle, func: F) {
        let core = core.core();
        let mut config_handle = core.device_config.write().await;

        let mut folders = if let Some(folders_data) = config_handle.plugin_data.get("folders") {
            if let Ok(folders) = serde_json::from_value::<FolderMap>(folders_data.clone()) {
                folders
            } else {
                Default::default()
            }
        } else {
            Default::default()
        };

        func(&mut folders);
        config_handle.plugin_data.insert("folders".to_string(), serde_json::to_value(folders).unwrap());
    }

    /// Sets folder in plugin data
    async fn update_folder(&self, core: &CoreHandle, folder_id: &str, folder_content: RawButtonPanel) {
        self.update_folders_data(core, |f| {
            f.insert(folder_id.to_string(), folder_content.clone());
        }).await;
    }

    /// Copies folder recursively
    #[async_recursion]
    async fn copy_folder_recursively(&self, core: &CoreHandle, folder_id: &str) -> Option<String> {
        let mut folder = self.get_folder(core, folder_id).await?;

        for val in folder.buttons.values_mut() {
            if let Ok(mut folder) = parse_button_to_component::<FolderComponent>(val) {
                if let Some(folder_id) = self.copy_folder_recursively(core, &folder.id).await {
                    folder.id = folder_id;
                }

                val.insert_component(folder).ok();
            }
        }

        let str = self.random_unique_name(core).await;
        self.update_folder(core, &str, folder).await;
        Some(str)
    }

    /// Deletes folder with all folders that are linked from the folder recursively
    async fn clean_unused_folders(&self, core: &CoreHandle) {
        fn count(folders: &FolderMap, folder_id: &str, ids: &mut HashSet<String>) {
            if let Some(folder) = folders.get(folder_id) {
                ids.insert(folder_id.to_string());
                for (_, item) in &folder.buttons {
                    if let Ok(folder) = parse_button_to_component::<FolderComponent>(item) {
                        ids.insert(folder.id.clone());
                        count(folders, &folder.id, ids);
                    }
                }
            }
        }

        let folders = self.list_folders(core).await;

        let mut ids = HashSet::new();
        let root_buttons = core.core.device_config.read().await.layout.clone();

        for button in root_buttons.buttons.values() {
            if let Ok(folder) = parse_button_to_component::<FolderComponent>(button) {
                count(&folders, &folder.id, &mut ids);
            }
        }

        log::info!("Found: {:?}", ids);

        self.update_folders_data(core, |f| {
            f.retain(|x, _| ids.contains(x))
        }).await
    }
}


#[derive(Serialize, Deserialize)]
pub struct FolderComponent {
    #[serde(default)]
    pub id: String,
    pub name: String,
}

impl Component for FolderComponent {
    const NAME: &'static str = "folder";
}

#[derive(Serialize, Deserialize)]
pub struct FolderLinkComponent {
    #[serde(default)]
    pub id: String
}

impl Component for FolderLinkComponent {
    const NAME: &'static str = "folder_link";
}

#[derive(Serialize, Deserialize)]
pub struct FolderUpComponent {}

impl Component for FolderUpComponent {
    const NAME: &'static str = "folder_up";
}

#[derive(Serialize, Deserialize)]
pub struct FolderStackData {
    folder_id: String,
}