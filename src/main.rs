extern crate toml;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serial;
extern crate ncurses;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use serial::prelude::*;
use ncurses::*;

const COLOR_BACKGROUND: i16 = 16;
const COLOR_FOREGROUND: i16 = 17;
const COLOR_PAIR_DEFAULT: i16 = 1;

const KEY_TOGGLE: i32 = 32;
const KEY_MARKER: i32 = 10;
const KEY_QUIT: i32 = 113;

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

    initscr();
    keypad(stdscr(), true);
    nodelay(stdscr(), true);
    noecho();

    start_color();
    init_color(COLOR_BACKGROUND, 0, 0, 0);
    init_color(COLOR_FOREGROUND, 200, 300, 200);
    init_pair(COLOR_PAIR_DEFAULT, COLOR_FOREGROUND, COLOR_BACKGROUND);

    bkgd(' ' as chtype | COLOR_PAIR(COLOR_PAIR_DEFAULT) as chtype);

    loop {
        match getch() {
            KEY_TOGGLE => println!("TOGGLE!"),
            KEY_MARKER => println!("MARKER!"),
            KEY_QUIT => break,
            _ => ()
        }
    }

    endwin();
}

#[cfg(windows)]
fn main() {
    println!("No windows for you good sir.");
}