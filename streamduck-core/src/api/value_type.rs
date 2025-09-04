use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Value};
use serde::{Deserialize, Serialize};

/// Type information to see what to expect from an event
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum ValueType {
    Any,
    Boolean,
    Integer,
    UnsignedInteger,
    Double,
    String,
    Nullable(Box<ValueType>),
    Union(Vec<ValueType>),
    Tuple(Vec<ValueType>),
    Array(Box<ValueType>),
    /// Keys must be String
    Map(Box<ValueType>),
    Object(Vec<(String, ValueType)>)
}

impl FromLua for ValueType {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        lua.from_value(value)
    }
}

impl IntoLua for ValueType {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        lua.to_value(&self)
    }
}