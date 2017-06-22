extern crate toml;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serial;
extern crate ncurses;
extern crate time;

mod command;
mod logfile;

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
    data_rate: Option<u16>,
    split: Option<u16>,
    prefix: Option<String>
}

fn change_status(state: bool) {
    attron(A_BOLD() | COLOR_PAIR(if state {COLOR_PAIR_ON} else {COLOR_PAIR_OFF}));
    let width = getmaxx(stdscr()) - 1;
    (1..width).fold((), |_, x| {
        mvaddch(11, x, 32);
    });
    mvaddstr(11, (width - 3) / 2, (if state {"ON"} else {"OFF"}));
    attr_off(A_BOLD() | COLOR_PAIR(if state {COLOR_PAIR_ON} else {COLOR_PAIR_OFF}));
}

#[cfg(unix)]
fn main() {
    let mut logging = false;

    let mut config_file: File = File::open(Path::new("meganlogs.toml")).unwrap();
    let mut config_data: String = String::new();
    config_file.read_to_string(&mut config_data).unwrap();
    let config: Config = toml::from_str(&config_data).unwrap();

    let device: String = config.serial_device.unwrap_or("/dev/ttyUSB0".to_string());
    let mut port: serial::SystemPort = serial::open(&device).unwrap();
    port.configure(&SETTINGS).unwrap();
    port.set_timeout(std::time::Duration::from_secs(3)).unwrap();

    // get signatures from ECU
    let sig_firmware: String = command::signature_firmware(&mut port).unwrap();
    let sig_comms: String = command::signature_comms(&mut port).unwrap();
    let monitor_version: u16 = command::monitor_version(&mut port).unwrap();

    // initial log row to get row size
    let init_row = command::realtime_data(&mut port).unwrap();

    // Create initial mutable data
    let frd_header: Vec<u8> = logfile::create_frd_header(&sig_firmware, init_row.len());

    // calculate sleep value in ms
    let sleep_value: u64 = (1000_f32 / config.data_rate.unwrap_or(15) as f32 - 20_f32).ceil() as u64;

    // Start UI
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
    mvaddstr(3, 12, &device);
    mvaddstr(4, 1, "Firmware:");
    mvaddstr(4, 12, &sig_firmware);
    mvaddstr(5, 1, "Comms:");
    mvaddstr(5, 12, &sig_comms);
    mvaddstr(6, 1, "Monitor:");
    mvaddstr(6, 12, &format!("{}", monitor_version));
    mvaddstr(7, 1, "Data rate:");
    mvaddstr(7, 12, &format!("{} rps ({} ms)", &config.data_rate.unwrap_or(15), sleep_value));
    mvaddstr(8, 1, "Log file:");
    mvaddstr(9, 1, "Log size:");

    change_status(logging);
    let mut log_path: String;
    let mut log_file: Option<File> = None;
    let path: &String = &config.log_path.unwrap_or(".".to_string());
    let prefix: &String = &config.prefix.unwrap_or("logfile".to_string());
    let timestamp: String = time::strftime("%Y%m%d_%H%M%S", &time::now()).unwrap();

    loop {
        match getch() {
            KEY_TOGGLE => {
                logging = !logging;
                if logging {
                    log_path = logfile::create_path(path, prefix, &timestamp, 1, 1);
                    log_file = Some(logfile::create_logfile(&log_path, &frd_header).unwrap());
                    mvaddstr(8, 12, &log_path);
                } else {
                    log_file = None;
                }
                change_status(logging);
            },
            KEY_MARKER => println!("MARKER!"),
            KEY_QUIT => break,
            _ => ()
        }

        match log_file.as_mut() {
            Some(active) => {
                active.write(&command::realtime_data(&mut port).unwrap()).unwrap();
                mvaddstr(9, 12, &format!("{} bytes", active.metadata().unwrap().len()));
                std::thread::sleep(std::time::Duration::from_millis(sleep_value));
            },
            None => ()
        }
    }

    endwin();
}

#[cfg(windows)]
fn main() {
    println!("No windows for you good sir.");
}