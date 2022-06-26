use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use streamduck_core::core::button::{Button, parse_unique_button_to_component};
use streamduck_core::modules::{PluginMetadata, SDModule};
use streamduck_core::versions::{CORE_EVENTS, PLUGIN_API, RENDERING, SDMODULE_TRAIT};
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};
use streamduck_core::modules::components::{ComponentDefinition, map_ui_values, UIFieldType, UIFieldValue, UIScalar, UIValue};
use streamduck_core::modules::events::SDCoreEvent;
use streamduck_core::core::{CoreHandle, UniqueButton};
use streamduck_core::core::manager::CoreManager;
use streamduck_core::image::{DynamicImage, Rgba};
use streamduck_core::images::convert_image;
use streamduck_core::modules::plugins::{PluginModuleManager, PluginRenderingManager, PluginSocketManager};
use streamduck_core_derive::component;
use streamduck_core_derive::plugin_config;
use streamduck_core::socket::{SocketHandle, SocketListener, SocketPacket};
use streamduck_core::streamdeck::{DeviceImage, Kind};
use streamduck_core::thread::rendering::{ButtonBackground, RendererComponent, RendererComponentBuilder};
use streamduck_core::thread::rendering::custom::{CustomRenderer, DeviceReference};
use streamduck_core::thread::util::{image_from_horiz_gradient, image_from_solid, render_box_on_image};
use streamduck_core::util::rusttype::{Point, Scale};
use streamduck_core::util::straight_copy;
use streamduck_core::async_trait;

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
            CORE_EVENTS,
            RENDERING
        ]
    )
}

#[no_mangle]
pub fn register(socket_manager: Arc<PluginSocketManager>, render_manager: Arc<PluginRenderingManager>, module_manager: Arc<PluginModuleManager>) {
    socket_manager.add_listener(Arc::new(ExampleListener));
    render_manager.add_renderer(Arc::new(ExampleRenderer::new()));
    //module_manager.add_module(Arc::new(ExampleModule));
}

pub struct ExampleListener;

#[async_trait]
impl SocketListener for ExampleListener {
    async fn message(&self, _socket: SocketHandle<'_>, packet: SocketPacket) {
        println!("packet: {:?}", packet);
    }
}

#[derive(Debug)]
pub struct ExampleModule;

#[async_trait]
impl SDModule for ExampleModule {
    fn name(&self) -> String {
        "example".to_string()
    }

    fn components(&self) -> HashMap<String, ComponentDefinition> {
        let mut map = HashMap::new();

        map.insert("example".to_string(), ComponentDefinition {
            display_name: "Example".to_string(),
            description: "Example component".to_string(),
            default_looks: RendererComponentBuilder::new()
                .background(ButtonBackground::Solid((255, 0, 255, 255)))
                .build()
        });

        map
    }


    async fn add_component(&self, _: CoreHandle, button: &mut Button, name: &str) {
        match name {
            "example" => {
                button.insert_component(
                    ExampleComponent { }
                ).ok();
            }

            _ => {}
        }
    }

    async fn remove_component(&self, _: CoreHandle, button: &mut Button, name: &str) {
        match name {
            "example" => {
                button.remove_component::<ExampleComponent>();
            }

            _ => {}
        }
    }

    async fn paste_component(&self, _: CoreHandle, reference_button: &Button, new_button: &mut Button) {
        straight_copy(reference_button, new_button, "example");
    }

    async fn component_values(&self, _: CoreHandle, _: &Button, _: &str) -> Vec<UIValue> {
        vec![
            UIValue {
                name: "header".to_string(),
                display_name: "Header".to_string(),
                description: "".to_string(),
                ty: UIFieldType::Header,
                value: UIFieldValue::Header
            },
            UIValue {
                name: "label".to_string(),
                display_name: "Label".to_string(),
                description: "".to_string(),
                ty: UIFieldType::Label,
                value: UIFieldValue::Label("label".to_string())
            },
            UIValue {
                name: "float".to_string(),
                display_name: "Float".to_string(),
                description: "".to_string(),
                ty: UIFieldType::InputFieldFloat,
                value: UIFieldValue::InputFieldFloat(2.43)
            },
            UIValue {
                name: "int".to_string(),
                display_name: "Integer".to_string(),
                description: "".to_string(),
                ty: UIFieldType::InputFieldInteger,
                value: UIFieldValue::InputFieldInteger(3)
            },
            UIValue {
                name: "str".to_string(),
                display_name: "String".to_string(),
                description: "".to_string(),
                ty: UIFieldType::InputFieldString,
                value: UIFieldValue::InputFieldString("string".to_string())
            },
            UIValue {
                name: "float2".to_string(),
                display_name: "Float2".to_string(),
                description: "".to_string(),
                ty: UIFieldType::InputFieldFloat2,
                value: UIFieldValue::InputFieldFloat2(13.23, 23.1)
            },
            UIValue {
                name: "int2".to_string(),
                display_name: "Integer2".to_string(),
                description: "".to_string(),
                ty: UIFieldType::InputFieldInteger2,
                value: UIFieldValue::InputFieldInteger2(13, 23)
            },
            UIValue {
                name: "uint".to_string(),
                display_name: "Unsigned Integer".to_string(),
                description: "".to_string(),
                ty: UIFieldType::InputFieldUnsignedInteger,
                value: UIFieldValue::InputFieldUnsignedInteger(232)
            },
            UIValue {
                name: "float_slider".to_string(),
                display_name: "Float Slider".to_string(),
                description: "".to_string(),
                ty: UIFieldType::ValueSliderFloat(UIScalar {
                    max_value: 100.0,
                    min_value: 0.0,
                    step: 0.1,
                    allow_out_of_bounds: false
                }),
                value: UIFieldValue::ValueSliderFloat(23.0)
            },
            UIValue {
                name: "int_slider".to_string(),
                display_name: "Integer Slider".to_string(),
                description: "".to_string(),
                ty: UIFieldType::ValueSliderInteger(UIScalar {
                    max_value: 100,
                    min_value: 0,
                    step: 1,
                    allow_out_of_bounds: false
                }),
                value: UIFieldValue::ValueSliderInteger(54)
            }
        ]
    }

    async fn set_component_value(&self, _: CoreHandle, _: &mut Button, _: &str, values: Vec<UIValue>) {
        println!("{:?}", values);
    }

    fn listening_for(&self) -> Vec<String> {
        vec![
            "renderer".to_string()
        ]
    }

    async fn settings(&self, _: Arc<CoreManager>) -> Vec<UIValue> {
        vec![
            UIValue {
                name: "test".to_string(),
                display_name: "test".to_string(),
                description: "".to_string(),
                ty: UIFieldType::ExistingImage,
                value: UIFieldValue::ExistingImage("0".to_string())
            }
        ]
    }

    async fn event(&self, core: CoreHandle, event: SDCoreEvent) {
        match event {
            SDCoreEvent::ButtonAction { pressed_button, .. } => {
                if let Ok(_) = parse_unique_button_to_component::<ExampleComponent>(&pressed_button).await {
                    let config = core.config();

                    let mut my_config: ExampleSettings = config.get_plugin_settings().await.unwrap_or_default();
                    my_config.test += 1;
                    config.set_plugin_settings(my_config).await;

                    println!("Example button pressed");
                }
            }

            _ => {}
        }
    }

    async fn render(&self, _: CoreHandle, _: &UniqueButton, frame: &mut DynamicImage) {
        render_box_on_image(frame, Scale::uniform(15.0), Point {x: 10.0, y: 25.0}, (255, 0, 0, 255));
    }

    fn render_hash(&self, _: CoreHandle, _: &UniqueButton, hash: &mut Box<dyn Hasher>) {
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

pub struct ExampleRenderer {
    tex: DeviceImage,
    already_rendered: Mutex<HashSet<u8>>,
}

impl ExampleRenderer {
    fn new() -> Self {
        Self {
            tex: convert_image( &Kind::OriginalV2, image_from_horiz_gradient((72, 72), Rgba([255, 0, 255, 255]), Rgba([255, 255, 255, 255]))),
            already_rendered: Default::default()
        }
    }
}

#[async_trait]
impl CustomRenderer for ExampleRenderer {
    fn name(&self) -> String {
        "example".to_string()
    }

    async fn refresh(&self, _: &CoreHandle) {
        self.already_rendered.lock().unwrap().clear();
    }

    async fn render(&self, key: u8, _: &UniqueButton, _: &CoreHandle, streamdeck: &mut DeviceReference) {
        let mut lock = self.already_rendered.lock().unwrap();

        if !lock.contains(&key) {
            streamdeck.write_image(&self.tex).ok();
            lock.insert(key);
        }
    }

    async fn representation(&self, _: u8, _: &UniqueButton, core: &CoreHandle) -> Option<DynamicImage> {
        Some(image_from_solid(core.core().image_size, Rgba([255, 0, 0, 255])))
    }

    async fn component_values(&self, _: &Button, component: &RendererComponent, _: &CoreHandle) -> Vec<UIValue> {
        let my_int = component.custom_data.as_i64().unwrap_or_default();

        vec![
            UIValue {
                name: "my_int".to_string(),
                display_name: "My Integer".to_string(),
                description: "Some example integer".to_string(),
                ty: UIFieldType::InputFieldInteger,
                value: UIFieldValue::InputFieldInteger(my_int as i32)
            }
        ]
    }

    async fn set_component_value(&self, _: &mut Button, component: &mut RendererComponent, _: &CoreHandle, value: Vec<UIValue>) {
        let change_map = map_ui_values(value);

        if let Some(value) = change_map.get("my_int") {
            if let Ok(value) = value.value.try_into_i32() {
                component.custom_data = Value::Number(Number::from(value))
            }
        }
    }
}