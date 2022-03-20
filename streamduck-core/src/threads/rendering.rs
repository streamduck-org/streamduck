//! Rendering thread
//!
//! A separate thread for processing and rendering images on streamdeck

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io::{Cursor};
use std::iter::Cycle;
use serde::{Serialize, Deserialize};
use std::sync::{Arc, RwLock};
use std::sync::mpsc::{channel, Sender, TryRecvError};
use std::thread::{sleep, spawn};
use std::time::{Duration, Instant};
use std::vec::IntoIter;
use image::{DynamicImage, Frame, Rgba, RgbaImage};
use image::imageops::{FilterType, tile};
use image::io::Reader;
use rusttype::Scale;
use crate::core::{SDCore};
use crate::core::button::{Component, parse_unique_button_to_component};
use crate::core::methods::{CoreHandle, get_current_screen};
use crate::font::get_font_from_collection;
use crate::images::SDImage;
use crate::threads::streamdeck::StreamDeckCommand;
use crate::util::rendering::{image_from_horiz_gradient, image_from_solid, image_from_vert_gradient, render_aligned_shadowed_text_on_image, render_aligned_text_on_image, resize_for_streamdeck, TextAlignment};

pub type ImageCollection = Arc<RwLock<HashMap<String, SDImage>>>;

/// Handle for contacting renderer thread
#[derive(Debug)]
pub struct RendererHandle {
    tx: Sender<RendererCommunication>,
    pub state: Arc<RendererState>,
}

impl RendererHandle {
    /// Asks rendering thread to redraw current screen
    pub fn redraw(&self) {
        self.tx.send(RendererCommunication::Redraw).ok();
    }
}

#[allow(dead_code)]
enum RendererCommunication {
    Nothing,
    Redraw,
}

#[derive(Debug)]
pub struct RendererState {
    pub render_cache: RwLock<HashMap<u64, DynamicImage>>,
    pub current_images: RwLock<HashMap<u8, DynamicImage>>,
}

/// Spawns rendering thread from a core reference
pub fn spawn_rendering_thread(core: Arc<SDCore>) -> RendererHandle {
    let (tx, rx) = channel::<RendererCommunication>();

    let state = Arc::new(RendererState {
        render_cache: Default::default(),
        current_images: Default::default(),
    });

    let renderer_state = state.clone();
    spawn(move || {
        let core = core.clone();

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
        let mut delta_time = 0.0;
        loop {
            if core.is_closed() {
                break;
            }

            match rx.try_recv() {
                Ok(com) => {
                    match com {
                        RendererCommunication::Redraw => redraw(core.clone(), &renderer_state, &missing),
                        _ => {}
                    }
                }
                Err(err) => {
                    match err {
                        TryRecvError::Empty => {}
                        TryRecvError::Disconnected => break,
                    }
                }
            }

            process_animations(delta_time, &core, &mut animation_counters);

            // Rate limiter
            let rate = 1.0 / core.frame_rate as f32;
            let time_since_last = last_iter.elapsed().as_secs_f32();

            let to_wait = rate - time_since_last;
            if to_wait > 0.0 {
                sleep(Duration::from_secs_f32(to_wait));
            }

            delta_time = last_iter.elapsed().as_secs_f32();

            last_iter = Instant::now();
        }

        log::trace!("rendering closed");
    });

    tx.send(RendererCommunication::Redraw).ok();

    RendererHandle {
        tx,
        state,
    }
}

#[derive(Clone)]
struct AnimationFrame {
    image: DynamicImage,
    delay: f32,
}

struct AnimationCounter {
    iterator: Cycle<IntoIter<AnimationFrame>>,
    current_frame: Option<AnimationFrame>,
    current_time: f32,
}

impl AnimationCounter {
    fn new(frames: Vec<Frame>, size: (usize, usize)) -> AnimationCounter {
        AnimationCounter {
            iterator: frames.into_iter()
                .map(|x| {
                    let delay = Duration::from(x.delay()).as_secs_f32();
                    AnimationFrame {
                        image: resize_for_streamdeck(size, DynamicImage::ImageRgba8(x.into_buffer())),
                        delay,
                    }
                }).collect::<Vec<AnimationFrame>>()
                .into_iter()
                .cycle(),
            current_frame: None,
            current_time: 0.0
        }
    }

    fn advance_counter(&mut self, delta_time: f32) {
        if let Some(_) = &self.current_frame {
            self.current_time += delta_time;

            while self.current_time > self.current_frame.as_ref().unwrap().delay {
                self.current_frame = self.iterator.next();
                self.current_time -= self.current_frame.as_ref().unwrap().delay;
            }
        } else {
            self.current_frame = self.iterator.next();
            self.current_time = 0.0;
        }
    }

    fn get_frame(&self) -> Option<DynamicImage> {
        self.current_frame.clone().map(|x| x.image)
    }
}

fn process_animations(delta_time: f32, core: &Arc<SDCore>, counters: &mut HashMap<String, AnimationCounter>) {
    let core_handle = CoreHandle::wrap(core.clone());
    let current_screen = get_current_screen(&core_handle);

    if current_screen.is_none() {
        return;
    }

    let current_screen = current_screen.unwrap();
    let screen_handle = current_screen.read().unwrap();
    let current_screen = screen_handle.buttons.clone();
    drop(screen_handle);

    let mut used_counters = vec![];

    let mut commands = vec![];

    for (key, button) in current_screen {
        if let Ok(component) = parse_unique_button_to_component::<RendererComponent>(&button) {
            if let ButtonBackground::ExistingImage(identifier) = &component.background {
                if let Some(SDImage::AnimatedImage(frames)) = core.image_collection.read().unwrap().get(identifier).cloned() {
                    used_counters.push(identifier.clone());

                    let counter = if let Some(counter) = counters.get_mut(identifier) {
                        counter
                    } else {
                        let counter = AnimationCounter::new(frames, core.image_size);
                        counters.insert(identifier.clone(), counter);
                        counters.get_mut(identifier).unwrap()
                    };

                    if let Some(frame) = counter.get_frame() {
                        let frame = draw_button(&component, frame, core);
                        commands.push(StreamDeckCommand::SetButtonImage(key, frame));
                    }
                }
            }
        }
    }

    counters.retain(|x, _| used_counters.contains(x));
    counters.iter_mut().for_each(|(_, counter)| counter.advance_counter(delta_time));

    core.send_commands(commands);
}

fn draw_background(renderer: &RendererComponent, core: &Arc<SDCore>, missing: &DynamicImage) -> DynamicImage {
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
                        image_from_solid(core.image_size, Rgba([0, 0, 0, 0]))
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

fn redraw(core: Arc<SDCore>, state: &RendererState, missing: &DynamicImage) {
    let core_handle = CoreHandle::wrap(core.clone());
    let current_screen = get_current_screen(&core_handle);

    if current_screen.is_none() {
        return;
    }

    let current_screen = current_screen.unwrap();
    let screen_handle = current_screen.read().unwrap();
    let current_screen = screen_handle.buttons.clone();
    drop(screen_handle);

    let mut commands = vec![];

    let mut current_images = HashMap::new();

    for i in 0..core.key_count {
        if let Some(button) = current_screen.get(&i) {
            if let Ok(component) = parse_unique_button_to_component::<RendererComponent>(&button) {
                let image = draw_button(&component, draw_background(&component, &core, missing), &core);
                current_images.insert(i, image.clone());
                commands.push(StreamDeckCommand::SetButtonImage(i, image));
            } else {
                current_images.insert(i, image_from_solid(core.image_size, Rgba([0, 0, 0, 255])));
                commands.push(StreamDeckCommand::ClearButtonImage(i));
            }
        } else {
            current_images.insert(i, image_from_solid(core.image_size, Rgba([0, 0, 0, 255])));
            commands.push(StreamDeckCommand::ClearButtonImage(i));
        }
    }

    {
        *state.current_images.write().unwrap() = current_images;
    }

    core.send_commands(commands);
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