use std::{env, io};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use toml;

#[derive(PartialEq)]
pub enum DataPoint {
    DriverDeviceList,
}

#[derive(Debug, Deserialize, Serialize, Default, Copy, Clone)]
pub struct Config {
    pub test_driver_device_list: Duration
}

pub struct Benchmark {
    start: Instant,
    data_point: Option<DataPoint>
}

/// benchmark output
impl Benchmark {
    pub fn stop(&self) {
        let duration = self.start.elapsed();
        println!("time: {:?}", duration);

        if let Ok(conf) = self.read_toml()  {
            if let Some(point) = &self.data_point {
                let values = match point {
                    DataPoint::DriverDeviceList => { (Config { test_driver_device_list: duration, ..conf}, conf.test_driver_device_list) }
                };
                let duration_before = values.1;
                let new_config = values.0;
                self.create_output(&duration_before, &duration);
                // writing it to the toml
                self.write_toml(new_config).unwrap();
            } else {
                println!("No data point selected")
            }
        } else {
            println!("Data file not available: {:?}", self.read_toml().err().unwrap().to_string())
        }
    }

    fn read_toml(&self) -> io::Result<Config> {
        let path = "./performance.toml";
        if !std::path::Path::new(path).exists() {
            std::fs::File::create(path)?;
            self.write_toml(Config::default())?;
        }
        // read content
        let content = std::fs::read_to_string("./performance.toml")?;
        Ok(toml::from_str(&content)?)
    }

    fn write_toml(&self, config: Config) -> io::Result<()> {
        if env::var("WRITE_TEST_RESULT").is_ok() {
            println!("writing result");
            let path = "./performance.toml";
            let tml = toml::to_string(&config).expect("Could not encode toml.");
            std::fs::write(path, tml)?;
        }
        Ok(())
    }

    fn create_output(&self, elapsed_before: &Duration, elapsed_after: &Duration) {
        let before = elapsed_before.as_secs_f32();
        let after = elapsed_after.as_secs_f32();
        if elapsed_before < elapsed_after {
            println!("performance: \u{1b}[41mWORSE\u{1b}[0m");
        } else {
            println!("performance: \u{1b}[42mBETTER\u{1b}[0m");
        }
        // can't divide by zero
        if before > 0.0 && after > 0.0 {
            println!("loss/gain: \u{1b}[33m{:?}%\u{1b}[0m ({}s)", after/before*100.0, before-after);
        }
    }
}

/// start a benchmark
pub fn start_benchmark(data_point: Option<DataPoint>) -> Benchmark {
    Benchmark {
        start: Instant::now(),
        data_point
    }
}