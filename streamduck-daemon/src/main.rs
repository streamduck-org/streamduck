use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use clap::{Arg, ArgAction, command, value_parser};
use clap::parser::ArgMatches;
use flexi_logger::{DeferredNow, FileSpec, Logger, LogSpecification, style, TS_DASHES_BLANK_COLONS_DOT_BLANK};
use log::{LevelFilter, log_enabled, Record};
use rayon::ThreadPoolBuilder;
use tokio::runtime::Builder;
use tokio::signal;
use tokio::sync::Mutex;

use streamduck_core::config::Config;
use streamduck_core::core::manager::CoreManager;
use streamduck_core::font::{load_default_font, load_fonts_from_resources};
use streamduck_core::modules::{load_base_modules, ModuleManager};
use streamduck_core::modules::plugins::load_plugins_from_folder;
use streamduck_core::socket::SocketManager;
use streamduck_core::thread::rendering::custom::RenderingManager;
use streamduck_daemon::daemon_data::DaemonListener;

#[cfg(target_family = "unix")]
mod unix;
#[cfg(target_family = "windows")]
mod windows;

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
    // Init parser
    let matches = command!()
        .arg(
            Arg::new("debug")
                .short('d')
                .long("debug")
                .action(ArgAction::SetTrue)
                .value_parser(value_parser!(bool))
                .help("Turn on debug mode")
            )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config-path")
                .value_parser(value_parser!(String))
                .help("Specify from where the config should be loaded")
            )
        .get_matches();
    
    // Setting up Tokio runtime
    let runtime = Builder::new_multi_thread()
        .enable_all()
        .thread_keep_alive(Duration::from_secs(3600))
        .build().unwrap();

    // Setting up Rayon limitations
    let cpu_count = num_cpus::get() / 3 * 2;

    ThreadPoolBuilder::new()
        .num_threads(cpu_count)
        .build_global().unwrap();

    // Spawning root task
    runtime.block_on(async { root(matches).await });
}

async fn root(matches: ArgMatches) {
    // Initializing logger
    let mut builder = LogSpecification::builder();

    let level = || -> LevelFilter {
        match matches
            .get_one::<bool>("debug")
            .unwrap_or(&false) {
                true => LevelFilter::Debug,
                false => LevelFilter::Info
            }
    };

    let custom_path = || -> Option<PathBuf> {
        match matches
            .get_one::<String>("config") {
                Some(v) => Some(PathBuf::from(v)),
                None => None
            }
    };

    builder.default(level())
        .module("streamdeck", LevelFilter::Off);

    Logger::with(builder.build())
        .log_to_file(FileSpec::default().suppress_timestamp().basename("streamduck-daemon"))
        .log_to_stdout()
        .format(logging_format)
        .start().unwrap();

    log::info!("Streamduck Daemon");

    if log_enabled!(log::Level::Debug) {
        log::warn!("Debugging output enabled");
    }

    // Initializing module manager
    let module_manager = ModuleManager::new();

    // Initializing rendering manager
    let render_manager = RenderingManager::new();

    // Reading config
    let config = Arc::new(Config::get(custom_path()).await);

    // Initializing socket manager
    let socket_manager = SocketManager::new();

    // Initializing core stuff
    load_base_modules(module_manager.clone(), socket_manager.clone()).await;
    load_default_font();
    load_fonts_from_resources();

    // Initializing built-in modules
    streamduck_actions::init_module(&module_manager).await;

    // Initializing core manager
    let core_manager = CoreManager::new(module_manager.clone(), render_manager.clone(), socket_manager.clone(), config.clone());

    // Adding daemon listener
    socket_manager.add_listener(Arc::new(DaemonListener {
        core_manager: core_manager.clone(),
        module_manager: module_manager.clone(),
        config: config.clone(),
        clipboard: Mutex::new(None)
    })).await;

    // Loading plugins
    load_plugins_from_folder(module_manager.clone(), socket_manager.clone(), render_manager.clone(), config.plugin_path()).await;

    // Announcing loaded modules
    for (module_name, _) in module_manager.get_modules().await {
        log::info!("Loaded module: {}", module_name)
    }

    // Loading device configs
    config.reload_device_configs().await.ok();

    // Adding devices from config
    core_manager.add_devices_from_config().await;

    // Spawning reconnect routine
    {
        let manager = core_manager.clone();
        tokio::spawn(async move { manager.reconnect_routine().await });
    }

    // Registering interrupt handle
    tokio::spawn(async {
        signal::ctrl_c().await.ok();
        clean_socket();
        std::process::exit(0);
    });

    hide_console();

    run_socket(socket_manager.clone()).await;
}

#[cfg(target_family = "windows")]
fn hide_console() {
    use itertools::Itertools;

    // Don't hide the console window only if requested
    if std::env::args().contains(&"c".to_string()) {
        return;
    }

    use std::ptr;
    use winapi::um::wincon::GetConsoleWindow;
    use winapi::um::winuser::{ShowWindow, SW_HIDE};

    let window = unsafe {GetConsoleWindow()};
    // https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindow
    if window != ptr::null_mut() {
        unsafe {
            ShowWindow(window, SW_HIDE);
        }
    }
}

#[cfg(target_family = "windows")]
async fn run_socket(socket_manager: Arc<SocketManager>) {
    windows::open_socket(socket_manager).await
}

#[cfg(target_family = "windows")]
fn clean_socket() {
    // cleanup not needed
}

#[cfg(target_family = "unix")]
fn hide_console() {
    // this is really only needed for windows
}

#[cfg(target_family = "unix")]
async fn run_socket(socket_manager: Arc<SocketManager>) {
    unix::open_socket(socket_manager).await
}

#[cfg(target_family = "unix")]
fn clean_socket() {
    unix::remove_socket()
}

