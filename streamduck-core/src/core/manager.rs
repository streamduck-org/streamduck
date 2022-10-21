//! Manager of streamduck cores

use std::collections::HashMap;
use std::sync::{Arc};
use std::time::Duration;
use futures::{stream, StreamExt};
use crate::core::{RawButtonPanel, SDCore};
use crate::core::methods::CoreHandle;
use hidapi::HidApi;
use serde_json::Value;
use tokio::sync::RwLock;
use tokio::time::sleep;
use crate::config::{Config, DeviceConfig};
use crate::{connect, find_decks, ModuleManager, RenderingManager, SocketManager};
use crate::util::{make_panel_unique};

/// Core manager struct
pub struct CoreManager {
    hid: RwLock<HidApi>,
    pub config: Arc<Config>,
    devices: RwLock<HashMap<String, DeviceData>>,
    pub module_manager: Arc<ModuleManager>,
    pub render_manager: Arc<RenderingManager>,
    pub socket_manager: Arc<SocketManager>,
}

#[allow(dead_code)]
impl CoreManager {
    /// Creates new core manager with provided module manager and config
    pub fn new(module_manager: Arc<ModuleManager>, render_manager: Arc<RenderingManager>, socket_manager: Arc<SocketManager>, config: Arc<Config>) -> Arc<CoreManager> {
        let hid = HidApi::new().expect("could not connect to hidapi");

        Arc::new(CoreManager {
            hid: RwLock::new(hid),
            config,
            devices: Default::default(),
            module_manager,
            render_manager,
            socket_manager
        })
    }

    /// Adds all devices from config to managed devices, used at start of the software
    pub async fn add_devices_from_config(&self) {
        for config in self.config.get_all_device_configs().await {
            let config_handle = config.read().await;
            self.add_device(config_handle.vid, config_handle.pid, &config_handle.serial).await;
        }
    }

    /// Lists detected unmanaged devices
    pub async fn list_available_devices(&self) -> Vec<(u16, u16, String)> {
        let mut handle = self.hid.write().await;

        handle.refresh_devices().ok();

        let mut devices = vec![];

        for (vid, pid, serial) in find_decks(&handle) {
            if let Some(serial) = serial {
                if !self.is_device_added(&serial).await {
                    devices.push((vid, pid, serial));
                }
            }
        }

        devices
    }

    /// Adds device to automatic reconnection
    pub async fn add_device(&self, vid: u16, pid: u16, serial: &str) {
        let mut handle = self.devices.write().await;

        if !handle.contains_key(serial) {
            let data = DeviceData {
                core: SDCore::blank(self.module_manager.clone(), self.render_manager.clone(), self.socket_manager.clone(), self.config.clone(), Default::default(), Default::default()).await,
                vid,
                pid,
                serial: serial.to_string()
            };

            self.config.restore_device_config(serial).await;

            handle.insert(serial.to_string(), data.clone());
        }
    }

    /// Connects to a device
    pub async fn connect_device(&self, vid: u16, pid: u16, serial: &str) -> Result<DeviceData, String> {
        let hid_handle = self.hid.read().await;
        let collection = self.config.get_image_collection(serial).await;

        let config = if let Some(config) = self.config.get_device_config(serial).await {
            config
        } else {
            self.config.set_device_config(serial, DeviceConfig {
                vid,
                pid,
                serial: serial.to_string(),
                brightness: 50,
                layout: RawButtonPanel {
                    display_name: "Root".to_string(),
                    data: Value::Null,
                    buttons: Default::default()
                },
                images: Default::default(),
                plugin_data: Default::default(),
                commit_time: Default::default(),
                dirty_state: false,
            }).await;
            self.config.save_device_config(serial).await.ok();
            self.config.get_device_config(serial).await.unwrap()
        };

        if let Ok(core) = connect(self.module_manager.clone(), self.render_manager.clone(), self.socket_manager.clone(), self.config.clone(), config.clone(), collection,&hid_handle, vid, pid, serial, self.config.frame_rate()).await {
            let data = DeviceData {
                core: core.clone(),
                vid,
                pid,
                serial: serial.to_string()
            };

            let core_handle = CoreHandle::wrap(core.clone());

            let config_handle = config.read().await;

            let brightness = config_handle.brightness;
            let layout = config_handle.layout.clone();

            drop(config_handle);

            core_handle.set_brightness(brightness).await;
            core_handle.reset_stack(make_panel_unique(layout)).await;


            let mut handle = self.devices.write().await;

            handle.insert(serial.to_string(), data.clone());

            Ok(data)
        } else {
            Err("Failed to connect".to_string())
        }
    }

    /// Removes device from automatic reconnection and stops current connection to it
    pub async fn remove_device(&self, serial: &str) {
        let mut handle = self.devices.write().await;
        let data = handle.remove(serial);

        if let Some(data) = data {
            data.core.close().await;
            self.config.disable_device_config(serial).await;
            self.config.reload_device_configs().await.ok();
        }
    }

    /// Lists managed devices
    pub async fn list_added_devices(&self) -> HashMap<String, DeviceData> {
        self.devices.read().await.iter()
            .map(|(s, d)| (s.clone(), d.clone()))
            .collect()
    }

    /// Returns if specific device is in managed list
    pub async fn is_device_added(&self, serial: &str) -> bool {
        self.devices.read().await.contains_key(serial)
    }

    /// Gets device data from managed devices
    pub async fn get_device(&self, serial: &str) -> Option<DeviceData> {
        if let Some(device_data) = self.devices.read().await.get(serial) {
            if !device_data.core.is_closed().await {
                Some(device_data.clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Starts running reconnection routine on current thread, probably spawn it out as a separate thread
    pub async fn reconnect_routine(&self) {
        loop {
            sleep(Duration::from_secs_f32(self.config.reconnect_rate())).await;

            let disconnected = self.get_disconnected().await;

            if !disconnected.is_empty() {
                for (serial, device) in disconnected {
                    log::warn!("{} is disconnected, attempting to reconnect", serial);
                    if let Ok(_) = self.connect_device(device.vid, device.pid, &device.serial).await {
                        log::info!("Reconnected {}", serial);
                    }
                }
            }
        }
    }

    /// Retrieves currently disconnected devices from managed devices list
    async fn get_disconnected(&self) -> HashMap<String, DeviceData> {
        let handle = self.devices.read().await;

        let map = stream::iter(handle.iter())
            .filter(|(_, d)| async { d.core.is_closed().await })
            .map(|(s, d)| (s.clone(), d.clone()))
            .collect().await;

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
