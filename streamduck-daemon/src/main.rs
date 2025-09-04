#![allow(dead_code)]

use std::thread;
use std::time::{Duration, Instant};
use ::log::{error, info};
use flexi_logger::Logger;
use streamduck_core::config;
use streamduck_core::core::{Core, CoreStatus};
use crate::log::log_format;

pub mod log;

fn main() -> anyhow::Result<()> {
    Logger::try_with_env_or_str("debug")?
        .format(log_format)
        .set_palette("1;3;-;7;8".to_string())
        .start()?;

    if let Err(e) = config::load_config() {
        error!("Failed to load config, continuing with default settings.\nReason: {}", e);
    }

    loop {
        let start_time = Instant::now();

        let status = {
            Core::new().run()
        };

        match status {
            Ok(status) => {
                match status {
                    CoreStatus::StillAlive => panic!("uhhh.. what? died while being alive???"),
                    CoreStatus::RestartNeeded => {
                        info!("Core exited with intent to restart!");
                    }
                    CoreStatus::ClientThreadDied => {
                        if start_time.elapsed() < Duration::from_secs(5) {
                            error!("Client thread died within 5 seconds since launch, is this a configuration issue? Terminating...");
                            break;
                        }
                    }
                    CoreStatus::NormalExit => {
                        info!("Core intended to exit! Terminating...");
                        break;
                    }
                }
            }
            Err(e) => {
                error!("Core exited with error: {}", e);
            }
        }

        info!("Restarting Core in 5 seconds...");
        thread::sleep(Duration::from_secs(5));
    }


    Ok(())
}
