use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use hidapi::{HidApi, HidError};
use streamdeck::{Colour, DeviceImage, Error, ImageMode, Kind, Mirroring, pids, Rotation, StreamDeck};
use tokio::sync::{Mutex, RwLock};
use tokio::task::spawn_blocking;

use streamduck_core::devices::drivers::{Driver, DriverError};
use streamduck_core::devices::metadata::{ButtonLayout, DeviceMetadata};
use streamduck_core::devices::{Device, SharedDevice};
use streamduck_core::devices::buttons::ButtonPosition;
use streamduck_core::image_lib::codecs::jpeg::JpegEncoder;
use streamduck_core::image_lib::{ColorType, DynamicImage, GenericImageView};
use streamduck_core::image_lib::imageops::FilterType;
use streamduck_core::init_managers;

const DRIVER_NAME: &'static str = "streamdeck";
const ELGATO_VENDOR: u16 = 0x0fd9;

pub struct StreamDeckDriver {
    hid_api: HidApi
}

impl StreamDeckDriver {
    /// Attempts to create instance of the driver and connect to HidApi
    pub fn new() -> Result<Arc<StreamDeckDriver>, HidError> {
        Ok(Arc::new(StreamDeckDriver {
            hid_api: HidApi::new()?
        }))
    }
}

fn match_streamdeck(product: u16) -> Option<Kind> {
    match product {
        pids::ORIGINAL => Some(Kind::Original),
        pids::ORIGINAL_V2 => Some(Kind::OriginalV2),
        pids::MINI => Some(Kind::Mini),
        pids::XL => Some(Kind::Xl),
        pids::MK2 => Some(Kind::Mk2),
        _ => None,
    }
}

fn get_product(kind: Kind) -> u16 {
    match kind {
        Kind::Original => pids::ORIGINAL,
        Kind::OriginalV2 => pids::ORIGINAL_V2,
        Kind::Mini => pids::MINI,
        Kind::Xl => pids::XL,
        Kind::Mk2 => pids::MK2
    }
}

fn make_metadata(kind: Kind, serial: &str) -> DeviceMetadata {
    DeviceMetadata {
        driver_name: DRIVER_NAME.to_string(),
        identifier: format!("{}-{}", get_product(kind), serial),
        has_screen: true,
        resolution: kind.image_size(),
        layout: match kind {
            Kind::Original | Kind::OriginalV2 | Kind::Mk2 => ButtonLayout(vec![5, 5, 5]),
            Kind::Mini => ButtonLayout(vec![3, 3]),
            Kind::Xl => ButtonLayout(vec![8, 8, 8, 8])
        },
    }
}

fn parse_identifier(identifier: String) -> Option<(u16, u16, Option<String>)> {
    let split = identifier.split('-').collect::<Vec<&str>>();

    let product = (|| {
        if let Some(product) = split.get(0) {
            if let Ok(product_id) = product.parse::<u16>() {
                return Some(product_id)
            }
        }

        None
    })()?;

    let serial = (|| {
        if let Some(serial) = split.get(1) {
            return Some(serial.to_string())
        }

        None
    })();

    Some((ELGATO_VENDOR, product, serial))
}

fn convert_image(kind: Kind, image: DynamicImage) -> Option<DeviceImage> {
    let (d_w, d_h) = kind.image_size();
    let (og_w, og_h) = image.dimensions();

    // Making sure image is of needed size
    let image = if og_w as usize != d_w || og_h as usize != d_h {
        image.resize_exact(d_w as u32, d_h as u32, FilterType::Triangle)
    } else {
        image
    };

    // Rotating the image
    let image = match kind.image_rotation() {
        Rotation::Rot0 => image,
        Rotation::Rot90 => image.rotate90(),
        Rotation::Rot180 => image.rotate180(),
        Rotation::Rot270 => image.rotate270(),
    };

    // Mirroring the image
    let image = match kind.image_mirror() {
        Mirroring::None => image,
        Mirroring::X => image.flipv(),
        Mirroring::Y => image.fliph(),
        Mirroring::Both => image.flipv().fliph(),
    };

    // Getting byte vector
    let mut data = image.into_rgb8().into_vec();

    // Flipping bytes if device uses BGR
    if let Kind::Original | Kind::Mini = kind {
        println!("flipping");
        for chunk in data.chunks_exact_mut(3) {
            chunk.swap(0, 2);
        }
    }

    // Encoding the image
    Some(DeviceImage::from_bytes(match kind.image_mode() {
        ImageMode::Bmp => data,
        ImageMode::Jpeg => {
            let mut buf = Vec::new();

            let mut encoder = JpegEncoder::new_with_quality(&mut buf, 100);
            encoder.encode(data.as_slice(), d_w as u32, d_h as u32, ColorType::Rgb8).ok()?;

            buf
        }
    }))
}

fn position_to_index(kind: Kind, position: ButtonPosition) -> u8 {
    let row_button_count = match kind {
        Kind::Original | Kind::OriginalV2 | Kind::Mk2 => 5,
        Kind::Mini => 3,
        Kind::Xl => 8
    };

    (position.row * row_button_count + position.column) as u8
}

#[streamduck_core::async_trait]
impl Driver for StreamDeckDriver {
    fn name(&self) -> String {
        "streamdeck".to_string()
    }

    async fn list_devices(&self) -> Vec<DeviceMetadata> {
        self.hid_api
            .device_list()
            .filter_map(|d| { // Checking if the device is valid
                if d.vendor_id() != ELGATO_VENDOR {
                    return None;
                }

                if let Some(serial) = d.serial_number() {
                    if !serial.chars().all(|c| c.is_alphanumeric()) {
                        return None;
                    }

                    Some((
                        match_streamdeck(d.product_id())?,
                        serial.to_string()
                    ))
                } else {
                    None
                }
            })
            .map(|(kind, serial)| { // Mapping device info into metadata
                make_metadata(kind, &serial)
            })
            .collect()
    }

    async fn connect_device(&self, identifier: String) -> Result<SharedDevice, DriverError> {
        if let Some((vendor, product, serial)) = parse_identifier(identifier) {
            match StreamDeck::connect_with_hid(&self.hid_api, vendor, product, serial) {
                Ok(mut device) => {
                    let kind = device.kind();
                    let serial = device.serial().unwrap_or_else(|_| "".to_string());

                    Ok(Arc::new(StreamDeckDevice {
                        streamdeck: Mutex::new(device),
                        kind,
                        serial,
                        image_cache: Default::default()
                    }))
                }

                Err(e) => Err(DriverError::FailedToConnect(
                    format!("Failed to connect: {:?}", e)
                ))
            }
        } else {
            Err(DriverError::InvalidIdentifier)
        }
    }
}

pub struct StreamDeckDevice {
    streamdeck: Mutex<StreamDeck>,
    kind: Kind,
    serial: String,
    image_cache: RwLock<HashMap<u128, DeviceImage>>
}

#[streamduck_core::async_trait]
impl Device for StreamDeckDevice {
    fn metadata(&self) -> DeviceMetadata {
        make_metadata(self.kind, &self.serial)
    }

    async fn poll(&self) {
        todo!()
    }

    async fn reset(&self) {
        self.streamdeck.lock().await.reset().ok();
    }

    async fn clear_screen(&self) {
        let mut lock = self.streamdeck.lock().await;

        for i in 0 .. self.kind.keys() {
            lock.set_button_rgb(i, &Colour {
                r: 0,
                g: 0,
                b: 0
            }).ok();
        }
    }

    async fn set_brightness(&self, brightness: u8) {
        let mut lock = self.streamdeck.lock().await;

        lock.set_brightness(brightness).ok();
    }

    async fn contains_image(&self, key: u128) -> bool {
        self.image_cache.read().await
            .contains_key(&key)
    }

    async fn add_image(&self, key: u128, image: DynamicImage) {
        let image = convert_image(self.kind, image).unwrap(); // temp unwrap
        self.image_cache.write().await
            .insert(key, image);
    }

    async fn remove_image(&self, key: u128) -> bool {
        self.image_cache.write().await
            .remove(&key).is_some()
    }

    async fn set_button_image(&self, position: ButtonPosition, key: u128) {
        let mut lock = self.streamdeck.lock().await;

        if let Some(image) = self.image_cache.read().await.get(&key) {
            lock.write_button_image(
                position_to_index(self. kind, position),
                image
            ).ok();
        }
    }
}