use crate::api::device::event::DeviceEvent;
use crate::api::device::setting::DeviceSetting;
use crate::api::device::{Device, DeviceError, DeviceLayout};
use crate::api::driver::{DeviceConnectionError, DeviceIdentifier, DriverspacedDeviceIdentifier};
use crate::api::lua::image::LuaDynamicImage;
use crate::api::lua::table_utils::LuaTablesUtils;
use crate::api::lua::validator::validate_json_value;
use mlua::{Error, Lua, LuaSerdeExt, ObjectLike, Table, UserData, UserDataMethods, Value};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use anyhow::anyhow;
use thiserror::Error;

#[derive(Debug)]
pub struct LuaDevice {
    state: Lua,
    obj: Table,
    id: DeviceIdentifier,
    driver_name: String,
    layout: Rc<RefCell<DeviceLayout>>
}

const LAYOUT_METHOD: &str = "layout";
const TICK_METHOD: &str = "tick";
const DISCONNECT_METHOD: &str = "disconnect";

#[derive(Debug, Error)]
pub enum LuaDeviceCreationError {
    #[error("{0}")]
    LuaError(#[from] mlua::Error),
    #[error("device table is supposed to have `{LAYOUT_METHOD}` and `{DISCONNECT_METHOD}` methods")]
    IncompatibleObject
}

impl From<LuaDeviceCreationError> for mlua::Error {
    fn from(value: LuaDeviceCreationError) -> Self {
        match value {
            LuaDeviceCreationError::LuaError(e) => e,
            _ => Error::ExternalError(Arc::new(value))
        }
    }
}

impl From<LuaDeviceCreationError> for DeviceConnectionError {
    fn from(value: LuaDeviceCreationError) -> Self {
        match value {
            LuaDeviceCreationError::LuaError(e) => Self::LuaError(e),
            _ => Self::OtherError(anyhow!("{}", value))
        }
    }
}

impl LuaDevice {
    pub fn new(state: Lua, obj: Table, id: DeviceIdentifier, driver: String) -> Result<Box<LuaDevice>, LuaDeviceCreationError> {
        if !obj.contains_keys(&[LAYOUT_METHOD, DISCONNECT_METHOD])? {
            return Err(LuaDeviceCreationError::IncompatibleObject)
        }

        let layout: DeviceLayout = state.from_value(obj.call_method(LAYOUT_METHOD, ())?)?;

        Ok(Box::new(LuaDevice {
            state,
            obj,
            id,
            driver_name: driver,
            layout: Rc::new(layout.into())
        }))
    }
}

impl Device for LuaDevice {
    fn id(&self) -> DriverspacedDeviceIdentifier {
        DriverspacedDeviceIdentifier {
            id: self.id.clone(),
            driver: self.driver_name.clone()
        }
    }

    fn disconnect(&self) -> Result<(), DeviceError> {
        Ok(self.obj.call_method(DISCONNECT_METHOD, ())?)
    }

    fn layout(&self) -> DeviceLayout {
        self.layout.borrow().clone()
    }

    fn settings(&self) -> Result<Vec<DeviceSetting>, DeviceError> {
        todo!()
    }

    fn set_setting(&self, name: &str, value: serde_json::Value) -> Result<(), DeviceError> {
        todo!()
    }

    fn set_display_image(&self, image: &LuaDynamicImage, hash: u64) -> Result<(), DeviceError> {
        todo!()
    }

    fn clear_image_cache(&self, hash: u64) -> Result<(), DeviceError> {
        todo!()
    }

    fn set_light(&self, state: bool) -> Result<(), DeviceError> {
        todo!()
    }

    fn set_rgb_light(&self, color: (u8, u8, u8)) -> Result<(), DeviceError> {
        todo!()
    }

    fn tick(&self) -> Result<Vec<DeviceEvent>, DeviceError> {
        let mut collector = LuaDeviceEventCollector {
            layout: self.layout.clone(),
            events: vec![],
        };

        self.state.scope(|s| {
            let borrowed = s.create_userdata_ref_mut(&mut collector)?;

            if self.obj.contains_key(TICK_METHOD)? {
                self.obj.call_method::<()>(TICK_METHOD, borrowed)?;
            }

            Ok(())
        })?;

        Ok(collector.events)
    }
}

pub struct LuaDeviceEventCollector {
    layout: Rc<RefCell<DeviceLayout>>,
    pub events: Vec<DeviceEvent>
}

impl UserData for LuaDeviceEventCollector {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("input", |lua, this, (index, name, value): (usize, String, Value)| {
            let json_value = lua.from_value::<serde_json::Value>(value)?;
            let index = index - 1;
            let layout = this.layout.borrow();

            let Some(input) = layout.inputs.get(index) else {
                return Err(Error::runtime(format!("Specified input index ({}) wasn't found!", index + 1)))
            };

            let Some(event_info) = input.events.iter().find(|e| e.name == name) else {
                return Err(Error::runtime(format!("Undeclared event '{}' was invoked!", name)))
            };

            if !validate_json_value(&json_value, &event_info.signature) {
                return Err(Error::runtime(
                    format!(
                        "Provided data doesn't match the event signature!\nExpected: {:#?}\nGot: {:#?}",
                        event_info.signature,
                        json_value
                    )
                ))
            }

            this.events.push(DeviceEvent::InputEvent {
                input: index,
                name,
                value: json_value
            });

            Ok(())
        });

        methods.add_method_mut("layoutChanged", |_, this, _: ()| {
            this.events.push(DeviceEvent::DeviceLayoutChanged);
            Ok(())
        })
    }
}
