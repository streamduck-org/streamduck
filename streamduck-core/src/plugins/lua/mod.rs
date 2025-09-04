mod plugin;
mod manifest;
mod driver;
mod device;

use std::fs::DirEntry;
use std::{fs, io};
use std::path::Path;
use std::sync::{LazyLock, Mutex};
use log::{error, info};
use mlua::{AnyUserData, Lua};
use thiserror::Error;
use crate::api::id::PluginID;
use crate::api::lua::hid::HidForLua;
use crate::api::plugin::{BoxPlugin, PluginSource};
use crate::plugins::lua::manifest::LuaPluginManifest;
use crate::plugins::lua::plugin::{LuaPlugin, LuaPluginCreationError};
use crate::util::id_set::{IdSet, IdSetError};

pub static HID_TO_USE: LazyLock<Mutex<fn(&Lua) -> Result<AnyUserData, mlua::Error>>> = LazyLock::new(|| 
    Mutex::new(|l| l.create_userdata(HidForLua::new()))
);

pub struct LuaPluginSource;

#[derive(Debug, Error)]
pub enum LuaPluginLoadError {
    #[error("failed to interact with filesystem:\n{0}")]
    IOError(#[from] io::Error),

    #[error("failed to read manifest:\n{0}")]
    DeserializationError(#[from] serde_json::Error),

    #[error("failed to load plugin:\n{0}")]
    CreationError(#[from] LuaPluginCreationError),

    #[error("{0}")]
    IdCollision(#[from] IdSetError),

    #[error("Lua plugin is supposed to consist of a folder containing manifest.json file")]
    InvalidPluginStructure,
}

fn load_plugin(entry: DirEntry, id_set: &mut IdSet<PluginID>) -> Result<BoxPlugin, LuaPluginLoadError> {
    if !entry.file_type()?.is_dir() {
        return Err(LuaPluginLoadError::InvalidPluginStructure)
    }

    let plugin_folder = entry.path();
    let folder_contents = plugin_folder.read_dir()?
        .collect::<Result<Vec<DirEntry>, _>>()?;

    let Some(manifest_location) = folder_contents.iter()
        .find(|e| e.path().ends_with("manifest.json"))
    else {
        return Err(LuaPluginLoadError::InvalidPluginStructure)
    };

    let manifest: LuaPluginManifest = serde_json::from_str(
        &fs::read_to_string(manifest_location.path())?
    )?;

    let plugin_name = manifest.name.clone();
    id_set.reserve(plugin_name.clone())?;

    let plugin = match LuaPlugin::new(plugin_folder, manifest) {
        Ok(v) => v,
        Err(e) => {
            id_set.remove(&plugin_name);
            return Err(e.into())
        }
    };

    Ok(plugin)
}

impl PluginSource for LuaPluginSource {
    fn name(&self) -> &'static str {
        "Lua Plugin Source"
    }

    fn load_plugins(&self, id_set: &mut IdSet<PluginID>, plugin_path: impl AsRef<Path>) -> Vec<BoxPlugin> {
        let read_dir = match fs::read_dir(plugin_path) {
            Ok(v) => v,
            Err(e) => {
                error!("unable to read plugins folder: {}", e);
                return vec![];
            }
        };

        let mut loaded_plugins = vec![];

        for child in read_dir {
            let child = match child {
                Ok(v) => v,
                Err(e) => {
                    error!("failed to interact with plugin folder: {}", e);
                    continue;
                }
            };

            let child_path = child.path();

            let plugin = match load_plugin(child, id_set) {
                Ok(v) => v,
                Err(e) => {
                    error!("error processing plugin at '{}': {}", child_path.to_string_lossy(), e);
                    continue;
                }
            };

            info!("Successfully loaded '{}'", plugin.name());

            loaded_plugins.push(plugin);
        }

        loaded_plugins
    }
}