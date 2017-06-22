extern crate serial;
extern crate byteorder;

use std::io::*;
use self::byteorder::{BigEndian, ReadBytesExt};

const CMD_FIRMWARE: [u8;7] = [0x00, 0x01, 0x53, 0x20, 0x60, 0xEF, 0xC3];
const CMD_COMMS: [u8;7] = [0x00, 0x01, 0x51, 0xCE, 0x6E, 0x8E, 0xEF];
const CMD_MONITOR: [u8;7] = [0x00, 0x01, 0x4D, 0xDA, 0x6F, 0xD2, 0xA0];
const CMD_LOG: [u8;7] = [0x00, 0x01, 0x41, 0xD3, 0xD9, 0x9E, 0x8B];

const FLAG_OK: u8 = 0x00;
const FLAG_REALTIME_DATA: u8 = 0x01;

pub fn signature_firmware(port: &mut serial::SystemPort) -> Option<String> {
    return do_command(port, &CMD_FIRMWARE, FLAG_OK).and_then(|data| String::from_utf8(data).ok());
}

pub fn signature_comms(port: &mut serial::SystemPort) -> Option<String> {
    return do_command(port, &CMD_COMMS, FLAG_OK).and_then(|mut data| {
        data.retain(|&a| a >= 32);  // take out non-ascii chars
        String::from_utf8(data).ok()
    });
}

pub fn monitor_version(port: &mut serial::SystemPort) -> Option<u16> {
    return do_command(port, &CMD_MONITOR, FLAG_OK).and_then(|data| Cursor::new(data).read_u16::<BigEndian>().ok());
}

pub fn realtime_data(port: &mut serial::SystemPort) -> Option<Vec<u8>> {
    return do_command(port, &CMD_LOG, FLAG_REALTIME_DATA);
}

fn do_command(port: &mut serial::SystemPort, cmd: &[u8], flag: u8) -> Option<Vec<u8>> {
    return match port.write(cmd) {
        Err(_) => None,
        Ok(_) => {
            let mut buf: [u8;3] = [0;3];
            port.read_exact(&mut buf).unwrap();
            if buf[2] == flag {
                let size: usize = Cursor::new(vec![buf[0], buf[1]]).read_u16::<BigEndian>().unwrap() as usize;
                let mut payload: Vec<u8> = vec![0;size - 1];  // minus flag byte
                port.read_exact(&mut payload).unwrap();
                let mut crc: [u8;4] = [0;4];
                port.read_exact(&mut crc).unwrap();
                // TODO: check CRC
                return Some(payload);
            }
            return None;
        }
    };
}