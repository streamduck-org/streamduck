use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use serde_json::{Error, Value};
use crate::core::button::Button;
use crate::core::{ButtonPanel, RawButtonPanel, UniqueButton};

/// Rendering utilities
pub mod rendering;

/// Wraps button in Arc and RwLock, but packed into a more convenient function
pub fn make_button_unique(button: Button) -> UniqueButton {
    Arc::new(RwLock::new(button))
}

/// Parses button panel to Value, serializing all the unique buttons in process
pub fn serialize_panel(panel: ButtonPanel) -> Result<Value, Error> {
    let mut buttons = HashMap::new();

    for (key, button) in panel {
        buttons.insert(key, button_to_raw(&button));
    }

    serialize_panel_raw(buttons)
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
    Ok(serde_json::from_value::<HashMap<u8, Button>>(value)?)
}

/// Converts raw button panel into button panel
pub fn make_panel_unique(raw_panel: RawButtonPanel) -> ButtonPanel {
    raw_panel.into_iter().map(|(key, button)| (key, make_button_unique(button))).collect()
}


/// Converts button panel to raw button panel
pub fn panel_to_raw(panel: &ButtonPanel) -> RawButtonPanel {
    panel.iter().map(|(k, b)| (*k, b.read().unwrap().deref().clone())).collect()
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