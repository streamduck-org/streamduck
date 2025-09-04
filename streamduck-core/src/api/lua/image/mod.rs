mod enums;

use crate::api::lua::image::enums::{LuaColorType, LuaFilterType};
use image::codecs::bmp::BmpEncoder;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::{ColorType, DynamicImage};
use mlua::{FromLua, IntoLua, Lua, UserData, UserDataFields, UserDataMethods, Value};
use std::ops::{Deref, DerefMut};
use crate::api::lua::hid::utils::LuaByteArray;

/// Wrapper to make dynamic image accessible to Lua
#[derive(Debug, PartialEq, Clone)]
pub struct LuaDynamicImage {
    image: DynamicImage
}

impl From<DynamicImage> for LuaDynamicImage {
    fn from(value: DynamicImage) -> Self {
        Self { image: value }
    }
}

impl From<LuaDynamicImage> for DynamicImage {
    fn from(value: LuaDynamicImage) -> Self {
        value.image
    }
}

impl Deref for LuaDynamicImage {
    type Target = DynamicImage;

    fn deref(&self) -> &Self::Target {
        &self.image
    }
}

impl DerefMut for LuaDynamicImage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.image
    }
}

impl FromLua for LuaDynamicImage {
    fn from_lua(value: Value, _lua: &Lua) -> mlua::Result<Self> {
        match value {
            Value::UserData(userdata) => Ok(userdata.take::<LuaDynamicImage>()?),
            _ => Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "LuaDynamicImage".to_string(),
                message: None,
            }),
        }
    }
}

impl UserData for LuaDynamicImage {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("colorType", |_, this| {
            Ok(LuaColorType::from(this.color()))
        });

        fields.add_field_method_get("width", |_, this| {
            Ok(this.width())
        });

        fields.add_field_method_get("height", |_, this| {
            Ok(this.height())
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_function(
            "resized",
            |_lua, (this, width, height, filter, exact): (LuaDynamicImage, u32, u32, LuaFilterType, Option<bool>)| {
                Ok(LuaDynamicImage::from(if exact.unwrap_or(false) {
                    this.resize_exact(width, height, filter.into())
                } else {
                    this.resize(width, height, filter.into())
                }))
            },
        );

        methods.add_function(
            "cropResized",
            |_lua, (this, width, height, filter): (LuaDynamicImage, u32, u32, LuaFilterType)| {
                Ok(LuaDynamicImage::from(this.resize_to_fill(width, height, filter.into())))
            },
        );

        methods.add_function(
            "rotated90",
            |_lua, (this, counter_clockwise): (LuaDynamicImage, Option<bool>)| {
                Ok(LuaDynamicImage::from(if counter_clockwise.unwrap_or(false) {
                    this.rotate270()
                } else {
                    this.rotate90()
                }))
            },
        );

        methods.add_function(
            "rotated180",
            |_lua, this: LuaDynamicImage| {
                Ok(LuaDynamicImage::from(this.rotate180()))
            },
        );

        methods.add_function(
            "hFlipped",
            |_lua, this: LuaDynamicImage| {
                Ok(LuaDynamicImage::from(this.fliph()))
            },
        );

        methods.add_function(
            "vFlipped",
            |_lua, this: LuaDynamicImage| {
                Ok(LuaDynamicImage::from(this.flipv()))
            },
        );

        methods.add_method("clone", |_lua, this, _: ()| {
            Ok(this.clone())
        });

        methods.add_method("encodeJpeg", |_lua, this, quality: Option<u8>| {
            let buffer_size = this.width() * this.height() * this.color().bytes_per_pixel() as u32;
            let mut buffer: Vec<u8> = vec![0u8; buffer_size as usize];

            let encoder = JpegEncoder::new_with_quality(&mut buffer, quality.unwrap_or(75));
            this.write_with_encoder(encoder).map_err(mlua::Error::external)?;

            Ok(LuaByteArray::from(buffer))
        });

        methods.add_method("encodePng", |_lua, this, _: ()| {
            let buffer_size = this.width() * this.height() * this.color().bytes_per_pixel() as u32;
            let mut buffer: Vec<u8> = vec![0u8; buffer_size as usize];

            let encoder = PngEncoder::new(&mut buffer);
            this.write_with_encoder(encoder).map_err(mlua::Error::external)?;

            Ok(LuaByteArray::from(buffer))
        });

        methods.add_method("encodeBmp", |_lua, this, _: ()| {
            let buffer_size = this.width() * this.height() * this.color().bytes_per_pixel() as u32;
            let mut buffer: Vec<u8> = vec![0u8; buffer_size as usize];

            let encoder = BmpEncoder::new(&mut buffer);
            this.write_with_encoder(encoder).map_err(mlua::Error::external)?;

            Ok(LuaByteArray::from(buffer))
        });

        methods.add_function("toRaw", |lua, this: LuaDynamicImage| {
            let color = this.color();

            let image = match this.image {
                DynamicImage::ImageLuma8(i) => i.to_vec().into_lua(lua),
                DynamicImage::ImageLumaA8(i) => i.to_vec().into_lua(lua),
                DynamicImage::ImageRgb8(i) => i.to_vec().into_lua(lua),
                DynamicImage::ImageRgba8(i) => i.to_vec().into_lua(lua),
                DynamicImage::ImageLuma16(i) => i.to_vec().into_lua(lua),
                DynamicImage::ImageLumaA16(i) => i.to_vec().into_lua(lua),
                DynamicImage::ImageRgb16(i) => i.to_vec().into_lua(lua),
                DynamicImage::ImageRgba16(i) => i.to_vec().into_lua(lua),
                DynamicImage::ImageRgb32F(i) => i.to_vec().into_lua(lua),
                DynamicImage::ImageRgba32F(i) => i.to_vec().into_lua(lua),
                _ => return Err(mlua::Error::RuntimeError("image is of unknown color type".to_string())),
            }?;

            Ok((image, LuaColorType::from(color)))
        });

        methods.add_function("toColor", |_, (this, value): (LuaDynamicImage, LuaColorType)| {
            Ok(LuaDynamicImage::from(match value.inner {
                ColorType::L8 => DynamicImage::from(this.image.to_luma8()),
                ColorType::La8 => DynamicImage::from(this.image.to_luma_alpha8()),
                ColorType::Rgb8 => DynamicImage::from(this.image.to_rgb8()),
                ColorType::Rgba8 => DynamicImage::from(this.image.to_rgba8()),
                ColorType::L16 => DynamicImage::from(this.image.to_luma16()),
                ColorType::La16 => DynamicImage::from(this.image.to_luma_alpha16()),
                ColorType::Rgb16 => DynamicImage::from(this.image.to_rgb16()),
                ColorType::Rgba16 => DynamicImage::from(this.image.to_rgba16()),
                ColorType::Rgb32F => DynamicImage::from(this.image.to_rgb32f()),
                ColorType::Rgba32F => DynamicImage::from(this.image.to_rgba32f()),
                _ => this.image
            }))
        });
    }
}

pub struct LuaDynamicImageStatic;

impl UserData for LuaDynamicImageStatic {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("newRGB8", |_lua, (width, height): (u32, u32)| {
            Ok(LuaDynamicImage::from(DynamicImage::new_rgb8(width, height)))
        });

        methods.add_function("newRGBA8", |_lua, (width, height): (u32, u32)| {
            Ok(LuaDynamicImage::from(DynamicImage::new_rgba8(width, height)))
        });

        methods.add_function("newRGB16", |_lua, (width, height): (u32, u32)| {
            Ok(LuaDynamicImage::from(DynamicImage::new_rgb16(width, height)))
        });

        methods.add_function("newRGBA16", |_lua, (width, height): (u32, u32)| {
            Ok(LuaDynamicImage::from(DynamicImage::new_rgba16(width, height)))
        });

        methods.add_function("newL8", |_lua, (width, height): (u32, u32)| {
            Ok(LuaDynamicImage::from(DynamicImage::new_luma8(width, height)))
        });

        methods.add_function("newLA8", |_lua, (width, height): (u32, u32)| {
            Ok(LuaDynamicImage::from(DynamicImage::new_luma_a8(width, height)))
        });

        methods.add_function("newL16", |_lua, (width, height): (u32, u32)| {
            Ok(LuaDynamicImage::from(DynamicImage::new_luma16(width, height)))
        });

        methods.add_function("newLA16", |_lua, (width, height): (u32, u32)| {
            Ok(LuaDynamicImage::from(DynamicImage::new_luma_a16(width, height)))
        });

        methods.add_function("newRGB32F", |_lua, (width, height): (u32, u32)| {
            Ok(LuaDynamicImage::from(DynamicImage::new_rgb32f(width, height)))
        });

        methods.add_function("newRGBA32F", |_lua, (width, height): (u32, u32)| {
            Ok(LuaDynamicImage::from(DynamicImage::new_rgba32f(width, height)))
        });
    }
}