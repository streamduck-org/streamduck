//! Device Thread
//!
//! A separate thread for processing, rendering images on streamdeck and reading buttons

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io::{Cursor};
use std::ops::Deref;
use serde::{Serialize, Deserialize};
use std::sync::{Arc, RwLock};
use std::sync::mpsc::{channel, Sender, TryRecvError};
use std::thread::{sleep, spawn};
use std::time::{Duration, Instant};
use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
use image::imageops::{FilterType, tile};
use rusttype::Scale;
use streamdeck::{Colour, DeviceImage, ImageMode, StreamDeck};
use crate::core::{SDCore, UniqueButton};
use crate::core::button::{Component, parse_unique_button_to_component};
use crate::core::methods::{CoreHandle, get_current_screen};
use crate::font::get_font_from_collection;
use crate::images::{AnimationFrame, SDImage};
use crate::modules::UniqueSDModule;
use crate::util::rendering::{image_from_horiz_gradient, image_from_solid, image_from_vert_gradient, render_aligned_shadowed_text_on_image, render_aligned_text_on_image, TextAlignment};

pub type ImageCollection = Arc<RwLock<HashMap<String, SDImage>>>;

/// Handle for contacting renderer thread
pub struct DeviceThreadHandle {
    tx: Sender<Vec<DeviceThreadCommunication>>
}

impl DeviceThreadHandle {
    /// Sends commands to device thread
    pub fn send(&self, commands: Vec<DeviceThreadCommunication>) {
        self.tx.send(commands).ok();
    }
}

#[allow(dead_code)]
pub enum DeviceThreadCommunication {
    /// Tells renderer that screen should be updated
    RefreshScreen,

    /// Sets streamdeck brightness to provided value
    SetBrightness(u8),

    /// Sets button image to specified image
    SetButtonImage(u8, DynamicImage),

    /// Sets button image to raw buffer of image
    SetButtonImageRaw(u8, Arc<DeviceImage>),

    /// Clears button and sets it to black color
    ClearButtonImage(u8),
}

/// Spawns device thread from a core reference
pub fn spawn_device_thread(core: Arc<SDCore>, streamdeck: StreamDeck, key_tx: Sender<(u8, bool)>) -> DeviceThreadHandle {
    let (tx, rx) = channel::<Vec<DeviceThreadCommunication>>();

    spawn(move || {
        let core = CoreHandle::wrap(core.clone());
        let mut streamdeck = streamdeck;
        let mut last_buttons = Vec::new();

        streamdeck.set_blocking(false).ok();

        let missing = draw_missing_texture(core.core.image_size);

        let mut animation_counters = HashMap::new();
        let mut last_iter = Instant::now();
        let mut renderer_map = HashMap::new();
        let mut animation_cache: HashMap<u64, Arc<DeviceImage>> = HashMap::new();
        let mut previous_state: HashMap<u8, u64> = HashMap::new();
        loop {
            if core.core.is_closed() {
                break;
            }

            // Reading buttons
            match streamdeck.read_buttons(None) {
                Ok(buttons) => {
                    for (key, value) in buttons.iter().enumerate() {
                        if let Some(last_value) = last_buttons.get(key) {
                            if last_value != value {
                                if key_tx.send((key as u8, *last_value == 0)).is_err() {
                                    log::error!("Key Handler thread crashed, killing connection...");
                                    core.core.close();
                                }
                            }
                        } else {
                            if *value > 0 {
                                if key_tx.send((key as u8, true)).is_err() {
                                    log::error!("Key Handler thread crashed, killing connection...");
                                    core.core.close();
                                }
                            }
                        }
                    }
                    last_buttons = buttons;
                }
                Err(err) => {
                    match err {
                        streamdeck::Error::NoData => {}
                        streamdeck::Error::Hid(_) => {
                            log::trace!("hid connection failed");
                            core.core.close()
                        }
                        _ => {
                            panic!("Error on streamdeck thread: {:?}", err);
                        }
                    }
                }
            }

            // Reading commands
            match rx.try_recv() {
                Ok(com) => {
                    for com in com {
                        match com {
                            DeviceThreadCommunication::SetBrightness(brightness) => {
                                streamdeck.set_brightness(brightness).ok();
                            }

                            DeviceThreadCommunication::SetButtonImage(key, image) => {
                                let mut buffer = vec![];

                                image.write_to(&mut Cursor::new(&mut buffer), match streamdeck.kind().image_mode() {
                                    ImageMode::Bmp => ImageFormat::Bmp,
                                    ImageMode::Jpeg => ImageFormat::Jpeg,
                                }).ok();

                                streamdeck.write_button_image(key, &DeviceImage::from(buffer)).ok();
                            }

                            DeviceThreadCommunication::SetButtonImageRaw(key, image) => {
                                streamdeck.write_button_image(key, image.deref()).ok();
                            }

                            DeviceThreadCommunication::ClearButtonImage(key) => {
                                streamdeck.set_button_rgb(key, &Colour {
                                    r: 0,
                                    g: 0,
                                    b: 0
                                }).ok();
                            }

                            DeviceThreadCommunication::RefreshScreen => {
                                let current_screen = get_current_screen(&core);

                                if current_screen.is_none() {
                                    return;
                                }

                                let current_screen = current_screen.unwrap();
                                let screen_handle = current_screen.read().unwrap();
                                let current_screen = screen_handle.buttons.clone();
                                drop(screen_handle);

                                renderer_map.clear();
                                renderer_map.extend(
                                    current_screen.into_iter()
                                        .filter(|(_, button)| button.read().unwrap().0.contains_key(RendererComponent::NAME))
                                        .map(|(key, x)| {
                                            let names = x.read().unwrap().component_names();
                                            let modules = core.module_manager().get_modules_for_rendering(&names);

                                            (key, (parse_unique_button_to_component::<RendererComponent>(&x).unwrap(), x, modules.into_values().collect::<Vec<UniqueSDModule>>()))
                                        })
                                )
                            }
                        }
                    }
                }
                Err(err) => {
                    match err {
                        TryRecvError::Empty => {}
                        TryRecvError::Disconnected => break,
                    }
                }
            }

            process_animations(&core, &mut streamdeck, &mut animation_cache, &mut animation_counters, &mut renderer_map, &mut previous_state, &missing);

            // Rate limiter
            let rate = 1.0 / core.core.pool_rate as f32;
            let time_since_last = last_iter.elapsed().as_secs_f32();

            let to_wait = rate - time_since_last;
            if to_wait > 0.0 {
                sleep(Duration::from_secs_f32(to_wait));
            }

            last_iter = Instant::now();
        }

        log::trace!("rendering closed");
    });

    DeviceThreadHandle {
        tx
    }
}

struct AnimationCounter {
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

fn process_animations(
    core: &CoreHandle,
    streamdeck: &mut StreamDeck,
    cache: &mut HashMap<u64, Arc<DeviceImage>>,
    counters: &mut HashMap<String, AnimationCounter>,
    renderer_map: &mut HashMap<u8, (RendererComponent, UniqueButton, Vec<UniqueSDModule>)>,
    previous_state: &mut HashMap<u8, u64>,
    missing: &DynamicImage
) {

    for key in 0..core.core.key_count {
        if let Some((component, button, modules)) = renderer_map.get(&key) {
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
                    if counter.new_frame {
                        let frame = counter.get_frame();

                        let mut hasher: Box<dyn Hasher> = Box::new(DefaultHasher::new());

                        component.hash(&mut hasher);
                        frame.index.hash(&mut hasher);

                        for module in modules {
                            module.render_hash(core.clone_for(module), &button, &mut hasher);
                        }

                        let hash = hasher.finish();

                        let variant = cache.get(&hash);

                        if component.to_cache && variant.is_some() {
                            let previous = previous_state.get(&key).unwrap_or(&0);
                            if hash != *previous {
                                streamdeck.write_button_image(key, variant.unwrap().deref()).ok();
                            }
                        } else {
                            let mut buffer = vec![];

                            draw_foreground(&component, &button, modules,frame.image.clone(), core).rotate180().write_to(&mut Cursor::new(&mut buffer), match core.core.kind.image_mode() {
                                ImageMode::Bmp => ImageFormat::Bmp,
                                ImageMode::Jpeg => ImageFormat::Jpeg,
                            }).ok();

                            let arc = Arc::new(DeviceImage::from(buffer));

                            if component.to_cache {
                                cache.insert(hash, arc.clone());
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

            let variant = cache.get(&hash);

            if component.to_cache && variant.is_some() {
                let previous = previous_state.get(&key).unwrap_or(&0);
                if hash != *previous {
                    streamdeck.write_button_image(key, variant.unwrap().deref()).ok();
                }
            } else {
                let mut buffer = vec![];

                draw_foreground(&component, &button, modules, draw_background(component, core, missing), core).rotate180().write_to(&mut Cursor::new(&mut buffer), match core.core.kind.image_mode() {
                    ImageMode::Bmp => ImageFormat::Bmp,
                    ImageMode::Jpeg => ImageFormat::Jpeg,
                }).ok();

                let arc = Arc::new(DeviceImage::from(buffer));

                if component.to_cache {
                    cache.insert(hash, arc.clone());
                }

                streamdeck.write_button_image(key, arc.deref()).ok();
            }

            previous_state.insert(key, hash);
        } else {
            let previous = previous_state.get(&key).unwrap_or(&0);

            if *previous != 0 {
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
#[derive(Serialize, Deserialize, Clone, Hash, Debug)]
pub struct RendererComponent {
    #[serde(default)]
    pub background: ButtonBackground,
    #[serde(default)]
    pub text: Vec<ButtonText>,
    #[serde(default = "make_true")]
    pub to_cache: bool,
}

fn make_true() -> bool { true }

impl Default for RendererComponent {
    fn default() -> Self {
        Self {
            background: ButtonBackground::Solid((255, 255, 255, 255)),
            text: vec![],
            to_cache: true,
        }
    }
}

impl Component for RendererComponent {
    const NAME: &'static str = "renderer";
}

#[allow(dead_code)]
pub(crate) fn hash_renderer(renderer: &RendererComponent) -> u64 {
    let mut hasher = DefaultHasher::new();
    renderer.hash(&mut hasher);
    hasher.finish()
}