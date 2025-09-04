use semver::Version;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Clone)]
pub struct LuaPluginManifest {
    pub name: String,
    pub api_version: Version,
    pub main_file: PathBuf
}