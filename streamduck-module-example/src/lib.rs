use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use streamduck_core::core::button::{Button, parse_unique_button_to_component};
use streamduck_core::modules::{PluginMetadata, SDModule, SDModulePointer};
use streamduck_core::versions::{EVENTS, PLUGIN_API, RENDERING, SDMODULE_TRAIT};
use serde::{Serialize, Deserialize};
use streamduck_core::core::methods::CoreHandle;
use streamduck_core::modules::components::{ComponentDefinition, UIFieldType, UIFieldValue, UIScalar, UIValue};
use streamduck_core::modules::events::SDEvent;
use streamduck_core::core::thread::{ButtonBackground, RendererComponent};
use streamduck_core::core::UniqueButton;
use streamduck_core::image::DynamicImage;
use streamduck_core_derive::component;
use streamduck_core_derive::plugin_config;
use streamduck_core::socket::{SocketHandle, SocketListener, SocketManager, SocketPacket};
use streamduck_core::util::rendering::render_box_on_image;
use streamduck_core::util::rusttype::{Point, Scale};

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
            EVENTS,
            RENDERING
        ]
    )
}

#[no_mangle]
pub fn get_module(socket_manager: Arc<SocketManager>) -> SDModulePointer {
    socket_manager.add_listener(Box::new(ExampleListener));
    Box::into_raw(Box::new(ExampleModule))
}

pub struct ExampleListener;

impl SocketListener for ExampleListener {
    fn message(&self, _socket: SocketHandle, packet: SocketPacket) {
        if packet.ty == "set_component_value" {
            println!("packet: {:?}", packet)
        }
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
            default_looks: RendererComponent {
                background: ButtonBackground::Solid((255, 0, 255, 255)),
                text: vec![],
                plugin_blacklist: vec![],
                to_cache: true
            }
        });

        map
    }


    fn add_component(&self, _: CoreHandle, button: &mut Button, name: &str) {
        match name {
            "example" => {
                button.insert_component(
                    ExampleComponent { }
                ).ok();
            }

            _ => {}
        }
    }

    fn remove_component(&self, _: CoreHandle, button: &mut Button, name: &str) {
        match name {
            "example" => {
                button.remove_component::<ExampleComponent>();
            }

            _ => {}
        }
    }

    fn component_values(&self, _: CoreHandle, _: &Button, _: &str) -> Vec<UIValue> {
        vec![
            UIValue {
                name: "header".to_string(),
                display_name: "Header".to_string(),
                ty: UIFieldType::Header,
                value: UIFieldValue::Header
            },
            UIValue {
                name: "label".to_string(),
                display_name: "Label".to_string(),
                ty: UIFieldType::Label,
                value: UIFieldValue::Label("label".to_string())
            },
            UIValue {
                name: "float".to_string(),
                display_name: "Float".to_string(),
                ty: UIFieldType::InputFieldFloat,
                value: UIFieldValue::InputFieldFloat(2.43)
            },
            UIValue {
                name: "int".to_string(),
                display_name: "Integer".to_string(),
                ty: UIFieldType::InputFieldInteger,
                value: UIFieldValue::InputFieldInteger(3)
            },
            UIValue {
                name: "str".to_string(),
                display_name: "String".to_string(),
                ty: UIFieldType::InputFieldString,
                value: UIFieldValue::InputFieldString("string".to_string())
            },
            UIValue {
                name: "float2".to_string(),
                display_name: "Float2".to_string(),
                ty: UIFieldType::InputFieldFloat2,
                value: UIFieldValue::InputFieldFloat2(13.23, 23.1)
            },
            UIValue {
                name: "int2".to_string(),
                display_name: "Integer2".to_string(),
                ty: UIFieldType::InputFieldInteger2,
                value: UIFieldValue::InputFieldInteger2(13, 23)
            },
            UIValue {
                name: "uint".to_string(),
                display_name: "Unsigned Integer".to_string(),
                ty: UIFieldType::InputFieldUnsignedInteger,
                value: UIFieldValue::InputFieldUnsignedInteger(232)
            },
            UIValue {
                name: "float_slider".to_string(),
                display_name: "Float Slider".to_string(),
                ty: UIFieldType::ValueSliderFloat(UIScalar {
                    default_value: 5.0,
                    max_value: 100.0,
                    min_value: 0.0,
                    step: 1.0,
                    allow_out_of_bounds: false
                }),
                value: UIFieldValue::ValueSliderFloat(23.0)
            },
            UIValue {
                name: "int_slider".to_string(),
                display_name: "Integer Slider".to_string(),
                ty: UIFieldType::ValueSliderInteger(UIScalar {
                    default_value: 5,
                    max_value: 100,
                    min_value: 0,
                    step: 1,
                    allow_out_of_bounds: false
                }),
                value: UIFieldValue::ValueSliderInteger(23)
            }
        ]
    }

    fn set_component_value(&self, _: CoreHandle, _: &mut Button, _: &str, values: Vec<UIValue>) {
        println!("{:?}", values);
    }

    fn listening_for(&self) -> Vec<String> {
        vec![
            "renderer".to_string()
        ]
    }

    fn event(&self, core: CoreHandle, event: SDEvent) {
        match event {
            SDEvent::ButtonAction { pressed_button, .. } => {
                if let Ok(_) = parse_unique_button_to_component::<ExampleComponent>(&pressed_button) {
                    let config = core.config();

                    let mut my_config: ExampleSettings = config.get_plugin_settings().unwrap_or_default();
                    my_config.test += 1;
                    config.set_plugin_settings(my_config);

                    println!("Example button pressed");
                }
            }

            _ => {}
        }
    }

    fn render(&self, core: CoreHandle, button: &UniqueButton, frame: &mut DynamicImage) {
        render_box_on_image(frame, Scale::uniform(15.0), Point {x: 10.0, y: 25.0}, (255, 0, 0, 255));
    }

    fn render_hash(&self, core: CoreHandle, button: &UniqueButton, hash: &mut Box<dyn Hasher>) {
        0.hash(hash);
    }
}

#[component("example")]
#[derive(Serialize, Deserialize, Default)]
pub struct ExampleComponent {

}

#[plugin_config("example")]
#[derive(Serialize, Deserialize, Default)]
pub struct ExampleSettings {
    test: i64
}