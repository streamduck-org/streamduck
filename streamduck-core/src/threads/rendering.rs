//! Rendering thread
//!
//! A separate thread for processing and rendering images on streamdeck

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
use image::io::Reader;
use rusttype::Scale;
use streamdeck::{Colour, DeviceImage, ImageMode, StreamDeck};
use crate::core::{SDCore};
use crate::core::button::{Component, parse_unique_button_to_component};
use crate::core::methods::{CoreHandle, get_current_screen};
use crate::font::get_font_from_collection;
use crate::images::{AnimationFrame, convert_image, SDImage};
use crate::util::rendering::{image_from_horiz_gradient, image_from_solid, image_from_vert_gradient, render_aligned_shadowed_text_on_image, render_aligned_text_on_image, TextAlignment};

pub type ImageCollection = Arc<RwLock<HashMap<String, SDImage>>>;

/// Handle for contacting renderer thread
pub struct DeviceThreadHandle {
    tx: Sender<Vec<DeviceThreadCommunication>>,
    pub state: Arc<RendererState>,
}

impl DeviceThreadHandle {
    /// Asks rendering thread to redraw current screen
    pub fn redraw(&self) {
        self.tx.send(vec![DeviceThreadCommunication::Redraw]).ok();
    }

    pub fn send(&self, commands: Vec<DeviceThreadCommunication>) {
        self.tx.send(commands).ok();
    }
}

#[allow(dead_code)]
pub enum DeviceThreadCommunication {
    /// Triggers redraw
    Redraw,

    /// Sets streamdeck brightness to provided value
    SetBrightness(u8),

    /// Sets button image to specified image
    SetButtonImage(u8, DynamicImage),

    /// Sets button image to raw buffer of image
    SetButtonImageRaw(u8, Arc<DeviceImage>),

    /// Clears button and sets it to black color
    ClearButtonImage(u8),
}

pub struct RendererState {
    pub render_cache: RwLock<HashMap<u64, Arc<DeviceImage>>>,
    pub current_images: RwLock<HashMap<u8, DynamicImage>>,
}

/// Spawns device thread from a core reference
pub fn spawn_device_thread(core: Arc<SDCore>, streamdeck: StreamDeck, key_tx: Sender<(u8, bool)>) -> DeviceThreadHandle {
    let (tx, rx) = channel::<Vec<DeviceThreadCommunication>>();

    let state = Arc::new(RendererState {
        render_cache: Default::default(),
        current_images: Default::default(),
    });

    let renderer_state = state.clone();
    spawn(move || {
        let core = core.clone();
        let mut streamdeck = streamdeck;
        let mut last_buttons = Vec::new();

        streamdeck.set_blocking(false).ok();

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

        let (iw, ih) = core.image_size;
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

        let mut animation_counters = HashMap::new();
        let mut last_iter = Instant::now();
        let mut renderer_map = HashMap::new();
        loop {
            if core.is_closed() {
                break;
            }

            // Reading buttons
            match streamdeck.read_buttons(None) {
                Ok(buttons) => {
                    for (key, value) in buttons.iter().enumerate() {
                        if let Some(last_value) = last_buttons.get(key) {
                            if last_value != value {
                                key_tx.send((key as u8, *last_value == 0)).ok();
                            }
                        } else {
                            if *value > 0 {
                                key_tx.send((key as u8, true)).ok();
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
                            core.close()
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
                            DeviceThreadCommunication::Redraw => redraw(core.clone(), &mut streamdeck, &renderer_state, &missing, &mut renderer_map, &mut animation_counters),

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

            process_animations(&core, &mut streamdeck, &renderer_state, &mut animation_counters, &mut renderer_map);

            // Rate limiter
            let rate = 1.0 / core.pool_rate as f32;
            let time_since_last = last_iter.elapsed().as_secs_f32();

            let to_wait = rate - time_since_last;
            if to_wait > 0.0 {
                sleep(Duration::from_secs_f32(to_wait));
            }

            last_iter = Instant::now();
        }

        log::trace!("rendering closed");
    });

    tx.send(vec![DeviceThreadCommunication::Redraw]).ok();

    DeviceThreadHandle {
        tx,
        state,
    }
}

struct AnimationCounter {
    frames: Vec<AnimationFrame>,
    current_time: Instant,
    index: usize,
    advanced: bool,
}

impl AnimationCounter {
    fn new(frames: Vec<AnimationFrame>) -> AnimationCounter {
        AnimationCounter {
            frames,
            current_time: Instant::now(),
            index: 0,
            advanced: false
        }
    }

    fn get_frame(&self) -> &AnimationFrame {
        &self.frames[self.index]
    }

    fn next_frame(&mut self) {
        if self.index < self.frames.len() - 1 {
            self.index += 1;
        } else {
            self.index = 0;
        }
    }

    fn advance_counter(&mut self) {
        let mut missing_time = self.current_time.elapsed().as_secs_f32();

        if missing_time > self.get_frame().delay {
            while missing_time > self.get_frame().delay {
                self.next_frame();
                missing_time -= self.get_frame().delay;
            }
            self.advanced = true;
            self.current_time = Instant::now();
        }
    }
}

fn process_animations(core: &Arc<SDCore>, streamdeck: &mut StreamDeck, state: &Arc<RendererState>, counters: &mut HashMap<String, AnimationCounter>, renderer_map: &mut HashMap<u8, RendererComponent>) {
    let mut cache = state.render_cache.write().unwrap();

    for (key, component) in renderer_map {
        if let ButtonBackground::ExistingImage(identifier) = &component.background {
            let counter = if let Some(counter) = counters.get_mut(identifier) {
                Some(counter)
            } else {
                if let Some(SDImage::AnimatedImage(frames)) = core.image_collection.read().unwrap().get(identifier).cloned() {
                    let counter = AnimationCounter::new(frames);
                    counters.insert(identifier.clone(), counter);
                    Some(counters.get_mut(identifier).unwrap())
                } else {
                    None
                }
            };

            if let Some(counter) = counter {
                if counter.advanced {
                    let frame = counter.get_frame();

                    let mut hasher = DefaultHasher::new();
                    component.hash(&mut hasher);
                    frame.index.hash(&mut hasher);
                    let hash = hasher.finish();

                    if let Some(image) = cache.get(&hash) {
                        streamdeck.write_button_image(*key, image.deref()).ok();
                    } else {
                        let mut buffer = vec![];

                        draw_button(&component, frame.image.clone(), core).rotate180().write_to(&mut Cursor::new(&mut buffer), match core.kind.image_mode() {
                            ImageMode::Bmp => ImageFormat::Bmp,
                            ImageMode::Jpeg => ImageFormat::Jpeg,
                        }).ok();

                        let arc = Arc::new(DeviceImage::from(buffer));

                        cache.insert(hash, arc.clone());
                        streamdeck.write_button_image(*key, arc.deref()).ok();
                    }
                }
            }
        }
    }

    for (_, counter) in counters {
        counter.advanced = false;
        counter.advance_counter()
    };
}

fn draw_background(renderer: &RendererComponent, core: &Arc<SDCore>, missing: &DynamicImage, counters: &mut HashMap<String, AnimationCounter>) -> DynamicImage {
    match &renderer.background {
        ButtonBackground::Solid(color) => {
            image_from_solid(core.image_size, Rgba([color.0, color.1, color.2, 255]))
        }

        ButtonBackground::HorizontalGradient(start, end) => {
            image_from_horiz_gradient(core.image_size, Rgba([start.0, start.1, start.2, 255]), Rgba([end.0, end.1, end.2, 255]))
        }

        ButtonBackground::VerticalGradient(start, end) => {
            image_from_vert_gradient(core.image_size, Rgba([start.0, start.1, start.2, 255]), Rgba([end.0, end.1, end.2, 255]))
        }

        ButtonBackground::ExistingImage(identifier) => {
            if let Some(image) = core.image_collection.read().unwrap().get(identifier) {
                match image {
                    SDImage::SingleImage(image) => {
                        image.resize_to_fill(core.image_size.0 as u32, core.image_size.1 as u32, FilterType::Triangle)
                    }

                    SDImage::AnimatedImage(_) => {
                        if let Some(counter) = counters.get(identifier) {
                            counter.get_frame().image.clone()
                        } else {
                            image_from_solid(core.image_size, Rgba([0, 0, 0, 0]))
                        }
                    }
                }
            } else {
                missing.clone()
            }
        }

        ButtonBackground::NewImage(blob) => {
            fn get_image(blob: String) -> Option<DynamicImage> {
                if let Ok(byte_array) = base64::decode(blob) {
                    if let Ok(recognized_image) = Reader::new(Cursor::new(byte_array)).with_guessed_format() {
                        if let Ok(decoded_image) = recognized_image.decode() {
                            return Some(decoded_image);
                        }
                    }
                }

                None
            }

            if let Some(image) = get_image(blob.to_string()) {
                image.resize_to_fill(core.image_size.0 as u32, core.image_size.1 as u32, FilterType::Triangle)
            } else {
                missing.clone()
            }
        }
    }
}

fn draw_button(renderer: &RendererComponent, mut background: DynamicImage, core: &Arc<SDCore>) -> DynamicImage {
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
                    core.image_size,
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
                    core.image_size,
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

fn redraw(core: Arc<SDCore>, streamdeck: &mut StreamDeck, state: &RendererState, missing: &DynamicImage, renderer_map: &mut HashMap<u8, RendererComponent>, counters: &mut HashMap<String, AnimationCounter>) {
    let core_handle = CoreHandle::wrap(core.clone());
    let current_screen = get_current_screen(&core_handle);

    if current_screen.is_none() {
        return;
    }

    let current_screen = current_screen.unwrap();
    let screen_handle = current_screen.read().unwrap();
    let current_screen = screen_handle.buttons.clone();
    drop(screen_handle);

    let mut current_images = HashMap::new();

    for i in 0..core.key_count {
        if let Some(button) = current_screen.get(&i) {
            if let Ok(component) = parse_unique_button_to_component::<RendererComponent>(&button) {
                renderer_map.insert(i, component.clone());

                let image = draw_button(&component, draw_background(&component, &core, missing, counters), &core);
                current_images.insert(i, image.clone());
                streamdeck.write_button_image(i, &convert_image(&streamdeck.kind(), image)).ok();
            } else {
                renderer_map.remove(&i);

                current_images.insert(i, image_from_solid(core.image_size, Rgba([0, 0, 0, 255])));
                streamdeck.set_button_rgb(i, &Colour { r: 0, g: 0, b: 0 }).ok();
            }
        } else {
            renderer_map.remove(&i);

            current_images.insert(i, image_from_solid(core.image_size, Rgba([0, 0, 0, 255])));
            streamdeck.set_button_rgb(i, &Colour { r: 0, g: 0, b: 0 }).ok();
        }
    }

    {
        *state.current_images.write().unwrap() = current_images;
    }
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