use serde::{Serialize, Deserialize};
use crate::trigger::TriggerCondition;

/// Map of key index to input description
pub type InputLayout = Vec<Input>;

/// An input piece for a device
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Input {
    /// Horizontal position of the input. Left is negative value, right is positive value
    pub x: i32,

    /// Vertical position of the input. Down is negative value, Up is positive value
    pub y: i32,

    /// Width of the input
    pub w: u32,

    /// Height of the input
    pub h: u32,

    /// Input type
    pub ty: InputType,

    /// If Some, the input will be considered to have a screen and specified resolution will be used for it
    pub resolution: Option<(u32, u32)>,

    /// Trigger conditions that should usually be used with the input
    pub trigger_presets: Vec<TriggerCondition>,
}

/// Defines what kind of input the thing provides
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum InputType {
    /// Digital button
    Button,

    /// A toggle, switches state with each press
    Toggle,

    /// Analog button, value varies depending on depth or force of the press
    AnalogButton,

    /// Slider, value can be adjusted by moving the slider back and forth
    Slider,

    /// Knob, value can be adjusted by twisting the knob, but is limited
    Knob,

    /// Endless knob, can be endlessly turned in any direction
    EndlessKnob,

    /// XY panel, absolute position 2 dimensional input
    XYPanel,

    /// Joystick, 2 dimensional input where values are -1.0 to 1.0 floating point numbers
    XYStick,

    /// Trackball / Trackpad, 2 dimensional input where values are the distance the ball covered
    Trackball,

    /// Wildcard input that could express anything else, eg. proximity sensor, accelerometer, temperature sensor...
    Sensor,
}