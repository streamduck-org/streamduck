use crate::core::{ButtonPanel, RawButtonPanel, UniqueButton};
use serde::{Serialize, Deserialize};
use crate::core::button::Button;
use crate::util::{button_to_raw, panel_to_raw};

/// Core event enumeration for events related to specific cores, needs to be converted to [SDGlobalEvent] to be serialized
#[derive(Clone, Debug)]
pub enum SDCoreEvent {
    /// Called when a new button is created on current screen
    ButtonAdded {key: u8, panel: ButtonPanel, added_button: UniqueButton},
    /// Called when a button gets updated or overridden with another button
    ButtonUpdated {key: u8, panel: ButtonPanel, new_button: UniqueButton, old_button: UniqueButton},
    /// Called when a button gets deleted
    ButtonDeleted {key: u8, panel: ButtonPanel, deleted_button: UniqueButton},

    /// Called when a valid button was pressed on
    ButtonAction {key: u8, panel: ButtonPanel, pressed_button: UniqueButton},

    /// Called when a button is pressed down
    ButtonDown {key: u8},
    /// Called when a button is released
    ButtonUp {key: u8},

    /// Called when a new panel gets pushed into the stack
    PanelPushed {new_panel: ButtonPanel},
    /// Called when panel gets popped from the stack
    PanelPopped {popped_panel: ButtonPanel},
    /// Called when panel gets replaced with different one
    PanelReplaced {old_panel: Option<ButtonPanel>, new_panel: ButtonPanel},
    /// Called when stack gets cleared and set with a root panel
    StackReset {new_panel: ButtonPanel},
}

/// Global event enumeration for events that are related to whole program, serializable
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SDGlobalEvent {
    /// Called when a new button is created on a screen
    ButtonAdded {serial_number: String, key: u8, panel: RawButtonPanel, added_button: Button},
    /// Called when a button gets updated or overridden with another button
    ButtonUpdated {serial_number: String, key: u8, panel: RawButtonPanel, new_button: Button, old_button: Button},
    /// Called when a button gets deleted
    ButtonDeleted {serial_number: String, key: u8, panel: RawButtonPanel, deleted_button: Button},

    /// Called when a valid button was pressed on
    ButtonAction {serial_number: String, key: u8, panel: RawButtonPanel, pressed_button: Button},

    /// Called when a button is pressed down
    ButtonDown {serial_number: String, key: u8},
    /// Called when a button is released
    ButtonUp {serial_number: String, key: u8},

    /// Called when a new panel gets pushed into the stack
    PanelPushed {serial_number: String, new_panel: RawButtonPanel},
    /// Called when panel gets popped from the stack
    PanelPopped {serial_number: String, popped_panel: RawButtonPanel},
    /// Called when panel gets replaced with different one
    PanelReplaced {serial_number: String, old_panel: Option<RawButtonPanel>, new_panel: RawButtonPanel},
    /// Called when stack gets cleared and set with a root panel
    StackReset {serial_number: String, new_panel: RawButtonPanel},

    /// Called when device has connected
    DeviceConnected {serial_number: String},

    /// Called when device has disconnected
    DeviceDisconnected {serial_number: String},
}

pub async fn core_event_to_global(event: SDCoreEvent, serial: &str) -> SDGlobalEvent {
    let serial_number = serial.to_string();
    match event {
        SDCoreEvent::ButtonAdded { key, panel, added_button } => SDGlobalEvent::ButtonAdded {
            serial_number,
            key,
            panel: panel_to_raw(&panel).await,
            added_button: button_to_raw(&added_button).await
        },

        SDCoreEvent::ButtonUpdated { key, panel, new_button, old_button } => SDGlobalEvent::ButtonUpdated {
            serial_number,
            key,
            panel: panel_to_raw(&panel).await,
            new_button: button_to_raw(&new_button).await,
            old_button: button_to_raw(&old_button).await
        },

        SDCoreEvent::ButtonDeleted { key, panel, deleted_button } => SDGlobalEvent::ButtonDeleted {
            serial_number,
            key,
            panel: panel_to_raw(&panel).await,
            deleted_button: button_to_raw(&deleted_button).await
        },

        SDCoreEvent::ButtonAction { key, panel, pressed_button } => SDGlobalEvent::ButtonAction {
            serial_number,
            key,
            panel: panel_to_raw(&panel).await,
            pressed_button: button_to_raw(&pressed_button).await
        },

        SDCoreEvent::ButtonDown { key } => SDGlobalEvent::ButtonDown {
            serial_number,
            key
        },

        SDCoreEvent::ButtonUp { key } => SDGlobalEvent::ButtonUp {
            serial_number,
            key
        },

        SDCoreEvent::PanelPushed { new_panel } => SDGlobalEvent::PanelPushed {
            serial_number,
            new_panel: panel_to_raw(&new_panel).await
        },

        SDCoreEvent::PanelPopped { popped_panel } => SDGlobalEvent::PanelPopped {
            serial_number,
            popped_panel: panel_to_raw(&popped_panel).await
        },

        SDCoreEvent::PanelReplaced { old_panel, new_panel } => SDGlobalEvent::PanelReplaced {
            serial_number,
            old_panel: if let Some(panel) = old_panel { Some(panel_to_raw(&panel).await) } else { None },
            new_panel: panel_to_raw(&new_panel).await
        },

        SDCoreEvent::StackReset { new_panel } => SDGlobalEvent::StackReset {
            serial_number,
            new_panel: panel_to_raw(&new_panel).await
        }
    }
}