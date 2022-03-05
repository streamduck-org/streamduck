use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use serde_json::{Error, Value};
use crate::core::button::Button;
use crate::core::{ButtonPanel, Panel, RawButtonPanel, UniqueButton, UniqueButtonMap};

/// Rendering utilities
pub mod rendering;

/// Wraps button in Arc and RwLock, but packed into a more convenient function
pub fn make_button_unique(button: Button) -> UniqueButton {
    Arc::new(RwLock::new(button))
}

/// Parses button panel to Value, serializing all the unique buttons in process
pub fn serialize_panel(panel: ButtonPanel) -> Result<Value, Error> {
    let panel = panel_to_raw(&panel);

    serialize_panel_raw(panel)
}

/// Serializes raw button panel to Value in more understandable function
pub fn serialize_panel_raw(panel: RawButtonPanel) -> Result<Value, Error> {
    serde_json::to_value(panel)
}

/// Parses value into a button panel
pub fn deserialize_panel(value: Value) -> Result<ButtonPanel, Error> {
    Ok(make_panel_unique(deserialize_panel_raw(value)?))
}

/// Parses value into a raw button panel
pub fn deserialize_panel_raw(value: Value) -> Result<RawButtonPanel, Error> {
    Ok(serde_json::from_value(value)?)
}

/// Converts raw button panel into button panel
pub fn make_panel_unique(raw_panel: RawButtonPanel) -> ButtonPanel {
    Arc::new(RwLock::new(
        Panel::<UniqueButtonMap> {
            display_name: raw_panel.display_name,
            data: raw_panel.data,
            buttons: raw_panel.buttons.into_iter().map(|(key, button)| (key, make_button_unique(button))).collect()
        }
    ))
}


/// Converts button panel to raw button panel
pub fn panel_to_raw(panel: &ButtonPanel) -> RawButtonPanel {
    let handle = panel.read().unwrap();
    let panel = (*handle).clone();
    drop(handle);

    RawButtonPanel {
        display_name: panel.display_name,
        data: panel.data,
        buttons: panel.buttons.into_iter().map(|(key, button)| (key, button_to_raw(&button))).collect()
    }
}

/// Converts unique button to raw button
pub fn button_to_raw(button: &UniqueButton) -> Button {
    button.read().unwrap().deref().clone()
}

/// Hashes image blob
pub fn hash_image(data: &String) -> String {
    let mut hasher = DefaultHasher::new();

    data.hash(&mut hasher);

    hasher.finish().to_string()
}