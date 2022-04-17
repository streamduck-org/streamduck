#[cfg(target_family = "unix")]
mod unix;

use std::sync::{Arc, Mutex};
use std::thread::spawn;
use flexi_logger::{DeferredNow, Logger, LogSpecification, style, TS_DASHES_BLANK_COLONS_DOT_BLANK};
use log::{LevelFilter, Record};
use streamduck_core::font::{load_default_font, load_fonts_from_resources};
use streamduck_core::modules::{load_base_modules, ModuleManager};
use streamduck_core::config::Config;
use streamduck_core::core::manager::CoreManager;
use streamduck_core::socket::SocketManager;
use streamduck_core::modules::plugins::load_plugins_from_folder;
use streamduck_core::thread::rendering::custom::RenderingManager;
use streamduck_daemon::daemon_data::DaemonListener;

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

    log::info!("Streamduck Daemon");

    // Initializing module manager
    let module_manager = ModuleManager::new();

    // Initializing rendering manager
    let render_manager = RenderingManager::new();

    // Reading config
    let config = Arc::new(Config::get());

    // Initializing socket manager
    let socket_manager = SocketManager::new();

    // Initializing core stuff
    load_base_modules(module_manager.clone(), socket_manager.clone());
    load_default_font();
    load_fonts_from_resources();

    // Initializing built-in modules
    streamduck_actions::init_module(&module_manager);

    // Initializing core manager
    let core_manager = CoreManager::new(module_manager.clone(), render_manager.clone(), socket_manager.clone(), config.clone());

    // Adding daemon listener
    socket_manager.add_listener(Box::new(DaemonListener {
        core_manager: core_manager.clone(),
        module_manager: module_manager.clone(),
        config: config.clone(),
        clipboard: Mutex::new(None)
    }));

    // Loading plugins
    load_plugins_from_folder(module_manager.clone(), socket_manager.clone(), render_manager.clone(), config.plugin_path());

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