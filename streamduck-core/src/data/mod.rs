use std::error::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Visitor;
use std::fmt::{Display, Formatter};
use rmpv::Value;
use tokio::sync::RwLock;
use crate::device::DeviceIdentifier;
use crate::plugin::Plugin;
use crate::ui::UISchema;

/// Name that also contains plugin name, used to differentiate things made by different plugins
#[derive(Clone, Debug, Hash, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct NamespacedName {
    /// Plugin name that the thing originated from
    pub(crate) plugin_name: String,

    /// Name of the thing
    pub name: String
}

impl NamespacedName {
    /// Creates a new name based on a plugin
    pub fn from_plugin(plugin: &Plugin, name: &str) -> NamespacedName {
        NamespacedName {
            plugin_name: plugin.name.clone(),
            name: name.to_string(),
        }
    }
}

impl Display for NamespacedName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.name,self.plugin_name)
    }
}

/// Source of the thing, used when user changes an option on an action, or presses an action, or on which input tick happened
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Source {
    /// Device the thing originated from
    pub device: DeviceIdentifier,

    /// Input where the thing originated from
    pub input: u16,
}

/// Dynamic options
#[derive(Debug)]
pub struct Options {
    /// Options data
    pub data: RwLock<Value>,

    /// UI Schema that should be used by UI to let users change the data
    pub ui: UISchema
}

impl Default for Options {
    fn default() -> Self {
        Self {
            data: RwLock::new(Value::Nil),
            ui: vec![],
        }
    }
}

/// Any numberable value
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub enum Number {
    /// 8 byte integer
    Integer(i64),

    /// 8 byte unsigned integer
    UnsignedInteger(u64),

    /// 4 byte floating point number
    Float(f32),

    /// Double precision floating point number
    Double(f64),
}

impl Number {
    /// Checks if number is signed 8 byte integer
    pub fn is_i64(&self) -> bool {
        match self {
            Number::Integer(_) => true,
            _ => false,
        }
    }

    /// Checks if number is unsigned 8 byte integer
    pub fn is_u64(&self) -> bool {
        match self {
            Number::UnsignedInteger(_) => true,
            _ => false,
        }
    }

    /// Checks if number is 4 byte floating point number
    pub fn is_f32(&self) -> bool {
        match self {
            Number::Float(_) => true,
            _ => false,
        }
    }

    /// Checks if number is double precision floating point number
    pub fn is_f64(&self) -> bool {
        match self {
            Number::Double(_) => true,
            _ => false,
        }
    }

    /// Returns i64 if the number is that
    pub fn as_i64(&self) -> Option<i64> {
        i64::try_from(*self).ok()
    }

    /// Returns u64 if the number is that
    pub fn as_u64(&self) -> Option<u64> {
        u64::try_from(*self).ok()
    }

    /// Returns f32 if the number is that
    pub fn as_f32(&self) -> Option<f32> {
        f32::try_from(*self).ok()
    }

    /// Returns f64 if the number is that
    pub fn as_f64(&self) -> Option<f64> {
        f64::try_from(*self).ok()
    }
}

impl From<i64> for Number {
    fn from(value: i64) -> Self {
        Number::Integer(value)
    }
}

impl From<u64> for Number {
    fn from(value: u64) -> Self {
        Number::UnsignedInteger(value)
    }
}

impl From<f32> for Number {
    fn from(value: f32) -> Self {
        Number::Float(value)
    }
}

impl From<f64> for Number {
    fn from(value: f64) -> Self {
        Number::Double(value)
    }
}

/// Various errors that could happen while dealing with other types
#[derive(Debug, Clone)]
pub enum NumberError {
    /// The variant is not compatible
    IncompatibleVariant,

    /// When null was encountered
    NoValue
}

impl Display for NumberError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for NumberError {}

impl TryFrom<&Value> for Number {
    type Error = NumberError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Integer(i) => {
                match () {
                    _ if i.is_i64() => Ok(Number::Integer(i.as_i64().unwrap())),
                    _ => Ok(Number::UnsignedInteger(i.as_u64().unwrap_or_else(|| 0)))
                }
            }
            Value::F32(f) => Ok(Number::Float(*f)),
            Value::F64(d) => Ok(Number::Double(*d)),
            Value::Nil => Err(NumberError::NoValue),
            _ => Err(NumberError::IncompatibleVariant)
        }
    }
}

impl TryFrom<Number> for i64 {
    type Error = NumberError;

    fn try_from(value: Number) -> Result<Self, Self::Error> {
        let Number::Integer(value) = value else {
            return Err(NumberError::IncompatibleVariant)
        };

        Ok(value)
    }
}

impl TryFrom<Number> for u64 {
    type Error = NumberError;

    fn try_from(value: Number) -> Result<Self, Self::Error> {
        let Number::UnsignedInteger(value) = value else {
            return Err(NumberError::IncompatibleVariant)
        };

        Ok(value)
    }
}

impl TryFrom<Number> for f32 {
    type Error = NumberError;

    fn try_from(value: Number) -> Result<Self, Self::Error> {
        let Number::Float(value) = value else {
            return Err(NumberError::IncompatibleVariant)
        };

        Ok(value)
    }
}

impl TryFrom<Number> for f64 {
    type Error = NumberError;

    fn try_from(value: Number) -> Result<Self, Self::Error> {
        let Number::Double(value) = value else {
            return Err(NumberError::IncompatibleVariant)
        };

        Ok(value)
    }
}

impl Serialize for Number {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        match self {
            Number::Integer(i) => serializer.serialize_i64(*i),
            Number::UnsignedInteger(u) => serializer.serialize_u64(*u),
            Number::Float(f) => serializer.serialize_f32(*f),
            Number::Double(d) => serializer.serialize_f64(*d),
        }
    }
}

struct NumberVisitor;

impl<'a> Visitor<'a> for NumberVisitor {
    type Value = Number;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        writeln!(formatter, "any number value")
    }

    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E> where E: Error {
        Ok(Number::Integer(v as i64))
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E> where E: Error {
        Ok(Number::Integer(v as i64))
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E> where E: Error {
        Ok(Number::Integer(v as i64))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> where E: Error {
        Ok(Number::Integer(v))
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E> where E: Error {
        Ok(Number::UnsignedInteger(v as u64))
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E> where E: Error {
        Ok(Number::UnsignedInteger(v as u64))
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E> where E: Error {
        Ok(Number::UnsignedInteger(v as u64))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> where E: Error {
        Ok(Number::UnsignedInteger(v))
    }

    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E> where E: Error {
        Ok(Number::Float(v))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> where E: Error {
        Ok(Number::Double(v))
    }
}

impl<'de> Deserialize<'de> for Number {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_any(NumberVisitor)
    }
}

/// Array of keys/indexes to reach the value
pub type ValuePath = Vec<Value>;

