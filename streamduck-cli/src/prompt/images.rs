use std::io::Cursor;
use std::str::Split;
use image::imageops::FilterType;
use image::io::Reader;
use streamduck_client::daemon::socket::daemon_data::{AddImageResult, ListImagesResult, RemoveImageResult};
use crate::prompt::ClientRef;

pub fn show_image(data: String, size: u32) {
    if let Ok(byte_array) = base64::decode(data) {
        if let Ok(recognized_image) = Reader::new(Cursor::new(byte_array)).with_guessed_format() {
            if let Ok(decoded_image) = recognized_image.decode() {
                let decoded_image = decoded_image.resize_exact(size, size, FilterType::Lanczos3);

                let mut output = vec![];

                termimage::ops::write_ansi_truecolor(&mut output, &decoded_image);

                if let Ok(output) = String::from_utf8(output) {
                    println!("{}", output.replace("\x1B[40m", "\x1B[0m"));
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
                72
            }
        } else {
            72
        };

        let result = client.list_images(current_sn).expect("Failed to get list of images");

        match result {
            ListImagesResult::DeviceNotFound => println!("image list: Device not found"),
            ListImagesResult::Images(images) => {
                for (identifier, image) in images {
                    println!("Image");
                    println!("Identifier: {}", identifier);
                    show_image(image, size);
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