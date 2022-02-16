use serde::{Serialize, Deserialize};
use crate::threads::rendering::RendererComponent;

/// Component definition
#[derive(Serialize, Deserialize, Clone)]
pub struct ComponentDefinition {
    /// Display name for the component
    pub display_name: String,

    /// Description of the component
    pub description: String,

    /// Controls that should be exposed to UI
    pub exposed_fields: Vec<UIField>,

    /// Default looks for a button, in case user doesn't want to setup one on their own
    pub default_looks: RendererComponent
}

/// UI Field, will be represented in a list similar to Unity's inspector
#[derive(Serialize, Deserialize, Clone)]
pub struct UIField {
    pub name: String,
    pub ty: UIFieldType,
}

/// UI Value, represents what fields currently have
#[derive(Serialize, Deserialize, Clone)]
pub struct UIValue {
    pub name: String,
    pub ty: UIFieldType,
    pub value: UIFieldValue,
}

/// UI Field Types, defines types that fields will have
#[derive(Serialize, Deserialize, Clone)]
pub enum UIFieldType {
    /// Displays a header for separation reasons
    Header,

    /// Text field that accepts float values
    InputFieldFloat,
    /// Text field that accepts integer values
    InputFieldInteger,
    /// Text field that accepts strings
    InputFieldString,

    /// Float slider of specified bounds
    ValueSliderFloat(UIScalar<f32>),
    /// Integer slider of specified bounds
    ValueSliderInteger(UIScalar<i32>),

    /// Collapsable submenu
    Collapsable(Vec<UIField>)
}

/// UI Field value, current state of the settings
#[derive(Serialize, Deserialize, Clone)]
pub enum UIFieldValue {
    /// Text field that accepts float values
    InputFieldFloat(f32),
    /// Text field that accepts integer values
    InputFieldInteger(i32),
    /// Text field that accepts strings
    InputFieldString(String),

    /// Float slider of specified bounds
    ValueSliderFloat(f32),
    /// Integer slider of specified bounds
    ValueSliderInteger(i32),

    /// Collapsable submenu
    Collapsable(Vec<UIFieldValue>)
}

/// Information for running sliders in UI
#[derive(Serialize, Deserialize, Clone)]
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