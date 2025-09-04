//! HIDAPI interface for Lua

pub mod utils;

use crate::api::lua::hid::utils::LuaByteArray;
use hidapi::{BusType, DeviceInfo, HidApi, HidDevice};
use mlua::{FromLua, Lua, LuaSerdeExt, UserData, UserDataMethods, Value};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::iter::{Enumerate, Map};
use std::ops::{Deref, DerefMut};
use std::time::Instant;

pub fn lua_enumerate<T, I: IntoIterator<Item=T>>(iter: I) -> Map<Enumerate<<I as IntoIterator>::IntoIter>, fn((usize, T)) -> (usize, T)> {
    iter.into_iter()
        .enumerate()
        .map(|(i, v)| (i + 1, v))
}

pub struct HidForLua {
    hid: RefCell<HidApi>,
    last_refresh: RefCell<Instant>
}

impl HidForLua {
    pub fn new() -> Self {
        Self {
            hid: HidApi::new().unwrap().into(),
            last_refresh: Instant::now().into()
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct LuaDeviceInfo {
    pub path: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub serial_number: Option<String>,
    pub release_number: u16,
    pub manufacturer_string: Option<String>,
    pub product_string: Option<String>,
    pub interface_number: i32,
    pub bus_type: String
}

impl From<DeviceInfo> for LuaDeviceInfo {
    fn from(value: DeviceInfo) -> Self {
        LuaDeviceInfo::from(&value)
    }
}

impl From<&DeviceInfo> for LuaDeviceInfo {
    fn from(value: &DeviceInfo) -> Self {
        Self {
            path: value.path().to_string_lossy().to_string(),
            vendor_id: value.vendor_id(),
            product_id: value.product_id(),
            serial_number: value.serial_number().map(str::to_string),
            release_number: value.release_number(),
            manufacturer_string: value.manufacturer_string().map(str::to_string),
            product_string: value.product_string().map(str::to_string),
            interface_number: value.interface_number(),
            bus_type: match value.bus_type() {
                BusType::Unknown => "Unknown",
                BusType::Usb => "Usb",
                BusType::Bluetooth => "Bluetooth",
                BusType::I2c => "I2c",
                BusType::Spi => "Spi"
            }.to_string()
        }
    }
}

impl UserData for HidForLua {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("refresh", |_lua, this, _: ()| {
            let mut hid = this.hid.borrow_mut();
            
            hid.refresh_devices().map_err(mlua::Error::external)?;
            this.last_refresh.replace(Instant::now());

            Ok(())
        });

        methods.add_method("timeSinceRefresh", |_lua, this, _: ()| {
            let time = this.last_refresh.borrow();
            Ok(time.elapsed().as_secs_f32())
        });

        methods.add_method("list", |lua, this, (vid, pid): (Option<u16>, Option<u16>)| {
            let hid = this.hid.borrow();

            let mut infos = hid.device_list()
                .filter(|d| vid.is_none_or(|v| d.vendor_id() == v))
                .filter(|d| pid.is_none_or(|p| d.product_id() == p))
                .map(LuaDeviceInfo::from)
                .collect::<Vec<_>>();
            infos.dedup();

            lua.to_value(&infos)
        });
        
        methods.add_method("open", |_lua, this, (vendor, product, serial): (u16, u16, Option<String>)| {
            let hid = this.hid.borrow();

            let device = if let Some(serial) = serial {
                hid.open_serial(vendor, product, &serial)
            } else {
                hid.open(vendor, product)
            }.map_err(mlua::Error::external)?;

            device.set_blocking_mode(false).map_err(mlua::Error::external)?;

            Ok(LuaHidDevice {
                device
            })
        });
    }
}

pub struct LuaHidDevice {
    device: HidDevice
}

impl Deref for LuaHidDevice {
    type Target = HidDevice;

    fn deref(&self) -> &Self::Target {
        &self.device
    }
}

impl DerefMut for LuaHidDevice {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.device
    }
}

impl FromLua for LuaHidDevice {
    fn from_lua(value: Value, _lua: &Lua) -> mlua::Result<Self> {
        match value {
            Value::UserData(userdata) => Ok(userdata.take::<LuaHidDevice>()?),
            _ => Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "LuaHidDevice".to_string(),
                message: None,
            }),
        }
    }
}

impl UserData for LuaHidDevice {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("getFeatureReport", |_lua, me, amount: usize| {
            let mut buf: LuaByteArray = vec![0u8; amount].into();
            let written = me.get_feature_report(&mut buf).map_err(mlua::Error::external)?;
            Ok((written, buf))
        });

        methods.add_method("sendFeatureReport", |_lua, me, arr: LuaByteArray| {
            me.send_feature_report(arr.as_slice()).map_err(mlua::Error::external)?;
            Ok(())
        });

        methods.add_method("read", |_lua, me, amount: usize| {
            let mut buf: LuaByteArray = vec![0u8; amount].into();
            let written = me.read(&mut buf).map_err(mlua::Error::external)?;
            Ok((written, buf))
        });

        methods.add_method("write", |_lua, me, arr: LuaByteArray| {
            me.write(arr.as_slice()).map_err(mlua::Error::external)?;
            Ok(())
        });

        methods.add_method("getIndexedString", |_lua, me, index: i32| {
            Ok(me.get_indexed_string(index).map_err(mlua::Error::external)?)
        });

        methods.add_method("info", |lua, me, _: ()| {
            Ok(lua.to_value(&me.get_device_info()
                .map_err(mlua::Error::external)
                .map(LuaDeviceInfo::from)?))
        });

        methods.add_function("destroy", |_lua, _this: LuaHidDevice| {
            Ok(())
        });
    }
}