//! Streamdeck connection thread
//!
//! A separate thread for interacting and pooling events from streamdeck
use std::sync::Arc;
use std::sync::mpsc::{channel, Sender, TryRecvError};
use std::thread::{sleep, spawn};
use std::time::{Duration, Instant};
use image::DynamicImage;
use streamdeck::{Colour, StreamDeck};
use crate::core::SDCore;

/// Handle for contacting streamdeck thread
#[derive(Debug)]
pub struct StreamDeckHandle {
    tx: Sender<Vec<StreamDeckCommand>>,
}

impl StreamDeckHandle {
    pub fn send(&self, commands: Vec<StreamDeckCommand>) {
        self.tx.send(commands).ok();
    }
}

/// Spawns streamdeck thread from a core reference
pub fn spawn_streamdeck_thread(core: Arc<SDCore>, streamdeck: StreamDeck, key_tx: Sender<(u8, bool)>) -> StreamDeckHandle {
    let (tx, rx) = channel::<Vec<StreamDeckCommand>>();

    spawn(move || {
        let core = core.clone();
        let mut streamdeck = streamdeck;
        let mut last_iter = Instant::now();
        let mut last_buttons = Vec::new();

        streamdeck.set_blocking(false).ok();

        loop {
            if core.is_closed() {
                break;
            }

            // Pool for buttons
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

            // Check for commands
            match rx.try_recv() {
                Ok(commands) => {
                    for command in commands {
                        match command {
                            StreamDeckCommand::SetButtonImage(key, image) => {
                                streamdeck.set_button_image(key, image).ok();
                            }
                            StreamDeckCommand::ClearButtonImage(key) => {
                                streamdeck.set_button_rgb(key, &Colour {
                                    r: 0,
                                    g: 0,
                                    b: 0
                                }).ok();
                            }
                            StreamDeckCommand::SetBrightness(brightness) => {
                                streamdeck.set_brightness(brightness).ok();
                            }
                        }
                    }
                }

                Err(err) => {
                    match err {
                        TryRecvError::Empty => {}
                        TryRecvError::Disconnected => {
                            break;
                        }
                    }
                }
            }

            // Rate limiter
            let rate = 1.0 / core.pool_rate as f32;
            let time_since_last = last_iter.elapsed().as_secs_f32();

            let to_wait = rate - time_since_last;
            if to_wait > 0.0 {
                sleep(Duration::from_secs_f32(to_wait));
            }

            last_iter = Instant::now();
        }

        log::trace!("streamdeck closed");
    });

    StreamDeckHandle {
        tx,
    }
}

/// Supported streamdeck commands
#[allow(dead_code)]
pub enum StreamDeckCommand {
    /// Sets streamdeck brightness to provided value
    SetBrightness(u8),

    /// Sets button image to specified image
    SetButtonImage(u8, DynamicImage),

    /// Clears button and sets it to black color
    ClearButtonImage(u8),
}