use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Value};
use serde::{Deserialize, Serialize};
use crate::api::value_type::ValueType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub position_size: ((i32, i32), (u32, u32)),
    pub icon: InputIcon,
    pub events: Vec<InputEvent>
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum InputIcon {
    Button,
    Toggle,
    AnalogButton,
    Slider,
    Knob,
    Encoder,
    TouchScreen,
    Joystick,
    Trackball,
    Touchpad,
    Sensor
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputEvent {
    pub name: String,
    pub description: Option<String>,
    pub signature: ValueType
}

impl FromLua for InputEvent {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        lua.from_value(value)
    }
}

impl IntoLua for InputEvent {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        lua.to_value(&self)
    }
}