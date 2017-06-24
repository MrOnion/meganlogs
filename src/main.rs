extern crate toml;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serial;
extern crate ncurses;
extern crate time;

mod command;
mod logfile;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use serial::prelude::*;
use ncurses::*;

const COLOR_PAIR_OFF: i16 = 1;
const COLOR_PAIR_ON: i16 = 2;

const KEY_TOGGLE: i32 = 32;
const KEY_MARKER: i32 = 10;
const KEY_SESSION: i32 = 115;
const KEY_QUIT: i32 = 113;

const ROW_PADDING: i32 = 12;
const ROW_SESSION_ID: i32 = 9;
const ROW_LOG_SIZE: i32 = 10;
const ROW_BANNER: i32 = 12;

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
    prefix: Option<String>
}

// Helpers

mod ui_help {
    use ncurses::*;

    pub fn clear_row(row: i32, from: i32) {
        let width = getmaxx(stdscr()) - 1;
        clear(width, row, from);
    }

    pub fn clear_rows(rows: &[i32], from: i32) {
        let width = getmaxx(stdscr()) - 1;
        for row in rows.iter() {
            clear(width, row.clone(), from);
        }
    }

    fn clear(width: i32, row: i32, from: i32) {
        for x in from..width {
            mvaddch(row, x, 32);
        };
    }
}

fn update_banner(state: bool) {
    attron(A_BOLD() | COLOR_PAIR(if state {COLOR_PAIR_ON} else {COLOR_PAIR_OFF}));
    let width = getmaxx(stdscr()) - 1;
    ui_help::clear_row(ROW_BANNER, 1);
    mvaddstr(ROW_BANNER, (width - 3) / 2, (if state {"ON"} else {"OFF"}));
    attr_off(A_BOLD() | COLOR_PAIR(if state {COLOR_PAIR_ON} else {COLOR_PAIR_OFF}));
}

// Main

#[cfg(unix)]
fn main() {
    // Parse config
    let mut config_file: File = File::open(Path::new("meganlogs.toml")).unwrap();
    let mut config_data: String = String::new();
    config_file.read_to_string(&mut config_data).unwrap();
    let config: Config = toml::from_str(&config_data).unwrap();

    // Setup serial
    let device: String = config.serial_device.unwrap_or("/dev/ttyUSB0".to_string());
    let mut port: serial::SystemPort = serial::open(&device).unwrap();
    port.configure(&SETTINGS).unwrap();
    port.set_timeout(std::time::Duration::from_secs(3)).unwrap();

    // get signatures from ECU
    let sig_firmware: String = command::signature_firmware(&mut port).unwrap();
    let sig_comms: String = command::signature_comms(&mut port).unwrap();
    let monitor_version: u16 = command::monitor_version(&mut port).unwrap();

    // initial log row to get metadata
    let init_row: Vec<u8> = command::realtime_data(&mut port).unwrap();
    let row_size: usize = init_row.len() as usize;

    // calculate sleep value between reads in ms (ugly feature but usable for now)
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
    mvaddstr(3, ROW_PADDING, &device);
    mvaddstr(4, 1, "Firmware:");
    mvaddstr(4, ROW_PADDING, &sig_firmware);
    mvaddstr(5, 1, "Comms:");
    mvaddstr(5, ROW_PADDING, &sig_comms);
    mvaddstr(6, 1, "Monitor:");
    mvaddstr(6, ROW_PADDING, &format!("{}", monitor_version));
    mvaddstr(8, 1, "Data rate:");
    mvaddstr(8, ROW_PADDING, &format!("{} rps ({} ms)", &config.data_rate.unwrap_or(15), sleep_value));
    mvaddstr(ROW_SESSION_ID, 1, "Session:");
    mvaddstr(ROW_LOG_SIZE, 1, "Log size:");

    // Beef

    let mut logging = false;
    update_banner(logging);

    let mut session_ids: (u16, u16) = (1, 1);
    let m_log: logfile::MLog = logfile::MLog::init(
        &config.log_path.unwrap_or(".".to_string()),
        &config.prefix.unwrap_or("logfile".to_string()),
        &sig_firmware,
        row_size
    );
    let mut log_file: Option<File> = None;

    loop {
        match getch() {
            KEY_TOGGLE => {
                logging = !logging;
                if logging {
                    log_file = Some(m_log.open(session_ids).unwrap());
                    mvaddstr(ROW_SESSION_ID, ROW_PADDING, &format!("{} / {}", session_ids.0, session_ids.1));
                    ui_help::clear_row(ROW_LOG_SIZE, ROW_PADDING);
                } else {
                    log_file.map(|f| f.sync_all());
                    log_file = None;
                    session_ids.1 += 1;
                }
                update_banner(logging);
            },
            KEY_SESSION => {
                if !logging {
                    session_ids = (session_ids.0 + 1, 1);
                    ui_help::clear_rows(&[ROW_SESSION_ID, ROW_LOG_SIZE], ROW_PADDING);
                }
            },
            KEY_MARKER => println!("MARKER!"),
            KEY_QUIT => break,
            _ => ()
        }

        match log_file.as_mut() {
            Some(active) => {
                active.write(&command::realtime_data(&mut port).unwrap()).unwrap();
                mvaddstr(ROW_LOG_SIZE, ROW_PADDING, &format!("{} bytes", active.metadata().unwrap().len()));
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