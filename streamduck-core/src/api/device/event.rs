use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceEvent {
    InputEvent {
        input: usize,
        name: String,
        value: serde_json::Value
    },
    DeviceLayoutChanged
}