extern crate byteorder;

use byteorder::{LittleEndian, ReadBytesExt};
use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;

fn main() -> io::Result<()> {
    let prefix = "mygraph";
    let upper_filename = format!("{}.upper", prefix);
    let lower_filename = format!("{}.lower", prefix);

    let upper_path = Path::new(&upper_filename);
    let lower_path = Path::new(&lower_filename);

    // Open the upper and lower files
    let upper_file = File::open(upper_path)?;
    let lower_file = File::open(lower_path)?;

    // Create buffer readers
    let mut upper_reader = BufReader::new(upper_file);
    let mut lower_reader = BufReader::new(lower_file);

    // Read from the upper file
    loop {
        // Read the upper x, y coordinates and count
        let _ux = match upper_reader.read_u16::<LittleEndian>() {
            Ok(x) => x,
            Err(_) => break,
        };
        let _uy = upper_reader.read_u16::<LittleEndian>()?;
        let count = upper_reader.read_u32::<LittleEndian>()?;

        // Read each lower coordinate pair count times
        for _ in 0..count {
            let lx = lower_reader.read_u16::<LittleEndian>()?;
            let ly = lower_reader.read_u16::<LittleEndian>()?;

            println!("{} {}", lx, ly);
        }
    }

    Ok(())
}
