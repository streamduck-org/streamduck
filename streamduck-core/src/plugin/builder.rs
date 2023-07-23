use std::collections::HashMap;
use std::sync::Arc;
use crate::config::SharedConfig;
use crate::core::{Action, ActionImpl, Overlay, OverlayImpl};
use crate::data::Options;
use crate::device::{Driver, DriverImpl};
use crate::plugin::{Plugin, PluginHolder, SharedPlugin};
use crate::ui::UISchema;

/// Builds Plugin structs for easier filling out of all the information
pub struct PluginBuilder {
    config: SharedConfig,
    name: String,
    overlays: HashMap<String, OverlayTemplate>,
    actions: HashMap<String, ActionTemplate>,
    drivers: HashMap<String, DriverTemplate>,
    holder: Option<Arc<dyn PluginHolder>>
}

impl PluginBuilder {
    /// Creates a new plugin builder
    pub fn new(name: &str, config: SharedConfig) -> PluginBuilder {
        PluginBuilder {
            config,
            name: name.to_string(),
            overlays: Default::default(),
            actions: Default::default(),
            drivers: Default::default(),
            holder: None,
        }
    }

    /// Adds overlay to the map
    pub fn add_overlay(mut self, overlay_name: &str, implementation: impl OverlayImpl + 'static, ui: UISchema) -> Self {
        let overlay = OverlayTemplate {
            implement: Arc::new(implementation),
            ui,
        };

        self.overlays.insert(overlay_name.to_string(), overlay);

        self
    }

    /// Adds action to the map
    pub fn add_action(mut self, action_name: &str, implementation: impl ActionImpl + 'static, ui: UISchema) -> Self {
        let action = ActionTemplate {
            implement: Arc::new(implementation),
            ui,
        };

        self.actions.insert(action_name.to_string(), action);

        self
    }

    /// Adds driver to the map
    pub fn add_driver(mut self, driver_name: &str, implementation: impl DriverImpl + 'static, options: Options) -> Self {
        let driver = DriverTemplate {
            implement: Arc::new(implementation),
            options,
        };

        self.drivers.insert(driver_name.to_string(), driver);

        self
    }

    /// Sets plugin holder
    pub fn holder(mut self, holder: impl PluginHolder + 'static) -> Self {
        self.holder = Some(Arc::new(holder));

        self
    }

    /// Builds the plugin struct
    pub fn build(self) -> SharedPlugin {
        let plugin = Arc::new(Plugin {
            config: self.config.clone(),
            name: self.name,
            busy: Default::default(),
            overlays: Default::default(),
            actions: Default::default(),
            drivers: Default::default(),
            holder: self.holder.unwrap_or_else(|| Arc::new(EmptyHolder)),
        });

        let weak_plugin = Arc::downgrade(&plugin);

        plugin.overlays.set(self.overlays.into_iter()
            .map(|(name, template)| {
                (name.clone(), Arc::new(Overlay {
                    config: self.config.clone(),
                    original_plugin: weak_plugin.clone(),
                    name: plugin.new_name(&name),
                    implement: template.implement,
                    ui: template.ui,
                }))
            })
            .collect()).ok();

        plugin.actions.set(self.actions.into_iter()
            .map(|(name, template)| {
                (name.clone(), Arc::new(Action {
                    config: self.config.clone(),
                    original_plugin: weak_plugin.clone(),
                    name: plugin.new_name(&name),
                    implement: template.implement,
                    ui: template.ui,
                }))
            })
            .collect()).ok();

        plugin.drivers.set(self.drivers.into_iter()
            .map(|(name, template)| {
                (name.clone(), Arc::new(Driver {
                    config: self.config.clone(),
                    original_plugin: weak_plugin.clone(),
                    name: plugin.new_name(&name),
                    implement: template.implement,
                    options: template.options,
                }))
            })
            .collect()).ok();

        plugin
    }
}

struct OverlayTemplate {
    implement: Arc<dyn OverlayImpl>,
    ui: UISchema
}

struct ActionTemplate {
    implement: Arc<dyn ActionImpl>,
    ui: UISchema
}

struct DriverTemplate {
    implement: Arc<dyn DriverImpl>,
    options: Options
}

/// Represents a PluginHolder that doesn't hold anything
pub struct EmptyHolder;
impl PluginHolder for EmptyHolder {}