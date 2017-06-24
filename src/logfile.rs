extern crate time;
extern crate byteorder;

use std::io::*;
use std::fs::File;
use std::path::Path;
use self::byteorder::{BigEndian, WriteBytesExt};

pub struct MLog {
    path: String,
    prefix: String,
    signature: String,
    row_size: usize
}

impl MLog {
    pub fn init(path: &String, prefix: &String, signature: &String, row_size: usize) -> MLog {
        MLog {
            path: path.to_string(),
            prefix: prefix.to_string(),
            signature: signature.to_string(),
            row_size,
        }
    }

    pub fn open(&self, ids: (u16, u16)) -> Option<File> {
        let ts: time::Tm = time::now();
        let ts_format: String = time::strftime("%Y%m%d_%H%M%S", &ts).unwrap();
        let filename: String = format!("{}/{}_{}_s{}_run{}.frd", self.path, self.prefix, ts_format, ids.0, ids.1);
        let frd_header: Vec<u8> = self.create_frd_header(&ts);
        return match File::create(Path::new(&filename)) {
            Err(_) => None,
            Ok(mut f) => {
                f.write(&frd_header).unwrap();
                f.flush().unwrap();
                return Some(f);
            }
        }
    }

    pub fn marker() -> Vec<u8> {
        let mut marker = Vec::with_capacity(5);
        marker.extend([0x02].iter());
        let mut ts: Vec<u8> = Vec::with_capacity(4);
        ts.write_u32::<BigEndian>(time::precise_time_s() as u32).unwrap();
        marker.extend(ts);
        return marker;
    }

    fn create_frd_header(&self, timestamp: &time::Tm) -> Vec<u8> {
        let mut header: Vec<u8> = Vec::with_capacity(81);

        // FRD file format + version
        header.extend([0x46, 0x52, 0x44, 0x00, 0x00, 0x00, 0x00, 0x01].iter());
        // Timestamp
        let mut ts: Vec<u8> = Vec::with_capacity(4);
        ts.write_u32::<BigEndian>(timestamp.to_timespec().sec as u32).unwrap();
        header.extend(ts);
        // Signature
        let padded: String = format!("{:63}", self.signature);
        header.extend(padded.as_bytes().iter());
        // Data index
        header.extend([0x00, 0x00, 0x00, 0x51].iter());
        // Row size
        let mut rs: Vec<u8> = Vec::with_capacity(2);
        rs.write_u16::<BigEndian>(self.row_size as u16).unwrap();
        header.extend(rs);

        return header;
    }
}