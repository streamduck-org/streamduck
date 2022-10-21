//! Plugin API for loading dynamic library files

use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::hash::Hasher;
use std::path::Path;
use std::sync::{Arc, Mutex};
use dlopen::Error;
use crate::modules::{ModuleManager, PluginMetadata, SDModule, UniqueSDModule};
use dlopen::wrapper::{Container, WrapperApi};
use dlopen_derive::WrapperApi;
use image::DynamicImage;
use tokio::task::{JoinError, spawn_blocking};
use crate::core::button::Button;
use crate::core::manager::CoreManager;
use crate::core::{check_feature_list_for_feature, CoreHandle, UniqueButton, warn_for_feature};
use crate::modules::components::{ComponentDefinition, UIValue};
use crate::modules::events::{SDCoreEvent, SDGlobalEvent};
use crate::{Config, RenderingManager};
use crate::socket::{SocketManager, UniqueSocketListener};
use crate::thread::rendering::custom::UniqueRenderer;
use crate::versions::SUPPORTED_FEATURES;

#[derive(WrapperApi)]
struct PluginApi {
    get_metadata: extern fn() -> PluginMetadata,
    register: fn(socket_manager: Arc<PluginSocketManager>, render_manager: Arc<PluginRenderingManager>, module_manager: Arc<PluginModuleManager>),
}

#[allow(dead_code)]
struct PluginProxy {
    pub wrapper: Arc<Container<PluginApi>>,
    pub metadata: PluginMetadata,
    pub plugin: UniqueSDModule
}

#[async_trait]
impl SDModule for PluginProxy {
    fn name(&self) -> String {
        self.plugin.name()
    }

    fn components(&self) -> HashMap<String, ComponentDefinition> {
        self.plugin.components()
    }

    async fn add_component(&self, core: CoreHandle, button: &mut Button, name: &str) {
        self.plugin.add_component(core, button, name).await
    }

    async fn remove_component(&self, core: CoreHandle, button: &mut Button, name: &str) {
        self.plugin.remove_component(core, button, name).await
    }

    async fn paste_component(&self, core: CoreHandle, reference_button: &Button, new_button: &mut Button) {
        self.plugin.paste_component(core, reference_button, new_button).await
    }

    async fn component_values(&self, core: CoreHandle, button: &Button, name: &str) -> Vec<UIValue> {
        self.plugin.component_values(core, button, name).await
    }

    async fn set_component_value(&self, core: CoreHandle, button: &mut Button, name: &str, value: Vec<UIValue>) {
        self.plugin.set_component_value(core, button, name, value).await
    }

    fn listening_for(&self) -> Vec<String> {
        self.plugin.listening_for()
    }

    async fn settings(&self, core: Arc<CoreManager>) -> Vec<UIValue> {
        self.plugin.settings(core).await
    }

    async fn set_setting(&self, core: Arc<CoreManager>, value: Vec<UIValue>) {
        self.plugin.set_setting(core, value).await
    }

    async fn global_event(&self, event: SDGlobalEvent) {
        if check_feature_list_for_feature(&self.metadata.used_features, "global_events") {
            self.plugin.global_event(event).await
        }
    }

    async fn event(&self, core: CoreHandle, event: SDCoreEvent) {
        if core.check_for_feature("core_events") {
            self.plugin.event(core, event).await
        }
    }

    async fn render(&self, core: CoreHandle, button: &UniqueButton, frame: &mut DynamicImage) {
        if core.check_for_feature("rendering") {
            self.plugin.render(core, button, frame).await
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

/// Wrapper of socket manager for plugin initialization to use
pub struct PluginSocketManager {
    socket_manager: Arc<SocketManager>,

    listeners: Arc<Mutex<Vec<UniqueSocketListener>>>
}

impl PluginSocketManager {
    /// Add listener to daemon
    pub fn add_listener(&self, listener: UniqueSocketListener) {
        self.listeners.lock().unwrap().push(listener);
    }

    async fn load_listeners(&self) -> Result<(), JoinError> {
        let listeners = self.listeners.clone();
        let listeners = spawn_blocking(move || listeners.lock().unwrap().clone()).await?;

        for listener in listeners {
            self.socket_manager.add_listener(listener).await;
        }

        Ok(())
    }
}

/// Wrapper of rendering manager for plugin initialization to use
pub struct PluginRenderingManager {
    rendering_manager: Arc<RenderingManager>,

    renderers: Arc<Mutex<Vec<UniqueRenderer>>>
}

impl PluginRenderingManager {
    /// Add renderer to daemon
    pub fn add_renderer(&self, renderer: UniqueRenderer) {
        self.renderers.lock().unwrap().push(renderer);
    }

    async fn load_renderers(&self) -> Result<(), JoinError> {
        let renderers = self.renderers.clone();
        let renderers = spawn_blocking(move || renderers.lock().unwrap().clone()).await?;

        for renderer in renderers {
            self.rendering_manager.add_custom_renderer(renderer).await;
        }

        Ok(())
    }
}


/// Wrapper of module manager for plugin initialization to use
pub struct PluginModuleManager {
    // Things required to register modules in core
    module_manager: Arc<ModuleManager>,
    metadata: PluginMetadata,
    wrapper: Arc<Container<PluginApi>>,

    // Collection that plugin will be writing to
    modules: Arc<Mutex<Vec<UniqueSDModule>>>,
}

impl PluginModuleManager {
    /// Add module to daemon
    pub fn add_module(&self, module: UniqueSDModule) {
        self.modules.lock().unwrap().push(module);
    }

    async fn load_modules(&self) -> Result<(), PluginError> {
        let modules = self.modules.clone();
        let modules = spawn_blocking(move || modules.lock().unwrap().clone()).await?;

        if modules.is_empty() {
            return Err(PluginError::NoModulesFound);
        }

        for module in modules {
            for component in module.components().keys() {
                if self.module_manager.get_component(component).await.is_some() {
                    return Err(PluginError::ComponentConflict(module.name(), component.to_string()))
                }
            }

            self.module_manager.add_module(Arc::new(PluginProxy {
                wrapper: self.wrapper.clone(),
                metadata: self.metadata.clone(),
                plugin: module
            })).await;
        }

        Ok(())
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
fn warn_about_essential_features(meta: &PluginMetadata) {
    let name = &meta.name;
    let features = &meta.used_features;

    warn_for_feature(name, &features, "compiler_version");
    warn_for_feature(name, &features, "plugin_api");
    warn_for_feature(name, &features, "sdmodule_trait");
}

/// Loads a plugin into module manager
pub async fn load_plugin<T: AsRef<OsStr>>(config: Arc<Config>, module_manager: Arc<ModuleManager>, socket_manager: Arc<SocketManager>, render_manager: Arc<RenderingManager>, path: T) -> Result<(), PluginError> {
    // Loading file as a library, error if cannot load
    let wrapper: Container<PluginApi> = unsafe { Container::load(path) }?;

    let wrapper = Arc::new(wrapper);

    // Retrieving metadata and comparing versions
    let metadata = wrapper.get_metadata();

    // Performing checks if enabled
    if config.plugin_compatibility_checks() {
        compare_plugin_versions(&metadata.used_features)?;
    }

    // Warn plugin if metadata doesn't contain essential plugins
    warn_about_essential_features(&metadata);

    // Adding module if it wasn't defined before
    if module_manager.get_module(&metadata.name).await.is_none() {
        let plugin_manager = Arc::new(PluginModuleManager {
            module_manager,
            metadata: metadata.clone(),
            wrapper,
            modules: Default::default()
        });

        let plugin_socket_manager = Arc::new(PluginSocketManager {
                socket_manager,
                listeners: Default::default()
        });

        let plugin_rendering_manager = Arc::new(PluginRenderingManager {
            rendering_manager: render_manager,
            renderers: Default::default()
        });

        // Calling register after all checks were done
        plugin_manager.wrapper.register(plugin_socket_manager.clone(), plugin_rendering_manager.clone(), plugin_manager.clone());

        plugin_manager.load_modules().await?;
        plugin_socket_manager.load_listeners().await?;
        plugin_rendering_manager.load_renderers().await?;

        Ok(())
    } else {
        Err(PluginError::AlreadyExists(metadata.name))
    }
}

/// Loads plugins into module manager from path
pub async fn load_plugins_from_folder<T: AsRef<OsStr>>(config: Arc<Config>, module_manager: Arc<ModuleManager>, socket_manager: Arc<SocketManager>, render_manager: Arc<RenderingManager>, path: T) {
    let path = Path::new(&path);
    match fs::read_dir(path) {
        Ok(read_dir) => {
            for item in read_dir {
                match item {
                    Ok(entry) => {
                        if entry.path().is_file() {
                            if let Some(file_name) = entry.path().file_name() {
                                log::info!("Loading plugin {:?}", file_name);
                                match load_plugin(config.clone(), module_manager.clone(), socket_manager.clone(), render_manager.clone(), entry.path()).await {
                                    Err(err) => match err {
                                        PluginError::LoadError(err) => log::error!("Failed to load plugin: {}", err),
                                        PluginError::WrongVersion(plugin, software) => log::error!("Failed to load plugin: Plugin is using unsupported version of '{}', software's using '{}'", plugin, software),
                                        PluginError::TooNew(version) => log::error!("Failed to load plugin: Software doesn't support '{}', try updating the software", version),
                                        PluginError::AlreadyExists(name) => log::error!("Failed to load plugin: Module '{}' was already defined", name),
                                        PluginError::ComponentConflict(name, component_name) => log::error!("Failed to load plugin: Module '{}' is declaring '{}' component, but it was already previously declared by other module", name, component_name),
                                        PluginError::JoinError(err) => log::error!("Failed to load plugin: {}", err),
                                        PluginError::NoModulesFound => log::error!("Failed to load plugin: No modules found")
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
    /// Plugin didn't register any modules
    NoModulesFound,
    /// Failed to load the plugin library
    LoadError(dlopen::Error),
    /// Plugin uses different version of a feature
    WrongVersion(String, String),
    /// Plugin uses unknown feature for the daemon
    TooNew(String),
    /// Module name already exists
    AlreadyExists(String),
    /// Component with the name was already declared (Soon to be removed due to better naming)
    ComponentConflict(String, String),
    /// Error spawning a blocking task
    JoinError(tokio::task::JoinError)
}

impl From<dlopen::Error> for PluginError {
    fn from(err: Error) -> Self {
        PluginError::LoadError(err)
    }
}

impl From<tokio::task::JoinError> for PluginError {
    fn from(err: JoinError) -> Self {
        PluginError::JoinError(err)
    }
}