mod device;
mod config;
mod info;
mod utils;
mod buttons;
mod module;

use std::io::Write;
use std::sync::Arc;
use streamduck_client::daemon::socket::daemon_data::{DoButtonActionResult, GetDeviceResult, PopScreenResult, SetBrightnessResult};
use streamduck_client::SDClient;
use crate::prompt::buttons::{button_component, button_from, button_new, button_remove};
use crate::prompt::config::{reload_config, save_config};
use crate::prompt::device::{add_device, device_list, remove_device};
use crate::prompt::info::{button_info, component_info, list_buttons, list_components, prompt_help};
use crate::prompt::module::{list_modules, module_info, module_list_params, module_params_add, module_params_remove, module_params_set};

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

                "device" | "d" => {
                    if let Some(command) = args.next() {
                        match command {
                            "list" | "l" => device_list(&client),
                            "add" | "a" => add_device(&client, args),
                            "remove" | "r" => remove_device(&client, args),
                            _ => println!("device: Unknown command"),
                        }
                    } else {
                        println!("device: Unknown command");
                    }
                }

                "select" | "s" => {
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

                "config" | "con" => {
                    if let Some(command) = args.next() {
                        match command {
                            "reload" | "r" => reload_config(&client, args, &current_sn),
                            "save" | "s" => save_config(&client, args, &current_sn),
                            _ => println!("config: Unknown command"),
                        }
                    } else {
                        println!("config: Unknown command");
                    }
                }

                "brightness" | "br" => {
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

                "back" | "ba" => {
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

                "press" | "p" => {
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

                "module" | "m" => {
                    if let Some(command) = args.next() {
                        match command {
                            "list" | "l" => list_modules(&client),
                            "info" | "i" => module_info(&client, args),
                            "params" | "p" => {
                                if let Some(command) = args.next() {
                                    match command {
                                        "add" | "a" => module_params_add(&client, args),
                                        "remove" | "r" => module_params_remove(&client, args),
                                        "set" | "s" => module_params_set(&client, args),
                                        "list" | "l" => module_list_params(&client, args),

                                        _ => println!("module params: Unknown command"),
                                    }
                                } else {
                                    println!("module params: Unknown command");
                                }
                            }

                            _ => println!("module: Unknown command"),
                        }
                    } else {
                        println!("module: Unknown command");
                    }
                }

                "component" | "com" => {
                    if let Some(command) = args.next() {
                        match command {
                            "list" | "l" => list_components(&client),
                            "info" | "i" => component_info(&client, args),

                            _ => println!("component: Unknown command"),
                        }
                    } else {
                        println!("component: Unknown command");
                    }
                }

                "button" | "b" => {
                    if let Some(command) = args.next() {
                        match command {
                            "list" | "l" => list_buttons(&client, &current_sn),
                            "info" | "i" => button_info(&client, &current_sn, args),
                            "new" | "n" => button_new(&client, &current_sn, args),
                            "from" | "f" => button_from(&client, &current_sn, args),
                            "remove" | "r" => button_remove(&client, &current_sn, args),
                            "component" | "c" => button_component(&client, &current_sn, args),

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