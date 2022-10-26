use image::{DynamicImage};

/// Represents width and height of an image in pixels
pub type ImageSize = (u16, u16);

/// Single image or animated
pub enum CoreImage {
    /// Single image
    SingleImage(DynamicImage),
    /// Animated image
    AnimatedImage(AnimatedImage)
}

/// Animated image
pub struct AnimatedImage(Vec<AnimationFrame>);

/// Frame of an animated image
pub struct AnimationFrame {
    /// Contents of the frame
    frame: DynamicImage,
    /// Index of the frame
    index: usize,
    /// Delay of the frame
    delay: f32
}