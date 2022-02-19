use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use dlopen::Error;
use streamduck_core::modules::{BoxedSDModule, ModuleManager, PluginMetadata, SDModule, SDModulePointer, UniqueSDModule};
use dlopen::wrapper::{Container, WrapperApi};
use dlopen_derive::WrapperApi;
use streamduck_core::core::button::Button;
use streamduck_core::core::methods::{CoreHandle, warn_for_feature};
use streamduck_core::modules::components::{ComponentDefinition, UIValue};
use streamduck_core::modules::events::SDEvent;
use streamduck_core::versions::SUPPORTED_FEATURES;
use streamduck_daemon::socket::SocketManager;

#[derive(WrapperApi)]
struct PluginApi {
    get_metadata: extern fn() -> PluginMetadata,
    get_module: extern fn() -> SDModulePointer,
    register: extern fn(socket_manager: Arc<SocketManager>),
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

    fn event(&self, core: CoreHandle, event: SDEvent) {
        if core.check_for_feature("events") {
            self.plugin.event(core, event)
        }
    }

    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }
}

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

pub fn warn_about_essential_features(module: UniqueSDModule) {
    let name = &module.name();
    let features = module.metadata().used_features;

    warn_for_feature(name, &features, "plugin_api");
    warn_for_feature(name, &features, "sdmodule_trait");
}

pub fn load_plugin<T: AsRef<OsStr>>(module_manager: Arc<ModuleManager>, socket_manager: Arc<SocketManager>, path: T) -> Result<(), PluginError> {
    // Loading file as a library, error if cannot load
    let wrapper: Container<PluginApi> = unsafe { Container::load(path) }?;

    // Retrieving metadata and comparing versions
    let metadata = wrapper.get_metadata();
    compare_plugin_versions(&metadata.used_features)?;

    // Running register function, so plugin can hook their listeners to various parts of the daemon
    wrapper.register(socket_manager);

    // Attempting to get module from the plugin
    let module: BoxedSDModule = unsafe { Box::from_raw(wrapper.get_module()) };

    // Wrapping plugin's module into a wrapper that contains loaded library
    let module_proxy: UniqueSDModule = Arc::new(Box::new(PluginProxy { wrapper, metadata, plugin: module }));

    // Warn plugin if metadata doesn't contain essential plugins
    warn_about_essential_features(module_proxy.clone());

    // Adding module if it wasn't defined before
    if module_manager.get_module(&module_proxy.name()).is_none() {
        // TODO: Check for component name conflicts
        module_manager.add_module(module_proxy);
        Ok(())
    } else {
        Err(PluginError::AlreadyExists(module_proxy.name()))
    }
}

pub fn load_plugins_from_folder<T: AsRef<OsStr>>(module_manager: Arc<ModuleManager>, socket_manager: Arc<SocketManager>, path: T) {
    let path = Path::new(&path);
    if let Ok(read_dir) = fs::read_dir(path) {
        for item in read_dir {
            match item {
                Ok(entry) => {
                    if entry.path().is_file() {
                        if let Some(file_name) = entry.path().file_name() {
                            log::info!("Loading plugin {:?}", file_name);
                            match load_plugin(module_manager.clone(), socket_manager.clone(), entry.path()) {
                                Err(err) => match err {
                                    PluginError::LoadError(err) => log::error!("Failed to load plugin: {}", err),
                                    PluginError::WrongVersion(plugin, software) => log::error!("Failed to load plugin: Plugin is using unsupported version of '{}', software's using '{}'", plugin, software),
                                    PluginError::TooNew(version) => log::error!("Failed to load plugin: Software doesn't support '{}', try updating the software", version),
                                    PluginError::AlreadyExists(name) => log::error!("Failed to load plugin: Module '{}' was already defined", name),
                                },
                                _ => {}
                            }
                        }
                    }
                }
                Err(err) => log::error!("Failed to reach entry. {}", err),
            }
        }
    } else {
        log::error!("Plugins folder is unreachable: {:?}", path);
    }
}

#[derive(Debug)]
pub enum PluginError {
    LoadError(dlopen::Error),
    WrongVersion(String, String),
    TooNew(String),
    AlreadyExists(String)
}

impl From<dlopen::Error> for PluginError {
    fn from(err: Error) -> Self {
        PluginError::LoadError(err)
    }
}