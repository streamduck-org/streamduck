use std::io::Write;
use std::str::Split;
use std::sync::Arc;
use streamduck_client::daemon::socket::daemon_data::{AddDeviceResult, DoButtonActionResult, GetButtonResult, GetCurrentScreenResult, GetDeviceResult, PopScreenResult, ReloadDeviceConfigResult, ReloadDeviceConfigsResult, RemoveDeviceResult, SaveDeviceConfigResult, SaveDeviceConfigsResult, SetBrightnessResult};
use streamduck_client::SDClient;
use crate::helps::COMMANDS;

type ClientRef<'a> = &'a Arc<Box<dyn SDClient>>;

pub fn prompt(client: Arc<Box<dyn SDClient>>) {
    println!("Streamduck CLI Prompt\n\nTo enter interactive UI mode, enter 'ui' command.\nTo view commands, enter 'help' command.\nTo exit, enter 'exit'.\n");


    let mut current_sn = String::new();

    loop {
        let mut line = String::new();
        print!("{}{}> ", current_sn, if current_sn.is_empty() { "" } else { " " });
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut line).unwrap();

        let mut args = line.trim().split(" ");

        if let Some(command) = args.next() {
            match command {
                "help" => println!("{}", prompt_help()),
                "exit" => break,

                "device" => {
                    if let Some(command) = args.next() {
                        match command {
                            "list" => device_list(&client),
                            "add" => add_device(&client, args),
                            "remove" => remove_device(&client, args),
                            _ => println!("device: Unknown command"),
                        }
                    } else {
                        println!("device: Unknown command");
                    }
                }

                "select" => {
                    if let Some(serial) = args.next() {
                        if let GetDeviceResult::Found(_) = client.get_device(serial).expect("Failed to check for device") {
                            current_sn = serial.to_string();
                            println!("select: Selected device '{}'", serial);
                        } else {
                            println!("select: Device not found");
                        }
                    } else {
                        println!("select: Unselected device");
                        current_sn = "".to_string();
                    }
                }

                "config" => {
                    if let Some(command) = args.next() {
                        match command {
                            "reload" => reload_config(&client, args, &current_sn),
                            "save" => save_config(&client, args, &current_sn),
                            _ => println!("config: Unknown command"),
                        }
                    } else {
                        println!("config: Unknown command");
                    }
                }

                "brightness" => {
                    if let Some(brightness) = args.next() {
                        if let Ok(brightness) = brightness.parse::<u8>() {
                            if !current_sn.is_empty() {
                                match client.set_brightness(&current_sn, brightness).expect("Failed to set brightness") {
                                    SetBrightnessResult::DeviceNotFound => println!("brightness: Device not found"),
                                    SetBrightnessResult::Set => println!("brightness: Set"),
                                }
                            } else {
                                println!("brightness: No device is selected");
                            }
                        } else {
                            println!("brightness: Input valid brightness value (0-255)");
                        }
                    } else {
                        println!("brightness: Input valid brightness value (0-255)");
                    }
                }

                "back" => {
                    if !current_sn.is_empty() {
                        match client.pop_screen(&current_sn).expect("Failed to pop screen") {
                            PopScreenResult::DeviceNotFound => println!("back: Device not found"),
                            PopScreenResult::OnlyOneRemaining => println!("back: Only one remaining"),
                            PopScreenResult::Popped => println!("back: Popped screen"),
                        }
                    } else {
                        println!("back: No device is selected");
                    }
                }

                "press" => {
                    if let Some(key_index) = args.next() {
                        if let Ok(key_index) = key_index.parse::<u8>() {
                            if !current_sn.is_empty() {
                                match client.do_button_action(&current_sn, key_index).expect("Failed to do button action") {
                                    DoButtonActionResult::DeviceNotFound => println!("press: Device not found"),
                                    DoButtonActionResult::Activated => println!("press: Pressed"),
                                }
                            } else {
                                println!("press: No device is selected");
                            }
                        } else {
                            println!("press: Input valid key index (0-255)");
                        }
                    } else {
                        println!("press: Input valid key index (0-255)");
                    }
                }

                "module" => {
                    if let Some(command) = args.next() {
                        match command {
                            "list" => list_modules(&client),
                            "info" => module_info(&client, args),

                            _ => println!("module: Unknown command"),
                        }
                    } else {
                        println!("module: Unknown command");
                    }
                }

                "component" => {
                    if let Some(command) = args.next() {
                        match command {
                            "list" => list_components(&client),
                            "info" => component_info(&client, args),

                            _ => println!("component: Unknown command"),
                        }
                    } else {
                        println!("component: Unknown command");
                    }
                }

                "button" => {
                    if let Some(command) = args.next() {
                        match command {
                            "list" => list_buttons(&client, &current_sn),
                            "info" => button_info(&client, &current_sn, args),

                            _ => println!("button: Unknown command"),
                        }
                    } else {
                        println!("button: Unknown command");
                    }
                }

                _ => println!("Unknown command"),
            }
        } else {
            println!("Unknown command");
        }
    }
}

pub fn prompt_help() -> String {
    let mut help = String::new();

    for (name, help_line) in COMMANDS {
        help += &format!("{} {}\n", name.replace("-", ""), help_line.replace("[<serial>] ", "").trim());
    }

    help
}

pub fn print_table(table: Vec<Vec<&str>>, first_separator: &str, separator: &str) {
    let mut max_len = vec![];

    // Calculating max size for each column
    for column in &table {
        let mut len = 0;

        for item in column {
            len = len.max(item.len());
        }

        max_len.push(len);
    }

    // Printing table
    if table.len() > 0 {
        for y in 0..table[0].len() {
            let separator = if y == 0 {
                first_separator
            } else {
                separator
            };

            for x in 0..table.len() {
                if y == 0 {
                    print!("{} {: <w$} ", separator, table[x][y], w = max_len[x])
                } else {
                    print!("{} {: >w$} ", separator, table[x][y], w = max_len[x])
                }
            }

            println!("{}", separator);
        }
    }
}

pub fn print_table_with_strings(table: Vec<Vec<String>>, first_separator: &str, separator: &str) {
    print_table(
        table.iter()
            .map(|v| {
                v.iter().map(|s| s.as_str()).collect()
            })
            .collect(),
        first_separator,
        separator
    );
}

// Devices
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

pub fn list_modules(client: ClientRef) {
    let mut table = vec![
        vec!["Name"],
        vec!["Version"],
        vec!["Description"]
    ];

    let module_list = client.list_modules().expect("Failed to list modules");

    for module in &module_list {
        table[0].push(module.name.as_str());
        table[1].push(module.version.as_str());
        table[2].push(module.description.as_str());
    }

    print_table(table, "-", "|");

    println!("\nFor more information on module, enter 'module info <name>'")
}

pub fn module_info(client: ClientRef, mut args: Split<&str>) {
    if let Some(name) = args.next() {
        let module_list = client.list_modules().expect("Failed to list modules");

        for module in module_list {
            if name == module.name {
                println!(
                    "- {} v{} by {}\n{}\n- Using features: {}",
                    module.name,
                    module.version,
                    module.author,
                    module.description,
                    {
                        let mut names = vec![];

                        for (name, _) in &module.used_features {
                            names.push(name.as_str())
                        }

                        names.join(", ")
                    }
                );
                return;
            }
        }

        println!("module info: Module not found");
    } else {
        println!("module info: Specify name");
    }
}

pub fn list_components(client: ClientRef) {
    let mut table = vec![
        vec!["Display Name"],
        vec!["Name"],
        vec!["Module"]
    ];

    let module_list = client.list_components().expect("Failed to list components");

    for (module, component_list) in &module_list {
        for (component, definition) in component_list {
            table[0].push(definition.display_name.as_str());
            table[1].push(component.as_str());
            table[2].push(module.as_str());
        }
    }

    print_table(table, "-", "|");

    println!("\nFor more information on component, enter 'component info <name>'")
}

pub fn component_info(client: ClientRef, mut args: Split<&str>) {
    if let Some(name) = args.next() {
        let component_list = client.list_components().expect("Failed to list components");

        for (module_name, component_list) in component_list {
            for (component_name, definition) in component_list {
                if name == component_name {
                    println!(
                        "- {} ({})\n{}\n- Provided by '{}' module",
                        definition.display_name,
                        component_name,
                        definition.description,
                        module_name
                    );
                    return;
                }
            }
        }

        println!("component info: Component not found");
    } else {
        println!("component info: Specify name");
    }
}

pub fn list_buttons(client: ClientRef, current_sn: &str) {
    if !current_sn.is_empty() {
        let screen = client.get_current_screen(current_sn).expect("Failed to get current screen");

        match screen {
            GetCurrentScreenResult::NoScreen => println!("button list: No screen"),
            GetCurrentScreenResult::DeviceNotFound => println!("button list: Device not found"),
            GetCurrentScreenResult::Screen(screen) => {
                let mut table = vec![
                    vec!["Index".to_string()],
                    vec!["Components".to_string()],
                ];

                for (key, button) in &screen {
                    table[0].push(key.to_string());

                    let mut names = button.component_names();
                    names.sort();

                    table[1].push(names.join(", ").to_string());
                }

                print_table_with_strings(table, "-", "|");

                println!("\nFor more information on button, enter 'button info <index>'")
            }
        }
    } else {
        println!("button list: No device is selected")
    }
}

pub fn button_info(client: ClientRef, current_sn: &str, mut args: Split<&str>) {
    if !current_sn.is_empty() {
        if let Some(key) = args.next() {
            if let Ok(key) = key.parse::<u8>() {
                let button = client.get_button(current_sn, key).expect("Failed to get current screen");

                match button {
                    GetButtonResult::DeviceNotFound => println!("button info: Device not found"),
                    GetButtonResult::NoButton => println!("button info: Button not found"),
                    GetButtonResult::Button(_) => {
                        println!("To Do, currently no dynamic component management")
                    }
                }
            } else {
                println!("button info: Input valid key index (0-255)");
            }
        } else {
            println!("button info: Input valid key index (0-255)");
        }
    } else {
        println!("button info: No device is selected")
    }
}

pub fn add_device(client: ClientRef, mut args: Split<&str>) {
    if let Some(serial) = args.next() {
        match client.add_device(serial).expect("Failed to add device") {
            AddDeviceResult::AlreadyRegistered => println!("device add: Device is already managed"),
            AddDeviceResult::NotFound => println!("device add: Device not found"),
            AddDeviceResult::Added => println!("device add: Added to managed list"),
        }
    }
}

pub fn remove_device(client: ClientRef, mut args: Split<&str>) {
    if let Some(serial) = args.next() {
        match client.remove_device(serial).expect("Failed to add device") {
            RemoveDeviceResult::NotRegistered => println!("device remove: Device already wasn't managed"),
            RemoveDeviceResult::Removed => println!("device remove: Removed from managed list"),
        }
    }
}

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