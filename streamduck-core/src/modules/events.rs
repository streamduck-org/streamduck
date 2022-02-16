use crate::core::{ButtonPanel, UniqueButton};

/// Event enumeration
#[derive(Clone, Debug)]
pub enum SDEvent {
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
    /// Called when stack gets cleared and set with a root panel
    StackReset {new_panel: ButtonPanel},
}