//! Rendering functions that represent default Streamduck renderer

pub mod custom;
pub mod component_values;

use std::hash::{Hash, Hasher};
use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
use rusttype::Scale;
use image::imageops::{FilterType, tile};
use streamdeck::{DeviceImage, ImageMode, StreamDeck};
use std::collections::HashMap;
use std::sync::Arc;
use std::collections::hash_map::DefaultHasher;
use std::io::Cursor;
use std::time::Instant;
use std::ops::Deref;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use crate::core::button::Component;
use crate::core::{CoreHandle, UniqueButton};
use crate::font::get_font_from_collection;
use crate::images::{AnimationFrame, SDImage};
use crate::modules::UniqueSDModule;
use crate::thread::rendering::custom::DeviceReference;
use crate::thread::util::{image_from_horiz_gradient, image_from_solid, image_from_vert_gradient, render_aligned_shadowed_text_on_image, render_aligned_text_on_image, TextAlignment};
use crate::util::hash_value;

/// Animation counter that counts frames for animated images
pub struct AnimationCounter {
    frames: Vec<(AnimationFrame, f32)>,
    time: Instant,
    wakeup_time: f32,
    index: usize,
    duration: f32,
    new_frame: bool,
}

impl AnimationCounter {
    fn new(frames: Vec<AnimationFrame>) -> AnimationCounter {
        let mut time_counter = 0.0;
        let frames: Vec<(AnimationFrame, f32)> = frames.into_iter()
            .map(|x| {
                let end_time = time_counter + x.delay;
                time_counter = end_time;
                (x, end_time)
            })
            .collect();

        let duration = time_counter;

        AnimationCounter {
            frames,
            time: Instant::now(),
            wakeup_time: 0.0,
            index: 0,
            duration,
            new_frame: false
        }
    }

    fn get_frame(&self) -> &AnimationFrame {
        &self.frames[self.index].0
    }

    fn advance_counter(&mut self) {
        let time = self.time.elapsed().as_secs_f32();

        if time > self.wakeup_time {
            let looped_time = time % self.duration;
            for i in 0..self.frames.len() {
                if looped_time < self.frames[i].1 {
                    self.index = i;
                    self.new_frame = true;
                    self.wakeup_time = time + self.frames[i].0.delay;
                    break;
                }
            }
        }
    }
}

/// Rendering code that's being called every loop
pub fn process_frame(
    core: &CoreHandle,
    streamdeck: &mut StreamDeck,
    cache: &mut HashMap<u64, (Arc<DeviceImage>, u64)>,
    counters: &mut HashMap<String, AnimationCounter>,
    renderer_map: &mut HashMap<u8, (RendererComponent, UniqueButton, Vec<UniqueSDModule>)>,
    previous_state: &mut HashMap<u8, u64>,
    missing: &DynamicImage,
    time: u64
) {

    for key in 0..core.core.key_count {
        if let Some((component, button, modules)) = renderer_map.get(&key) {
            if !component.renderer.is_empty() {
                // Custom renderer detected
                let lock = core.core.render_manager.read_renderers();

                if let Some(renderer) = lock.get(&component.renderer) {
                    // Stopping any further process if custom renderer is found
                    renderer.render(key, button, core, &mut DeviceReference::new(streamdeck, key));
                    previous_state.insert(key, 1);
                    continue;
                }
            }


            if let ButtonBackground::ExistingImage(identifier) = &component.background {
                let counter = if let Some(counter) = counters.get_mut(identifier) {
                    Some(counter)
                } else {
                    if let Some(SDImage::AnimatedImage(frames)) = core.core.image_collection.read().unwrap().get(identifier).cloned() {
                        let counter = AnimationCounter::new(frames);
                        counters.insert(identifier.clone(), counter);
                        Some(counters.get_mut(identifier).unwrap())
                    } else {
                        None
                    }
                };

                if let Some(counter) = counter {
                    let frame = counter.get_frame();

                    let mut hasher: Box<dyn Hasher> = Box::new(DefaultHasher::new());

                    component.hash(&mut hasher);
                    frame.index.hash(&mut hasher);

                    for module in modules {
                        module.render_hash(core.clone_for(module), &button, &mut hasher);
                    }

                    let hash = hasher.finish();

                    if counter.new_frame || (hash != *previous_state.get(&key).unwrap_or(&1)) {
                        let variant = cache.get_mut(&hash);

                        if component.to_cache && variant.is_some() {
                            let (variant, time_to_die) = variant.unwrap();
                            *time_to_die = time + 20000;

                            let previous = previous_state.get(&key).unwrap_or(&1);
                            if hash != *previous {
                                streamdeck.write_button_image(key, variant.deref()).ok();
                            }

                        } else {
                            let mut buffer = vec![];

                            draw_foreground(&component, &button, modules,frame.image.clone(), core).rotate180().write_to(&mut Cursor::new(&mut buffer), match core.core.kind.image_mode() {
                                ImageMode::Bmp => ImageFormat::Bmp,
                                ImageMode::Jpeg => ImageFormat::Jpeg,
                            }).ok();

                            let arc = Arc::new(DeviceImage::from(buffer));

                            if component.to_cache {
                                cache.insert(hash, (arc.clone(), time + 20000));
                            }

                            streamdeck.write_button_image(key, arc.deref()).ok();
                        }

                        previous_state.insert(key, hash);
                    }

                    // Skipping anything else if we already processed an animated image
                    continue;
                }
            }

            // If not animated, continuing with normal process of rendering a button
            let mut hasher: Box<dyn Hasher> = Box::new(DefaultHasher::new());

            component.hash(&mut hasher);
            for module in modules {
                module.render_hash(core.clone_for(module), &button, &mut hasher);
            }

            let hash = hasher.finish();

            let variant = cache.get_mut(&hash);

            if component.to_cache && variant.is_some() {
                let (variant, time_to_die) = variant.unwrap();
                *time_to_die = time + 20000;

                let previous = previous_state.get(&key).unwrap_or(&1);
                if hash != *previous {
                    streamdeck.write_button_image(key, variant.deref()).ok();
                }
            } else {
                let mut buffer = vec![];

                draw_foreground(&component, &button, modules, draw_background(component, core, missing), core).rotate180().write_to(&mut Cursor::new(&mut buffer), match core.core.kind.image_mode() {
                    ImageMode::Bmp => ImageFormat::Bmp,
                    ImageMode::Jpeg => ImageFormat::Jpeg,
                }).ok();

                let arc = Arc::new(DeviceImage::from(buffer));

                if component.to_cache {
                    cache.insert(hash, (arc.clone(), time + 20000));
                }

                streamdeck.write_button_image(key, arc.deref()).ok();
            }

            previous_state.insert(key, hash);
        } else {
            let previous = previous_state.get(&key).unwrap_or(&1);

            if *previous != 0 {
                previous_state.insert(key, 0);
                streamdeck.set_button_rgb(key, &streamdeck::Colour {
                    r: 0,
                    g: 0,
                    b: 0
                }).ok();
            }
        }
    }

    for (_, counter) in counters {
        counter.new_frame = false;
        counter.advance_counter()
    };
}

/// Draws background for static images
pub fn draw_background(renderer: &RendererComponent, core: &CoreHandle, missing: &DynamicImage) -> DynamicImage {
    match &renderer.background {
        ButtonBackground::Solid(color) => {
            image_from_solid(core.core.image_size, Rgba([color.0, color.1, color.2, 255]))
        }

        ButtonBackground::HorizontalGradient(start, end) => {
            image_from_horiz_gradient(core.core.image_size, Rgba([start.0, start.1, start.2, 255]), Rgba([end.0, end.1, end.2, 255]))
        }

        ButtonBackground::VerticalGradient(start, end) => {
            image_from_vert_gradient(core.core.image_size, Rgba([start.0, start.1, start.2, 255]), Rgba([end.0, end.1, end.2, 255]))
        }

        ButtonBackground::ExistingImage(identifier) => {
            if let Some(image) = core.core.image_collection.read().unwrap().get(identifier) {
                match image {
                    SDImage::SingleImage(image) => {
                        image.resize_to_fill(core.core.image_size.0 as u32, core.core.image_size.1 as u32, FilterType::Triangle)
                    }

                    SDImage::AnimatedImage(frames) => {
                        frames[0].image.clone().resize_to_fill(core.core.image_size.0 as u32, core.core.image_size.1 as u32, FilterType::Triangle)
                    }
                }
            } else {
                missing.clone()
            }
        }

        ButtonBackground::NewImage(blob) => {
            if let Ok(image) = SDImage::from_base64(blob, core.core.image_size) {
                image.get_image()
            } else {
                missing.clone()
            }
        }
    }
}

/// Draws foreground of a button (text, plugin layers)
pub fn draw_foreground(renderer: &RendererComponent, button: &UniqueButton, modules: &Vec<UniqueSDModule>, mut background: DynamicImage, core: &CoreHandle) -> DynamicImage {
    // Render any additional things plugins want displayed
    for module in modules {
        module.render(core.clone_for(module), button, &mut background);
    }


    for button_text in &renderer.text {
        let text = button_text.text.as_str();
        let scale = Scale { x: button_text.scale.0, y: button_text.scale.1 };
        let align = button_text.alignment.clone();
        let padding = button_text.padding;
        let offset = button_text.offset.clone();
        let color = button_text.color.clone();

        if let Some(font) = get_font_from_collection(&button_text.font) {
            if let Some(shadow) = &button_text.shadow {
                render_aligned_shadowed_text_on_image(
                    core.core.image_size,
                    &mut background,
                    font.as_ref(),
                    text,
                    scale,
                    align,
                    padding,
                    offset,
                    color,
                    shadow.offset.clone(),
                    shadow.color.clone(),
                )
            } else {
                render_aligned_text_on_image(
                    core.core.image_size,
                    &mut background,
                    font.as_ref(),
                    text,
                    scale,
                    align,
                    padding,
                    offset,
                    color,
                )
            }
        }
    }

    background
}

/// Draws missing texture from HL2
pub fn draw_missing_texture(size: (usize, usize)) -> DynamicImage {
    let mut pattern = RgbaImage::new(16, 16);

    for x in 0..16 {
        for y in 0..16 {
            let color = if y < 8 {
                if x < 8 {
                    Rgba([255, 0, 255, 255])
                } else {
                    Rgba([0, 0, 0, 255])
                }
            } else {
                if x >= 8 {
                    Rgba([255, 0, 255, 255])
                } else {
                    Rgba([0, 0, 0, 255])
                }
            };

            pattern.put_pixel(x, y, color);
        }
    }

    let (iw, ih) = size;
    let mut frame = RgbaImage::new(iw as u32, ih as u32);

    tile(&mut frame, &pattern);

    let mut missing = DynamicImage::ImageRgba8(frame);

    if let Some(font) = get_font_from_collection("default") {
        render_aligned_shadowed_text_on_image(
            (iw, ih),
            &mut missing,
            &font,
            "ГДЕ",
            Scale { x: 30.0, y: 30.0 },
            TextAlignment::Center,
            0,
            (0.0, -13.0),
            (255, 0, 255, 255),
            (2, 2),
            (0, 0, 0, 255),
        );

        render_aligned_shadowed_text_on_image(
            (iw, ih),
            &mut missing,
            &font,
            "Where",
            Scale { x: 25.0, y: 25.0 },
            TextAlignment::Center,
            0,
            (0.0, 8.0),
            (255, 0, 255, 255),
            (1, 1),
            (0, 0, 0, 255),
        );
    }

    missing
}

/// Draws texture that says "Custom Renderer"
pub fn draw_custom_renderer_texture(size: (usize, usize)) -> DynamicImage {
    let font = get_font_from_collection("default").unwrap();
    let mut frame = image_from_solid(size, Rgba([55, 55, 55, 255]));

    render_aligned_text_on_image(size, &mut frame, font.deref(), "Custom", Scale::uniform(16.0), TextAlignment::Center, 0, (0.0, -8.0), (255, 255, 255, 255));
    render_aligned_text_on_image(size, &mut frame, font.deref(), "Renderer", Scale::uniform(16.0), TextAlignment::Center, 0, (0.0, 8.0), (255, 255, 255, 255));

    frame
}

/// Definition for color format
pub type Color = (u8, u8, u8, u8);

/// Button Background definition for button renderer
#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub enum ButtonBackground {
    Solid(Color),
    HorizontalGradient(Color, Color),
    VerticalGradient(Color, Color),
    ExistingImage(String),
    NewImage(String),
}

impl Default for ButtonBackground {
    fn default() -> Self {
        Self::Solid((0, 0, 0, 0))
    }
}

/// Button Text definition for button renderer
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ButtonText {
    pub text: String,
    pub font: String,
    pub scale: (f32, f32),
    pub alignment: TextAlignment,
    pub padding: u32,
    pub offset: (f32, f32),
    pub color: Color,
    pub shadow: Option<ButtonTextShadow>,
}

impl Hash for ButtonText {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.text.hash(state);
        self.font.hash(state);
        ((self.scale.0 * 100.0) as i32).hash(state);
        ((self.scale.1 * 100.0) as i32).hash(state);
        self.alignment.hash(state);
        self.padding.hash(state);
        ((self.offset.0 * 100.0) as i32).hash(state);
        ((self.offset.1 * 100.0) as i32).hash(state);
        self.color.hash(state);
        self.shadow.hash(state);
    }
}

/// Button text shadow
#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct ButtonTextShadow {
    pub offset: (i32, i32),
    pub color: Color,
}

/// Renderer component that contains button background and array of text structs
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RendererComponent {
    /// Uses default renderer if empty
    #[serde(default)]
    pub renderer: String,
    #[serde(default)]
    pub background: ButtonBackground,
    #[serde(default)]
    pub text: Vec<ButtonText>,
    #[serde(default)]
    pub plugin_blacklist: Vec<String>,
    #[serde(default = "make_true")]
    pub to_cache: bool,
    /// Anything that custom renderers might want to remember
    #[serde(default)]
    pub custom_data: Value,
}

fn make_true() -> bool { true }

impl Default for RendererComponent {
    fn default() -> Self {
        Self {
            renderer: "".to_string(),
            background: ButtonBackground::Solid((255, 255, 255, 255)),
            text: vec![],
            plugin_blacklist: vec![],
            to_cache: true,
            custom_data: Default::default()
        }
    }
}

impl Hash for RendererComponent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.renderer.hash(state);
        self.plugin_blacklist.hash(state);
        self.text.hash(state);
        self.to_cache.hash(state);
        self.background.hash(state);
        hash_value(&self.custom_data, state);
    }
}

impl Component for RendererComponent {
    const NAME: &'static str = "renderer";
}

/// Builder for renderer component
#[derive(Default)]
pub struct RendererComponentBuilder {
    component: RendererComponent
}

impl RendererComponentBuilder {
    /// Creates new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets custom renderer
    pub fn renderer(mut self, renderer: &str) -> Self {
        self.component.renderer = renderer.to_string(); self
    }

    /// Sets background
    pub fn background(mut self, background: ButtonBackground) -> Self {
        self.component.background = background; self
    }

    /// Adds a text object
    pub fn add_text(mut self, text: ButtonText) -> Self {
        self.component.text.push(text); self
    }

    /// Adds a plugin to rendering blacklist for the component
    pub fn add_to_blacklist(mut self, plugin: &str) -> Self {
        self.component.plugin_blacklist.push(plugin.to_string()); self
    }

    /// Sets caching state
    pub fn caching(mut self, cache: bool) -> Self {
        self.component.to_cache = cache; self
    }

    /// Builds the component
    pub fn build(self) -> RendererComponent {
        self.into()
    }
}

impl From<RendererComponentBuilder> for RendererComponent {
    fn from(builder: RendererComponentBuilder) -> Self {
        builder.component
    }
}

/// Renderer settings
#[derive(Serialize, Deserialize, Default)]
pub struct RendererSettings {
    /// Blacklist of plugins that aren't allowed to render
    pub plugin_blacklist: Vec<String>
}

#[allow(dead_code)]
pub(crate) fn hash_renderer(renderer: &RendererComponent) -> u64 {
    let mut hasher = DefaultHasher::new();
    renderer.hash(&mut hasher);
    hasher.finish()
}
