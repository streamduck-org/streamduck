use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use serde::{Serialize, Deserialize};
use crate::threads::rendering::{Color, RendererComponent};

/// Component definition
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ComponentDefinition {
    /// Display name for the component
    pub display_name: String,

    /// Description of the component
    pub description: String,

    /// Default looks for a button, in case user doesn't want to setup one on their own
    pub default_looks: RendererComponent
}

/// UI Field, will be represented in a list similar to Unity's inspector
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UIField {
    pub name: String,
    pub display_name: String,
    pub ty: UIFieldType,
    pub default_value: UIFieldValue
}

/// UI Value, represents what fields currently have
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UIValue {
    pub name: String,
    pub display_name: String,
    pub ty: UIFieldType,
    pub value: UIFieldValue,
}

/// UI Field Types, defines types that fields will have
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum UIFieldType {
    /// Displays a header for separation reasons
    Header,

    /// Text field that accepts float values
    InputFieldFloat,
    /// Text field that accepts integer values
    InputFieldInteger,
    /// Text field that accepts strings
    InputFieldString,

    /// Text field that accepts 2 float values
    InputFieldFloat2,

    /// Text field that accepts 2 integer values
    InputFieldInteger2,

    /// Text field that accepts only positive integer values
    InputFieldUnsignedInteger,

    // TODO: Add more types of inputs

    /// Float slider of specified bounds
    ValueSliderFloat(UIScalar<f32>),
    /// Integer slider of specified bounds
    ValueSliderInteger(UIScalar<i32>),

    /// Collapsable submenu
    Collapsable(Vec<UIField>),

    /// Array of menus, this definition acts as a template of how to construct the array, each field will be duplicated for each item in the array like structs
    Array(Vec<UIField>),

    /// Choice dropdown
    Choice(Vec<String>),

    /// Checkbox
    Checkbox {
        disabled: bool
    },

    /// Color picker
    Color,

    /// Image data encoded in base64
    ImageData,

    /// Image from image collection
    ExistingImage,

    /// Font name
    Font,
}

/// UI Field value, current state of the settings
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum UIFieldValue {
    /// Displays a header for separation reasons
    Header,

    /// Text field that accepts float values
    InputFieldFloat(f32),
    /// Text field that accepts integer values
    InputFieldInteger(i32),
    /// Text field that accepts strings
    InputFieldString(String),

    /// Text field that accepts 2 float values
    InputFieldFloat2(f32, f32),

    /// Text field that accepts 2 integer values
    InputFieldInteger2(i32, i32),

    /// Text field that accepts only positive integer values
    InputFieldUnsignedInteger(u32),

    /// Float slider of specified bounds
    ValueSliderFloat(f32),
    /// Integer slider of specified bounds
    ValueSliderInteger(i32),

    /// Collapsable submenu
    Collapsable(Vec<UIValue>),

    /// Array of menus
    Array(Vec<Vec<UIValue>>),

    /// Choice dropdown
    Choice(String),

    /// Checkbox
    Checkbox(bool),

    /// Color picker
    Color(u8, u8, u8, u8),

    /// Image data encoded in base64
    ImageData(String),

    /// Image from image collection
    ExistingImage(String),

    /// Font name
    Font(String),
}

impl UIFieldValue {
    pub fn try_into_bool(&self) -> Result<bool, String> {
        TryInto::<bool>::try_into(self)
    }

    pub fn try_into_f32(&self) -> Result<f32, String> {
        TryInto::<f32>::try_into(self)
    }

    pub fn try_into_i32(&self) -> Result<i32, String> {
        TryInto::<i32>::try_into(self)
    }

    pub fn try_into_u32(&self) -> Result<u32, String> {
        TryInto::<u32>::try_into(self)
    }

    pub fn try_into_f32_f32(&self) -> Result<(f32, f32), String> {
        TryInto::<(f32, f32)>::try_into(self)
    }

    pub fn try_into_i32_i32(&self) -> Result<(i32, i32), String> {
        TryInto::<(i32, i32)>::try_into(self)
    }

    pub fn try_into_color(&self) -> Result<Color, String> {
        TryInto::<Color>::try_into(self)
    }

    pub fn try_into_string(&self) -> Result<String, String> {
        TryInto::<String>::try_into(self)
    }
}

/// From conversions
impl Into<UIFieldValue> for Color {
    fn into(self) -> UIFieldValue {
        UIFieldValue::Color(self.0, self.1, self.2, self.3)
    }
}

impl Into<UIFieldValue> for &Color {
    fn into(self) -> UIFieldValue {
        UIFieldValue::Color(self.0, self.1, self.2, self.3)
    }
}

/// To conversions
impl TryInto<bool> for UIFieldValue {
    type Error = String;

    fn try_into(self) -> Result<bool, Self::Error> {
        if let UIFieldValue::Checkbox(b) = self {
            Ok(b)
        } else {
            Err("Incorrect value".to_string())
        }
    }
}

impl TryInto<bool> for &UIFieldValue {
    type Error = String;

    fn try_into(self) -> Result<bool, Self::Error> {
        if let UIFieldValue::Checkbox(b) = self {
            Ok(*b)
        } else {
            Err("Incorrect value".to_string())
        }
    }
}

impl TryInto<f32> for UIFieldValue {
    type Error = String;

    fn try_into(self) -> Result<f32, Self::Error> {
        if let UIFieldValue::InputFieldFloat(f) | UIFieldValue::ValueSliderFloat(f) = self {
            Ok(f)
        } else {
            Err("Incorrect value".to_string())
        }
    }
}

impl TryInto<f32> for &UIFieldValue {
    type Error = String;

    fn try_into(self) -> Result<f32, Self::Error> {
        if let UIFieldValue::InputFieldFloat(f) | UIFieldValue::ValueSliderFloat(f) = self {
            Ok(*f)
        } else {
            Err("Incorrect value".to_string())
        }
    }
}

impl TryInto<i32> for UIFieldValue {
    type Error = String;

    fn try_into(self) -> Result<i32, Self::Error> {
        if let UIFieldValue::InputFieldInteger(i) | UIFieldValue::ValueSliderInteger(i) = self {
            Ok(i)
        } else {
            Err("Incorrect value".to_string())
        }
    }
}

impl TryInto<i32> for &UIFieldValue {
    type Error = String;

    fn try_into(self) -> Result<i32, Self::Error> {
        if let UIFieldValue::InputFieldInteger(i) | UIFieldValue::ValueSliderInteger(i) = self {
            Ok(*i)
        } else {
            Err("Incorrect value".to_string())
        }
    }
}

impl TryInto<u32> for UIFieldValue {
    type Error = String;

    fn try_into(self) -> Result<u32, Self::Error> {
        if let UIFieldValue::InputFieldUnsignedInteger(u) = self {
            Ok(u)
        } else {
            Err("Incorrect value".to_string())
        }
    }
}

impl TryInto<u32> for &UIFieldValue {
    type Error = String;

    fn try_into(self) -> Result<u32, Self::Error> {
        if let UIFieldValue::InputFieldUnsignedInteger(u) = self {
            Ok(*u)
        } else {
            Err("Incorrect value".to_string())
        }
    }
}

impl TryInto<(f32, f32)> for UIFieldValue {
    type Error = String;

    fn try_into(self) -> Result<(f32, f32), Self::Error> {
        if let UIFieldValue::InputFieldFloat2(f1, f2) = self {
            Ok((f1, f2))
        } else {
            Err("Incorrect value".to_string())
        }
    }
}

impl TryInto<(f32, f32)> for &UIFieldValue {
    type Error = String;

    fn try_into(self) -> Result<(f32, f32), Self::Error> {
        if let UIFieldValue::InputFieldFloat2(f1, f2) = self {
            Ok((*f1, *f2))
        } else {
            Err("Incorrect value".to_string())
        }
    }
}

impl TryInto<(i32, i32)> for UIFieldValue {
    type Error = String;

    fn try_into(self) -> Result<(i32, i32), Self::Error> {
        if let UIFieldValue::InputFieldInteger2(i1, i2) = self {
            Ok((i1, i2))
        } else {
            Err("Incorrect value".to_string())
        }
    }
}

impl TryInto<(i32, i32)> for &UIFieldValue {
    type Error = String;

    fn try_into(self) -> Result<(i32, i32), Self::Error> {
        if let UIFieldValue::InputFieldInteger2(i1, i2) = self {
            Ok((*i1, *i2))
        } else {
            Err("Incorrect value".to_string())
        }
    }
}

impl TryInto<Color> for UIFieldValue {
    type Error = String;

    fn try_into(self) -> Result<Color, Self::Error> {
        if let UIFieldValue::Color(c1, c2, c3, c4) = self {
            Ok((c1, c2, c3, c4))
        } else {
            Err("Incorrect value".to_string())
        }
    }
}

impl TryInto<Color> for &UIFieldValue {
    type Error = String;

    fn try_into(self) -> Result<Color, Self::Error> {
        if let UIFieldValue::Color(c1, c2, c3, c4) = self {
            Ok((*c1, *c2, *c3, *c4))
        } else {
            Err("Incorrect value".to_string())
        }
    }
}

impl TryInto<String> for UIFieldValue {
    type Error = String;

    fn try_into(self) -> Result<String, Self::Error> {
        if let UIFieldValue::InputFieldString(str) | UIFieldValue::Choice(str) | UIFieldValue::ImageData(str) | UIFieldValue::ExistingImage(str) | UIFieldValue::Font(str) = self {
            Ok(str)
        } else {
            Err("Incorrect value".to_string())
        }
    }
}

impl TryInto<String> for &UIFieldValue {
    type Error = String;

    fn try_into(self) -> Result<String, Self::Error> {
        if let UIFieldValue::InputFieldString(str) | UIFieldValue::Choice(str) | UIFieldValue::ImageData(str) | UIFieldValue::ExistingImage(str) | UIFieldValue::Font(str) = self {
            Ok(str.clone())
        } else {
            Err("Incorrect value".to_string())
        }
    }
}

impl TryInto<PathBuf> for UIFieldValue {
    type Error = String;

    fn try_into(self) -> Result<PathBuf, Self::Error> {
        if let UIFieldValue::InputFieldString(str) | UIFieldValue::Choice(str) = self {
            if let Ok(path) = PathBuf::from_str(&str) {
                Ok(path)
            } else {
                Err("Failed to parse path".to_string())
            }
        } else {
            Err("Incorrect enum value".to_string())
        }
    }
}

impl TryInto<PathBuf> for &UIFieldValue {
    type Error = String;

    fn try_into(self) -> Result<PathBuf, Self::Error> {
        if let UIFieldValue::InputFieldString(str) | UIFieldValue::Choice(str) = self {
            if let Ok(path) = PathBuf::from_str(str) {
                Ok(path)
            } else {
                Err("Failed to parse path".to_string())
            }
        } else {
            Err("Incorrect value".to_string())
        }
    }
}

/// Information for running sliders in UI
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UIScalar<T: PartialEq> {
    /// Default value slider will have
    pub default_value: T,
    /// Maximum value for the slider
    pub max_value: T,
    /// Minimum value for the slider
    pub min_value: T,
    /// How precise the slider will be in UI
    pub step: T,
    /// To allow manually inputting values outside of the range
    pub allow_out_of_bounds: bool
}

pub fn map_ui_values(values: Vec<UIValue>) -> HashMap<String, UIValue> {
    values.into_iter()
        .map(|x| (x.name.clone(), x))
        .collect()
}

pub fn map_ui_values_ref(values: &Vec<UIValue>) -> HashMap<String, UIValue> {
    values.into_iter()
        .map(|x| (x.name.clone(), x.clone()))
        .collect()
}