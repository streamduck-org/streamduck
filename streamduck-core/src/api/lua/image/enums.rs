use std::ops::Deref;
use image::ColorType;
use image::imageops::FilterType;
use mlua::{FromLua, IntoLua, Lua, Value};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct LuaFilterType {
    pub inner: FilterType
}

impl Default for LuaFilterType {
    fn default() -> Self {
        Self {
            inner: FilterType::CatmullRom
        }
    }
}

impl From<FilterType> for LuaFilterType {
    fn from(value: FilterType) -> Self {
        Self { inner: value }
    }
}

impl From<LuaFilterType> for FilterType {
    fn from(value: LuaFilterType) -> Self {
        value.inner
    }
}

impl Deref for LuaFilterType {
    type Target = FilterType;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl FromLua for LuaFilterType {
    fn from_lua(value: Value, _lua: &Lua) -> mlua::Result<Self> {
        Ok(match value.to_string()?.as_str() {
            "nearest" => FilterType::Nearest,
            "linear" | "triangle" => FilterType::Triangle,
            "cubic" | "catmull" | "catmullrom" => FilterType::CatmullRom,
            "gaussian" => FilterType::Gaussian,
            "lanczos" | "lanczos3" => FilterType::Lanczos3,
            _ => {
                return Ok(LuaFilterType::default())
            },
        }.into())
    }
}

impl IntoLua for LuaFilterType {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        match self.inner {
            FilterType::Nearest => "nearest",
            FilterType::Triangle => "triangle",
            FilterType::CatmullRom => "cubic",
            FilterType::Gaussian => "gaussian",
            FilterType::Lanczos3 => "lanczos"
        }.into_lua(lua)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct LuaColorType {
    pub inner: ColorType
}

impl From<ColorType> for LuaColorType {
    fn from(value: ColorType) -> Self {
        Self { inner: value }
    }
}

impl From<LuaColorType> for ColorType {
    fn from(value: LuaColorType) -> Self {
        value.inner
    }
}

impl Deref for LuaColorType {
    type Target = ColorType;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl FromLua for LuaColorType {
    fn from_lua(value: Value, _lua: &Lua) -> mlua::Result<Self> {
        let value_str = value.to_string()?;
        Ok(match value_str.as_str() {
            "l8" => ColorType::L8,
            "la8" => ColorType::La8,
            "rgb8" => ColorType::Rgb8,
            "rgba8" => ColorType::Rgba8,
            "l16" => ColorType::L16,
            "la16" => ColorType::La16,
            "rgb16" => ColorType::Rgb16,
            "rgba16" => ColorType::Rgba16,
            "rgb32f" => ColorType::Rgb32F,
            "rgba32f" => ColorType::Rgba32F,
            _ => {
                return Err(mlua::Error::FromLuaConversionError {
                    from: value.type_name(),
                    to: "ColorType".to_string(),
                    message: Some(format!("Invalid color type '{}'", value_str)),
                })
            }
        }.into())
    }
}

impl IntoLua for LuaColorType {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        match self.inner {
            ColorType::L8 => "l8",
            ColorType::La8 => "la8",
            ColorType::Rgb8 => "rgb8",
            ColorType::Rgba8 => "rgba8",
            ColorType::L16 => "l16",
            ColorType::La16 => "la16",
            ColorType::Rgb16 => "rgb16",
            ColorType::Rgba16 => "rgba16",
            ColorType::Rgb32F => "rgb32f",
            ColorType::Rgba32F => "rgba32f",
            _ => "unknown"
        }.into_lua(lua)
    }
}
