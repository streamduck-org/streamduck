use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, LockResult, MutexGuard};
use image::{DynamicImage, Rgba};
use rusttype::Scale;
use serde_json::{Map, Value};
use crate::core::{ButtonPanel, UniqueButton};
use crate::{Config, ModuleManager, SDCore};
use crate::util::{add_array_function, button_to_raw, change_from_path, convert_value_to_path, deserialize_panel, make_button_unique, panel_to_raw, remove_array_function, serialize_panel, set_value_function};
use serde::de::Error as DeError;
use serde_json::Error as JSONError;
use crate::core::button::{Button, parse_unique_button_to_component};
use crate::font::get_font_from_collection;
use crate::modules::events::SDEvent;
use crate::modules::{features_to_vec, UniqueSDModule};
use crate::modules::components::{UIPathValue, UIValue};
use crate::thread::DeviceThreadCommunication;
use crate::thread::rendering::{draw_background, draw_foreground, draw_missing_texture, RendererComponent};
use crate::thread::util::{image_from_solid, render_aligned_text_on_image, TextAlignment};
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

    /// Returns config reference
    pub fn config(&self) -> Arc<Config> {
        self.required_feature("config");
        self.core.config.clone()
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
        let handle = screen.read().unwrap();
        handle.buttons.get(&key).cloned()
    } else {
        None
    }
}

/// Sets button to current screen with specified position
pub fn set_button(core: &CoreHandle, key: u8, button: UniqueButton) -> bool {
    core.required_feature("core_methods");
    if let Some(screen) = get_current_screen(core) {
        let mut handle = screen.write().unwrap();
        let previous_button = handle.buttons.get(&key).cloned();

        handle.buttons.insert(key, button.clone());

        drop(handle);

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

        core.core.mark_for_redraw();

        true
    } else {
        false
    }
}

/// Clears button from current screen on specified position
pub fn clear_button(core: &CoreHandle, key: u8) -> bool {
    core.required_feature("core_methods");
    if let Some(screen) = get_current_screen(core) {
        let mut handle = screen.write().unwrap();
        if let Some(button) = handle.buttons.remove(&key) {
            drop(handle);

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

            core.core.mark_for_redraw();

            true
        } else {
            false
        }
    } else {
        false
    }
}

/// Adds component onto a button, returns success boolean
pub fn add_component(core: &CoreHandle, key: u8, component_name: &str) -> bool {
    core.required_feature("core_methods");

    let module_manager = core.module_manager();

    if let Some(screen) = get_current_screen(&core) {
        let handle = screen.read().unwrap();
        if let Some(button) = handle.buttons.get(&key).cloned() {
            let previous = make_button_unique(button_to_raw(&button));

            let mut button_handle = button.write().unwrap();
            drop(handle);

            if !button_handle.component_names().contains(&component_name.to_string()) {
                let components = module_manager.read_component_map();

                if let Some((_, module)) = components.get(component_name) {
                    module.add_component(core.clone_for(&module), button_handle.deref_mut(), component_name);

                    drop(button_handle);
                    drop(components);

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

                    core.core.mark_for_redraw();

                    return true;
                }
            }
        }
    }

    false
}

/// Gets component values from a component on a button
pub fn get_component_values(core: &CoreHandle, key: u8, component_name: &str) -> Option<Vec<UIValue>> {
    core.required_feature("core_methods");

    let module_manager = core.module_manager();

    if let Some(screen) = get_current_screen(&core) {
        let handle = screen.read().unwrap();
        if let Some(button) = handle.buttons.get(&key).cloned() {
            let mut button_handle = button.write().unwrap();
            drop(handle);

            if button_handle.component_names().contains(&component_name.to_string()) {
                let components = module_manager.read_component_map();

                if let Some((_, module)) = components.get(component_name) {
                    return Some(module.component_values(core.clone_for(&module), button_handle.deref_mut(), component_name));
                }
            }
        }
    }

    None
}

/// Gets component values from component on a button, but with paths for easier interaction with values
pub fn get_component_values_with_paths(core: &CoreHandle, key: u8, component_name: &str) -> Option<Vec<UIPathValue>> {
    if let Some(values) = get_component_values(core, key, component_name) {
        Some(values.into_iter().map(|x| convert_value_to_path(x, "")).collect())
    } else {
        None
    }
}

/// Sets component values based on changes for component on a button
pub fn set_component_value(core: &CoreHandle, key: u8, component_name: &str, value: Vec<UIValue>) -> bool {
    core.required_feature("core_methods");

    let module_manager = core.module_manager();

    if let Some(screen) = get_current_screen(&core) {
        let handle = screen.read().unwrap();
        if let Some(button) = handle.buttons.get(&key).cloned() {
            let previous = make_button_unique(button_to_raw(&button));

            let mut button_handle = button.write().unwrap();
            drop(handle);

            if button_handle.component_names().contains(&component_name.to_string()) {
                let components = module_manager.read_component_map();

                if let Some((_, module)) = components.get(component_name) {
                    module.set_component_value(core.clone_for(&module), button_handle.deref_mut(), component_name, value);
                    drop(button_handle);
                    drop(components);

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

                    core.core.mark_for_redraw();

                    return true;
                }
            }
        }
    }

    false
}

/// Adds new array element to a component value
pub fn add_element_component_value(core: &CoreHandle, key: u8, component_name: &str, path: &str) -> bool {
    if let Some(values) = get_component_values(core, key, component_name) {
        let (changes, success) = change_from_path(path, values, &add_array_function(), false);

        if success {
            if !changes.is_empty() {
                set_component_value(core, key, component_name, changes)
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    }
}

/// Removes element from array in component value
pub fn remove_element_component_value(core: &CoreHandle, key: u8, component_name: &str, path: &str, index: usize) -> bool {
    if let Some(values) = get_component_values(core, key, component_name) {
        let (changes, success) = change_from_path(path, values, &remove_array_function(index), false);

        if success {
            if !changes.is_empty() {
                set_component_value(core, key, component_name, changes)
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    }
}

/// Sets value based on path for component value
pub fn set_component_value_by_path(core: &CoreHandle, key: u8, component_name: &str, value: UIPathValue) -> bool {
    if let Some(values) = get_component_values(core, key, component_name) {
        let (changes, success) = change_from_path(&value.path, values, &set_value_function(value.clone()), false);

        if success {
            if !changes.is_empty() {
                set_component_value(core, key, component_name, changes)
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    }
}

/// Removes component from a button
pub fn remove_component(core: &CoreHandle, key: u8, component_name: &str) -> bool {
    core.required_feature("core_methods");

    let module_manager = core.module_manager();

    if let Some(screen) = get_current_screen(&core) {
        let handle = screen.read().unwrap();
        if let Some(button) = handle.buttons.get(&key).cloned() {
            let previous = make_button_unique(button_to_raw(&button));

            let mut button_handle = button.write().unwrap();
            drop(handle);

            if button_handle.component_names().contains(&component_name.to_string()) {
                let components = module_manager.read_component_map();

                if let Some((_, module)) = components.get(component_name) {
                    module.remove_component(core.clone_for(&module), button_handle.deref_mut(), component_name);

                    drop(button_handle);
                    drop(components);

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

                    core.core.mark_for_redraw();

                    return true;
                }
            }
        }
    }

    false
}

pub fn paste_button(core: &CoreHandle, key: u8, reference_button: &Button) -> bool {
    let mut new_button = Button::new();

    let responsible_modules = core.module_manager().get_modules_for_declared_components(reference_button.component_names().as_slice());
    for module in responsible_modules {
        module.paste_component(core.clone_for(&module), reference_button, &mut new_button);
    }

    println!("resulting button: {:?}", new_button);

    set_button(core, key, make_button_unique(new_button))
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

    core.core.mark_for_redraw();
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

    core.core.mark_for_redraw();
}

/// Returns first panel of the stack for saving purposes
pub fn get_root_screen(core: &CoreHandle) -> ButtonPanel {
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
pub fn reset_stack(core: &CoreHandle, panel: ButtonPanel) {
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

    core.core.mark_for_redraw();
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

            core.core.mark_for_redraw();

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
        let handle = screen.read().unwrap();
        if let Some(button) = handle.buttons.get(&key).cloned() {
            drop(handle);
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

            core.core.mark_for_redraw();
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

pub fn get_button_images(core: &CoreHandle) -> Option<HashMap<u8, DynamicImage>> {
    let missing = draw_missing_texture(core.core.image_size);
    let custom = {
        let size = core.core.image_size;
        let font = get_font_from_collection("default").unwrap();
        let mut frame = image_from_solid(size, Rgba([55, 55, 55, 255]));

        render_aligned_text_on_image(size, &mut frame, font.deref(), "Custom", Scale::uniform(16.0), TextAlignment::Center, 0, (0.0, -8.0), (255, 255, 255, 255));
        render_aligned_text_on_image(size, &mut frame, font.deref(), "Renderer", Scale::uniform(16.0), TextAlignment::Center, 0, (0.0, 8.0), (255, 255, 255, 255));

        frame
    };

    let panel = get_current_screen(core)?;
    let current_screen = panel.read().unwrap();
    let buttons = current_screen.buttons.clone();

    let renderers = core.core.render_manager.read_renderers();

    Some(buttons.into_iter()
        .filter_map(|(key, button)| {
            if let Ok(component) = parse_unique_button_to_component::<RendererComponent>(&button) {
                let modules = core.module_manager().get_modules_for_rendering(&button.read().unwrap().component_names());
                let modules = modules.into_values()
                    .filter(|x| !component.plugin_blacklist.contains(&x.name()))
                    .collect::<Vec<UniqueSDModule>>();

                let image = if component.renderer.is_empty() {
                    draw_foreground(
                        &component,
                        &button,
                        &modules,
                        draw_background(
                            &component,
                            core,
                            &missing
                        ),
                        core
                    )
                } else {
                    if let Some(renderer) = renderers.get(&component.renderer) {
                        if let Some(image) = renderer.representation(key, &button, core) {
                            image
                        } else {
                            custom.clone()
                        }
                    } else {
                        custom.clone()
                    }
                };

                Some((key, image))
            } else {
                None
            }
        })
        .collect())
}

/// Replaces current screen with specified one
pub fn replace_screen(core: &CoreHandle, screen: ButtonPanel) {
    core.required_feature("core_methods");
    let mut stack = core.current_stack().unwrap();
    stack.pop();
    stack.push(screen);
    core.core.mark_for_redraw();
}

/// Sets brightness of the streamdeck to specified (Range from 0 to 100)
pub fn set_brightness(core: &CoreHandle, brightness: u8) {
    core.required_feature("core_methods");
    core.core().send_commands(vec![DeviceThreadCommunication::SetBrightness(brightness)]);

    let core = core.core();
    let mut handle = core.device_config.write().unwrap();
    handle.brightness = brightness;
}

/// Commits all changes to layout to device config so it can be later saved
pub fn commit_changes(core: &CoreHandle) {
    core.required_feature("core_methods");
    let stack = get_root_screen(core);

    let core = core.core();
    let mut handle = core.device_config.write().unwrap();
    handle.layout = panel_to_raw(&stack);
}