use std::error::Error;
use std::fmt::{Display, Formatter};
use serde::{Serialize, Deserialize};
use crate::localization::LocalizedString;

#[derive(Serialize, Deserialize, Clone, Debug)]
/// Dynamic parameter that can be used with devices, plugins, etc. to display options for the user
pub struct Parameter {
    /// Name of the parameter
    pub name: String,
    /// Display name of the parameter
    pub display_name: LocalizedString,
    /// Description of the parameter
    pub description: LocalizedString,
    /// Path to the field, handled by the type itself
    path: String,
    /// Parameter variant
    pub variant: ParameterVariant,
}

fn append_path(name: &str, variant: &mut ParameterVariant, first: bool) {
    match variant {
        ParameterVariant::CollapsableMenu(m) => {
            for param in m {
                param.path = format!("{name}.{}", param.path);

                append_path(name, &mut param.variant, false)
            }
        }

        ParameterVariant::Array(a) => {
            for (_i, element) in a.into_iter().enumerate() {
                for param in element {
                    param.path = if first {
                        format!("{name}.{_i}.{}", param.path)
                    } else {
                        format!("{name}.{}", param.path)
                    };

                    append_path(name, &mut param.variant, false)
                }
            }
        }

        _ => {}
    }
}

impl Parameter {
    /// Creates new parameter with separate localized strings for display name and description
    pub fn new(
        name: &str,
        display_name: LocalizedString,
        description: LocalizedString,
        mut variant: ParameterVariant
    ) -> Parameter {
        append_path(name, &mut variant, true);

        Parameter {
            name: name.to_string(),
            display_name,
            description,
            path: name.to_string(),
            variant
        }
    }

    /// Creates new parameter with localized strings derived from localization key
    ///
    /// display_name = localization_key + ".name"
    ///
    /// description = localization_key + ".desc"
    pub fn new_from_key(name: &str, localization_key: &str, variant: ParameterVariant) -> Parameter {
        let display_name = format!("{localization_key}.name");
        let description = format!("{localization_key}.desc");

        Self::new(
            name,
            LocalizedString::new(&display_name),
            LocalizedString::new(&description),
            variant
        )
    }

    /// Retrieves path of the parameter
    pub fn path(&self) -> String {
        self.path.clone()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
/// Parameter variant
pub enum ParameterVariant {
    /// Displays a header block with display name and description
    Header,
    /// Displays text that cannot be changed
    Label(String),
    /// Has nested parameters hidden behind a dropdown
    CollapsableMenu(Vec<Parameter>),
    /// Has array of elements which contain nested parameters
    Array(Vec<Vec<Parameter>>),
    /// Displays text input field
    TextInput {
        /// If the field should appear disabled
        disabled: bool,
        /// Value for the text input field
        value: String
    },
    /// Displays integer input field
    IntegerInput {
        /// If the field should appear disabled
        disabled: bool,
        /// Value for the integer input field
        value: i64
    },
    /// Displays real number input field
    NumberInput {
        /// If the field should appear disabled
        disabled: bool,
        /// Value for the real number input field
        value: f64
    }
}

/// All possible errors that can happen with parameters
#[derive(Debug)]
pub enum ParameterError {
    /// Enum contained wrong variant for this operation
    WrongVariant,
    /// Data provided to the parameter was invalid
    InvalidData,
}

impl Display for ParameterError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ParameterError {}

impl From<i64> for ParameterVariant {
    fn from(i: i64) -> Self {
        Self::IntegerInput {
            disabled: false,
            value: i
        }
    }
}

impl From<f64> for ParameterVariant {
    fn from(f: f64) -> Self {
        Self::NumberInput {
            disabled: false,
            value: f
        }
    }
}

impl From<String> for ParameterVariant {
    fn from(s: String) -> Self {
        Self::TextInput {
            disabled: false,
            value: s
        }
    }
}

impl TryFrom<ParameterVariant> for i64 {
    type Error = ParameterError;

    fn try_from(value: ParameterVariant) -> Result<Self, Self::Error> {
        if let ParameterVariant::IntegerInput {value, ..} = value {
            Ok(value)
        } else {
            Err(ParameterError::WrongVariant)
        }
    }
}

impl TryFrom<ParameterVariant> for f64 {
    type Error = ParameterError;

    fn try_from(value: ParameterVariant) -> Result<Self, Self::Error> {
        if let ParameterVariant::NumberInput {value, ..} = value {
            Ok(value)
        } else {
            Err(ParameterError::WrongVariant)
        }
    }
}

impl TryFrom<ParameterVariant> for String {
    type Error = ParameterError;

    fn try_from(value: ParameterVariant) -> Result<Self, Self::Error> {
        match value {
            ParameterVariant::Label(value) |
            ParameterVariant::TextInput { value, .. } => {
                Ok(value)
            }

            _ => {
                Err(ParameterError::WrongVariant)
            }
        }
    }
}

/// Shared behavior for handling parameters, mainly should be used by macros
pub trait ParameterImpl {
    /// Lists all available parameters
    fn list_parameters(&self) -> Vec<Parameter>;
    /// Gets parameter by path
    fn get_parameter(&self, path: &str) -> Option<Parameter>;
    /// Sets parameter's value
    fn set_parameter(&mut self, path: &str, value: ParameterVariant) -> Result<(), ParameterError>;
    /// Adds new element to an array
    fn add_element(&mut self, path: &str) -> Result<(), ParameterError>;
    /// Removes element from an array
    fn remove_element(&mut self, path: &str, index: usize) -> Result<(), ParameterError>;
    /// Moves element in the array
    fn move_element(&mut self, path: &str, from_index: usize, to_index: usize) -> Result<(), ParameterError>;
}