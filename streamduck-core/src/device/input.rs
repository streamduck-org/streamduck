use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Map of key index to input description
pub type InputLayout = HashMap<u32, Input>;

/// An input piece for a device
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Input {
    /// Horizontal position of the input. Left is negative value, right is positive value
    pub x: i32,

    /// Vertical position of the input. Down is negative value, Up is positive value
    pub y: i32,

    /// Input type
    pub ty: InputType,

    /// If Some, the input will be considered to have a screen and specified resolution will be used for it
    pub resolution: Option<(u32, u32)>,
}

/// Defines what kind of input the thing provides
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum InputType {
    /// Digital button, either off (0) or on (255)
    Button,

    /// Analog button, value varies depending on depth or force of the press
    AnalogButton,

    /// Slider, value can be adjusted by moving the slider back and forth
    Slider,

    /// Knob, value can be adjusted by twisting the knob
    Knob,

    /// X in XY panel, 2 dimensional inputs must be defined in 2 separate inputs
    XPanel,

    /// Y in XY panel, 2 dimensional inputs must be defined in 2 separate inputs
    YPanel,

    /// X in a joystick, 2 dimensional inputs must be defined in 2 separate inputs
    XStick,

    /// Y in a joystick, 2 dimensional inputs must be defined in 2 separate inputs
    YStick,
}