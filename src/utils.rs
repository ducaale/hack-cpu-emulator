use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

// https://stackoverflow.com/a/35820003/5915221
pub fn lines_from_file(filename: impl AsRef<Path>) -> io::Result<Vec<String>> {
    BufReader::new(File::open(filename)?).lines().collect()
}

pub fn get_bit(bits: i16, index: u16) -> bool {
    (bits >> index) & 1 == 1
}

pub fn get_bit_slice(bits: i16, start: u16, end: u16) -> i16 {
    let mask = !(-1 << (end - start));
    (bits >> start) & mask
}