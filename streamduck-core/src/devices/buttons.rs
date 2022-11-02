use serde::{Serialize, Deserialize};

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