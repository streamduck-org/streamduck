use std::error::Error;
use std::fmt::{Display, Formatter};
use serde::{Serialize, Deserialize};
use crate::localization::LocalizedString;

#[cfg(feature = "derive")]
#[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
pub use streamduck_derive::ParameterImpl;

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
    },
    /// Displays a checkbox
    Checkbox {
        /// If the field should appear disabled
        disabled: bool,
        /// Checkbox state
        value: bool
    },
    /// Displays a toggle switch
    Toggle {
        /// If the field should appear disabled
        disabled: bool,
        /// Toggle switch state
        value: bool
    },
    /// Displays a dropdown with provided choices
    Choice {
        /// If the field should appear disabled
        disabled: bool,
        /// Possible choices
        choices: Vec<String>,
        /// Current choice
        value: String
    }
}

/// RGBA Color tuple
pub type Color = (u8, u8, u8, u8);

/// Attempts to flatten provided parameter if possible
pub fn flatten_parameter(parameter: Parameter) -> Vec<Parameter> {
    match &parameter.variant {
        ParameterVariant::CollapsableMenu(params) => params.clone(),
        _ => vec![parameter]
    }
}

/// All possible errors that can happen with parameters
#[derive(Debug)]
pub enum ParameterError {
    /// Enum contained wrong parameter variant for this operation
    WrongVariant,
    /// Cannot set value since the parameter is disabled
    ParameterIsDisabled,
    /// Data provided to the parameter was invalid
    InvalidData,
    /// This method was not implemented
    NotImplemented,
    /// Path was not found in the list of the parameters
    NotFound,
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

/// Shared behavior for handling parameters,
/// mainly should be used by macros or custom implementations for types.
pub trait ParameterImpl {
    /// Lists all parameters
    fn parameter(&self, options: ParameterOptions) -> Parameter;
    /// Sets parameter's value
    fn set_parameter(&mut self, options: ParameterOptions, value: Parameter) -> Result<(), ParameterError> { Err(ParameterError::NotImplemented) }
    /// Adds new element to an array
    fn add_element(&mut self, options: ParameterOptions) -> Result<(), ParameterError> { Err(ParameterError::NotImplemented) }
    /// Removes element from an array
    fn remove_element(&mut self, options: ParameterOptions, index: usize) -> Result<(), ParameterError> { Err(ParameterError::NotImplemented) }
    /// Moves element in the array
    fn move_element(&mut self, options: ParameterOptions, from_index: usize, to_index: usize) -> Result<(), ParameterError> { Err(ParameterError::NotImplemented) }
}

/// Additional options that could be asked for upon retrieval of the parameter
pub struct ParameterOptions {
    /// Name that was assigned to the parameter, is empty if the parameter is not part of a struct
    pub name: String,
    /// Display name localization key
    pub display_name: String,
    /// Description localization key
    pub description: String,
    /// If the parameter should be disabled
    pub disabled: bool,
    /// Preferred variant
    pub preferred_variant: PreferredParameterVariant
}

/// Option that tells which kind of variant the parameter should be retrieved as
#[derive(PartialEq)]
pub enum PreferredParameterVariant {
    /// There's no preference on what variant should be returned
    NoPreference,
    /// If choice parameter should be generated with this
    Choice,
    /// If label parameter should be generated with this
    Label,
    /// If text input parameter should be generated
    TextInput,
    /// If toggle should be generated
    Toggle,
    /// If checkbox should be generated
    Checkbox,
}

impl Default for PreferredParameterVariant {
    fn default() -> Self {
        Self::NoPreference
    }
}

/// Type that could be used for ParameterImpl macro to define dynamic choice parameter
pub struct DynamicChoice {
    choices: Vec<String>,
    value: String
}

impl ParameterImpl for i64 {
    fn parameter(&self, options: ParameterOptions) -> Parameter {
        Parameter::new(
            &options.name,
            LocalizedString::new(&options.display_name),
            LocalizedString::new(&options.description),
            ParameterVariant::IntegerInput {
                disabled: options.disabled,
                value: *self
            }
        )
    }

    fn set_parameter(&mut self, options: ParameterOptions, value: Parameter) -> Result<(), ParameterError> {
        if let ParameterVariant::IntegerInput {value, disabled} = value.variant {
            if options.disabled == disabled && disabled {
                Err(ParameterError::ParameterIsDisabled)
            } else {
                *self = value;
                Ok(())
            }
        } else {
            Err(ParameterError::WrongVariant)
        }
    }
}

impl ParameterImpl for f64 {
    fn parameter(&self, options: ParameterOptions) -> Parameter {
        Parameter::new(
            &options.name,
            LocalizedString::new(&options.display_name),
            LocalizedString::new(&options.description),
            ParameterVariant::NumberInput {
                disabled: options.disabled,
                value: *self
            }
        )
    }

    fn set_parameter(&mut self, options: ParameterOptions, value: Parameter) -> Result<(), ParameterError> {
        if let ParameterVariant::NumberInput {value, disabled} = value.variant {
            if options.disabled == disabled && disabled {
                Err(ParameterError::ParameterIsDisabled)
            } else {
                *self = value;
                Ok(())
            }
        } else {
            Err(ParameterError::WrongVariant)
        }
    }
}

impl ParameterImpl for String {
    fn parameter(&self, options: ParameterOptions) -> Parameter {
        match options.preferred_variant {
            PreferredParameterVariant::Label => {
                Parameter::new(
                    &options.name,
                    LocalizedString::new(&options.display_name),
                    LocalizedString::new(&options.description),
                    ParameterVariant::Label(self.clone())
                )
            }
            _ => {
                Parameter::new(
                    &options.name,
                    LocalizedString::new(&options.display_name),
                    LocalizedString::new(&options.description),
                    ParameterVariant::TextInput {
                        disabled: options.disabled,
                        value: self.clone()
                    }
                )
            }
        }
    }

    fn set_parameter(&mut self, options: ParameterOptions, value: Parameter) -> Result<(), ParameterError> {
        if let ParameterVariant::TextInput {value, disabled} = value.variant {
            if options.preferred_variant == PreferredParameterVariant::Label {
                Err(ParameterError::WrongVariant)
            } else {
                if options.disabled == disabled && disabled {
                    Err(ParameterError::ParameterIsDisabled)
                } else {
                    *self = value;
                    Ok(())
                }
            }
        } else {
            Err(ParameterError::WrongVariant)
        }
    }
}