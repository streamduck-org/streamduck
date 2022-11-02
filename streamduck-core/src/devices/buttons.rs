use serde::{Serialize, Deserialize};

/// Button position
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct ButtonPosition {
    /// Row of the button
    pub row: u16,

    /// Column of the button / Number of the button in the row
    pub column: u16
}