use crate::api::plugin::Plugin;
use mlua::{Lua, LuaSerdeExt, MultiValue, ObjectLike, Table, UserData, UserDataMethods};
use semver::{Version, VersionReq};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use log::{error, info, warn};
use thiserror::Error;
use crate::api::driver::Driver;
use crate::api::lua::common::CommonLuaStatic;
use crate::api::lua::hid::utils::{buf_util, byte_util};
use crate::api::lua::image::LuaDynamicImageStatic;
use crate::api::lua::validator::validate_json_value;
use crate::api::value_type::ValueType;
use crate::plugins::lua::driver::LuaDriver;
use crate::plugins::lua::HID_TO_USE;
use crate::plugins::lua::manifest::LuaPluginManifest;

pub static CURRENT_LUA_API_VERSION_REQ: LazyLock<VersionReq> = LazyLock::new(|| {
    VersionReq::parse("~1").unwrap()
});

pub struct LuaPlugin {
    state: Lua,
    manifest: LuaPluginManifest,
    plugin_folder: PathBuf,
    registry: LuaPluginComponentRegistry
}

#[derive(Default, Debug)]
struct LuaPluginComponentRegistry {
    plugin_name: String,
    drivers: Vec<Box<LuaDriver>>
}

impl UserData for LuaPluginComponentRegistry {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("registerDriver",
       |lua, reg, (name, obj): (String, Table)| {
           info!("[{}]: Registered driver '{}'", reg.plugin_name, name);
           reg.drivers.push(LuaDriver::new(lua.clone(), obj, name)?);
           Ok(())
        })
    }
}

#[derive(Error, Debug)]
pub enum LuaPluginCreationError {
    #[error("failed to load lua script:\n{0}")]
    LuaError(#[from] mlua::Error),

    #[error("plugin is using too new or too old Lua API (expected to match {0}, but got {1})")]
    IncompatibleLuaApi(&'static VersionReq, Version),

    #[error("main_file path ({0}) is not found in the plugin folder")]
    MainFileNotFound(PathBuf)
}

const REGISTER_FUNCTION: &str = "Register";
const POST_INIT_FUNCTION: &str = "PostInit";
const TICK_FUNCTION: &str = "Tick";

impl LuaPlugin {
    fn configure_lua_state(plugin_folder: &Path, manifest: &LuaPluginManifest) -> Result<Lua, mlua::Error> {
        // This is fine!
        let lua = unsafe { Lua::unsafe_new() };

        // Configure globals
        let current_globals = lua.globals();

        {
            let package = current_globals.get::<Table>("package")?;

            let package_path = package.get::<String>("path")?;

            let lua_paths = &[
                plugin_folder.join("?.lua"),
                plugin_folder.join("share/luajit-2.1/?.lua"),
                plugin_folder.join("share/lua/5.1/?.lua"),
                plugin_folder.join("lib64/lua/5.1/?.lua"),
                plugin_folder.join("lib64/lua/5.1/loadall.lua"),
                plugin_folder.join("lib/lua/5.1/?.lua"),
                plugin_folder.join("lib/lua/5.1/loadall.lua")
            ];

            package.set("path", format!(
                "{0};{1}",
                lua_paths.into_iter()
                    .map(|p| p.to_string_lossy())
                    .collect::<Vec<_>>()
                    .join(";"),
                package_path
            ))?;

            let extension = if cfg!(target_os = "windows") {
                "dll"
            } else {
                "so"
            };

            let plain_variant = format!("?.{}", extension);
            let lib_variant = format!("lib?.{}", extension);
            let loadall = format!("loadall.{}", extension);

            let package_cpath = package.get::<String>("cpath")?;

            let c_paths = &[
                plugin_folder.join(&plain_variant),
                plugin_folder.join(&lib_variant),
                plugin_folder.join("lib64/lua/5.1/").join(&plain_variant),
                plugin_folder.join("lib64/lua/5.1/").join(&lib_variant),
                plugin_folder.join("lib64/lua/5.1/").join(&loadall),
                plugin_folder.join("lib/lua/5.1/").join(&plain_variant),
                plugin_folder.join("lib/lua/5.1/").join(&lib_variant),
                plugin_folder.join("lib/lua/5.1/").join(&loadall)
            ];

            package.set("cpath", format!(
                "{0};{1}",
                c_paths.into_iter()
                    .map(|p| p.to_string_lossy())
                    .collect::<Vec<_>>()
                    .join(";"),
                package_cpath,
            ))?;

            package.set("plugin_dir", plugin_folder.to_string_lossy())?;

            let plugin_name = manifest.name.clone();
            let print_func = lua.create_function(move |_, a: MultiValue| {
                let to_print = a.into_iter()
                    .map(|v| v.to_string())
                    .collect::<Result<Vec<String>, _>>()?
                    .join("\t");

                info!("[{}]: {}", plugin_name, to_print);

                Ok(())
            })?;
            current_globals.set("print", print_func)?;

            let hid = HID_TO_USE.lock().map_err(mlua::Error::external)?;

            current_globals.set("image", LuaDynamicImageStatic)?;
            current_globals.set("byte", byte_util(&lua)?)?;
            current_globals.set("buf", buf_util(&lua)?)?;
            current_globals.set("hid", hid(&lua)?)?;
            current_globals.set("common", CommonLuaStatic)?;

            current_globals.set("test_validation", lua.create_function(|l, (sig, val): (ValueType, mlua::Value)| {
                let json_val: serde_json::Value = l.from_value(val)?;
                
                Ok(validate_json_value(&json_val, &sig))
            })?)?;
        }

        Ok(lua)
    }

    pub fn new(plugin_folder: PathBuf, manifest: LuaPluginManifest) -> Result<Box<LuaPlugin>, LuaPluginCreationError> {
        if !CURRENT_LUA_API_VERSION_REQ.matches(&manifest.api_version) {
            return Err(LuaPluginCreationError::IncompatibleLuaApi(
                CURRENT_LUA_API_VERSION_REQ.deref(),
                manifest.api_version.clone()
            ))
        }

        let path_to_main = plugin_folder.join(&manifest.main_file);

        if !path_to_main.exists() {
            return Err(LuaPluginCreationError::MainFileNotFound(
                manifest.main_file
            ))
        }

        let state = Self::configure_lua_state(&plugin_folder, &manifest)?;

        state.load(path_to_main).exec()?;

        let mut registry = LuaPluginComponentRegistry {
            plugin_name: manifest.name.clone(),
            ..Default::default()
        };

        state.scope(|x| {
            let reference = x.create_userdata_ref_mut(&mut registry)?;

            if state.globals().contains_key(REGISTER_FUNCTION)? {
                state.globals().call_function::<()>(REGISTER_FUNCTION, reference)?;
            } else {
                warn!("Register function was not found for '{}', continuing as empty plugin", manifest.name)
            }

            Ok(())
        })?;

        Ok(Box::new(LuaPlugin {
            state,
            registry,
            manifest,
            plugin_folder,
        }))
    }

    fn run_post_init(&self) -> Result<(), mlua::Error> {
        let globals = self.state.globals();

        if globals.contains_key(POST_INIT_FUNCTION)? {
            globals.call_function::<()>(POST_INIT_FUNCTION, ())?;
        }

        Ok(())
    }

    fn run_tick(&self) -> Result<(), mlua::Error> {
        let globals = self.state.globals();

        if globals.contains_key(TICK_FUNCTION)? {
            globals.call_function::<()>(TICK_FUNCTION, ())?;
        }

        Ok(())
    }
}

impl Plugin for LuaPlugin {
    fn name(&self) -> String {
        self.manifest.name.clone()
    }

    fn post_init(&self) {
        match self.run_post_init() {
            Err(e) => {
                error!("'{}' failed to run post init:\n{}", &self.manifest.name, e)
            }
            _ => {}
        }
    }

    fn drivers(&self) -> Vec<&dyn Driver> {
        self.registry.drivers.iter()
            .map(|lua_driver| lua_driver.as_ref() as &dyn Driver)
            .collect()
    }

    fn tick(&self) {
        match self.run_tick() {
            Err(e) => {
                error!("'{}' failed to run tick:\n{}", &self.manifest.name, e)
            }
            _ => {}
        }
    }
}