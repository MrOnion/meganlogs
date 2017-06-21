extern crate toml;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serial;
extern crate ncurses;

mod command;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use serial::prelude::*;
use ncurses::*;

const COLOR_PAIR_OFF: i16 = 1;
const COLOR_PAIR_ON: i16 = 2;

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

fn change_status(state: bool) {
    attron(A_BOLD() | COLOR_PAIR(if state {COLOR_PAIR_ON} else {COLOR_PAIR_OFF}));
    let width = getmaxx(stdscr()) - 1;
    (1..width).fold((), |_, x| {
        let z = mvaddch(7, x, 32);
    });
    mvaddstr(7, (width - 3) / 2, (if state { "ON" } else { "OFF" }));
}

#[cfg(unix)]
fn main() {
    let mut logging = false;

    let mut config_file: File = File::open(Path::new("meganlogs.toml")).unwrap();
    let mut config_data: String = String::new();
    config_file.read_to_string(&mut config_data).unwrap();
    let config: Config = toml::from_str(&config_data).unwrap();

    let a = "/dev/ttyUSB0".to_string();
    let device: &String = config.serial_device.as_ref().unwrap_or(&a);
    let mut port: serial::SystemPort = serial::open(device).unwrap();
    port.configure(&SETTINGS).unwrap();
    port.set_timeout(std::time::Duration::from_secs(3)).unwrap();

    // get signatures from ECU
    let sig_firmware: String = command::signature_firmware(&mut port).unwrap();
    let sig_comms: String = command::signature_comms(&mut port).unwrap();

    initscr();
    keypad(stdscr(), true);
    nodelay(stdscr(), true);
    noecho();
    cbreak();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    start_color();
    init_pair(COLOR_PAIR_OFF, COLOR_WHITE, COLOR_RED);
    init_pair(COLOR_PAIR_ON, COLOR_WHITE, COLOR_GREEN);

    box_(stdscr(), 0, 0);

    mvaddstr(1, 1, "*** MeganLogs ***");
    mvaddstr(3, 1, "Serial:");
    mvaddstr(3, 12, device);
    mvaddstr(4, 1, "Firmware:");
    mvaddstr(4, 12, &sig_firmware);
    mvaddstr(5, 1, "Comms:");
    mvaddstr(5, 12, &sig_comms);

    change_status(logging);

    loop {
        match getch() {
            KEY_TOGGLE => {
                logging = !logging;
                change_status(logging);
            },
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