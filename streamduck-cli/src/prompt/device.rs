use std::str::Split;
use streamduck_client::daemon::daemon_data::devices::{AddDeviceResult, RemoveDeviceResult};
use crate::prompt::ClientRef;
use crate::prompt::utils::print_table;

pub fn device_list(client: ClientRef) {
    let list: Vec<(bool, bool, String, String)> = client.device_list().expect("Failed to get device list")
        .into_iter()
        .map(|d| (d.online, d.managed, d.device_type.to_string(), d.serial_number))
        .collect();

    let mut table: Vec<Vec<&str>> = vec![
        vec!["Online"],
        vec!["Managed"],
        vec!["Type"],
        vec!["Serial"]
    ];

    for (online, managed, ty, serial) in &list {
        table[0].push(if *online { "Yes" } else { "No" });
        table[1].push(if *managed { "Yes" } else { "No" });
        table[2].push(ty.as_str());
        table[3].push(serial.as_str());
    }

    print_table(table, "-", "|");

    println!("\nTo select device for device related operations, enter 'select <serial>'");
}

pub fn add_device(client: ClientRef, mut args: Split<&str>) {
    if let Some(serial) = args.next() {
        match client.add_device(serial).expect("Failed to add device") {
            AddDeviceResult::AlreadyRegistered => println!("device add: Device is already managed"),
            AddDeviceResult::NotFound => println!("device add: Device not found"),
            AddDeviceResult::Added => println!("device add: Added to managed list"),
        }
    } else {
        println!("device add: Specify serial number")
    }
}

pub fn remove_device(client: ClientRef, mut args: Split<&str>) {
    if let Some(serial) = args.next() {
        match client.remove_device(serial).expect("Failed to add device") {
            RemoveDeviceResult::NotRegistered => println!("device remove: Device already wasn't managed"),
            RemoveDeviceResult::Removed => println!("device remove: Removed from managed list"),
        }
    } else {
        println!("device remove: Specify serial number")
    }
}