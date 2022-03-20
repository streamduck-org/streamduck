use std::io::Cursor;
use image::{AnimationDecoder, Delay, DynamicImage, Frame, ImageFormat};
use image::gif::GifDecoder;
use image::io::Reader;
use image::png::PngDecoder;
use itertools::Itertools;
use serde::{Serialize, Deserialize};
use crate::util::rendering::resize_for_streamdeck;

/// Enum that represents various types of images Streamduck will use
#[derive(Clone)]
pub enum SDImage {
    /// Single normal image
    SingleImage(DynamicImage),

    /// Animated image
    AnimatedImage(Vec<Frame>)
}

impl SDImage {
    /// Attempts to decode base64 image to SDImage
    pub fn from_base64(image: &str) -> Result<SDImage, ImageDeserializationError> {
        let bytes = base64::decode(image)?;

        let decoder = Reader::new(Cursor::new(bytes)).with_guessed_format()?;

        if let Some(format) = decoder.format() {
            match format {
                // Only png and gif that need special handling
                ImageFormat::Png => {
                    let decoder = PngDecoder::new(decoder.into_inner())?;

                    if decoder.is_apng() {
                        let decoder = decoder.apng();

                        Ok(SDImage::AnimatedImage(decoder.into_frames().collect_frames()?.into_iter().map(
                            |x| {
                                let delay = x.delay();
                                let top = x.top();
                                let left = x.left();

                                let image = resize_for_streamdeck((100, 100), DynamicImage::ImageRgba8(x.into_buffer()));

                                Frame::from_parts(image.into_rgba8(), left, top, delay)
                            }
                        ).collect()))
                    } else {
                        Ok(SDImage::SingleImage(resize_for_streamdeck((100, 100), DynamicImage::from_decoder(decoder)?)))
                    }
                }

                ImageFormat::Gif => {
                    let decoder = GifDecoder::new(decoder.into_inner())?;
                    Ok(SDImage::AnimatedImage(decoder.into_frames().collect_frames()?.into_iter().map(
                        |x| {
                            let delay = x.delay();
                            let top = x.top();
                            let left = x.left();

                            let image = resize_for_streamdeck((100, 100), DynamicImage::ImageRgba8(x.into_buffer()));

                            Frame::from_parts(image.into_rgba8(), left, top, delay)
                        }
                    ).collect()))
                }

                _ => {
                    Ok(SDImage::SingleImage(resize_for_streamdeck((100, 100), decoder.decode()?)))
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

/// Serialized version of a frame
#[derive(Serialize, Deserialize, Hash, Clone, Debug)]
pub struct SerializedFrame {
    pub image: String,
    pub width: u32,
    pub height: u32,
    pub delay: (u32, u32),
    pub top: u32,
    pub left: u32
}

impl From<Frame> for SerializedFrame {
    fn from(frame: Frame) -> Self {
        SerializedFrame::from(&frame)
    }
}

impl From<&Frame> for SerializedFrame {
    fn from(frame: &Frame) -> Self {
        let image = DynamicImage::ImageRgba8(frame.buffer().clone());

        let mut buffer = vec![];
        image.write_to(&mut buffer, ImageFormat::Png).ok();

        SerializedFrame {
            image: base64::encode(buffer),
            width: frame.buffer().width(),
            height: frame.buffer().height(),
            delay: frame.delay().numer_denom_ms(),
            top: frame.top(),
            left: frame.left()
        }
    }
}

impl TryFrom<SerializedFrame> for Frame {
    type Error = ImageDeserializationError;

    fn try_from(value: SerializedFrame) -> Result<Self, Self::Error> {
        Frame::try_from(&value)
    }
}

impl TryFrom<&SerializedFrame> for Frame {
    type Error = ImageDeserializationError;

    fn try_from(value: &SerializedFrame) -> Result<Self, Self::Error> {
        let bytes = base64::decode(&value.image)?;

        let image = Reader::new(Cursor::new(bytes)).with_guessed_format()?.decode()?;

        Ok(Frame::from_parts(
            image.into_rgba8(),
            value.left,
            value.top,
            Delay::from_numer_denom_ms(value.delay.0, value.delay.1)
        ))
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
                    image.write_to(&mut buffer, ImageFormat::Png).ok();
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