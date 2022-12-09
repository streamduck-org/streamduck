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

    /// Creates new parameter taking name, display name and description from options
    pub fn new_from_options(options: &ParameterOptions, variant: ParameterVariant) -> Parameter {
        Self::new(
            &options.name,
            LocalizedString::new(&options.display_name),
            LocalizedString::new(&options.description),
            variant
        )
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
        value: i32
    },
    /// Displays 2 integer input fields
    Integer2Input {
        /// If the field should appear disabled
        disabled: bool,
        /// Value for the integer input field
        value: (i32, i32)
    },
    /// Displays positive integer input field
    PositiveIntegerInput {
        /// If the field should appear disabled
        disabled: bool,
        /// Value for the positive integer input field
        value: u32
    },
    /// Displays 2 positive integer input fields
    PositiveInteger2Input {
        /// If the field should appear disabled
        disabled: bool,
        /// Value for the positive integer input field
        value: (u32, u32)
    },
    /// Displays real number input field
    NumberInput {
        /// If the field should appear disabled
        disabled: bool,
        /// Value for the real number input field
        value: f64
    },
    /// Displays 2 real number input fields
    Number2Input {
        /// If the field should appear disabled
        disabled: bool,
        /// Value for the real number input field
        value: (f64, f64)
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
    },
    /// Displays color picker
    Color {
        /// If the field should appear disabled
        disabled: bool,
        /// Color value
        value: Color,
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
#[derive(Clone, Default)]
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
#[derive(PartialEq, Clone, Copy)]
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
#[derive(Default)]
pub struct DynamicChoice {
    choices: Vec<String>,
    value: String
}

impl ParameterImpl for DynamicChoice {
    fn parameter(&self, options: ParameterOptions) -> Parameter {
        Parameter::new_from_options(
            &options,
            ParameterVariant::Choice {
                disabled: options.disabled,
                choices: self.choices.clone(),
                value: self.value.clone()
            }
        )
    }
}

impl ParameterImpl for i32 {
    fn parameter(&self, options: ParameterOptions) -> Parameter {
        Parameter::new_from_options(
            &options,
            ParameterVariant::IntegerInput {
                disabled: options.disabled,
                value: *self
            }
        )
    }

    // fn set_parameter(&mut self, options: ParameterOptions, value: Parameter) -> Result<(), ParameterError> {
    //     if let ParameterVariant::IntegerInput {value, disabled} = value.variant {
    //         if options.disabled == disabled && disabled {
    //             Err(ParameterError::ParameterIsDisabled)
    //         } else {
    //             *self = value;
    //             Ok(())
    //         }
    //     } else {
    //         Err(ParameterError::WrongVariant)
    //     }
    // }
}

impl ParameterImpl for (i32, i32) {
    fn parameter(&self, options: ParameterOptions) -> Parameter {
        Parameter::new_from_options(
            &options,
            ParameterVariant::Integer2Input {
                disabled: options.disabled,
                value: *self
            }
        )
    }
}

impl ParameterImpl for u32 {
    fn parameter(&self, options: ParameterOptions) -> Parameter {
        Parameter::new_from_options(
            &options,
            ParameterVariant::PositiveIntegerInput {
                disabled: options.disabled,
                value: *self
            }
        )
    }
}

impl ParameterImpl for (u32, u32) {
    fn parameter(&self, options: ParameterOptions) -> Parameter {
        Parameter::new_from_options(
            &options,
            ParameterVariant::PositiveInteger2Input {
                disabled: options.disabled,
                value: *self
            }
        )
    }
}

impl ParameterImpl for f64 {
    fn parameter(&self, options: ParameterOptions) -> Parameter {
        Parameter::new_from_options(
            &options,
            ParameterVariant::NumberInput {
                disabled: options.disabled,
                value: *self
            }
        )
    }

    // fn set_parameter(&mut self, options: ParameterOptions, value: Parameter) -> Result<(), ParameterError> {
    //     if let ParameterVariant::NumberInput {value, disabled} = value.variant {
    //         if options.disabled == disabled && disabled {
    //             Err(ParameterError::ParameterIsDisabled)
    //         } else {
    //             *self = value;
    //             Ok(())
    //         }
    //     } else {
    //         Err(ParameterError::WrongVariant)
    //     }
    // }
}

impl ParameterImpl for (f64, f64) {
    fn parameter(&self, options: ParameterOptions) -> Parameter {
        Parameter::new_from_options(
            &options,
            ParameterVariant::Number2Input {
                disabled: options.disabled,
                value: *self
            }
        )
    }
}

impl ParameterImpl for Color {
    fn parameter(&self, options: ParameterOptions) -> Parameter {
        Parameter::new_from_options(
            &options,
            ParameterVariant::Color {
                disabled: options.disabled,
                value: *self
            }
        )
    }
}

impl ParameterImpl for String {
    fn parameter(&self, options: ParameterOptions) -> Parameter {
        match options.preferred_variant {
            PreferredParameterVariant::Label => {
                Parameter::new_from_options(
                    &options,
                    ParameterVariant::Label(self.clone())
                )
            }
            _ => {
                Parameter::new_from_options(
                    &options,
                    ParameterVariant::TextInput {
                        disabled: options.disabled,
                        value: self.clone()
                    }
                )
            }
        }
    }

    // fn set_parameter(&mut self, options: ParameterOptions, value: Parameter) -> Result<(), ParameterError> {
    //     if let ParameterVariant::TextInput {value, disabled} = value.variant {
    //         if let PreferredParameterVariant::NoPreference | PreferredParameterVariant::TextInput = options.preferred_variant {
    //             if options.disabled == disabled && disabled {
    //                 Err(ParameterError::ParameterIsDisabled)
    //             } else {
    //                 *self = value;
    //                 Ok(())
    //             }
    //         } else {
    //             Err(ParameterError::WrongVariant)
    //         }
    //     } else {
    //         Err(ParameterError::WrongVariant)
    //     }
    // }
}

impl ParameterImpl for bool {
    fn parameter(&self, options: ParameterOptions) -> Parameter {
        match options.preferred_variant {
            PreferredParameterVariant::Checkbox => {
                Parameter::new_from_options(
                    &options,
                    ParameterVariant::Checkbox {
                        disabled: options.disabled,
                        value: *self
                    }
                )
            }
            _ => {
                Parameter::new_from_options(
                    &options,
                    ParameterVariant::Toggle {
                        disabled: options.disabled,
                        value: *self
                    }
                )
            }
        }
    }

    // fn set_parameter(&mut self, options: ParameterOptions, value: Parameter) -> Result<(), ParameterError> {
    //     if let ParameterVariant::Checkbox {value, disabled} = value.variant {
    //         if let PreferredParameterVariant::Checkbox = options.preferred_variant {
    //             if options.disabled == disabled && disabled {
    //                 Err(ParameterError::ParameterIsDisabled)
    //             } else {
    //                 *self = value;
    //                 Ok(())
    //             }
    //         } else {
    //             Err(ParameterError::WrongVariant)
    //         }
    //     } else if let ParameterVariant::Toggle { value, disabled } = value.variant {
    //         if let PreferredParameterVariant::Toggle | PreferredParameterVariant::NoPreference = options.preferred_variant {
    //             if options.disabled == disabled && disabled {
    //                 Err(ParameterError::ParameterIsDisabled)
    //             } else {
    //                 *self = value;
    //                 Ok(())
    //             }
    //         } else {
    //             Err(ParameterError::WrongVariant)
    //         }
    //     } else {
    //         Err(ParameterError::WrongVariant)
    //     }
    // }
}

impl<T: ParameterImpl + Default> ParameterImpl for Option<T> {
    fn parameter(&self, options: ParameterOptions) -> Parameter {
        let mut params = self.as_ref().map_or_else(
            || vec![],
            |x| flatten_parameter(x.parameter(Default::default()))
        );

        match options.preferred_variant {
            PreferredParameterVariant::Checkbox => {
                params.insert(0, Parameter::new_from_options(
                    &options,
                    ParameterVariant::Checkbox {
                        disabled: options.disabled,
                        value: self.is_some()
                    }
                ));
            }

            PreferredParameterVariant::Toggle => {
                params.insert(0, Parameter::new_from_options(
                    &options,
                    ParameterVariant::Toggle {
                        disabled: options.disabled,
                        value: self.is_some()
                    }
                ));
            }

            _ => {}
        }

        Parameter::new_from_options(
            &options,
            ParameterVariant::CollapsableMenu(params)
        )
    }
    //
    // fn set_parameter(&mut self, options: ParameterOptions, value: Parameter) -> Result<(), ParameterError> {
    //     if value.name == options.name { // Setting toggle in this case if there's one
    //
    //     }
    //
    //     Ok(())
    // }
}

impl<T: ParameterImpl + Default> ParameterImpl for Vec<T> {
    fn parameter(&self, options: ParameterOptions) -> Parameter {
        Parameter::new_from_options(
            &options,
            ParameterVariant::Array(self.clone().into_iter()
                .map(|x| flatten_parameter(x.parameter(Default::default())))
                .collect()
            )
        )
    }
}