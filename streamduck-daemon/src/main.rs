#[cfg(target_family = "unix")]
mod unix;

mod plugins;

use std::sync::Arc;
use std::thread::spawn;
use flexi_logger::{DeferredNow, Logger, LogSpecification, style, TS_DASHES_BLANK_COLONS_DOT_BLANK};
use log::{LevelFilter, Record};
use streamduck_core::font::load_fonts_from_resources;
use streamduck_core::modules::{load_base_modules, ModuleManager};
use streamduck_daemon::config::Config;
use streamduck_daemon::core_manager::CoreManager;
use streamduck_daemon::socket::daemon_data::DaemonListener;
use streamduck_daemon::socket::SocketManager;
use crate::plugins::load_plugins_from_folder;

fn logging_format(
    w: &mut dyn std::io::Write,
    now: &mut DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    let level = record.level();
    write!(
        w,
        "{} [{}] {}",
        style(level).paint(now.format(TS_DASHES_BLANK_COLONS_DOT_BLANK)),
        style(level).paint(level.to_string()),
        style(level).paint(&record.args().to_string())
    )
}

fn main() {
    // Initializing logger
    let mut builder = LogSpecification::builder();
    builder.default(LevelFilter::Debug)
        .module("streamdeck", LevelFilter::Off);

    Logger::with(builder.build())
        .format(logging_format)
        .start().unwrap();

    log::info!("Streamduck v{}", get_version());

    // Initializing module manager
    let module_manager = ModuleManager::new();

    // Reading config
    let config = Arc::new(Config::get());

    // Initializing core stuff
    load_base_modules(module_manager.clone());
    load_fonts_from_resources();

    // Initializing built-in modules
    streamduck_actions::init_module(&module_manager);

    // Socket listener manager
    let socket_manager = SocketManager::new();

    // Initializing core manager
    let core_manager = CoreManager::new(module_manager.clone(), config.clone());

    // Adding daemon listener
    socket_manager.add_listener(Box::new(DaemonListener {
        core_manager: core_manager.clone(),
        module_manager: module_manager.clone(),
        config: config.clone()
    }));

    // Loading plugins
    load_plugins_from_folder(module_manager.clone(), socket_manager.clone(), config.plugin_path());

    // Announcing loaded modules
    for (module_name, _) in module_manager.get_modules() {
        log::info!("Loaded module: {}", module_name)
    }

    // Loading device configs
    config.reload_device_configs().ok();



    // Adding devices from config
    core_manager.add_devices_from_config();

    // Spawning reconnect routine
    {
        let manager = core_manager.clone();
        spawn(move || manager.reconnect_routine());
    }

    // Registering interrupt handle
    ctrlc::set_handler(move || {
        clean_socket();
        std::process::exit(0);
    }).expect("Error setting Ctrl-C handler");

    run_socket(socket_manager.clone());
}

#[cfg(target_family = "unix")]
fn run_socket(socket_manager: Arc<SocketManager>) {
    unix::open_socket(socket_manager)
}

#[cfg(target_family = "unix")]
fn clean_socket() {
    unix::remove_socket()
}

fn get_version() -> String {
    "0.0.6".to_string()
}