//! Device Thread
//!
//! A separate thread for processing, rendering images on streamdeck and reading buttons

use std::collections::HashMap;
use std::io::Cursor;
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::{channel, Sender, TryRecvError};
use std::thread::spawn;
use std::time::{Duration, Instant};
use image::{DynamicImage, ImageFormat};
use streamdeck::{Colour, DeviceImage, ImageMode, StreamDeck};
use rendering::RendererComponent;
use crate::core::{CoreHandle, SDCore};
use crate::core::button::{Component, parse_unique_button_to_component};
use crate::images::SDImage;
use crate::modules::core_module::CoreSettings;
use crate::modules::UniqueSDModule;

/// Rendering utilities
pub mod util;
pub mod rendering;

/// Collection of images
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

/// Enum of various operations that can be sent to device thread
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

        let missing = rendering::draw_missing_texture(core.core.image_size);

        let mut animation_counters = HashMap::new();
        let mut last_iter = Instant::now();
        let mut renderer_map = HashMap::new();
        let mut animation_cache: HashMap<u64, (Arc<DeviceImage>, u64)> = HashMap::new();
        let mut previous_state: HashMap<u8, u64> = HashMap::new();
        let mut time = 0;
        let mut last_time = time;
        loop {
            if core.core.is_closed() {
                break;
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
                                let current_screen = core.get_current_screen();

                                if current_screen.is_none() {
                                    return;
                                }

                                let current_screen = current_screen.unwrap();
                                let screen_handle = current_screen.read().unwrap();
                                let current_screen = screen_handle.buttons.clone();
                                drop(screen_handle);

                                let core_settings: CoreSettings = core.config().get_plugin_settings().unwrap_or_default();

                                renderer_map.clear();
                                renderer_map.extend(
                                    current_screen.into_iter()
                                        .filter(|(_, button)| button.read().unwrap().0.contains_key(RendererComponent::NAME))
                                        .map(|(key, x)| {
                                            let names = x.read().unwrap().component_names();
                                            let mut modules = core.module_manager().get_modules_for_rendering(&names);

                                            let component = parse_unique_button_to_component::<RendererComponent>(&x).unwrap();

                                            modules.retain(|x, _| !component.plugin_blacklist.contains(x));
                                            modules.retain(|x, _| !core_settings.renderer.plugin_blacklist.contains(x));

                                            (key, (component, x, modules.into_values().collect::<Vec<UniqueSDModule>>()))
                                        })
                                );

                                core.core.render_manager.read_renderers().iter()
                                    .for_each(|(_, renderer)| renderer.refresh(&core));
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

            rendering::process_frame(&core, &mut streamdeck, &mut animation_cache, &mut animation_counters, &mut renderer_map, &mut previous_state, &missing, time);
            time += 1;

            // Occasionally cleaning cache
            if time % 3000 == 0 && time != last_time {
                animation_cache.retain(|_, (_, t)| *t > time);
            }

            last_time = time;

            // Rate limiter
            let rate = 1.0 / core.core.pool_rate as f32;
            let time_since_last = last_iter.elapsed().as_secs_f32();
            let to_wait = match rate - time_since_last {
                n if n < 0.0 => None,
                n => Some(Duration::from_secs_f32(n)),
            };

            // Reading buttons
            match streamdeck.read_buttons(to_wait) {
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

            last_iter = Instant::now();
        }

        log::trace!("rendering closed");
    });

    DeviceThreadHandle {
        tx
    }
}
