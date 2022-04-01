/// Definitions of button structs
pub mod button;

/// Methods for interacting with the core
pub mod methods;
pub mod manager;
pub mod thread;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::sync::mpsc::{channel, Receiver};
use streamdeck::{Kind, StreamDeck};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::config::UniqueDeviceConfig;
use crate::core::button::Button;
use crate::core::thread::{DeviceThreadCommunication, DeviceThreadHandle, spawn_device_thread};
use crate::core::methods::{button_down, button_up, CoreHandle};
use crate::ImageCollection;
use crate::modules::ModuleManager;

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
    pub display_name: String,
    /// Data to keep with stack
    pub data: Value,
    /// Buttons of the panel
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
    /// Module manager
    pub module_manager: Arc<ModuleManager>,

    /// Device config associated with the device
    pub device_config: UniqueDeviceConfig,

    /// Current panel stack
    pub current_stack: Mutex<Vec<ButtonPanel>>,

    /// Image size supported by streamdeck
    pub image_size: (usize, usize),

    /// Image collection to use for rendering
    pub image_collection: ImageCollection,

    /// Kind of streamdeck device
    pub kind: Kind,

    /// Key count of the streamdeck device
    pub key_count: u8,

    /// Pool rate of how often should the core read events from the device
    pub pool_rate: u32,

    /// Decides if core is dead
    pub should_close: RwLock<bool>,

    handles: Mutex<Option<ThreadHandles>>
}

impl SDCore {
    /// Creates an instance of core that is already dead
    pub fn blank(module_manager: Arc<ModuleManager>, device_config: UniqueDeviceConfig, image_collection: ImageCollection) -> Arc<SDCore> {
        Arc::new(SDCore {
            module_manager,
            device_config,
            current_stack: Mutex::new(vec![]),
            handles: Mutex::new(None),
            image_size: (0, 0),
            image_collection,
            kind: Kind::Original,
            key_count: 0,
            pool_rate: 0,
            should_close: RwLock::new(true)
        })
    }

    /// Creates an instance of the core over existing streamdeck connection
    pub fn new(module_manager: Arc<ModuleManager>, device_config: UniqueDeviceConfig, image_collection: ImageCollection, connection: StreamDeck, pool_rate: u32) -> (Arc<SDCore>, KeyHandler) {
        let (key_tx, key_rx) = channel();

        let core = Arc::new(SDCore {
            module_manager,
            device_config,
            current_stack: Mutex::new(vec![]),
            handles: Mutex::new(None),
            image_size: connection.image_size(),
            image_collection,
            kind: connection.kind(),
            key_count: connection.kind().keys(),
            pool_rate,
            should_close: RwLock::new(false)
        });

        let renderer = spawn_device_thread(core.clone(), connection, key_tx);

        if let Ok(mut handles) = core.handles.lock() {
            *handles = Some(
                ThreadHandles {
                    renderer
                }
            )
        }

        (core.clone(), KeyHandler {
            core: CoreHandle::wrap(core.clone()),
            receiver: key_rx
        })
    }

    /// Sends commands to streamdeck thread
    pub fn send_commands(&self, commands: Vec<DeviceThreadCommunication>) {
        let handles = self.handles.lock().unwrap();

        handles.as_ref().unwrap().renderer.send(commands);
    }

    /// Checks if core is supposed to be closed
    pub fn is_closed(&self) -> bool {
        *self.should_close.read().unwrap()
    }

    /// Kills the core and all the related threads
    pub fn close(&self) {
        let mut lock = self.should_close.write().unwrap();
        *lock = true;
    }
}

struct ThreadHandles {
    pub renderer: DeviceThreadHandle
}

/// Routine that acts as a middleman between streamdeck thread and the core, was made as a way to get around Sync restriction
pub struct KeyHandler{
    core: CoreHandle,
    receiver: Receiver<(u8, bool)>
}

impl KeyHandler {
    /// Runs the key handling loop in current thread
    pub fn run_loop(&self) {
        loop {
            if self.core.core().is_closed() {
                break
            }

            if let Ok((key, state)) = self.receiver.recv() {
                if state {
                    button_down(&self.core, key);
                } else {
                    button_up(&self.core, key);
                }
            } else {
                break;
            }
        }
    }
}
