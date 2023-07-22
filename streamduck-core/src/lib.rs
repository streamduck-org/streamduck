#![warn(missing_docs)]
#![allow(unused_variables)]
#![allow(dead_code)]

//! Main functionality of the project

use std::collections::HashSet;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use arc_swap::{ArcSwap, AsRaw};
use futures::future::join_all;
use scc::{HashIndex, HashMap};
use scc::ebr::Barrier;
use scc::hash_map::OccupiedEntry;
use tokio::task;
use tokio::time::sleep;
use crate::config::{Config, SharedConfig};
use crate::core::{Core, SharedCore, WeakAction, WeakOverlay};
use crate::data::NamespacedName;
use crate::device::{DeviceError, DeviceIdentifier, DeviceMetadata, SharedDevice};
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
    plugins: HashMap<String, SharedPlugin>,
    discovered_devices: ArcSwap<HashSet<DeviceIdentifier>>,
    cores: HashMap<DeviceIdentifier, SharedCore>,
    overlays: HashMap<NamespacedName, WeakOverlay>,
    actions: HashMap<NamespacedName, WeakAction>,
    drivers: HashMap<NamespacedName, WeakDriver>,
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
        self.plugins.insert_async(plugin.name.clone(), plugin.clone()).await.ok();

        // Adding it to all indices
        join_all(plugin.overlays().iter()
            .map(|(name, overlay)| {
                (plugin.new_name(name), Arc::downgrade(overlay))
            })
            .map(|(k, v)| {
                self.overlays.insert_async(k, v)
            })).await;

        join_all(plugin.actions().iter()
            .map(|(name, action)| {
                (plugin.new_name(name), Arc::downgrade(action))
            })
            .map(|(k, v)| {
                self.actions.insert_async(k, v)
            })).await;

        join_all(plugin.drivers().iter()
            .map(|(name, driver)| {
                (plugin.new_name(name), Arc::downgrade(driver))
            })
            .map(|(k, v)| {
                self.drivers.insert_async(k, v)
            })).await;
    }

    /// Gets new devices from all loaded drivers
    pub async fn refresh_devices(&self) {
        let mut new_device_list = HashSet::new();

        let mut entry_iter = self.drivers.first_occupied_entry_async().await;

        while let Some(entry) = entry_iter {
            let name = entry.key();
            let driver = entry.get();

            println!("driver: {:?}", name);

            if let Some(driver) = driver.upgrade() {
                new_device_list.extend(driver.list_devices().await);
            } else {
                self.drivers.remove_async(name).await;
            }

            entry_iter = entry.next_async().await;
        }

        println!("devices: {:?}", new_device_list);

        self.discovered_devices.store(Arc::new(new_device_list));
    }

    /// Lists devices found by Streamduck
    pub async fn list_devices(&self) -> HashSet<DeviceIdentifier> {
        self.discovered_devices.load().as_ref().clone()
    }

    /// Describes device found under the identifier, returns none if driver no longer exists,
    /// or if device is no longer there in some cases
    pub async fn describe_device(&self, device_identifier: &DeviceIdentifier) -> Option<DeviceMetadata> {
        let driver = self.drivers.get_async(&device_identifier.driver_name).await?;
        let upgraded_driver = driver.get().upgrade()?;

        Some(upgraded_driver.describe_device(device_identifier).await)
    }

    /// Adds device to autoconnect
    pub async fn add_device_to_autoconnect(&self, device_identifier: &DeviceIdentifier) {
        if !self.config.load().autoconnect_devices.contains(device_identifier) {
            let mut new_config = self.config.load().as_ref().clone();

            new_config.autoconnect_devices.insert(device_identifier.clone());

            self.config.store(Arc::new(new_config));
        }

        if !self.cores.contains(device_identifier) {
            self.connect_to_device(device_identifier).await;
        }
    }

    /// Connects to the device and initializes a core for it
    pub async fn connect_to_device(&self, device_identifier: &DeviceIdentifier) -> bool {
        let Some(driver) = self.drivers.get_async(&device_identifier.driver_name).await else {
            return false
        };

        let Some(upgraded_driver) = driver.get().upgrade() else {
            return false
        };

        let Ok(device) = upgraded_driver.connect_device(device_identifier).await else {
            return false;
        };

        // Device config should exist at this point because of driver
        let device_config = self.config.load().get_device_config(device_identifier).await.unwrap();

        let core = Core::init_core(self.config.clone(), device, device_config).await;
        self.cores.insert_async(device_identifier.clone(), core).await.ok().is_some()
    }

    /// Removes device from autoconnect
    pub fn remove_device_to_autoconnect(&self, device_identifier: &DeviceIdentifier) {
        if self.config.load().autoconnect_devices.contains(device_identifier) {
            let mut new_config = self.config.load().as_ref().clone();

            new_config.autoconnect_devices.remove(device_identifier);

            self.config.store(Arc::new(new_config));
        }
    }

    /// Cleans up any dead device
    pub async fn clean_up_dead_devices(&self) {
        let mut entry = self.cores.first_occupied_entry_async().await;

        while let Some(some_entry) = entry {
            if !some_entry.get().active.load(Ordering::Acquire) {
                some_entry.get().save_data().await;
                self.cores.remove_async(some_entry.key()).await;
            }

            entry = some_entry.next_async().await;
        }
    }

    /// Drops the connected device, but lets it save first
    pub async fn drop_device(&self, device_identifier: &DeviceIdentifier) {
        if let Some(entry) = self.cores.get_async(device_identifier).await {
            entry.get().die().await;
            let _ = entry.remove_entry();
        }
    }

    /// Launches the root task for the software
    pub async fn run(self: &Arc<Self>) {
        self.running.store(true, Ordering::Release);

        // Rendering thread
        let self_copy = self.clone();
        std::thread::spawn(move || {
            rendering_tick(self_copy);
        });

        // Device check task
        let self_copy = self.clone();
        task::spawn(device_check_tick(self_copy));

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
            println!("rendering - frames counted: {}, frame diff: {}, target diff: {}", frame_count, elapsed_time, target_diff);
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

        // do actual code here
        sleep(Duration::from_micros(15000)).await;


        // tick limiter
        let current_time = start_time.elapsed().as_secs_f32();
        let elapsed_time = current_time - tick_start;

        let target_diff = 1.0 / config_guard.tick_rate;
        let sleep_time = target_diff - elapsed_time;

        tick_count += 1;

        if tick_start.floor() > last_tick_time.floor() {
            println!("tick - ticks counted: {}, tick diff: {}, target diff: {}", tick_count, elapsed_time, target_diff);
            tick_count = 0;
        }

        if sleep_time > 0.0 {
            sleep(Duration::from_secs_f32(sleep_time)).await;
        }

        last_tick_time = tick_start;
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
            if let Some(device) = self_copy.cores.get_async(identifier).await {
                if device.get().active.load(Ordering::Acquire) {
                    continue;
                }
            }

            // Connect to the device
            self_copy.connect_to_device(identifier).await;
        }

        sleep(Duration::from_secs_f32(config_guard.device_check_frequency)).await;
    }
}