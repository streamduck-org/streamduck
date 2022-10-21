use crate::core::{ButtonPanel, RawButtonPanel, UniqueButton};
use serde::{Serialize, Deserialize};
use crate::core::button::Button;
use crate::util::{button_to_raw, panel_to_raw};

/// Core event enumeration for events related to specific cores, needs to be converted to [SDGlobalEvent] to be serialized
#[derive(Clone, Debug)]
pub enum SDCoreEvent {
    /// Called when a new button is created on current screen
    ButtonAdded {
        /// Key index
        key: u8,
        /// Current panel
        panel: ButtonPanel,
        /// Button that was added
        added_button: UniqueButton
    },
    /// Called when a button gets updated or overridden with another button
    ButtonUpdated {
        /// Key index
        key: u8,
        /// Current panel
        panel: ButtonPanel,
        /// New version of the button
        new_button: UniqueButton,
        /// Old version of the button
        old_button: UniqueButton
    },
    /// Called when a button gets deleted
    ButtonDeleted {
        /// Key index
        key: u8,
        /// Current panel
        panel: ButtonPanel,
        /// Button that was deleted
        deleted_button: UniqueButton
    },

    /// Called when a valid button was pressed on
    ButtonAction {
        /// Key index
        key: u8,
        /// Current panel
        panel: ButtonPanel,
        /// Button that was pressed
        pressed_button: UniqueButton
    },

    /// Called when a button is pressed down
    ButtonDown {
        /// Key index
        key: u8
    },
    /// Called when a button is released
    ButtonUp {
        /// Key index
        key: u8
    },

    /// Called when a new panel gets pushed into the stack
    PanelPushed {
        /// Panel that was pushed into the stack
        new_panel: ButtonPanel
    },
    /// Called when panel gets popped from the stack
    PanelPopped {
        /// Panel that was popped from the stack
        popped_panel: ButtonPanel
    },
    /// Called when panel gets replaced with different one
    PanelReplaced {
        /// Old panel that used to be on the screen
        old_panel: Option<ButtonPanel>,
        /// New panel that was put on the screen
        new_panel: ButtonPanel
    },
    /// Called when stack gets cleared and set with a root panel
    StackReset {
        /// New root panel
        new_panel: ButtonPanel
    },
}

/// Global event enumeration for events that are related to whole program, serializable
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SDGlobalEvent {
    /// Called when a new button is created on a screen
    ButtonAdded {
        /// Serial number of the device
        serial_number: String,
        /// Key index
        key: u8,
        /// Current panel
        panel: RawButtonPanel,
        /// Button that was added
        added_button: Button
    },
    /// Called when a button gets updated or overridden with another button
    ButtonUpdated {
        /// Serial number of the device
        serial_number: String,
        /// Key index
        key: u8,
        /// Current panel
        panel: RawButtonPanel,
        /// New version of the button
        new_button: Button,
        /// Old version of the button
        old_button: Button
    },
    /// Called when a button gets deleted
    ButtonDeleted {
        /// Serial number of the device
        serial_number: String,
        /// Key index
        key: u8,
        /// Current panel
        panel: RawButtonPanel,
        /// Button that was deleted
        deleted_button: Button
    },

    /// Called when a valid button was pressed on
    ButtonAction {
        /// Serial number of the device
        serial_number: String,
        /// Key index
        key: u8,
        /// Current panel
        panel: RawButtonPanel,
        /// Button that was pressed
        pressed_button: Button
    },

    /// Called when a button is pressed down
    ButtonDown {
        /// Serial number of the device
        serial_number: String,
        /// Key index
        key: u8,
    },
    /// Called when a button is released
    ButtonUp {
        /// Serial number of the device
        serial_number: String,
        /// Key index
        key: u8,
    },

    /// Called when a new panel gets pushed into the stack
    PanelPushed {
        /// Serial number of the device
        serial_number: String,
        /// Panel that was pushed to the stack
        new_panel: RawButtonPanel
    },
    /// Called when panel gets popped from the stack
    PanelPopped {
        /// Serial number of the device
        serial_number: String,
        /// Panel that was popped from the stack
        popped_panel: RawButtonPanel
    },
    /// Called when panel gets replaced with different one
    PanelReplaced {
        /// Serial number of the device
        serial_number: String,
        /// Old panel that used to be on the screen
        old_panel: Option<RawButtonPanel>,
        /// New panel that was put on the screen
        new_panel: RawButtonPanel
    },
    /// Called when stack gets cleared and set with a root panel
    StackReset {
        /// Serial number of the device
        serial_number: String,
        /// New root panel
        new_panel: RawButtonPanel
    },

    /// Called when device has connected
    DeviceConnected {
        /// Serial number of the device
        serial_number: String
    },

    /// Called when device has disconnected
    DeviceDisconnected {
        /// Serial number of the device
        serial_number: String
    },
}

/// Converts [SDCoreEvent] to [SDGlobalEvent] by adding serial number
pub async fn core_event_to_global(event: SDCoreEvent, serial: &str) -> SDGlobalEvent {
    let serial_number = serial.to_string();
    match event {
        SDCoreEvent::ButtonAdded { key, panel, added_button } => SDGlobalEvent::ButtonAdded {
            serial_number,
            key,
            panel: panel_to_raw(&panel).await,
            added_button: button_to_raw(&added_button).await,
        },

        SDCoreEvent::ButtonUpdated { key, panel, new_button, old_button } => SDGlobalEvent::ButtonUpdated {
            serial_number,
            key,
            panel: panel_to_raw(&panel).await,
            new_button: button_to_raw(&new_button).await,
            old_button: button_to_raw(&old_button).await,
        },

        SDCoreEvent::ButtonDeleted { key, panel, deleted_button } => SDGlobalEvent::ButtonDeleted {
            serial_number,
            key,
            panel: panel_to_raw(&panel).await,
            deleted_button: button_to_raw(&deleted_button).await,
        },

        SDCoreEvent::ButtonAction { key, panel, pressed_button } => SDGlobalEvent::ButtonAction {
            serial_number,
            key,
            panel: panel_to_raw(&panel).await,
            pressed_button: button_to_raw(&pressed_button).await,
        },

        SDCoreEvent::ButtonDown { key } => SDGlobalEvent::ButtonDown {
            serial_number,
            key,
        },

        SDCoreEvent::ButtonUp { key } => SDGlobalEvent::ButtonUp {
            serial_number,
            key,
        },

        SDCoreEvent::PanelPushed { new_panel } => SDGlobalEvent::PanelPushed {
            serial_number,
            new_panel: panel_to_raw(&new_panel).await,
        },

        SDCoreEvent::PanelPopped { popped_panel } => SDGlobalEvent::PanelPopped {
            serial_number,
            popped_panel: panel_to_raw(&popped_panel).await,
        },

        SDCoreEvent::PanelReplaced { old_panel, new_panel } => SDGlobalEvent::PanelReplaced {
            serial_number,
            old_panel: if let Some(panel) = old_panel { Some(panel_to_raw(&panel).await) } else { None },
            new_panel: panel_to_raw(&new_panel).await,
        },

        SDCoreEvent::StackReset { new_panel } => SDGlobalEvent::StackReset {
            serial_number,
            new_panel: panel_to_raw(&new_panel).await,
        }
    }
}