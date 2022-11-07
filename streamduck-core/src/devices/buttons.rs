use serde::{Serialize, Deserialize};
use crate::events::Event;

/// Button position
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct ButtonPosition {
    /// Row of the button
    pub row: u16,

    /// Column of the button / Number of the button in the row
    pub column: u16
}

impl From<(u16, u16)> for ButtonPosition {
    fn from(tuple: (u16, u16)) -> Self {
        Self {
            row: tuple.0,
            column: tuple.1
        }
    }
}

/// Button events
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ButtonEvent {
    /// Button was pressed down
    ButtonDown(ButtonPosition),
    /// Button was released
    ButtonUp(ButtonPosition)
}

impl Event for ButtonEvent {
    fn name() -> String {
        "button_event".to_string()
    }
}