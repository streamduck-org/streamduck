mod device;
mod config;
mod info;
mod utils;
mod buttons;
mod module;
mod images;
mod helper;

use std::sync::Arc;
use rustyline::{Editor};
use rustyline::error::ReadlineError;
use streamduck_client::daemon::daemon_data::buttons::{CopyButtonResult, PasteButtonResult};
use streamduck_client::daemon::daemon_data::devices::{GetDeviceResult, SetBrightnessResult};
use streamduck_client::daemon::daemon_data::ops::DoButtonActionResult;
use streamduck_client::daemon::daemon_data::panels::{DropStackToRootResult, PopScreenResult};
use streamduck_client::SDSyncRequestClient;
use crate::prompt::buttons::{button_component, button_from, button_new, button_remove};
use crate::prompt::config::{export_config, import_config, reload_config, save_config};
use crate::prompt::device::{add_device, device_list, remove_device};
use crate::prompt::helper::StreamduckHelper;
use crate::prompt::images::{add_image, list_images, remove_image};
use crate::prompt::info::{button_info, component_info, list_buttons, list_components, list_fonts, prompt_help, show_stack};
use crate::prompt::module::{list_modules, module_info, module_list_params, module_params_add, module_params_remove, module_params_set, module_params_upload};

type ClientRef<'a> = &'a Arc<Box<dyn SDSyncRequestClient>>;

pub fn prompt(client: Arc<Box<dyn SDSyncRequestClient>>) {
    println!("Streamduck CLI Prompt\n\nTo view commands, enter 'help' command.\nTo exit, enter 'exit'.\n");
    let mut current_sn = String::new();

    let mut rl = Editor::<StreamduckHelper>::new();
    rl.set_helper(Some(StreamduckHelper));

    loop {
        let prefix = format!("{}{}> ", current_sn, if current_sn.is_empty() { "" } else { " " });

        match rl.readline(&prefix) {
            Ok(line) => {
                rl.add_history_entry(&line);

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
                                    "import" | "i" => import_config(&client, args, &current_sn),
                                    "export" | "e" => export_config(&client, args, &current_sn),
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
                                if let Some(drop) = args.next() {
                                    if drop == "drop" {
                                        match client.drop_stack_to_root(&current_sn).expect("Failed to drop stack") {
                                            DropStackToRootResult::DeviceNotFound => println!("back drop: Device not found"),
                                            DropStackToRootResult::Dropped => println!("back drop: Dropped to root screen")
                                        }
                                    } else {
                                        println!("back: Unknown command")
                                    }
                                } else {
                                    match client.pop_screen(&current_sn).expect("Failed to pop screen") {
                                        PopScreenResult::DeviceNotFound => println!("back: Device not found"),
                                        PopScreenResult::OnlyOneRemaining => println!("back: Only one remaining"),
                                        PopScreenResult::Popped => println!("back: Popped screen"),
                                    }
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
                                                "upload" | "u" => module_params_upload(&client, args),
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

                        "image" | "i" => {
                            if let Some(command) = args.next() {
                                match command {
                                    "list" | "l" => list_images(&client, &current_sn, args),
                                    "add" | "a" => add_image(&client, &current_sn, args),
                                    "remove" | "r" => remove_image(&client, &current_sn, args),
                                    _ => println!("image: Unknown command"),
                                }
                            } else {
                                println!("image: Unknown command");
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

                        "stack" => {
                            show_stack(&client, &current_sn);
                        }

                        "font" | "f" => {
                            if let Some(command) = args.next() {
                                match command {
                                    "list" | "l" => list_fonts(&client),

                                    _ => println!("font: Unknown command"),
                                }
                            } else {
                                println!("font: Unknown command");
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

                                    "copy" | "cp" => {
                                        if !current_sn.is_empty() {
                                            if let Some(key) = args.next() {
                                                if let Ok(key) = key.parse::<u8>() {
                                                    let result = client.copy_button(&current_sn, key).expect("Failed to copy button");

                                                    match result {
                                                        CopyButtonResult::DeviceNotFound => println!("button copy: Device not found"),
                                                        CopyButtonResult::NoButton => println!("button copy: No button to copy"),
                                                        CopyButtonResult::Copied => println!("button copy: Copied"),
                                                    }
                                                } else {
                                                    println!("button copy: Input valid key index (0-255)")
                                                }
                                            } else {
                                                println!("button copy: Input valid key index (0-255)")
                                            }
                                        } else {
                                            println!("button copy: No device is selected")
                                        }
                                    }

                                    "paste" | "p" => {
                                        if !current_sn.is_empty() {
                                            if let Some(key) = args.next() {
                                                if let Ok(key) = key.parse::<u8>() {
                                                    let result = client.paste_button(&current_sn, key).expect("Failed to paste button");

                                                    match result {
                                                        PasteButtonResult::DeviceNotFound => println!("button paste: Device not found"),
                                                        PasteButtonResult::FailedToPaste => println!("button paste: Failed to paste"),
                                                        PasteButtonResult::Pasted => println!("button paste: Pasted"),
                                                    }
                                                } else {
                                                    println!("button paste: Input valid key index (0-255)")
                                                }
                                            } else {
                                                println!("button paste: Input valid key index (0-255)")
                                            }
                                        } else {
                                            println!("button paste: No device is selected")
                                        }
                                    }

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

            Err(ReadlineError::Interrupted) => {
                break;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                eprintln!("{}", err);
                break;
            }
        }
    }
}