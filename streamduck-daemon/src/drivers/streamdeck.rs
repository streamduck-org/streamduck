use std::collections::HashMap;
use std::sync::Arc;

use elgato_streamdeck::{AsyncStreamDeck, list_devices, StreamDeckError};
use elgato_streamdeck::asynchronous::{ButtonStateReader, ButtonStateUpdate};
use elgato_streamdeck::images::convert_image_async;
use elgato_streamdeck::info::Kind;
use hidapi::{HidApi, HidError};
use tokio::sync::RwLock;

use streamduck_core::devices::{Device, DeviceError, SharedDevice};
use streamduck_core::devices::buttons::{ButtonEvent, ButtonPosition};
use streamduck_core::devices::drivers::{Driver, DriverError};
use streamduck_core::devices::metadata::{ButtonLayout, DeviceMetadata};
use streamduck_core::events::EventDispatcher;
use streamduck_core::image_lib::DynamicImage;

const DRIVER_NAME: &'static str = "streamdeck";

pub struct StreamDeckDriver;

impl StreamDeckDriver {
    /// Attempts to create instance of the driver and connect to HidApi
    pub fn new() -> Result<Arc<StreamDeckDriver>, HidError> {
        Ok(Arc::new(StreamDeckDriver))
    }
}

fn make_metadata(kind: Kind, serial: &str) -> DeviceMetadata {
    DeviceMetadata {
        driver_name: DRIVER_NAME.to_string(),
        identifier: format!("{}-{}", kind.product_id(), serial),
        has_screen: true,
        resolution: kind.key_image_format().size,
        layout: match kind {
            Kind::Original | Kind::OriginalV2 | Kind::Mk2 => ButtonLayout(vec![5, 5, 5]),
            Kind::Mini | Kind::MiniMk2 => ButtonLayout(vec![3, 3]),
            Kind::Xl | Kind::XlV2 => ButtonLayout(vec![8, 8, 8, 8]),
            Kind::Pedal => ButtonLayout(vec![3]),
        },
    }
}

fn parse_identifier(identifier: String) -> Option<(Kind, String)> {
    let split = identifier.split('-').collect::<Vec<&str>>();

    let product = (|| {
        if let Some(product) = split.get(0) {
            if let Ok(product_id) = product.parse::<u16>() {
                return Some(Kind::from_pid(product_id)?)
            }
        }

        None
    })()?;

    let serial = (|| {
        if let Some(serial) = split.get(1) {
            return Some(serial.to_string())
        }

        None
    })()?;

    Some((product, serial))
}


fn position_to_index(kind: Kind, position: ButtonPosition) -> u8 {
    ((position.row * kind.column_count() as u16) + position.column) as u8
}

fn index_to_position(kind: Kind, index: u8) -> ButtonPosition {
    ButtonPosition::from(((index / kind.column_count()) as u16, (index % kind.column_count()) as u16))
}

#[streamduck_core::async_trait]
impl Driver for StreamDeckDriver {
    fn name(&self) -> String {
        "streamdeck".to_string()
    }

    async fn list_devices(&self, hidapi: &HidApi) -> Vec<DeviceMetadata> {
        list_devices(hidapi).into_iter()
            .map(|(kind, serial)| { // Mapping device info into metadata
                make_metadata(kind, &serial)
            })
            .collect()
    }

    async fn connect_device(&self, hidapi: &HidApi, identifier: String) -> Result<SharedDevice, DriverError> {
        if let Some((kind, serial)) = parse_identifier(identifier) {
            match AsyncStreamDeck::connect(hidapi, kind, &serial) {
                Ok(streamdeck) => {
                    let kind = streamdeck.kind();
                    let serial = streamdeck.serial_number().await.unwrap_or_else(|_| "".to_string());

                    Ok(Arc::new(StreamDeckDevice {
                        reader: streamdeck.get_reader(),
                        streamdeck,
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
    streamdeck: Arc<AsyncStreamDeck>,
    reader: Arc<ButtonStateReader>,
    kind: Kind,
    serial: String,
    image_cache: RwLock<HashMap<u128, Vec<u8>>>
}

fn map_device_error(e: StreamDeckError) -> DeviceError {
    match e {
        StreamDeckError::HidError(e) => e.into(),
        StreamDeckError::ImageError(e) => e.into(),
        StreamDeckError::Utf8Error(e) => DeviceError::Other(Box::new(e)),
        StreamDeckError::JoinError(e) => DeviceError::Other(Box::new(e)),
        StreamDeckError::NoScreen => DeviceError::InvalidUse("There's no screen".to_string()),
        StreamDeckError::InvalidKeyIndex => DeviceError::InvalidUse("Invalid key index".to_string()),
        StreamDeckError::UnrecognizedPID => DeviceError::InvalidUse("Unrecognized PID".to_string()),
    }
}

#[streamduck_core::async_trait]
impl Device for StreamDeckDevice {
    fn metadata(&self) -> DeviceMetadata {
        make_metadata(self.kind, &self.serial)
    }

    async fn poll(&self, dispatcher: &Arc<EventDispatcher>) -> Result<(), DeviceError> {
        let updates = self.reader.read(40.0).await
            .map_err(map_device_error)?;

        for update in updates {
            dispatcher.invoke(match update {
                ButtonStateUpdate::ButtonDown(index) => {
                    ButtonEvent::ButtonDown(index_to_position(self.kind, index))
                }

                ButtonStateUpdate::ButtonUp(index) => {
                    ButtonEvent::ButtonUp(index_to_position(self.kind, index))
                }
            }).await;
        }

        Ok(())
    }

    async fn reset(&self) -> Result<(), DeviceError> {
        Ok(
            self.streamdeck.reset().await
                .map_err(map_device_error)?
        )
    }

    async fn clear_screen(&self) -> Result<(), DeviceError> {
        for i in 0 .. self.kind.key_count() {
            self.streamdeck.clear_button_image(i).await
                .map_err(map_device_error)?;
        }

        Ok(())
    }

    async fn set_brightness(&self, brightness: u8) -> Result<(), DeviceError> {
        Ok(
            self.streamdeck.set_brightness(brightness).await
                .map_err(map_device_error)?
        )
    }

    async fn contains_image(&self, key: u128) -> bool {
        self.image_cache.read().await
            .contains_key(&key)
    }

    async fn add_image(&self, key: u128, image: DynamicImage) -> Result<(), DeviceError> {
        let image = convert_image_async(self.kind, image).await
            .map_err(map_device_error)?;
        self.image_cache.write().await
            .insert(key, image);
        Ok(())
    }

    async fn remove_image(&self, key: u128) -> bool {
        self.image_cache.write().await
            .remove(&key).is_some()
    }

    async fn clear_button_image(&self, position: ButtonPosition) -> Result<(), DeviceError> {
        Ok(self.streamdeck.clear_button_image(position_to_index(self.kind, position)).await
            .map_err(map_device_error)?)
    }

    async fn set_button_image(&self, position: ButtonPosition, key: u128) -> Result<(), DeviceError> {
        if let Some(image) = self.image_cache.read().await.get(&key) {
            Ok(self.streamdeck.write_image(
                position_to_index(self.kind, position),
                image
            ).await.map_err(map_device_error)?)
        } else {
            Err(DeviceError::ImageMissing)
        }
    }
}
