extern crate toml;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serial;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use serial::prelude::*;

const SETTINGS: serial::PortSettings = serial::PortSettings {
    baud_rate:    serial::Baud115200,
    char_size:    serial::Bits8,
    parity:       serial::ParityNone,
    stop_bits:    serial::Stop1,
    flow_control: serial::FlowNone
};

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

    let a = "/dev/ttyUSB0".to_string();
    let device: &String = config.serial_device.as_ref().unwrap_or(&a);
    let mut port: serial::SystemPort = serial::open(device).unwrap();
    port.configure(&SETTINGS).unwrap();
    port.set_timeout(std::time::Duration::from_secs(3)).unwrap();

    
}

#[cfg(windows)]
fn main() {
    println!("No windows for you good sir.");
}