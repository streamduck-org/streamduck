//! Manager of streamduck cores

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread::{sleep, spawn};
use std::time::Duration;
use streamduck_core::core::{RawButtonPanel, SDCore};
use streamduck_core::{connect, find_decks};
use streamduck_core::core::methods::{CoreHandle, load_panels, push_screen, set_brightness};
use streamduck_core::hidapi::HidApi;
use streamduck_core::modules::ModuleManager;
use streamduck_core::util::{make_panel_unique};
use crate::config::{Config, DeviceConfig};

/// Core manager struct
pub struct CoreManager {
    hid: HidApi,
    config: Arc<Config>,
    devices: RwLock<HashMap<String, DeviceData>>,
    module_manager: Arc<ModuleManager>,
}

#[allow(dead_code)]
impl CoreManager {
    /// Creates new core manager with provided module manager and config
    pub fn new(module_manager: Arc<ModuleManager>, config: Arc<Config>) -> Arc<CoreManager> {
        let hid = HidApi::new().expect("could not connect to hidapi");

        Arc::new(CoreManager {
            hid,
            config,
            devices: Default::default(),
            module_manager
        })
    }

    /// Adds all devices from config to managed devices, used at start of the software
    pub fn add_devices_from_config(&self) {
        for config in self.config.get_all_device_configs() {
            self.add_device(config.vid, config.pid, &config.serial);
        }
    }

    /// Lists detected unmanaged devices
    pub fn list_available_devices(&self) -> Vec<(u16, u16, String)> {
        find_decks(&self.hid).iter()
            .filter(|(.., str)| str.is_some())
            .filter(|(.., str)| self.get_device(str.clone().unwrap().as_str()).is_none())
            .map(|(vid, pid, serial)| (*vid, *pid, serial.clone().unwrap()))
            .collect()
    }

    /// Adds device to automatic reconnection
    pub fn add_device(&self, vid: u16, pid: u16, serial: &str) {
        let mut handle = self.devices.write().unwrap();

        if !handle.contains_key(serial) {
            let data = DeviceData {
                core: SDCore::blank(self.module_manager.clone()),
                vid,
                pid,
                serial: serial.to_string()
            };

            self.config.restore_device_config(serial);

            handle.insert(serial.to_string(), data.clone());
        }
    }

    /// Connects to a device
    pub fn connect_device(&self, vid: u16, pid: u16, serial: &str) -> Result<DeviceData, String> {
        if let Ok((core, handler)) = connect(self.module_manager.clone(), &self.hid, vid, pid, serial, self.config.pool_rate()) {
            spawn(move || {
                handler.run_loop();
                log::trace!("key handler closed");
            });

            let data = DeviceData {
                core: core.clone(),
                vid,
                pid,
                serial: serial.to_string()
            };

            let core_handle = CoreHandle::wrap(core.clone());

            if let Some(config) = self.config.get_device_config(serial) {
                set_brightness(&core_handle, config.brightness);
                load_panels(&core_handle, make_panel_unique(config.layout))
            } else {
                self.config.set_device_config(serial, DeviceConfig {
                    vid,
                    pid,
                    serial: serial.to_string(),
                    brightness: 50,
                    layout: RawButtonPanel::new()
                });

                self.config.save_device_config(serial).ok();

                push_screen(&core_handle, make_panel_unique(HashMap::new()));
                set_brightness(&core_handle, 50);
            }

            let mut handle = self.devices.write().unwrap();

            handle.insert(serial.to_string(), data.clone());

            Ok(data)
        } else {
            Err("Failed to connect".to_string())
        }
    }

    /// Removes device from automatic reconnection and stops current connection to it
    pub fn remove_device(&self, serial: &str) {
        let mut handle = self.devices.write().unwrap();
        let data = handle.remove(serial);

        if let Some(data) = data {
            data.core.close();
            self.config.disable_device_config(serial);
            self.config.reload_device_configs().ok();
        }
    }

    /// Lists managed devices
    pub fn list_added_devices(&self) -> HashMap<String, DeviceData> {
        self.devices.read().unwrap().iter()
            .map(|(s, d)| (s.clone(), d.clone()))
            .collect()
    }

    /// Gets device data from managed devices
    pub fn get_device(&self, serial: &str) -> Option<DeviceData> {
        self.devices.read().unwrap().get(serial).cloned()
    }

    /// Starts running reconnection routine on current thread, probably spawn it out as a separate thread
    pub fn reconnect_routine(&self) {
        loop {
            sleep(Duration::from_secs_f32(self.config.reconnect_rate()));

            let disconnected = self.get_disconnected();

            if !disconnected.is_empty() {
                for (serial, device) in disconnected {
                    log::warn!("{} is disconnected, attempting to reconnect", serial);
                    if let Ok(_) = self.connect_device(device.vid, device.pid, &device.serial) {
                        log::info!("Reconnected {}", serial);
                    }
                }
            }
        }
    }

    /// Retrieves currently disconnected devices from managed devices list
    fn get_disconnected(&self) -> HashMap<String, DeviceData> {
        let handle = self.devices.read().unwrap();

        let map = handle.iter()
            .filter(|(_, d)| d.core.is_closed())
            .map(|(s, d)| (s.clone(), d.clone()))
            .collect();

        drop(handle);

        map
    }
}

/// Device data
#[derive(Clone)]
pub struct DeviceData {
    /// Core that holds connection to the device
    pub core: Arc<SDCore>,
    /// Vendor ID
    pub vid: u16,
    /// Product ID
    pub pid: u16,
    /// Serial number
    pub serial: String,
}