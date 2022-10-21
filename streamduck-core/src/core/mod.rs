use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use streamdeck::{Kind, StreamDeck};
use tokio::sync::{Mutex, RwLock};
use tokio::sync::mpsc::unbounded_channel;

pub use methods::check_feature_list_for_feature;
pub use methods::CoreHandle;
pub use methods::warn_for_feature;

use crate::ImageCollection;
use crate::config::{Config, UniqueDeviceConfig};
use crate::core::button::Button;
use crate::modules::events::SDGlobalEvent;
use crate::modules::ModuleManager;
use crate::socket::SocketManager;
use crate::thread::{DeviceThreadCommunication, DeviceThreadHandle, spawn_device_thread};
use crate::thread::rendering::custom::RenderingManager;

/// Definitions of button structs
pub mod button;

/// Methods for interacting with the core
mod methods;
pub mod manager;

/// Reference counted RwLock of a button, prevents data duplication and lets you edit buttons if they're in many stacks at once
pub type UniqueButton = Arc<RwLock<Button>>;

/// Map of UniqueButtons
pub type UniqueButtonMap = HashMap<u8, UniqueButton>;

/// Map of Buttons
pub type ButtonMap = HashMap<u8, Button>;

/// Hashmap of UniqueButtons
pub type ButtonPanel = Arc<RwLock<Panel<UniqueButtonMap>>>;

/// Hashmap of raw Buttons
pub type RawButtonPanel = Panel<ButtonMap>;

/// Panel definition
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Panel<T> {
    /// Display name that will be shown in UI
    #[serde(default)]
    pub display_name: String,
    /// Data to keep with stack
    #[serde(default)]
    pub data: Value,
    /// Buttons of the panel
    #[serde(default)]
    pub buttons: T
}

impl Into<ButtonMap> for Panel<ButtonMap> {
    fn into(self) -> ButtonMap {
        self.buttons
    }
}

impl Into<ButtonMap> for &Panel<ButtonMap> {
    fn into(self) -> ButtonMap {
        self.buttons.clone()
    }
}

impl Into<UniqueButtonMap> for Panel<UniqueButtonMap> {
    fn into(self) -> UniqueButtonMap {
        self.buttons
    }
}

impl Into<UniqueButtonMap> for &Panel<UniqueButtonMap> {
    fn into(self) -> UniqueButtonMap {
        self.buttons.clone()
    }
}

/// Core struct that contains all relevant information about streamdeck and manages the streamdeck
#[allow(dead_code)]
pub struct SDCore {
    /// Serial number of the device
    pub serial_number: String,

    /// Module manager
    pub module_manager: Arc<ModuleManager>,

    /// Rendering manager
    pub render_manager: Arc<RenderingManager>,

    /// Socket manager
    pub socket_manager: Arc<SocketManager>,

    /// Config
    pub config: Arc<Config>,

    /// Device config associated with the device
    pub device_config: UniqueDeviceConfig,

    /// Current panel stack
    pub current_stack: Mutex<Vec<ButtonPanel>>,

    /// Image size supported by streamdeck
    pub image_size: (usize, usize),

    /// Image collection to use for thread
    pub image_collection: ImageCollection,

    /// Kind of streamdeck device
    pub kind: Kind,

    /// Key count of the streamdeck device
    pub key_count: u8,

    /// Pool rate of how often should the core read events from the device
    pub frame_rate: u32,

    /// Decides if core is dead
    pub should_close: RwLock<bool>,

    handles: Mutex<Option<ThreadHandles>>
}

impl SDCore {
    /// Creates an instance of core that is already dead
    pub async fn blank(module_manager: Arc<ModuleManager>, render_manager: Arc<RenderingManager>, socket_manager: Arc<SocketManager>, config: Arc<Config>, device_config: UniqueDeviceConfig, image_collection: ImageCollection) -> Arc<SDCore> {
        let serial_number = device_config.read().await.serial.to_string();
        Arc::new(SDCore {
            serial_number,
            module_manager,
            render_manager,
            socket_manager,
            config,
            device_config,
            current_stack: Mutex::new(vec![]),
            handles: Mutex::new(None),
            image_size: (0, 0),
            image_collection,
            kind: Kind::Original,
            key_count: 0,
            frame_rate: 0,
            should_close: RwLock::new(true)
        })
    }

    /// Creates an instance of the core over existing streamdeck connection
    pub async fn new(module_manager: Arc<ModuleManager>, render_manager: Arc<RenderingManager>, socket_manager: Arc<SocketManager>, config: Arc<Config>, device_config: UniqueDeviceConfig, image_collection: ImageCollection, mut connection: StreamDeck, frame_rate: u32) -> Arc<SDCore> {
        let (key_tx, mut key_rx) = unbounded_channel();

        let serial_number = device_config.read().await.serial.to_string();
        let serial_number = connection.serial().unwrap_or_else(|_| serial_number);

        module_manager.send_global_event_to_modules(SDGlobalEvent::DeviceConnected {
            serial_number: serial_number.clone()
        }).await;

        let core = Arc::new(SDCore {
            serial_number,
            module_manager,
            render_manager,
            socket_manager,
            config,
            device_config,
            current_stack: Mutex::new(vec![]),
            handles: Mutex::new(None),
            image_size: connection.image_size(),
            image_collection,
            kind: connection.kind(),
            key_count: connection.kind().keys(),
            frame_rate,
            should_close: RwLock::new(false)
        });

        let renderer = spawn_device_thread(core.clone(), connection, key_tx);

        *core.handles.lock().await = Some(
            ThreadHandles {
                renderer
            }
        );

        let task_core = CoreHandle::wrap(core.clone());
        tokio::spawn(async move {
            loop {
                if task_core.core().is_closed().await {
                    break
                }

                if let Some((key, state)) = key_rx.recv().await {
                    if state {
                        task_core.button_down(key).await;
                    } else {
                        task_core.button_up(key).await;
                    }
                } else {
                    break;
                }
            }
        });

        core
    }

    /// Tells device thread to refresh screen
    pub async fn mark_for_redraw(&self) {
        let handles = self.handles.lock().await;

        handles.as_ref().unwrap().renderer.send(vec![DeviceThreadCommunication::RefreshScreen]);
    }

    /// Sends commands to streamdeck thread
    pub async fn send_commands(&self, commands: Vec<DeviceThreadCommunication>) {
        let handles = self.handles.lock().await;

        handles.as_ref().unwrap().renderer.send(commands);
    }

    /// Gets serial number of the core
    pub async fn serial_number(&self) -> String {
        self.device_config.read().await.serial.to_string()
    }

    /// Checks if core is supposed to be closed
    pub async fn is_closed(&self) -> bool {
        *self.should_close.read().await
    }

    /// Kills the core and all the related threads
    pub async fn close(&self) {
        self.module_manager.send_global_event_to_modules(SDGlobalEvent::DeviceDisconnected {
            serial_number: self.serial_number.to_string()
        }).await;

        let mut lock = self.should_close.write().await;
        *lock = true;
    }
}

struct ThreadHandles {
    pub renderer: DeviceThreadHandle
}
