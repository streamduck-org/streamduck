pub mod requests;
pub mod events;
pub mod device;

use std::cell::{Cell, OnceCell, RefCell};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::rc::Rc;
use std::thread::sleep;
use std::time::{Duration, Instant};
use log::{info};
use crate::api::index::PluginIndex;
use crate::api::driver::{NamespacedDeviceIdentifier, RefDriver};
use crate::api::id::{NamespacedID, PluginID};
use crate::api::plugin::{BoxPlugin, PluginSource, RefPlugin};
use crate::client_server::{boot_client_thread, get_channels, CoreChannels};
use crate::core::device::ManagedDevice;
use crate::plugins::lua::LuaPluginSource;
use crate::util::id_set::IdSet;
use crate::util::linked_once_list::LinkedOnceList;

const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");

pub struct Core<'core> {
    status: Cell<CoreStatus>,
    client: OnceCell<CoreChannels>,

    plugin_list: LinkedOnceList<BoxPlugin>,
    plugin_index: RefCell<HashMap<String, RefPlugin<'core>>>,

    drivers: RefCell<HashMap<NamespacedID, RefDriver<'core>>>,

    last_discovery: RefCell<Option<Instant>>,
    discovered_devices: RefCell<HashSet<NamespacedDeviceIdentifier>>,

    devices: RefCell<HashMap<NamespacedDeviceIdentifier, Rc<ManagedDevice<'core>>>>
}

impl<'core> PluginIndex for Core<'core> {
    fn all_plugins(&self) -> Vec<RefPlugin<'_>> {
        self.plugin_index.borrow().values().copied().collect()
    }

    fn find_plugin(&self, id: &str) -> Option<RefPlugin<'_>> {
        self.plugin_index.borrow().get(id).copied()
    }

    fn all_drivers(&self) -> HashMap<NamespacedID, RefDriver<'_>> {
        self.drivers.borrow().clone()
    }

    fn all_plugin_drivers(&self, id: &str) -> Vec<RefDriver<'_>> {
        self.drivers.borrow().iter()
            .filter(|(driver_id, _)| driver_id.plugin == id)
            .map(|(_, driver)| *driver)
            .collect()
    }

    fn find_driver(&self, id: &NamespacedID) -> Option<RefDriver<'_>> {
        self.drivers.borrow().get(id).copied()
    }
}

impl<'core> Core<'core> {
    pub fn new() -> Core<'core> {
        Core {
            status: Cell::new(CoreStatus::StillAlive),
            client: Default::default(),

            plugin_list: LinkedOnceList::new(),
            plugin_index: Default::default(),

            drivers: Default::default(),

            last_discovery: Default::default(),
            discovered_devices: Default::default(),

            devices: Default::default(),
        }
    }

    pub fn rebuild_indices(&'core self) {
        self.plugin_index.replace(self.plugin_list.iter().map(|p| (p.name(), p.as_ref())).collect());

        // Maps to plugin components
        let mut driver_map = HashMap::new();

        for plugin in self.plugin_index.borrow().values() {
            for driver in plugin.drivers() {
                let name = driver.name();
                driver_map.insert(NamespacedID::new(name, plugin.name()), driver);
            }
        }

        self.drivers.replace(driver_map);
    }

    fn load_plugins_from_source(&self, id_set: &mut IdSet<PluginID>, path: impl AsRef<Path>, source: impl PluginSource) {
        info!("Loading plugins from '{}'", source.name());
        self.plugin_list.extend(source.load_plugins(id_set, path))
    }

    pub fn run(&'core self) -> anyhow::Result<CoreStatus> {
        if self.status.get().is_dead() {
            panic!("Core was ran after already being ran!")
        }

        info!("Streamduck Core v{} starting...", VERSION.unwrap_or("unknown"));

        let mut id_set: IdSet<PluginID> = IdSet::new();
        let plugin_path = "./plugins";
        
        // Load plugins from plugin sources
        self.load_plugins_from_source(&mut id_set, plugin_path, LuaPluginSource);

        // Build up the indices for easy findability of plugins and components
        self.rebuild_indices();

        // Run post init functions on all plugins
        let plugins = self.plugin_index.borrow();
        info!("{} plugin(s) loaded! Running post-init...", plugins.len());
        for plugin in plugins.values() {
            plugin.post_init();
        }

        let (chan, client_channels) = get_channels();
        self.client.set(chan).unwrap();

        boot_client_thread(client_channels);

        while !self.status.get().is_dead() {
            self.check_device_discovery();
            self.process_requests();
            self.tick_connected_devices();

            sleep(Duration::from_millis(10))
        }

        Ok(self.status.get())
    }
}

#[derive(Clone, Copy)]
pub enum CoreStatus {
    StillAlive,
    RestartNeeded,
    ClientThreadDied,
    NormalExit
}

impl CoreStatus {
    pub fn is_dead(&self) -> bool {
        match self {
            CoreStatus::StillAlive => false,
            _ => true,
        }
    }
}