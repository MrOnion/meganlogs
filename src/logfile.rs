extern crate time;
extern crate byteorder;

use std::io::*;
use std::fs::File;
use std::path::Path;
use self::byteorder::{BigEndian, WriteBytesExt};

pub fn create_path(path: &String, prefix: &String, timestamp: &String, run_id: u16, split_id: u16) -> String {
    return format!("{}/{}_{}_run{}_{}.frd", path, prefix, timestamp, run_id, split_id);
}

pub fn create_logfile(filename: &String, frd_header: &Vec<u8>) -> Option<File> {
    return match File::create(Path::new(filename)) {
        Err(_) => None,
        Ok(mut f) => {
            f.write(frd_header).unwrap();
            f.flush().unwrap();
            return Some(f);
        }
    };
}

pub fn create_frd_header(signature: &String, row_size: usize) -> Vec<u8> {
    let mut header: Vec<u8> = Vec::with_capacity(81);

    // FRD file format + version
    header.extend([0x46, 0x52, 0x44, 0x00, 0x00, 0x00, 0x00, 0x01].iter().cloned());
    // Timestamp
    let mut ts = Vec::with_capacity(4);
    ts.write_u32::<BigEndian>(time::precise_time_s() as u32).unwrap();
    header.extend(ts);
    // Signature
    let padded: String = format!("{:63}", signature);
    header.extend(padded.as_bytes().iter().cloned());
    // Data index
    header.extend([0x00, 0x00, 0x00, 0x51].iter().cloned());
    // Row size
    let mut rs = Vec::with_capacity(2);
    rs.write_u16::<BigEndian>(row_size as u16).unwrap();
    header.extend(rs);

    return header;
}