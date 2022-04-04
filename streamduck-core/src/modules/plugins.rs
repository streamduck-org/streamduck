//! Plugin API for loading dynamic library files

use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::hash::Hasher;
use std::path::Path;
use std::sync::Arc;
use dlopen::Error;
use crate::modules::{BoxedSDModule, ModuleManager, PluginMetadata, SDModule, SDModulePointer, UniqueSDModule};
use dlopen::wrapper::{Container, WrapperApi};
use dlopen_derive::WrapperApi;
use image::DynamicImage;
use crate::core::button::Button;
use crate::core::manager::CoreManager;
use crate::core::methods::{CoreHandle, warn_for_feature};
use crate::core::UniqueButton;
use crate::modules::components::{ComponentDefinition, UIValue};
use crate::modules::events::SDEvent;
use crate::RenderingManager;
use crate::socket::SocketManager;
use crate::versions::SUPPORTED_FEATURES;

#[derive(WrapperApi)]
struct PluginApi {
    get_metadata: extern fn() -> PluginMetadata,
    get_module: extern fn(socket_manager: Arc<SocketManager>, render_manager: Arc<RenderingManager>) -> SDModulePointer,
}

#[allow(dead_code)]
struct PluginProxy {
    pub wrapper: Container<PluginApi>,
    pub metadata: PluginMetadata,
    pub plugin: BoxedSDModule
}

impl SDModule for PluginProxy {
    fn name(&self) -> String {
        self.plugin.name()
    }

    fn components(&self) -> HashMap<String, ComponentDefinition> {
        self.plugin.components()
    }

    fn add_component(&self, core: CoreHandle, button: &mut Button, name: &str) {
        self.plugin.add_component(core, button, name)
    }

    fn remove_component(&self, core: CoreHandle, button: &mut Button, name: &str) {
        self.plugin.remove_component(core, button, name)
    }

    fn component_values(&self, core: CoreHandle, button: &Button, name: &str) -> Vec<UIValue> {
        self.plugin.component_values(core, button, name)
    }

    fn set_component_value(&self, core: CoreHandle, button: &mut Button, name: &str, value: Vec<UIValue>) {
        self.plugin.set_component_value(core, button, name, value)
    }

    fn listening_for(&self) -> Vec<String> {
        self.plugin.listening_for()
    }

    fn settings(&self, core: Arc<CoreManager>) -> Vec<UIValue> {
        self.plugin.settings(core)
    }

    fn set_setting(&self, core: Arc<CoreManager>, value: Vec<UIValue>) {
        self.plugin.set_setting(core, value)
    }

    fn event(&self, core: CoreHandle, event: SDEvent) {
        if core.check_for_feature("events") {
            self.plugin.event(core, event)
        }
    }

    fn render(&self, core: CoreHandle, button: &UniqueButton, frame: &mut DynamicImage) {
        if core.check_for_feature("rendering") {
            self.plugin.render(core, button, frame)
        }
    }

    fn render_hash(&self, core: CoreHandle, button: &UniqueButton, hash: &mut Box<dyn Hasher>) {
        if core.check_for_feature("rendering") {
            self.plugin.render_hash(core, button, hash)
        }
    }

    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }
}

/// Returns error if plugin is incompatible
pub fn compare_plugin_versions(versions: &Vec<(String, String)>) -> Result<(), PluginError> {
    let core_versions = SUPPORTED_FEATURES.clone().into_iter()
        .map(|(n, v)| (*n, *v))
        .collect::<HashMap<&str, &str>>();

    for (name, version) in versions {
        if let Some(software_version) = core_versions.get(name.as_str()) {
            if software_version != version {
                return Err(PluginError::WrongVersion(format!("{} {}", name, version), format!("{} {}", name, software_version)))
            }
        } else {
            return Err(PluginError::TooNew(format!("{} {}", name, version)))
        }
    }

    Ok(())
}

/// Warns about essential features
pub fn warn_about_essential_features(module: UniqueSDModule) {
    let name = &module.name();
    let features = module.metadata().used_features;

    warn_for_feature(name, &features, "plugin_api");
    warn_for_feature(name, &features, "sdmodule_trait");
}

/// Loads a plugin into module manager
pub fn load_plugin<T: AsRef<OsStr>>(module_manager: Arc<ModuleManager>, socket_manager: Arc<SocketManager>, render_manager: Arc<RenderingManager>, path: T) -> Result<(), PluginError> {
    // Loading file as a library, error if cannot load
    let wrapper: Container<PluginApi> = unsafe { Container::load(path) }?;

    // Retrieving metadata and comparing versions
    let metadata = wrapper.get_metadata();
    compare_plugin_versions(&metadata.used_features)?;

    // Attempting to get module from the plugin
    let module: BoxedSDModule = unsafe { Box::from_raw(wrapper.get_module(socket_manager, render_manager)) };

    // Wrapping plugin's module into a wrapper that contains loaded library
    let module_proxy: UniqueSDModule = Arc::new(Box::new(PluginProxy { wrapper, metadata, plugin: module }));

    // Warn plugin if metadata doesn't contain essential plugins
    warn_about_essential_features(module_proxy.clone());

    // Adding module if it wasn't defined before
    if module_manager.get_module(&module_proxy.name()).is_none() {
        for component in module_proxy.components().keys() {
            if module_manager.get_component(component).is_some() {
                return Err(PluginError::ComponentConflict(module_proxy.name(), component.to_string()))
            }
        }

        module_manager.add_module(module_proxy);
        Ok(())
    } else {
        Err(PluginError::AlreadyExists(module_proxy.name()))
    }
}

/// Loads plugins into module manager from path
pub fn load_plugins_from_folder<T: AsRef<OsStr>>(module_manager: Arc<ModuleManager>, socket_manager: Arc<SocketManager>, render_manager: Arc<RenderingManager>, path: T) {
    let path = Path::new(&path);
    match fs::read_dir(path) {
        Ok(read_dir) => {
            for item in read_dir {
                match item {
                    Ok(entry) => {
                        if entry.path().is_file() {
                            if let Some(file_name) = entry.path().file_name() {
                                log::info!("Loading plugin {:?}", file_name);
                                match load_plugin(module_manager.clone(), socket_manager.clone(), render_manager.clone(), entry.path()) {
                                    Err(err) => match err {
                                        PluginError::LoadError(err) => log::error!("Failed to load plugin: {}", err),
                                        PluginError::WrongVersion(plugin, software) => log::error!("Failed to load plugin: Plugin is using unsupported version of '{}', software's using '{}'", plugin, software),
                                        PluginError::TooNew(version) => log::error!("Failed to load plugin: Software doesn't support '{}', try updating the software", version),
                                        PluginError::AlreadyExists(name) => log::error!("Failed to load plugin: Module '{}' was already defined", name),
                                        PluginError::ComponentConflict(name, component_name) => log::error!("Failed to load plugin: Module '{}' is declaring '{}' component, but it was already previously declared by other module", name, component_name),
                                    },
                                    _ => {}
                                }
                            }
                        }
                    }
                    Err(err) => log::error!("Failed to reach entry. {}", err),
                }
            }
        }
        Err(e) => {
            if let std::io::ErrorKind::NotFound = e.kind() {
                log::info!("Loaded no plugins, missing plugins folder")
            } else {
                log::error!("Plugins folder is unreachable: {:?}", path);
            }
        }
    }
}

/// Enum for anything wrong that might happen during plugin loading
#[derive(Debug)]
pub enum PluginError {
    LoadError(dlopen::Error),
    WrongVersion(String, String),
    TooNew(String),
    AlreadyExists(String),
    ComponentConflict(String, String),
}

impl From<dlopen::Error> for PluginError {
    fn from(err: Error) -> Self {
        PluginError::LoadError(err)
    }
}