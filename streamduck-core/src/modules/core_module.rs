//! Core module

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::config::PluginConfig;
use crate::core::button::{Button, Component};
use crate::core::{check_feature_list_for_feature, CoreHandle};
use crate::core::manager::CoreManager;
use crate::modules::components::{ComponentDefinition, map_ui_values, UIFieldType, UIFieldValue, UIValue};
use crate::modules::{PluginMetadata, SDModule};
use crate::modules::events::{core_event_to_global, SDCoreEvent};
use crate::socket::send_event_to_socket;
use crate::SocketManager;
use crate::thread::rendering::{RendererComponent, RendererSettings};
use crate::thread::rendering::component_values::{get_renderer_component_values, set_renderer_component_values};
use crate::util::straight_copy;
use crate::versions::{CORE, MODULE_MANAGER};

/// The core module, for exposing renderer component to requests and such
pub struct CoreModule {
    pub(crate) socket_manager: Arc<SocketManager>
}

#[async_trait]
impl SDModule for CoreModule {
    fn name(&self) -> String {
        "core".to_string()
    }

    fn components(&self) -> HashMap<String, ComponentDefinition> {
        let mut map = HashMap::new();

        map.insert("renderer".to_string(), ComponentDefinition {
            display_name: "Renderer".to_string(),
            description: "The only thing that makes a button render an image on streamdeck".to_string(),
            default_looks: Default::default()
        });

        map
    }

    async fn add_component(&self, _: CoreHandle, button: &mut Button, name: &str) {
        match name {
            "renderer" => {
                button.insert_component(RendererComponent::default()).ok();
            }
            _ => {}
        }
    }

    async fn remove_component(&self, _: CoreHandle, button: &mut Button, name: &str) {
        match name {
            "renderer" => {
                button.remove_component::<RendererComponent>();
            }
            _ => {}
        }
    }

    async fn paste_component(&self, _: CoreHandle, reference_button: &Button, new_button: &mut Button) {
        straight_copy(reference_button, new_button, RendererComponent::NAME);
    }

    async fn component_values(&self, core: CoreHandle, button: &Button, name: &str) -> Vec<UIValue> {
        match name {
            "renderer" => {
                get_renderer_component_values(&core, button).await
            }

            _ => vec![],
        }
    }

    async fn set_component_value(&self, core: CoreHandle, button: &mut Button, name: &str, value: Vec<UIValue>) {
        match name {
            "renderer" => {
                set_renderer_component_values(&core, button, value).await
            }

            _ => {}
        }
    }

    fn listening_for(&self) -> Vec<String> {
        vec![]
    }

    async fn settings(&self, core_manager: Arc<CoreManager>) -> Vec<UIValue> {
        let settings: CoreSettings = core_manager.config.get_plugin_settings().await.unwrap_or_default();

        let mut fields = vec![];

        fields.push(
            UIValue {
                name: "rendering".to_string(),
                display_name: "Rendering Settings".to_string(),
                description: "Settings related to rendering of buttons".to_string(),
                ty: UIFieldType::Collapsable,
                value: UIFieldValue::Collapsable({
                    let mut fields = vec![];

                    fields.push(
                        UIValue {
                            name: "plugin_blacklist".to_string(),
                            display_name: "Allowed plugins to render".to_string(),
                            description: "Disabled plugins will not appear on buttons".to_string(),
                            ty: UIFieldType::Collapsable,
                            value: UIFieldValue::Collapsable({
                                core_manager.module_manager.get_modules().await
                                    .into_values()
                                    .filter_map(|x| if check_feature_list_for_feature(&x.metadata().used_features, "rendering") {
                                        let name = x.name();
                                        Some(UIValue {
                                            name: name.clone(),
                                            display_name: name.clone(),
                                            description: "".to_string(),
                                            ty: UIFieldType::Checkbox { disabled: false },
                                            value: UIFieldValue::Checkbox(!settings.renderer.plugin_blacklist.contains(&name))
                                        })
                                    } else { None })
                                    .collect()
                            })
                        }
                    );

                    fields
                })
            }
        );

        fields
    }

    async fn set_setting(&self, core_manager: Arc<CoreManager>, value: Vec<UIValue>) {
        let mut settings: CoreSettings = core_manager.config.get_plugin_settings().await.unwrap_or_default();

        let change_map = map_ui_values(value);

        if let Some(value) = change_map.get("rendering") {
            if let UIFieldValue::Collapsable(value) = &value.value {
                let change_map = map_ui_values(value.clone());

                if let Some(value) = change_map.get("plugin_blacklist") {
                    if let UIFieldValue::Collapsable(value) = &value.value {
                        let change_map = map_ui_values(value.clone());

                        for (name, value) in change_map {
                            if let UIFieldValue::Checkbox(state) = value.value {
                                if state {
                                    settings.renderer.plugin_blacklist.retain(|x| *x != name);
                                } else {
                                    settings.renderer.plugin_blacklist.push(name);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Calling redraw for all devices
        for device in core_manager.list_added_devices().await.into_values() {
            device.core.mark_for_redraw().await;
        }

        core_manager.config.set_plugin_settings(settings).await;
    }

    async fn event(&self, core: CoreHandle, event: SDCoreEvent) {
        let global_event = core_event_to_global(event, &core.core.serial_number().await).await;
        send_event_to_socket(&self.socket_manager, global_event).await;
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata::from_literals(
            "core",
            "TheJebForge",
            "Core of the software, provides essential components",
            "0.1",
            &[
                CORE,
                MODULE_MANAGER
            ]
        )
    }
}

/// Settings related to various things around the core
#[derive(Serialize, Deserialize, Default)]
pub struct CoreSettings {
    pub renderer: RendererSettings
}

impl PluginConfig for CoreSettings {
    const NAME: &'static str = "core";
}

