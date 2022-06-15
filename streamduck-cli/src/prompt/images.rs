use std::io::Cursor;
use std::str::Split;
use image::{DynamicImage, GenericImageView, Rgba};
use image::imageops::FilterType;
use image::io::Reader;
use palette::{Hsl, IntoColor, Srgb};
use streamduck_client::daemon::daemon_data::assets::{AddImageResult, ListImagesResult, RemoveImageResult};
use crate::prompt::ClientRef;

fn check_for_true_color() -> bool {
    if let Ok(var) = std::env::var("COLORTERM") {
        if var.find("24bit").is_some() || var.find("truecolor").is_some() {
            return true;
        }
    }

    if let Ok(var) = std::env::var("TERMINAL_EMULATOR") {
        if var.find("JetBrains").is_some() {
            return true;
        }
    }

    false
}

fn print_image_truecolor(image: DynamicImage, size: u32) {
    let decoded_image = image.resize_exact(size, size, FilterType::Lanczos3);

    let mut output = vec![];

    termimage::ops::write_ansi_truecolor(&mut output, &decoded_image);

    if let Ok(output) = String::from_utf8(output) {
        println!("{}", output.replace("\x1B[40m", "\x1B[0m"));
    }
}

fn convert_pixel_to_value(pixel: Rgba<u8>) -> f32 {
    let color = Srgb::new(
        pixel.0[0] as f32 / 255.0,
        pixel.0[1] as f32 / 255.0,
        pixel.0[2] as f32 / 255.0
    );

    let luminance: Hsl = color.into_color();

    return luminance.lightness;
}

fn print_image_bw(image: DynamicImage, size: u32) {
    let ramp = " •:░░▒▒▓▓█";
    let ramp_len = unicode_segmentation::UnicodeSegmentation::graphemes(ramp, true).count();

    let image = image.resize_exact(size, size / 2, FilterType::Lanczos3);

    for y in 0..(size / 2) {
        for x in 0..size {
            let pixel = image.get_pixel(x, y);
            let lum = convert_pixel_to_value(pixel);

            let index = ((lum * (ramp_len - 1) as f32).floor() as usize).max(0).min(ramp_len - 1);

            print!("{}", ramp.chars().nth(index).unwrap_or('w'));
        }

        println!();
    }
}

pub fn show_image(data: String, size: u32) {
    if let Ok(byte_array) = base64::decode(data) {
        if let Ok(recognized_image) = Reader::new(Cursor::new(byte_array)).with_guessed_format() {
            if let Ok(decoded_image) = recognized_image.decode() {
                if check_for_true_color() {
                    print_image_truecolor(decoded_image, size);
                } else {
                    print_image_bw(decoded_image, size);
                }

                return;
            }
        }
    }

    println!("Failed to display image");
}

pub fn list_images(client: ClientRef, current_sn: &str, mut args: Split<&str>) {
    if !current_sn.is_empty() {
        let size = if let Some(size) = args.next() {
            if let Ok(size) = size.parse() {
                size
            } else {
                40
            }
        } else {
            40
        };

        let result = client.list_images(current_sn).expect("Failed to get list of images");

        match result {
            ListImagesResult::DeviceNotFound => println!("image list: Device not found"),
            ListImagesResult::Images(images) => {
                for (identifier, image) in images {
                    println!("{}", if image.animated { "GIF" } else { "Image" });
                    println!("Identifier: {}", identifier);
                    show_image(image.image_blob, size);
                    println!()
                }
            }
        }
    } else {
        println!("image list: No device is selected");
    }
}

pub fn add_image(client: ClientRef, current_sn: &str, args: Split<&str>) {
    if !current_sn.is_empty() {
        let file_path = args.collect::<Vec<&str>>().join(" ");

        if let Ok(byte_array) = std::fs::read(file_path) {
            let result = client.add_image(current_sn, &base64::encode(byte_array)).expect("Failed to add image");

            match result {
                AddImageResult::DeviceNotFound => println!("image add: Device not found"),
                AddImageResult::InvalidData => println!("image add: Invalid image data"),
                AddImageResult::Added(identifier) => println!("image add: Added under identifier '{}'", identifier),
            }
        } else {
            println!("image add: Failed to read file");
        }
    } else {
        println!("image add: No device is specified");
    }
}

pub fn remove_image(client: ClientRef, current_sn: &str, mut args: Split<&str>) {
    if !current_sn.is_empty() {
        if let Some(identifier) = args.next() {
            let result = client.remove_image(current_sn, identifier).expect("Failed to remove image");

            match result {
                RemoveImageResult::NotFound => println!("image add: Not found"),
                RemoveImageResult::Removed => println!("image add: Removed image"),
            }
        } else {
            println!("image remove: Specify identifier")
        }
    } else {
        println!("image remove: No device is specified");
    }
}