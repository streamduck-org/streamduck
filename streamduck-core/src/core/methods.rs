use std::ops::DerefMut;
use std::sync::{Arc, LockResult, MutexGuard};
use serde_json::{Map, Value};
use crate::core::{ButtonPanel, UniqueButton};
use crate::{ModuleManager, SDCore};
use crate::util::{button_to_raw, deserialize_panel, make_button_unique, panel_to_raw, serialize_panel};
use serde::de::Error as DeError;
use serde_json::Error as JSONError;
use crate::modules::events::SDEvent;
use crate::modules::{features_to_vec, UniqueSDModule};
use crate::modules::components::UIValue;
use crate::threads::streamdeck::StreamDeckCommand;
use crate::versions::SUPPORTED_FEATURES;

/// Handle that's given out to a module to perform actions on the core
#[derive(Clone)]
pub struct CoreHandle {
    pub(crate) core: Arc<SDCore>,
    pub(crate) module_name: String,
    pub(crate) module_features: Vec<(String, String)>,
}

/// Checks if slice of features contains a specific feature
pub fn check_feature_list_for_feature(features: &Vec<(String, String)>, feature: &str) -> bool {
    for (feat, _) in features {
        if *feat == feature {
            return true;
        }
    }

    false
}

/// Warns if slice of features doesn't contain a specific feature
pub fn warn_for_feature(module_name: &str, features: &Vec<(String, String)>, feature: &str) {
    if !check_feature_list_for_feature(features, feature) {
        log::warn!("Module '{}' is using unreported feature '{}', please add the feature into plugin metadata to prevent any future crashes due to version incompatibility", module_name, feature);
    }
}

impl CoreHandle {
    /// Wraps core reference with a handle, used for all core features to be able to bypass feature checking
    pub fn wrap(core: Arc<SDCore>) -> CoreHandle {
        CoreHandle {
            core,
            module_name: "-system-".to_string(),
            module_features: features_to_vec(SUPPORTED_FEATURES)
        }
    }

    /// Checks if module is allowed to use this feature
    pub fn check_for_feature(&self, feature: &str) -> bool {
        check_feature_list_for_feature(&self.module_features, feature)
    }

    /// Warns if module is using feature it hasn't reported
    pub fn required_feature(&self, feature: &str) {
        warn_for_feature(&self.module_name, &self.module_features, feature)
    }

    /// Clones the handle for specified module
    pub fn clone_for(&self, module: &UniqueSDModule) -> CoreHandle {
        CoreHandle {
            core: self.core.clone(),
            module_name: module.name(),
            module_features: module.metadata().used_features
        }
    }

    /// Returns core reference
    pub fn core(&self) -> Arc<SDCore> {
        self.required_feature("core");
        self.core.clone()
    }

    /// Returns module manager reference
    pub fn module_manager(&self) -> Arc<ModuleManager> {
        self.required_feature("module_manager");
        self.core.module_manager.clone()
    }

    /// Returns current stack lock
    pub fn current_stack(&self) -> LockResult<MutexGuard<'_, Vec<ButtonPanel>>> {
        self.required_feature("core");
        self.core.current_stack.lock()
    }

}

/// Returns a button from current screen on specified position
pub fn get_button(core: &CoreHandle, key: u8) -> Option<UniqueButton> {
    core.required_feature("core_methods");
    if let Some(screen) = get_current_screen(core) {
        screen.get(&key).cloned()
    } else {
        None
    }
}

/// Sets button to current screen with specified position
pub fn set_button(core: &CoreHandle, key: u8, button: UniqueButton) -> bool {
    core.required_feature("core_methods");
    if let Some(mut screen) = get_current_screen(core) {
        let previous_button = screen.get(&key).cloned();

        screen.insert(key, button.clone());

        replace_screen(core, screen.clone());

        if let Some(previous_button) = previous_button {
            for module in core.module_manager().get_module_list() {
                if module.name() == core.module_name {
                    continue;
                }

                module.event(core.clone_for(&module), SDEvent::ButtonUpdated {
                    key,
                    panel: screen.clone(),
                    new_button: button.clone(),
                    old_button: previous_button.clone()
                });
            }
        } else {
            for module in core.module_manager().get_module_list() {
                if module.name() == core.module_name {
                    continue;
                }

                module.event(core.clone_for(&module), SDEvent::ButtonAdded {
                    key,
                    panel: screen.clone(),
                    added_button: button.clone()
                });
            }
        }

        true
    } else {
        false
    }
}

/// Clears button from current screen on specified position
pub fn clear_button(core: &CoreHandle, key: u8) -> bool {
    core.required_feature("core_methods");
    if let Some(mut screen) = get_current_screen(core) {
        if let Some(button) = screen.remove(&key) {
            replace_screen(core, screen.clone());

            for module in core.module_manager().get_module_list() {
                if module.name() == core.module_name {
                    continue;
                }

                module.event(core.clone_for(&module), SDEvent::ButtonDeleted {
                    key,
                    panel: screen.clone(),
                    deleted_button: button.clone()
                });
            }

            true
        } else {
            false
        }
    } else {
        false
    }
}

pub fn add_component(core: &CoreHandle, key: u8, component_name: &str) -> bool {
    core.required_feature("core_methods");
    if let Some(screen) = get_current_screen(&core) {
        if let Some(button) = screen.get(&key) {
            let previous = make_button_unique(button_to_raw(button));

            let mut button_handle = button.write().unwrap();

            if !button_handle.component_names().contains(&component_name.to_string()) {
                let components = core.module_manager().get_components_list_by_modules();

                for (module, component_list) in components {
                    for (component, _) in component_list {
                        if component == component_name {
                            let module = core.module_manager().get_module(&module).unwrap();

                            module.add_component(core.clone_for(&module), button_handle.deref_mut(), &component);

                            drop(button_handle);

                            for module in core.module_manager().get_module_list() {
                                if module.name() == core.module_name {
                                    continue;
                                }

                                module.event(core.clone_for(&module), SDEvent::ButtonUpdated {
                                    key,
                                    panel: screen.clone(),
                                    new_button: button.clone(),
                                    old_button: previous.clone()
                                });
                            }

                            return true;
                        }
                    }
                }
            }
        }
    }

    false
}

pub fn get_component_values(core: &CoreHandle, key: u8, component_name: &str) -> Option<Vec<UIValue>> {
    core.required_feature("core_methods");
    if let Some(screen) = get_current_screen(&core) {
        if let Some(button) = screen.get(&key) {
            let mut button_handle = button.write().unwrap();

            if button_handle.component_names().contains(&component_name.to_string()) {
                let components = core.module_manager().get_components_list_by_modules();

                for (module, component_list) in components {
                    for (component, _) in component_list {
                        if component == component_name {
                            let module = core.module_manager().get_module(&module).unwrap();

                            return Some(module.component_values(core.clone_for(&module), button_handle.deref_mut(), &component));
                        }
                    }
                }
            }
        }
    }

    None
}

pub fn set_component_value(core: &CoreHandle, key: u8, component_name: &str, value: Vec<UIValue>) -> bool {
    core.required_feature("core_methods");
    if let Some(screen) = get_current_screen(&core) {
        if let Some(button) = screen.get(&key) {
            let previous = make_button_unique(button_to_raw(button));

            let mut button_handle = button.write().unwrap();

            if button_handle.component_names().contains(&component_name.to_string()) {
                let components = core.module_manager().get_components_list_by_modules();

                for (module, component_list) in components {
                    for (component, _) in component_list {
                        if component == component_name {
                            let module = core.module_manager().get_module(&module).unwrap();
                            module.set_component_value(core.clone_for(&module), button_handle.deref_mut(), &component, value);
                            drop(button_handle);

                            for module in core.module_manager().get_module_list() {
                                if module.name() == core.module_name {
                                    continue;
                                }

                                module.event(core.clone_for(&module), SDEvent::ButtonUpdated {
                                    key,
                                    panel: screen.clone(),
                                    new_button: button.clone(),
                                    old_button: previous.clone()
                                });
                            }

                            return true;
                        }
                    }
                }
            }
        }
    }

    false
}

pub fn remove_component(core: &CoreHandle, key: u8, component_name: &str) -> bool {
    core.required_feature("core_methods");
    if let Some(screen) = get_current_screen(&core) {
        if let Some(button) = screen.get(&key) {
            let previous = make_button_unique(button_to_raw(button));

            let mut button_handle = button.write().unwrap();

            if button_handle.component_names().contains(&component_name.to_string()) {
                let components = core.module_manager().get_components_list_by_modules();

                for (module, component_list) in components {
                    for (component, _) in component_list {
                        if component == component_name {
                            let module = core.module_manager().get_module(&module).unwrap();

                            module.remove_component(core.clone_for(&module), button_handle.deref_mut(), &component);

                            drop(button_handle);

                            for module in core.module_manager().get_module_list() {
                                if module.name() == core.module_name {
                                    continue;
                                }

                                module.event(core.clone_for(&module), SDEvent::ButtonUpdated {
                                    key,
                                    panel: screen.clone(),
                                    new_button: button.clone(),
                                    old_button: previous.clone()
                                });
                            }

                            return true;
                        }
                    }
                }
            }
        }
    }

    false
}

/// Pushes new panel into the stack
pub fn push_screen(core: &CoreHandle, screen: ButtonPanel) {
    core.required_feature("core_methods");
    let mut stack = core.current_stack().unwrap();

    stack.push(screen.clone());
    drop(stack);

    for module in core.module_manager().get_module_list() {
        if module.name() == core.module_name {
            continue;
        }

        module.event(core.clone_for(&module), SDEvent::PanelPushed {
            new_panel: screen.clone()
        });
    }

    core.core().mark_for_redraw();
}

/// Pops panel from stack
pub fn pop_screen(core: &CoreHandle) {
    core.required_feature("core_methods");
    let mut stack = core.current_stack().unwrap();

    let old_panel = stack.pop();
    drop(stack);

    if let Some(old_panel) = old_panel {
        for module in core.module_manager().get_module_list() {
            if module.name() == core.module_name {
                continue;
            }

            module.event(core.clone_for(&module), SDEvent::PanelPopped {
                popped_panel: old_panel.clone()
            })
        }
    }

    core.core().mark_for_redraw();
}

/// Returns first panel of the stack for saving purposes
pub fn save_panels(core: &CoreHandle) -> ButtonPanel {
    core.required_feature("core_methods");
    let stack = core.current_stack().unwrap();
    stack.get(0).unwrap().clone()
}

/// Returns first panel of the stack that's already been serialized
pub fn save_panels_to_value(core: &CoreHandle) -> Value {
    core.required_feature("core_methods");
    let stack = core.current_stack().unwrap();

    if let Some(panel) = stack.get(0) {
        let serialized_panel = serialize_panel(panel.clone()).unwrap();
        serde_json::to_value(&serialized_panel).unwrap()
    } else {
        Value::Object(Map::new())
    }
}

/// Clears the stack and loads provided panel into the stack
pub fn load_panels(core: &CoreHandle, panel: ButtonPanel) {
    core.required_feature("core_methods");
    let mut stack = core.current_stack().unwrap();

    stack.clear();
    stack.push(panel.clone());
    drop(stack);

    for module in core.module_manager().get_module_list() {
        if module.name() == core.module_name {
            continue;
        }

        module.event(core.clone_for(&module), SDEvent::StackReset {
            new_panel: panel.clone()
        });
    }

    core.core().mark_for_redraw();
}

/// Clears the stack, attempts to deserialize provided panel value into an actual panel and then pushes it into the stack
pub fn load_panels_from_value(core: &CoreHandle, panels: Value) -> Result<(), JSONError> {
    core.required_feature("core_methods");
    match deserialize_panel(panels) {
        Ok(panel) => {
            let mut stack = core.current_stack().unwrap();

            stack.clear();
            stack.push(panel.clone());
            drop(stack);

            for module in core.module_manager().get_module_list() {
                if module.name() == core.module_name {
                    continue;
                }

                module.event(core.clone_for(&module), SDEvent::StackReset {
                    new_panel: panel.clone()
                });
            }

            core.core().mark_for_redraw();

            Ok(())
        }
        Err(err) => {
            Err(DeError::custom(format!("Failed to load panels: {}", err)))
        }
    }
}

/// Triggers button down event on all modules
pub fn button_down(core: &CoreHandle, key: u8) {
    core.required_feature("core_methods");
    for module in core.module_manager().get_module_list() {
        if module.name() == core.module_name {
            continue;
        }

        module.event(core.clone_for(&module), SDEvent::ButtonDown {
            key
        })
    }
}

/// Triggers button up event on all modules
pub fn button_up(core: &CoreHandle, key: u8) {
    core.required_feature("core_methods");
    for module in core.module_manager().get_module_list() {
        if module.name() == core.module_name {
            continue;
        }

        module.event(core.clone_for(&module), SDEvent::ButtonUp {
            key
        })
    }

    button_action(core, key);
}

/// Triggers button action event for modules that are related to components of the button
pub fn button_action(core: &CoreHandle, key: u8) {
    core.required_feature("core_methods");
    if let Some(screen) = get_current_screen(core) {
        if let Some(button) = screen.get(&key) {
            for module in core.module_manager().get_modules_for_components(button.read().unwrap().component_names().as_slice()) {
                if module.name() == core.module_name {
                    continue;
                }

                module.event(core.clone_for(&module), SDEvent::ButtonAction {
                    key,
                    panel: screen.clone(),
                    pressed_button: button.clone()
                })
            }
        }
    }
}

/// Gets current panel stack
pub fn get_stack(core: &CoreHandle) -> Vec<ButtonPanel> {
    core.required_feature("core_methods");
    let stack = core.current_stack().unwrap();

    stack.iter().map(|x| x.clone()).collect()
}

/// Gets panel that's currently on top of the stack
pub fn get_current_screen(core: &CoreHandle) -> Option<ButtonPanel> {
    core.required_feature("core_methods");
    let stack = core.current_stack().unwrap();

    if let Some(screen) = stack.last() {
        Some(screen.clone())
    } else {
        None
    }
}

/// Replaces current screen with specified one
pub fn replace_screen(core: &CoreHandle, screen: ButtonPanel) {
    core.required_feature("core_methods");
    let mut stack = core.current_stack().unwrap();
    stack.pop();
    stack.push(screen);
    core.core().mark_for_redraw();
}

/// Sets brightness of the streamdeck to specified (Range from 0 to 100)
pub fn set_brightness(core: &CoreHandle, brightness: u8) {
    core.required_feature("core_methods");
    core.core().send_commands(vec![StreamDeckCommand::SetBrightness(brightness)]);

    let core = core.core();
    let mut handle = core.device_config.write().unwrap();
    handle.brightness = brightness;
}

/// Commits all changes to layout to device config so it can be later saved
pub fn commit_changes(core: &CoreHandle) {
    core.required_feature("core_methods");
    let stack = save_panels(core);

    let core = core.core();
    let mut handle = core.device_config.write().unwrap();
    handle.layout = panel_to_raw(&stack);
}