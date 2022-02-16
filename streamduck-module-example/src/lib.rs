use std::collections::HashMap;
use std::sync::Arc;
use streamduck_core::core::button::{Button, parse_unique_button_to_component};
use streamduck_core::modules::{PluginMetadata, SDModule, SDModulePointer};
use streamduck_core::versions::{EVENTS, PLUGIN_API, SDMODULE_TRAIT};
use serde::{Serialize, Deserialize};
use streamduck_core::core::methods::CoreHandle;
use streamduck_core::modules::components::{ComponentDefinition, UIValue};
use streamduck_core::modules::events::SDEvent;
use streamduck_core::threads::rendering::{ButtonBackground, RendererComponent};
use streamduck_core_derive::component;
use streamduck_daemon::socket::{SocketHandle, SocketListener, SocketManager, SocketPacket};

#[no_mangle]
pub fn get_metadata() -> PluginMetadata {
    PluginMetadata::from_literals(
        "example",
        "TheJebForge",
        "Just an example plugin crate",
        "0.1",
        &[
            PLUGIN_API,
            SDMODULE_TRAIT,
            EVENTS
        ]
    )
}

#[no_mangle]
pub fn get_module() -> SDModulePointer {
    Box::into_raw(Box::new(ExampleModule))
}

#[no_mangle]
pub fn register(socket_manager: Arc<SocketManager>) {
    socket_manager.add_listener(Box::new(ExampleListener))
}

pub struct ExampleListener;

impl SocketListener for ExampleListener {
    fn message(&self, _socket: SocketHandle, packet: SocketPacket) {
        println!("received packet {:?}", packet);
    }
}

#[derive(Debug)]
pub struct ExampleModule;

impl SDModule for ExampleModule {
    fn name(&self) -> String {
        "example".to_string()
    }

    fn components(&self) -> HashMap<String, ComponentDefinition> {
        let mut map = HashMap::new();

        map.insert("example".to_string(), ComponentDefinition {
            display_name: "Example".to_string(),
            description: "Example component".to_string(),
            exposed_fields: vec![],
            default_looks: RendererComponent {
                background: ButtonBackground::Solid((255, 0, 255, 255)),
                text: vec![],
                to_cache: true
            }
        });

        map
    }


    fn add_component(&self, button: &mut Button, name: &str) {
        match name {
            "example" => {
                button.insert_component(
                    ExampleComponent { }
                ).ok();
            }

            _ => {}
        }
    }

    fn component_values(&self, _: &Button, _: &str) -> Vec<UIValue> {
        vec![]
    }

    fn set_component_value(&self, _: &mut Button, _: &str, _: UIValue) {

    }

    fn listening_for(&self) -> Vec<String> {
        vec![
            "renderer".to_string()
        ]
    }

    fn event(&self, _core: CoreHandle, event: SDEvent) {
        println!("Received event: {:?}", event);

        match event {
            SDEvent::ButtonAction { pressed_button, .. } => {
                if let Ok(_) = parse_unique_button_to_component::<ExampleComponent>(&pressed_button) {
                    println!("Example button pressed");
                }
            }

            _ => {}
        }
    }
}

#[component("example")]
#[derive(Serialize, Deserialize, Default)]
pub struct ExampleComponent {

}