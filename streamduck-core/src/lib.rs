#![warn(missing_docs)]
#![allow(unused_variables)]
#![allow(dead_code)]

//! Main functionality of the project

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use arc_swap::{ArcSwap};
use tokio::task;
use tokio::time::sleep;
use tracing::{error, info, trace, warn};
use crate::config::{Config, SharedConfig};
use crate::core::{Core, SharedCore, WeakAction, WeakOverlay};
use crate::data::NamespacedName;
use crate::device::{DeviceIdentifier, DeviceMetadata};
use crate::device::driver::WeakDriver;
use crate::plugin::{SharedPlugin};

/// Core that would manage the device
pub mod core;

/// Device types
pub mod device;

/// Various data structures
pub mod data;

/// Event system
pub mod event;

/// Structures for UI schema
pub mod ui;

/// Trigger condition definitions
pub mod trigger;

/// Various utility functions
pub mod util;

/// Plugin definitions
pub mod plugin;

/// Configuration of the software
pub mod config;

/// Shared reference of Streamduck
pub type SharedStreamduck = Arc<Streamduck>;

/// Instance of the core of the software, is what loads the plugins and makes things tick
pub struct Streamduck {
    running: AtomicBool,
    config: SharedConfig,
    plugins: ArcSwap<HashMap<String, SharedPlugin>>,
    discovered_devices: ArcSwap<HashSet<DeviceIdentifier>>,
    cores: ArcSwap<HashMap<DeviceIdentifier, SharedCore>>,
    overlays: ArcSwap<HashMap<NamespacedName, WeakOverlay>>,
    actions: ArcSwap<HashMap<NamespacedName, WeakAction>>,
    drivers: ArcSwap<HashMap<NamespacedName, WeakDriver>>,
}

impl Streamduck {
    /// Initializes the core of the software
    pub async fn init() -> SharedStreamduck {
        let config = Config::load_or_create().await;

        Arc::new(Streamduck {
            running: Default::default(),
            config,
            plugins: Default::default(),
            discovered_devices: Default::default(),
            cores: Default::default(),
            overlays: Default::default(),
            actions: Default::default(),
            drivers: Default::default(),
        })
    }

    /// Gets instance of shared config
    pub fn get_config(&self) -> SharedConfig {
        self.config.clone()
    }

    /// Adds plugin to the software
    pub async fn load_plugin(&self, plugin: SharedPlugin) {
        // Adding it to main map
        self.plugins.rcu(|cache| {
            let mut new_plugin_map = cache.as_ref().clone();
            new_plugin_map.insert(plugin.name.clone(), plugin.clone());
            new_plugin_map
        });

        // Adding it to all indices
        self.overlays.rcu(|cache| {
            let mut new_overlay_map = cache.as_ref().clone();
            new_overlay_map.extend(plugin.overlays().iter()
                .map(|(name, overlay)| {
                    (plugin.new_name(name), Arc::downgrade(overlay))
                }));
            new_overlay_map
        });

        self.actions.rcu(|cache| {
            let mut new_action_map = cache.as_ref().clone();
            new_action_map.extend(plugin.actions().iter()
                .map(|(name, action)| {
                    (plugin.new_name(name), Arc::downgrade(action))
                }));
            new_action_map
        });

        self.drivers.rcu(|cache| {
            let mut new_driver_map = cache.as_ref().clone();
            new_driver_map.extend(plugin.drivers().iter()
                .map(|(name, driver)| {
                    (plugin.new_name(name), Arc::downgrade(driver))
                }));
            new_driver_map
        });
    }

    /// Gets new devices from all loaded drivers
    pub async fn refresh_devices(&self) {
        let mut new_device_list = HashSet::new();

        info!("Refreshing devices");

        for (name, driver) in self.drivers.load().iter() {
            if let Some(driver) = driver.upgrade() {
                new_device_list.extend(driver.list_devices().await);
            }
        }

        self.discovered_devices.store(Arc::new(new_device_list));
    }

    /// Lists devices found by Streamduck
    pub async fn list_devices(&self) -> HashSet<DeviceIdentifier> {
        self.discovered_devices.load().as_ref().clone()
    }

    /// Describes device found under the identifier, returns none if driver no longer exists,
    /// or if device is no longer there in some cases
    pub async fn describe_device(&self, device_identifier: &DeviceIdentifier) -> Option<DeviceMetadata> {
        let guard = self.drivers.load();
        let driver = guard.get(&device_identifier.driver_name)?;
        let upgraded_driver = driver.upgrade()?;

        Some(upgraded_driver.describe_device(device_identifier).await)
    }

    /// Adds device to autoconnect
    pub async fn add_device_to_autoconnect(&self, device_identifier: &DeviceIdentifier) {
        if !self.config.load().autoconnect_devices.contains(device_identifier) {
            info!(%device_identifier, "Adding device to autoconnect");

            self.config.rcu(|cache| {
                let mut new_config = cache.as_ref().clone();
                new_config.autoconnect_devices.insert(device_identifier.clone());
                new_config
            });

            self.save_config().await;
        }

        if !self.cores.load().contains_key(device_identifier) {
            self.connect_to_device(device_identifier).await;
        }
    }

    /// Connects to the device and initializes a core for it
    pub async fn connect_to_device(&self, device_identifier: &DeviceIdentifier) -> bool {
        info!(%device_identifier, "Connecting to the device");

        let guard = self.drivers.load();

        let Some(driver) = guard.get(&device_identifier.driver_name) else {
            warn!(%device_identifier, "Driver wasn't found for the device identifier");
            return false
        };

        let Some(upgraded_driver) = driver.upgrade() else {
            warn!(%device_identifier, "Driver for the device was already removed");
            return false
        };

        let device = match upgraded_driver.connect_device(device_identifier).await {
            Ok(d) => d,
            Err(e) => {
                error!(%e, "Error happened trying to connect to a device");
                return false;
            }
        };

        // Device config should exist at this point because of driver
        let device_config = self.config.load().get_device_config(device_identifier).await.unwrap();

        let core = Core::init_core(self.config.clone(), device, device_config).await;

        self.cores.rcu(|cores| {
            let mut new_cores = (**cores).clone();
            new_cores.insert(device_identifier.clone(), core.clone());
            new_cores
        });

        true
    }

    /// Removes device from autoconnect
    pub async fn remove_device_to_autoconnect(&self, device_identifier: &DeviceIdentifier) {
        if self.config.load().autoconnect_devices.contains(device_identifier) {
            info!(%device_identifier, "Removing device from autoconnect");

            self.config.rcu(|cache| {
                let mut new_config = cache.as_ref().clone();
                new_config.autoconnect_devices.remove(device_identifier);
                new_config
            });

            self.save_config().await;
        }
    }

    /// Cleans up any dead device
    pub async fn clean_up_dead_devices(&self) {
        let mut dead_entries= vec![];

        self.cores.rcu(|cache| {
            let (alive, dead): (Vec<_>, Vec<_>) = cache.as_ref().clone().into_iter()
                .partition(|(_, c)| c.active.load(Ordering::Acquire));

            dead_entries = dead;

            HashMap::from_iter(alive)
        });

        for (iden, dead_core) in dead_entries {
            trace!(%iden, "Device is dead, cleaning up");
            dead_core.die().await;
        }
    }

    /// Drops the connected device, but lets it save first
    pub async fn drop_device(&self, device_identifier: &DeviceIdentifier) {
        if let Some(entry) = self.cores.load().get(device_identifier) {
            entry.die().await;
        }
    }

    /// Writes global config to file
    pub async fn save_config(&self) {
        let guard = self.config.load();

        if let Err(e) = guard.save().await {
            error!(?e, "Error happened while trying to save global config");
        }
    }

    /// Launches the root task for the software
    pub async fn run(self: &Arc<Self>) {
        self.running.store(true, Ordering::Release);

        info!("Spawning rendering thread");
        // Rendering thread
        let self_copy = self.clone();
        std::thread::spawn(move || {
            rendering_tick(self_copy);
        });

        info!("Spawning device check task");
        // Device check task
        let self_copy = self.clone();
        task::spawn(device_check_tick(self_copy));

        info!("Running the tick task");
        // Tick
        let self_copy = self.clone();
        tick(self_copy).await;
    }
}

impl Drop for Streamduck {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Release);
    }
}

fn rendering_tick(self_copy: SharedStreamduck) {
    let start_time = Instant::now();
    let mut last_frame_time = 0.0_f32;

    let mut frame_count = 0;

    while self_copy.running.load(Ordering::Acquire) {
        let frame_start = start_time.elapsed().as_secs_f32();
        let config_guard = self_copy.config.load();

        // do actual code here
        std::thread::sleep(Duration::from_micros(15000));


        // frame limiter
        let current_time = start_time.elapsed().as_secs_f32();
        let elapsed_time = current_time - frame_start;

        let target_diff = 1.0 / config_guard.frame_rate;
        let sleep_time = target_diff - elapsed_time;

        frame_count += 1;

        if frame_start.floor() > last_frame_time.floor() {
            // trace!("rendering - frames counted: {}, frame diff: {}, target diff: {}", frame_count, elapsed_time, target_diff);
            frame_count = 0;
        }

        if sleep_time > 0.0 {
            std::thread::sleep(Duration::from_secs_f32(sleep_time));
        }

        last_frame_time = frame_start;
    }
}

async fn tick(self_copy: SharedStreamduck) {
    let start_time = Instant::now();
    let mut last_tick_time = 0.0_f32;

    let mut tick_count = 0;

    while self_copy.running.load(Ordering::Acquire) {
        let tick_start = start_time.elapsed().as_secs_f32();
        let config_guard = self_copy.config.load();

        // device tick
        tokio::task::spawn(device_poll_tick_impl(self_copy.clone()));


        // tick limiter
        let current_time = start_time.elapsed().as_secs_f32();
        let elapsed_time = current_time - tick_start;

        let target_diff = 1.0 / config_guard.tick_rate;
        let sleep_time = target_diff - elapsed_time;

        tick_count += 1;

        if tick_start.floor() > last_tick_time.floor() {
            // trace!("tick - ticks counted: {}, tick diff: {}, target diff: {}", tick_count, elapsed_time, target_diff);
            tick_count = 0;
        }

        if sleep_time > 0.0 {
            sleep(Duration::from_secs_f32(sleep_time)).await;
        }

        last_tick_time = tick_start;
    }
}

async fn device_poll_tick_impl(self_copy: SharedStreamduck) {
    for (_, core) in self_copy.cores.load().iter() {
        if core.active.load(Ordering::Acquire) {
            core.poll_device().await;
        }
    }
}

async fn device_check_tick(self_copy: SharedStreamduck) {
    while self_copy.running.load(Ordering::Acquire) {
        let config_guard = self_copy.config.load();

        self_copy.clean_up_dead_devices().await;

        self_copy.refresh_devices().await;

        for identifier in self_copy.discovered_devices.load().iter() {

            // Skip device if it's not included in autoconnect
            if !config_guard.autoconnect_devices.contains(identifier) {
                continue;
            }

            // Skip device if it's already connected and active
            if let Some(device) = self_copy.cores.load().get(identifier) {
                if device.active.load(Ordering::Acquire) {
                    continue;
                }
            }

            // Connect to the device
            self_copy.connect_to_device(identifier).await;
        }

        sleep(Duration::from_secs_f32(config_guard.device_check_frequency)).await;
    }
}