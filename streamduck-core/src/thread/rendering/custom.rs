use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard};
use image::DynamicImage;
use streamdeck::{DeviceImage, StreamDeck};
use crate::core::button::Button;
use crate::core::{CoreHandle, UniqueButton};
use crate::modules::components::UIValue;
use crate::thread::rendering::RendererComponent;

/// Reference to Stream Deck
///
/// This is used to restrain plugins from setting buttons that don't use their renderers, would be uncool if some plugin decided to draw to every button instead of button that user wanted
pub struct DeviceReference<'a> {
    streamdeck: &'a mut StreamDeck,
    key: u8,
}

#[allow(dead_code)]
impl<'a> DeviceReference<'a> {
    pub(crate) fn new(streamdeck: &'a mut StreamDeck, key: u8) -> Self {
        Self {
            streamdeck,
            key
        }
    }

    /// Writes image to streamdeck directly on button currently processed by core
    pub fn write_image(&mut self, image: &DeviceImage) -> Result<(), streamdeck::Error> {
        self.streamdeck.write_button_image(self.key, image)
    }
}

/// Custom renderer trait
#[allow(unused_variables)]
pub trait CustomRenderer: Send + Sync {
    /// Name of the renderer
    fn name(&self) -> String;

    /// Called whenever current screen changes, should be used for any things that shouldn't be called every tick
    fn refresh(&self, core_handle: &CoreHandle) {}

    /// Called on every tick with device reference provided
    fn render(&self, key: u8, button: &UniqueButton, core_handle: &CoreHandle, streamdeck: &mut DeviceReference) {}

    /// Called on get_button_images method, the returned image is what will be shown on GUI
    fn representation(&self, key: u8, button: &UniqueButton, core_handle: &CoreHandle) -> Option<DynamicImage> { None }

    /// Called when renderer component has custom renderer selected, can be used to give custom fields to renderer component
    fn component_values(&self, button: &Button, component: &RendererComponent, core_handle: &CoreHandle) -> Vec<UIValue> { vec![] }

    /// Called when renderer component has custom renderer selected, used to set custom fields to whatever structure plugin wishes
    fn set_component_value(&self, button: &mut Button, component: &mut RendererComponent, core_handle: &CoreHandle, value: Vec<UIValue>) { }
}

/// Reference counted renderer object
pub type UniqueRenderer = Arc<dyn CustomRenderer>;

/// Manager that keeps a bunch of related things to rendering thread
#[derive(Default)]
pub struct RenderingManager {
    renderers: RwLock<HashMap<String, UniqueRenderer>>
}

impl RenderingManager {
    /// Creates new rendering manager
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Adds renderer to the manager
    pub fn add_custom_renderer(&self, renderer: UniqueRenderer) {
        let mut lock = self.renderers.write().unwrap();
        lock.insert(renderer.name(), renderer);
    }

    /// Returns all renderers managed by the manager
    pub fn get_renderers(&self) -> HashMap<String, UniqueRenderer> {
        self.renderers.read().unwrap().clone()
    }

    /// Returns read lock for renderers
    pub fn read_renderers(&self) -> RwLockReadGuard<HashMap<String, UniqueRenderer>> {
        self.renderers.read().unwrap()
    }
}