use crate::api::device::event::DeviceEvent;
use crate::api::device::input::Input;
use crate::api::device::output::Output;
use crate::api::device::setting::{DeviceSetting, StrippedDeviceSetting};
use crate::api::driver::DriverspacedDeviceIdentifier;
use crate::api::lua::image::LuaDynamicImage;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

pub mod input;
pub mod output;
pub mod setting;
pub mod event;

pub trait Device {
    fn id(&self) -> DriverspacedDeviceIdentifier;
    fn disconnect(&self) -> Result<(), DeviceError>;
    fn layout(&self) -> DeviceLayout;
    fn settings(&self) -> Result<Vec<DeviceSetting>, DeviceError>;
    fn set_setting(&self, name: &str, value: Value) -> Result<(), DeviceError>;
    fn set_display_image(&self, image: &LuaDynamicImage, hash: u64) -> Result<(), DeviceError>;
    fn clear_image_cache(&self, hash: u64) -> Result<(), DeviceError>;
    fn set_light(&self, state: bool) -> Result<(), DeviceError>;
    fn set_rgb_light(&self, color: (u8, u8, u8)) -> Result<(), DeviceError>;
    fn tick(&self) -> Result<Vec<DeviceEvent>, DeviceError>;
    
    fn set_setting_stripped(&self, setting: StrippedDeviceSetting) -> Result<(), DeviceError> {
        self.set_setting(&setting.name, setting.value)
    }
}

pub type BoxDevice = Box<dyn Device>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceLayout {
    pub name: String,
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>
}

#[derive(Debug, Error)]
pub enum DeviceError {
    #[error("Device disconnected!")]
    DeviceDisconnected,
    #[error("Lua error happened while interacting with device: {0}")]
    LuaError(#[from] mlua::Error),
    #[error("Error happened while interacting with device: {0}")]
    Other(#[from] anyhow::Error)
}