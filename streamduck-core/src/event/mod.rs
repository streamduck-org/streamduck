use rmpv::Value;
use serde::{Serialize, Deserialize};
use crate::device::input::InputType;

pub struct EventDispatcher {

}

/// An event
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Event {
    /// Name of the event
    pub name: String,

    /// Payload of the event
    pub payload: Value,
}

/// Input payload, should be used by the device to send input events to the core
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InputPayload {
    /// Which input got the event
    input: u16,

    /// The event that happened to the input
    event: InputEvent
}

/// Input event, describes what happened to the input
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum InputEvent {
    /// Button press event
    ButtonPressed,

    /// Button release event
    ButtonReleased,

    /// State of the toggle got changed
    Toggle(bool),

    /// Pressure change of analog button
    AnalogButton(u8),

    /// Hover occured over a position for XY Panel
    XYPanelHover {
        position: (u32, u32)
    },

    /// XY Panel received a press at certain position
    XYPanelPress {
        position: (u32, u32)
    },

    /// XY Panel press was released at certain position
    XYPanelRelease {
        position: (u32, u32)
    },

    /// XY Panel drag
    XYPanelDrag {
        position: (u32, u32)
    },

    /// XY Panel swipe occured
    XYPanelSwipe {
        /// Starting position
        start: (u32, u32),

        /// End position
        end: (u32, u32),
    },

    /// Joystick state change
    Joystick(f32, f32),

    /// Slider or Knob state change
    SliderKnob(f32),

    /// Endless knob was twisted
    EndlessKnob(i16),

    /// Trackball was rolled
    Trackball(i16, i16),

    /// Sensor state change
    Sensor(Value)
}