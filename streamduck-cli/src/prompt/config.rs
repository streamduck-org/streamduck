use std::fs;
use std::path::PathBuf;
use std::str::{FromStr, Split};
use streamduck_client::daemon::socket::daemon_data::{ExportDeviceConfigResult, ImportDeviceConfigResult, ReloadDeviceConfigResult, ReloadDeviceConfigsResult, SaveDeviceConfigResult, SaveDeviceConfigsResult};
use crate::prompt::ClientRef;

pub fn reload_config(client: ClientRef, mut args: Split<&str>, current_sn: &String) {
    if let Some(arg) = args.next() {
        if arg == "all" {
            match client.reload_device_configs().expect("Failed to reload configs") {
                ReloadDeviceConfigsResult::ConfigError => println!("config reload: Error happened while reloading configs, check daemon logs"),
                ReloadDeviceConfigsResult::Reloaded => println!("config reload: Reloaded"),
            }
        } else {
            println!("config reload: Unknown operation");
        }
    } else {
        if !current_sn.is_empty() {
            match client.reload_device_config(current_sn).expect("Failed to reload config") {
                ReloadDeviceConfigResult::ConfigError => println!("config reload: Error happened while reloading config, check daemon logs"),
                ReloadDeviceConfigResult::DeviceNotFound => println!("config reload: Device not found"),
                ReloadDeviceConfigResult::Reloaded => println!("config reload: Reloaded"),
            }
        } else {
            println!("config reload: No device is selected");
        }
    }
}

pub fn save_config(client: ClientRef, mut args: Split<&str>, current_sn: &String) {
    if let Some(arg) = args.next() {
        if arg == "all" {
            match client.save_device_configs().expect("Failed to save configs") {
                SaveDeviceConfigsResult::ConfigError => println!("config save: Error happened while saving configs, check daemon logs"),
                SaveDeviceConfigsResult::Saved => println!("config save: Saved"),
            }
        } else {
            println!("config save: Unknown operation");
        }
    } else {
        if !current_sn.is_empty() {
            match client.save_device_config(current_sn).expect("Failed to save config") {
                SaveDeviceConfigResult::ConfigError => println!("config save: Error happened while saving config, check daemon logs"),
                SaveDeviceConfigResult::DeviceNotFound => println!("config save: Device not found"),
                SaveDeviceConfigResult::Saved => println!("config save: Saved"),
            }
        } else {
            println!("config save: No device is selected");
        }
    }
}

pub fn import_config(client: ClientRef, mut args: Split<&str>, current_sn: &String) {
    if !current_sn.is_empty() {
        if let Some(path) = args.next() {
            if let Ok(path) = PathBuf::from_str(path) {
                if let Ok(file) = fs::read_to_string(path) {
                    let result = client.import_device_config(current_sn, file).expect("Failed to import device config");

                    match result {
                        ImportDeviceConfigResult::DeviceNotFound => println!("config import: Device not found"),
                        ImportDeviceConfigResult::InvalidConfig => println!("config import: Invalid device config"),
                        ImportDeviceConfigResult::FailedToSave => println!("config import: Failed to save config"),
                        ImportDeviceConfigResult::Imported => println!("config import: Imported"),
                    }
                } else {
                    println!("config import: Failed to read file");
                }
            } else {
                println!("config import: Invalid path");
            }
        } else {
            println!("config import: Specify path");
        }
    } else {
        println!("config import: No device is selected");
    }
}

pub fn export_config(client: ClientRef, mut args: Split<&str>, current_sn: &String) {
    if !current_sn.is_empty() {
        if let Some(path) = args.next() {
            if let Ok(path) = PathBuf::from_str(path) {
                let data = client.export_device_config(current_sn).expect("Failed to export device config");

                match data {
                    ExportDeviceConfigResult::DeviceNotFound => println!("config export: Device not found"),
                    ExportDeviceConfigResult::Exported(config) => {
                        if let Ok(_) = fs::write(path, config) {
                            println!("config export: Exported");
                        } else {
                            println!("config export: Failed to write file");
                        }
                    }
                }
            } else {
                println!("config export: Invalid path");
            }
        } else {
            println!("config export: Specify path");
        }
    } else {
        println!("config export: No device is selected");
    }
}