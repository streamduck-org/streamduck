use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::api::value_type::ValueType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSetting {
    pub name: String,
    pub description: String,
    pub ty: ValueType,
    pub value: Value
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrippedDeviceSetting {
    pub name: String,
    pub value: Value
}

impl From<DeviceSetting> for StrippedDeviceSetting {
    fn from(value: DeviceSetting) -> Self {
        StrippedDeviceSetting {
            name: value.name,
            value: value.value
        }
    }
}