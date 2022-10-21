use serde::{Serialize, Deserialize};
use strum_macros::{EnumVariantNames, EnumString, Display};
use image::{DynamicImage, GenericImage, GenericImageView, Rgba};
use image::imageops::{FilterType, horizontal_gradient, vertical_gradient};
use rusttype::{Font, Point, point, Scale};

/// Resizes image to specified size
pub fn resize_for_streamdeck(size: (usize, usize), image: DynamicImage) -> DynamicImage {
    let (sx, sy) = size;
    DynamicImage::from(image.to_rgba8()).resize_to_fill(sx as u32, sy as u32, FilterType::Lanczos3)
}

/// Generates solid color image of specified size
pub fn image_from_solid(size: (usize, usize), color: Rgba<u8>) -> DynamicImage {
    let (sx, sy) = size;
    let mut image = DynamicImage::new_rgba8(sx as u32, sy as u32);

    horizontal_gradient(&mut image, &color, &color);

    image
}

/// Generates horizontal gradient image of specified size
pub fn image_from_horiz_gradient(size: (usize, usize), start: Rgba<u8>, end: Rgba<u8>) -> DynamicImage {
    let (sx, sy) = size;
    let mut image = DynamicImage::new_rgba8(sx as u32, sy as u32);

    horizontal_gradient(&mut image, &start, &end);

    image
}

/// Generates vertical gradient image of specified size
pub fn image_from_vert_gradient(size: (usize, usize), start: Rgba<u8>, end: Rgba<u8>) -> DynamicImage {
    let (sx, sy) = size;
    let mut image = DynamicImage::new_rgba8(sx as u32, sy as u32);

    vertical_gradient(&mut image, &start, &end);

    image
}

/// Renders text from font and parameters onto provided image
pub fn render_text_on_image(image: &mut DynamicImage, font: &Font, text: &str, scale: Scale, point: Point<f32>, color: (u8, u8, u8, u8)) {
    let (size_x, size_y) = image.dimensions();
    for glyph in font.layout(text, scale, point) {
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            glyph.draw(|x, y, v| {
                let bound_x = (x as i32 + bounding_box.min.x) as u32;
                let bound_y = (y as i32 + bounding_box.min.y) as u32;

                if (bound_x < size_x) && (bound_y < size_y) {
                    let pixel = image.get_pixel(bound_x, bound_y);
                    let color_mul = (v * (color.3 as f32 / 255.0)).clamp(0.0, 1.0);

                    image.put_pixel(
                        bound_x,
                        bound_y,
                        // Turn the coverage into an alpha value
                        Rgba([(pixel.0[0] as f32 * (1.0 - color_mul) + color.0 as f32 * color_mul) as u8, (pixel.0[1] as f32 * (1.0 - color_mul) + color.1 as f32 * color_mul) as u8, (pixel.0[2] as f32 * (1.0 - color_mul) + color.2 as f32 * color_mul) as u8, 255]),
                    )
                }
            })
        }
    }
}

/// Renders text with shadows from font and parameters onto provided image
pub fn render_shadowed_text_on_image(image: &mut DynamicImage, font: &Font, text: &str, scale: Scale, point: Point<f32>, color: (u8, u8, u8, u8), shadow_offset: (i32, i32), shadow_color: (u8, u8, u8, u8)) {
    let (size_x, size_y) = image.dimensions();
    for glyph in font.layout(text, scale, point) {
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            glyph.draw(|x, y, v| {
                let bound_x = (x as i32 + bounding_box.min.x) as u32;
                let bound_y = (y as i32 + bounding_box.min.y) as u32;

                if (bound_x < size_x) && (bound_y < size_y) {
                    let pixel = image.get_pixel(bound_x, bound_y);
                    let color_mul = (v * (color.3 as f32 / 255.0)).clamp(0.0, 1.0);

                    image.put_pixel(
                        bound_x,
                        bound_y,
                        // Turn the coverage into an alpha value
                        Rgba([(pixel.0[0] as f32 * (1.0 - color_mul) + color.0 as f32 * color_mul) as u8, (pixel.0[1] as f32 * (1.0 - color_mul) + color.1 as f32 * color_mul) as u8, (pixel.0[2] as f32 * (1.0 - color_mul) + color.2 as f32 * color_mul) as u8, 255]),
                    );

                    let shadow_x = (bound_x as i32 + shadow_offset.0) as u32;
                    let shadow_y = (bound_y as i32 + shadow_offset.1) as u32;

                    if (shadow_x < size_x) && (shadow_y < size_y) {
                        let pixel = image.get_pixel(shadow_x, shadow_y);
                        let shadow_mul = (v * (shadow_color.3 as f32 / 255.0)).clamp(0.0, 1.0);

                        if shadow_mul > 0.01 {
                            image.put_pixel(
                                shadow_x,
                                shadow_y,
                                // Turn the coverage into an alpha value
                                Rgba([(pixel.0[0] as f32 * (1.0 - shadow_mul) + shadow_color.0 as f32 * shadow_mul) as u8, (pixel.0[1] as f32 * (1.0 - shadow_mul) + shadow_color.1 as f32 * shadow_mul) as u8, (pixel.0[2] as f32 * (1.0 - shadow_mul) + shadow_color.2 as f32 * shadow_mul) as u8, 255]),
                            );
                        }
                    }
                }
            })
        }
    }
}

/// Renders box with provided parameters onto provided image
pub fn render_box_on_image(image: &mut DynamicImage, scale: Scale, point: Point<f32>, color: (u8, u8, u8, u8)) {
    let (size_x, size_y) = image.dimensions();
    for x in 0..(scale.x as isize) {
        for y in 0..(scale.y as isize) {
            let offset_x = (point.x + x as f32) as u32;
            let offset_y = (point.y - y as f32 - 1.0) as u32;

            if (offset_x < size_x) && (offset_y < size_y) {
                image.put_pixel(
                    offset_x,
                    offset_y,
                    Rgba([color.0, color.1, color.2, 255]),
                )
            }
        }
    }
}

/// Calculates bounds for text with provided font and parameters
pub fn calculate_bounds_for_text(font: &Font, text: &str, scale: Scale) -> (u32, u32) {
    let mut w: u32 = 0;
    let mut h: u32 = 0;

    for glyph in font.layout(text, scale, point(0.0, 0.0)) {
        if let Some(bounding) = glyph.pixel_bounding_box() {
            h = h.max(bounding.height() as u32);
            w = w.max(bounding.max.x as u32);
        }
    }

    (w, h)
}

/// Alignment enumeration
#[derive(Debug, Clone, Hash, Serialize, Deserialize, EnumVariantNames, EnumString, Display)]
#[strum(serialize_all = "title_case")]
pub enum TextAlignment {
    /// Top left alignment
    TopLeft,
    /// Top center alignment
    TopCenter,
    /// Top right alignment
    TopRight,
    /// Middle left alignment
    MiddleLeft,
    /// Center alignment
    Center,
    /// Middle right alignment
    MiddleRight,
    /// Bottom left alignment
    BottomLeft,
    /// Bottom center alignment
    BottomCenter,
    /// Bottom right alignment
    BottomRight,
}

/// Calculates where text should be rendered for specified alignment and other parameters
pub fn get_alignment_position_for_text(size: (usize, usize), font: &Font, text: &str, scale: Scale, align: TextAlignment, padding: u32, offset: (f32, f32)) -> Point<f32> {
    let (sw, sh) = size;
    let (tw, th) = calculate_bounds_for_text(font, text, scale);

    point(
        match align {
            TextAlignment::TopLeft | TextAlignment::MiddleLeft | TextAlignment::BottomLeft => {
                (padding) as f32 + offset.0
            }

            TextAlignment::TopCenter | TextAlignment::Center | TextAlignment::BottomCenter => {
                (sw as i32 / 2 - tw as i32 / 2 - 1) as f32 + offset.0
            }

            TextAlignment::TopRight | TextAlignment::MiddleRight | TextAlignment::BottomRight => {
                (sw as i32 - tw as i32 - padding as i32) as f32 + offset.0
            }
        },
        match align {
            TextAlignment::TopLeft | TextAlignment::TopCenter | TextAlignment::TopRight => {
                (padding + th) as f32 + offset.1
            }

            TextAlignment::MiddleLeft | TextAlignment::Center | TextAlignment::MiddleRight => {
                (sh as i32 / 2 - th as i32 / 2 + th as i32 - 1) as f32 + offset.1
            }

            TextAlignment::BottomLeft | TextAlignment::BottomCenter | TextAlignment::BottomRight => {
                (sh as i32 - padding as i32) as f32 + offset.1
            }
        },
    )
}

/// Renders aligned text onto provided image with specified parameters
pub fn render_aligned_text_on_image(size: (usize, usize), image: &mut DynamicImage, font: &Font, text: &str, scale: Scale, align: TextAlignment, padding: u32, offset: (f32, f32), color: (u8, u8, u8, u8)) {
    let point = get_alignment_position_for_text(size, font, text, scale.clone(), align, padding, offset);
    render_text_on_image(image, &font, text, scale, point, color);
}

/// Renders aligned shadowed text onto provided image with specified parameters
pub fn render_aligned_shadowed_text_on_image(size: (usize, usize), image: &mut DynamicImage, font: &Font, text: &str, scale: Scale, align: TextAlignment, padding: u32, offset: (f32, f32), color: (u8, u8, u8, u8), shadow_offset: (i32, i32), shadow_color: (u8, u8, u8, u8)) {
    let point = get_alignment_position_for_text(size, font, text, scale.clone(), align, padding, offset);
    render_shadowed_text_on_image(image, &font, text, scale, point, color, shadow_offset, shadow_color);
}