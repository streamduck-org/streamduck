use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use crate::thread::rendering::{Color, RendererComponent};

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
    /// Name of the field
    pub name: String,

    /// Display name of the field
    pub display_name: String,

    /// Description of the field
    pub description: String,

    /// Type of the field
    pub ty: UIFieldType,

    /// Default value that will be used when initializing this field
    pub default_value: UIFieldValue<UIValue>
}

/// UI Value, represents what fields currently have
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UIValue {
    /// Name of the value
    pub name: String,

    /// Display name of the value
    pub display_name: String,

    /// Description of the value
    pub description: String,

    /// Type of the value
    pub ty: UIFieldType,

    /// Actual value
    pub value: UIFieldValue<UIValue>,
}

/// UI Path Value, represents a value that has a path inside of the value hierarchy
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UIPathValue {
    /// Name of the value
    pub name: String,

    /// Path of the value
    pub path: String,

    /// Display name of the value
    pub display_name: String,

    /// Description of the value
    pub description: String,

    /// Type of the value
    pub ty: UIFieldType,

    /// Actual value
    pub value: UIFieldValue<UIPathValue>,
}

/// UI Field Types, defines types that fields will have
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum UIFieldType {
    /// Displays a header for separation reasons
    Header,

    /// Displays text
    Label,

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
    Collapsable,

    /// Array of menus, this definition acts as a template of how to construct the array, each field will be duplicated for each item in the array like structs
    Array(Vec<UIField>),

    /// Choice dropdown
    Choice(Vec<String>),

    /// Checkbox
    Checkbox {
        /// If checkbox should appear disabled in UI
        ///
        /// Note: Doesn't actually prevent setting the checkbox, so do internal checks if value should be changed
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

    /// Button for receiving impulses from clients
    Button {
        /// If button should appear disabled
        disabled: bool
    },

    /// Previews an image on UI
    ImagePreview,
}

/// UI Field value, current state of the settings
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum UIFieldValue<V> {
    /// Displays a header for separation reasons
    Header,

    /// Displays text
    Label(String),

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
    Collapsable(Vec<V>),

    /// Array of menus
    Array(Vec<Vec<V>>),

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

    /// Button
    Button,

    /// Previews an image on UI, png image data encoded in base64
    ImagePreview(String)
}

impl<V> UIFieldValue<V> {
    /// Attempts to parse the value into a boolean
    pub fn try_into_bool(&self) -> Result<bool, String> {
        TryInto::<bool>::try_into(self)
    }

    /// Attempts to parse the value into a floating number
    pub fn try_into_f32(&self) -> Result<f32, String> {
        TryInto::<f32>::try_into(self)
    }

    /// Attempts to parse the value into an integer
    pub fn try_into_i32(&self) -> Result<i32, String> {
        TryInto::<i32>::try_into(self)
    }

    /// Attempts to parse the value into an unsigned integer
    pub fn try_into_u32(&self) -> Result<u32, String> {
        TryInto::<u32>::try_into(self)
    }

    /// Attempts to parse the value into a pair of floating numbers
    pub fn try_into_f32_f32(&self) -> Result<(f32, f32), String> {
        TryInto::<(f32, f32)>::try_into(self)
    }

    /// Attempts to parse the value into a pair of integers
    pub fn try_into_i32_i32(&self) -> Result<(i32, i32), String> {
        TryInto::<(i32, i32)>::try_into(self)
    }

    /// Attempts to parse the value into a color
    pub fn try_into_color(&self) -> Result<Color, String> {
        TryInto::<Color>::try_into(self)
    }

    /// Attempts to parse the value into a string
    pub fn try_into_string(&self) -> Result<String, String> {
        TryInto::<String>::try_into(self)
    }
}

/// From conversions
impl<V> Into<UIFieldValue<V>> for Color {
    fn into(self) -> UIFieldValue<V> {
        UIFieldValue::Color(self.0, self.1, self.2, self.3)
    }
}

impl<V> Into<UIFieldValue<V>> for &Color {
    fn into(self) -> UIFieldValue<V> {
        UIFieldValue::Color(self.0, self.1, self.2, self.3)
    }
}

/// To conversions
impl<V> TryInto<bool> for UIFieldValue<V> {
    type Error = String;

    fn try_into(self) -> Result<bool, Self::Error> {
        if let UIFieldValue::Checkbox(b) = self {
            Ok(b)
        } else {
            Err("Incorrect value".to_string())
        }
    }
}

impl<V> TryInto<bool> for &UIFieldValue<V> {
    type Error = String;

    fn try_into(self) -> Result<bool, Self::Error> {
        if let UIFieldValue::Checkbox(b) = self {
            Ok(*b)
        } else {
            Err("Incorrect value".to_string())
        }
    }
}

impl<V> TryInto<f32> for UIFieldValue<V> {
    type Error = String;

    fn try_into(self) -> Result<f32, Self::Error> {
        if let UIFieldValue::InputFieldFloat(f) | UIFieldValue::ValueSliderFloat(f) = self {
            Ok(f)
        } else {
            Err("Incorrect value".to_string())
        }
    }
}

impl<V> TryInto<f32> for &UIFieldValue<V> {
    type Error = String;

    fn try_into(self) -> Result<f32, Self::Error> {
        if let UIFieldValue::InputFieldFloat(f) | UIFieldValue::ValueSliderFloat(f) = self {
            Ok(*f)
        } else {
            Err("Incorrect value".to_string())
        }
    }
}

impl<V> TryInto<i32> for UIFieldValue<V> {
    type Error = String;

    fn try_into(self) -> Result<i32, Self::Error> {
        if let UIFieldValue::InputFieldInteger(i) | UIFieldValue::ValueSliderInteger(i) = self {
            Ok(i)
        } else {
            Err("Incorrect value".to_string())
        }
    }
}

impl<V> TryInto<i32> for &UIFieldValue<V> {
    type Error = String;

    fn try_into(self) -> Result<i32, Self::Error> {
        if let UIFieldValue::InputFieldInteger(i) | UIFieldValue::ValueSliderInteger(i) = self {
            Ok(*i)
        } else {
            Err("Incorrect value".to_string())
        }
    }
}

impl<V> TryInto<u32> for UIFieldValue<V> {
    type Error = String;

    fn try_into(self) -> Result<u32, Self::Error> {
        if let UIFieldValue::InputFieldUnsignedInteger(u) = self {
            Ok(u)
        } else {
            Err("Incorrect value".to_string())
        }
    }
}

impl<V> TryInto<u32> for &UIFieldValue<V> {
    type Error = String;

    fn try_into(self) -> Result<u32, Self::Error> {
        if let UIFieldValue::InputFieldUnsignedInteger(u) = self {
            Ok(*u)
        } else {
            Err("Incorrect value".to_string())
        }
    }
}

impl<V> TryInto<(f32, f32)> for UIFieldValue<V> {
    type Error = String;

    fn try_into(self) -> Result<(f32, f32), Self::Error> {
        if let UIFieldValue::InputFieldFloat2(f1, f2) = self {
            Ok((f1, f2))
        } else {
            Err("Incorrect value".to_string())
        }
    }
}

impl<V> TryInto<(f32, f32)> for &UIFieldValue<V> {
    type Error = String;

    fn try_into(self) -> Result<(f32, f32), Self::Error> {
        if let UIFieldValue::InputFieldFloat2(f1, f2) = self {
            Ok((*f1, *f2))
        } else {
            Err("Incorrect value".to_string())
        }
    }
}

impl<V> TryInto<(i32, i32)> for UIFieldValue<V> {
    type Error = String;

    fn try_into(self) -> Result<(i32, i32), Self::Error> {
        if let UIFieldValue::InputFieldInteger2(i1, i2) = self {
            Ok((i1, i2))
        } else {
            Err("Incorrect value".to_string())
        }
    }
}

impl<V> TryInto<(i32, i32)> for &UIFieldValue<V> {
    type Error = String;

    fn try_into(self) -> Result<(i32, i32), Self::Error> {
        if let UIFieldValue::InputFieldInteger2(i1, i2) = self {
            Ok((*i1, *i2))
        } else {
            Err("Incorrect value".to_string())
        }
    }
}

impl<V> TryInto<Color> for UIFieldValue<V> {
    type Error = String;

    fn try_into(self) -> Result<Color, Self::Error> {
        if let UIFieldValue::Color(c1, c2, c3, c4) = self {
            Ok((c1, c2, c3, c4))
        } else {
            Err("Incorrect value".to_string())
        }
    }
}

impl<V> TryInto<Color> for &UIFieldValue<V> {
    type Error = String;

    fn try_into(self) -> Result<Color, Self::Error> {
        if let UIFieldValue::Color(c1, c2, c3, c4) = self {
            Ok((*c1, *c2, *c3, *c4))
        } else {
            Err("Incorrect value".to_string())
        }
    }
}

impl<V> TryInto<String> for UIFieldValue<V> {
    type Error = String;

    fn try_into(self) -> Result<String, Self::Error> {
        if let UIFieldValue::InputFieldString(str) |
                UIFieldValue::Choice(str) |
                UIFieldValue::ImageData(str) |
                UIFieldValue::ExistingImage(str) |
                UIFieldValue::Font(str) |
                UIFieldValue::Label(str) |
                UIFieldValue::ImagePreview(str) = self {
            Ok(str)
        } else {
            Err("Incorrect value".to_string())
        }
    }
}

impl<V> TryInto<String> for &UIFieldValue<V> {
    type Error = String;

    fn try_into(self) -> Result<String, Self::Error> {
        if let UIFieldValue::InputFieldString(str) |
                UIFieldValue::Choice(str) |
                UIFieldValue::ImageData(str) |
                UIFieldValue::ExistingImage(str) |
                UIFieldValue::Font(str) |
                UIFieldValue::Label(str) |
                UIFieldValue::ImagePreview(str) = self {
            Ok(str.clone())
        } else {
            Err("Incorrect value".to_string())
        }
    }
}

impl<V> TryInto<PathBuf> for UIFieldValue<V> {
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

impl<V> TryInto<PathBuf> for &UIFieldValue<V> {
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

impl From<UIFieldValue<UIValue>> for UIFieldValue<UIPathValue> {
    fn from(val: UIFieldValue<UIValue>) -> Self {
        match val {
            UIFieldValue::Header => UIFieldValue::Header,
            UIFieldValue::Label(s) => UIFieldValue::Label(s),
            UIFieldValue::InputFieldFloat(f) => UIFieldValue::InputFieldFloat(f),
            UIFieldValue::InputFieldInteger(i) => UIFieldValue::InputFieldInteger(i),
            UIFieldValue::InputFieldString(s) => UIFieldValue::InputFieldString(s),
            UIFieldValue::InputFieldFloat2(f1, f2) => UIFieldValue::InputFieldFloat2(f1, f2),
            UIFieldValue::InputFieldInteger2(i1, i2) => UIFieldValue::InputFieldInteger2(i1, i2),
            UIFieldValue::InputFieldUnsignedInteger(u) => UIFieldValue::InputFieldUnsignedInteger(u),
            UIFieldValue::ValueSliderFloat(f) => UIFieldValue::ValueSliderFloat(f),
            UIFieldValue::ValueSliderInteger(i) => UIFieldValue::ValueSliderInteger(i),

            UIFieldValue::Collapsable(_) => {
                panic!("Please use convert_value_to_path")
            }

            UIFieldValue::Array(_) => {
                panic!("Please use convert_value_to_path")
            }

            UIFieldValue::Choice(c) => UIFieldValue::Choice(c),
            UIFieldValue::Checkbox(b) => UIFieldValue::Checkbox(b),
            UIFieldValue::Color(c1, c2, c3, c4) => UIFieldValue::Color(c1, c2, c3, c4),
            UIFieldValue::ImageData(d) => UIFieldValue::ImageData(d),
            UIFieldValue::ExistingImage(i) => UIFieldValue::ExistingImage(i),
            UIFieldValue::Font(f) => UIFieldValue::Font(f),
            UIFieldValue::Button => UIFieldValue::Button,
            UIFieldValue::ImagePreview(d) => UIFieldValue::ImagePreview(d)
        }
    }
}

impl From<UIFieldValue<UIPathValue>> for UIFieldValue<UIValue> {
    fn from(val: UIFieldValue<UIPathValue>) -> Self {
        match val {
            UIFieldValue::Header => UIFieldValue::Header,
            UIFieldValue::Label(s) => UIFieldValue::Label(s),
            UIFieldValue::InputFieldFloat(f) => UIFieldValue::InputFieldFloat(f),
            UIFieldValue::InputFieldInteger(i) => UIFieldValue::InputFieldInteger(i),
            UIFieldValue::InputFieldString(s) => UIFieldValue::InputFieldString(s),
            UIFieldValue::InputFieldFloat2(f1, f2) => UIFieldValue::InputFieldFloat2(f1, f2),
            UIFieldValue::InputFieldInteger2(i1, i2) => UIFieldValue::InputFieldInteger2(i1, i2),
            UIFieldValue::InputFieldUnsignedInteger(u) => UIFieldValue::InputFieldUnsignedInteger(u),
            UIFieldValue::ValueSliderFloat(f) => UIFieldValue::ValueSliderFloat(f),
            UIFieldValue::ValueSliderInteger(i) => UIFieldValue::ValueSliderInteger(i),

            UIFieldValue::Collapsable(m) => {
                UIFieldValue::Collapsable(m.into_iter()
                    .map(|x| UIValue {
                        name: x.name,
                        display_name: x.display_name,
                        description: x.description,
                        ty: x.ty,
                        value: x.value.into()
                    })
                    .collect())
            }

            UIFieldValue::Array(a) => {
                UIFieldValue::Array(a.into_iter()
                    .map(|x| x.into_iter()
                        .map(|x| UIValue {
                            name: x.name,
                            display_name: x.display_name,
                            description: x.description,
                            ty: x.ty,
                            value: x.value.into()
                        })
                        .collect())
                    .collect())
            }

            UIFieldValue::Choice(c) => UIFieldValue::Choice(c),
            UIFieldValue::Checkbox(b) => UIFieldValue::Checkbox(b),
            UIFieldValue::Color(c1, c2, c3, c4) => UIFieldValue::Color(c1, c2, c3, c4),
            UIFieldValue::ImageData(d) => UIFieldValue::ImageData(d),
            UIFieldValue::ExistingImage(i) => UIFieldValue::ExistingImage(i),
            UIFieldValue::Font(f) => UIFieldValue::Font(f),
            UIFieldValue::Button => UIFieldValue::Button,
            UIFieldValue::ImagePreview(d) => UIFieldValue::ImagePreview(d)
        }
    }
}

impl From<UIPathValue> for UIValue {
    fn from(value: UIPathValue) -> Self {
        UIValue {
            name: value.name,
            display_name: value.display_name,
            description: value.description,
            ty: value.ty,
            value: value.value.into()
        }
    }
}

/// Information for running sliders in UI
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UIScalar<T: PartialEq> {
    /// Maximum value for the slider
    pub max_value: T,
    /// Minimum value for the slider
    pub min_value: T,
    /// How precise the slider will be in UI
    pub step: T,
    /// To allow manually inputting values outside of the range
    pub allow_out_of_bounds: bool
}

/// Converts array of values to map
pub fn map_ui_values(values: Vec<UIValue>) -> HashMap<String, UIValue> {
    values.into_iter()
        .map(|x| (x.name.clone(), x))
        .collect()
}

/// Converts reference to array of values to map
pub fn map_ui_values_ref(values: &Vec<UIValue>) -> HashMap<String, UIValue> {
    values.into_iter()
        .map(|x| (x.name.clone(), x.clone()))
        .collect()
}

/// Converts reference to array of path values to map
pub fn map_ui_path_values(values: &Vec<UIPathValue>) -> HashMap<String, UIPathValue> {
    let mut map = HashMap::new();

    fn add_values_to_map(values: &Vec<UIPathValue>, map: &mut HashMap<String, UIPathValue>) {
        values.into_iter()
            .for_each(|x| {
                match &x.value {
                    UIFieldValue::Collapsable(m) => {
                        add_values_to_map(m, map);
                    }

                    UIFieldValue::Array(a) => {
                        a.into_iter()
                            .for_each(|x| add_values_to_map(x, map))
                    }

                    _ => {
                        map.insert(x.path.to_string(), x.clone());
                    }
                }
            })
    }

    add_values_to_map(values, &mut map);

    return map;
}