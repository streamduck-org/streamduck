use std::str::Split;
use streamduck_client::daemon::socket::daemon_data::{ReloadDeviceConfigResult, ReloadDeviceConfigsResult, SaveDeviceConfigResult, SaveDeviceConfigsResult};
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