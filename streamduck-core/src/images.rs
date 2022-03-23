use std::hash::{Hash, Hasher};
use std::io::Cursor;
use std::time::Duration;
use image::{AnimationDecoder, DynamicImage, Frame, ImageFormat};
use image::codecs::gif::GifDecoder;
use image::codecs::png::PngDecoder;
use image::io::Reader;
use itertools::Itertools;
use serde::{Serialize, Deserialize};
use streamdeck::{DeviceImage, ImageMode, Kind};
use crate::util::rendering::resize_for_streamdeck;

/// Enum that represents various types of images Streamduck will use
#[derive(Clone)]
pub enum SDImage {
    /// Single normal image
    SingleImage(DynamicImage),

    /// Animated image
    AnimatedImage(Vec<AnimationFrame>)
}

impl SDImage {
    /// Attempts to decode base64 image to SDImage
    pub fn from_base64(image: &str, size: (usize, usize)) -> Result<SDImage, ImageDeserializationError> {
        let bytes = base64::decode(image)?;

        let decoder = Reader::new(Cursor::new(bytes)).with_guessed_format()?;

        if let Some(format) = decoder.format() {
            match format {
                // Only png and gif that need special handling
                ImageFormat::Png => {
                    let decoder = PngDecoder::new(decoder.into_inner())?;

                    if decoder.is_apng() {
                        let decoder = decoder.apng();

                        Ok(SDImage::AnimatedImage(convert_frames(decoder.into_frames().collect_frames()?, size)))
                    } else {
                        Ok(SDImage::SingleImage(resize_for_streamdeck(size, DynamicImage::from_decoder(decoder)?)))
                    }
                }

                ImageFormat::Gif => {
                    let decoder = GifDecoder::new(decoder.into_inner())?;
                    Ok(SDImage::AnimatedImage(convert_frames(decoder.into_frames().collect_frames()?, size)))
                }

                _ => {
                    Ok(SDImage::SingleImage(resize_for_streamdeck(size, decoder.decode()?)))
                }
            }
        } else {
            Err(ImageDeserializationError::UnrecognizedFormat)
        }
    }

    /// Checks if image is animated
    pub fn is_animated(&self) -> bool {
        match self {
            SDImage::SingleImage(_) => false,
            SDImage::AnimatedImage(_) => true,
        }
    }
}

/// Enum that represents serialized variant of [SDImage]
#[derive(Serialize, Deserialize, Hash, Debug, Clone)]
pub enum SDSerializedImage {
    SingleImage(String),
    AnimatedImage(Vec<SerializedFrame>)
}

impl SDSerializedImage {
    /// Gets image blob
    pub fn as_image_blob(&self) -> Result<String, ImageDeserializationError> {
        match self {
            SDSerializedImage::SingleImage(image) => Ok(image.clone()),
            SDSerializedImage::AnimatedImage(frames) => {
                if let Some(frame) = frames.get(0) {
                    Ok(frame.image.clone())
                } else {
                    Err(ImageDeserializationError::NoFrame)
                }
            }
        }
    }

    /// Checks if image is animated
    pub fn is_animated(&self) -> bool {
        match self {
            SDSerializedImage::SingleImage(_) => false,
            SDSerializedImage::AnimatedImage(_) => true,
        }
    }
}

/// Frame of animated image
#[derive(Clone)]
pub struct AnimationFrame {
    pub image: DynamicImage,
    pub index: usize,
    pub delay: f32,
}

/// Converts [Frame] vector to [AnimationFrame]
pub fn convert_frames(frames: Vec<Frame>, size: (usize, usize)) -> Vec<AnimationFrame> {
    frames.into_iter()
        .enumerate()
        .map(|(i, x)| {
            let delay = Duration::from(x.delay()).as_secs_f32() / 1.23;
            AnimationFrame {
                image: resize_for_streamdeck(size, DynamicImage::from(x.into_buffer())),
                index: i,
                delay
            }
        })
        .collect()
}

/// Serialized version of a frame
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializedFrame {
    pub image: String,
    pub index: usize,
    pub delay: f32,
}

impl Hash for SerializedFrame {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.image.hash(state);
        self.index.hash(state);
        ((self.delay * 100.0) as i32).hash(state);
    }
}

impl From<AnimationFrame> for SerializedFrame {
    fn from(frame: AnimationFrame) -> Self {
        SerializedFrame::from(&frame)
    }
}

impl From<&AnimationFrame> for SerializedFrame {
    fn from(frame: &AnimationFrame) -> Self {
        let mut buffer = vec![];
        frame.image.write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png).ok();

        SerializedFrame {
            image: base64::encode(buffer),
            index: frame.index,
            delay: frame.delay,
        }
    }
}

impl TryFrom<SerializedFrame> for AnimationFrame {
    type Error = ImageDeserializationError;

    fn try_from(value: SerializedFrame) -> Result<Self, Self::Error> {
        AnimationFrame::try_from(&value)
    }
}

impl TryFrom<&SerializedFrame> for AnimationFrame {
    type Error = ImageDeserializationError;

    fn try_from(value: &SerializedFrame) -> Result<Self, Self::Error> {
        let bytes = base64::decode(&value.image)?;

        let image = Reader::new(Cursor::new(bytes)).with_guessed_format()?.decode()?;

        Ok(AnimationFrame {
            image,
            index: value.index,
            delay: value.delay
        })
    }
}

impl From<SDImage> for SDSerializedImage {
    fn from(image: SDImage) -> Self {
        SDSerializedImage::from(&image)
    }
}

impl From<&SDImage> for SDSerializedImage {
    fn from(image: &SDImage) -> Self {
        match image {
            SDImage::SingleImage(image) => {
                SDSerializedImage::SingleImage({
                    let mut buffer = vec![];
                    image.write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png).ok();
                    base64::encode(buffer)
                })
            }

            SDImage::AnimatedImage(frames) => {
                SDSerializedImage::AnimatedImage({
                    frames.into_iter()
                        .map_into()
                        .collect()
                })
            }
        }
    }
}

impl TryFrom<SDSerializedImage> for SDImage {
    type Error = ImageDeserializationError;

    fn try_from(value: SDSerializedImage) -> Result<Self, Self::Error> {
        SDImage::try_from(&value)
    }
}

impl TryFrom<&SDSerializedImage> for SDImage {
    type Error = ImageDeserializationError;

    fn try_from(value: &SDSerializedImage) -> Result<Self, Self::Error> {
        match value {
            SDSerializedImage::SingleImage(image) => {
                let bytes = base64::decode(image)?;

                Ok(SDImage::SingleImage(Reader::new(Cursor::new(bytes)).with_guessed_format()?.decode()?))
            }

            SDSerializedImage::AnimatedImage(serialized_frames) => {
                Ok(SDImage::AnimatedImage({
                    let mut frames = vec![];

                    for serialized_frame in serialized_frames {
                        frames.push(serialized_frame.try_into()?)
                    }

                    frames
                }))
            }
        }
    }
}

/// Error for deserializing images
pub enum ImageDeserializationError {
    Base64Error(base64::DecodeError),
    IoError(std::io::Error),
    ImageError(image::ImageError),
    InvalidByteBuffer,
    UnrecognizedFormat,
    NoFrame
}

impl From<base64::DecodeError> for ImageDeserializationError {
    fn from(err: base64::DecodeError) -> Self {
        ImageDeserializationError::Base64Error(err)
    }
}

impl From<std::io::Error> for ImageDeserializationError {
    fn from(err: std::io::Error) -> Self {
        ImageDeserializationError::IoError(err)
    }
}

impl From<image::ImageError> for ImageDeserializationError {
    fn from(err: image::ImageError) -> Self {
        ImageDeserializationError::ImageError(err)
    }
}

/// Converts image to device image
pub fn convert_image(kind: &Kind, image: DynamicImage) -> DeviceImage {
    let mut buffer = vec![];

    image.rotate180().write_to(&mut Cursor::new(&mut buffer), match kind.image_mode() {
        ImageMode::Bmp => ImageFormat::Bmp,
        ImageMode::Jpeg => ImageFormat::Jpeg,
    }).ok();

    DeviceImage::from(buffer)
}