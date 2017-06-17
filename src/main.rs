extern crate toml;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[derive(Deserialize)]
struct Config {
    serial_device: Option<String>,
    log_path: Option<String>,
    data_rate: Option<f32>,
    split: Option<u16>,
    prefix: Option<String>
}

#[cfg(unix)]
fn main() {
    let mut config_file: File = File::open(Path::new("meganlogs.toml")).unwrap();
    let mut config_data: String = String::new();
    config_file.read_to_string(&mut config_data).unwrap();
    let config: Config = toml::from_str(&config_data).unwrap();

    println!("{:?} {:?} {:?} {:?} {:?}", config.serial_device, config.log_path, config.data_rate, config.split, config.prefix);
}

#[cfg(windows)]
fn main() {
    println!("No windows for you good sir.");
}