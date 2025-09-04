use crate::api::device::BoxDevice;
use crate::api::driver::{DeviceConnectionError, DeviceIdentifier, DeviceListError, Driver};
use crate::api::lua::table_utils::LuaTablesUtils;
use crate::plugins::lua::device::LuaDevice;
use mlua::{Error, Lua, LuaSerdeExt, ObjectLike, Table};
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug)]
pub struct LuaDriver {
    state: Lua,
    obj: Table,
    name: String
}

const LIST_DEVICES_METHOD: &str = "listDevices";
const CONNECT_DEVICE_METHOD: &str = "connectDevice";

#[derive(Debug, Error)]
pub enum LuaDriverCreationError {
    #[error("{0}")]
    LuaError(#[from] mlua::Error),
    #[error("driver table is supposed to have `{LIST_DEVICES_METHOD}` and `{CONNECT_DEVICE_METHOD}` methods")]
    IncompatibleObject
}

impl From<LuaDriverCreationError> for mlua::Error {
    fn from(value: LuaDriverCreationError) -> Self {
        match value {
            LuaDriverCreationError::LuaError(e) => e,
            _ => Error::ExternalError(Arc::new(value))
        }
    }
}

impl LuaDriver {
    pub fn new(state: Lua, obj: Table, name: String) -> Result<Box<LuaDriver>, LuaDriverCreationError> {
        if !obj.contains_keys(&[LIST_DEVICES_METHOD, CONNECT_DEVICE_METHOD])? {
            return Err(LuaDriverCreationError::IncompatibleObject)
        }

        Ok(Box::new(LuaDriver {
            state,
            obj,
            name
        }))
    }
}

impl Driver for LuaDriver {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn list_devices(&self) -> Result<Vec<DeviceIdentifier>, DeviceListError> {
        Ok(self.state.from_value(self.obj.call_method(LIST_DEVICES_METHOD, ())?)?)
    }

    fn connect_device(&self, id: &DeviceIdentifier) -> Result<BoxDevice, DeviceConnectionError> {
        let device_table: Option<Table> = self.obj.call_method(
            CONNECT_DEVICE_METHOD,
            self.state.to_value(id)?
        )?;

        let Some(device_table) = device_table else {
            return Err(DeviceConnectionError::DeviceNotFound)
        };

        Ok(LuaDevice::new(self.state.clone(), device_table, id.clone(), self.name.clone())?)
    }
}