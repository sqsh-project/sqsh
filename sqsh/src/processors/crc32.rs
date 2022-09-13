//! CRC32 checksum
//!
//! Implementation of the CRC32 checksum algorithm as described [here](https://en.wikipedia.org/wiki/Cyclic_redundancy_check).
use std::fmt::Display;

// use super::{Checksum, ChecksumError};
use crate::processors::adler32::Checksum;
use crc::{crc32, Hasher32};
use log::info;

use crate::core::Process;

/// CRC32 struct to save inner Digest element from `crc32` crate
pub struct CRC32 {
    a: crc32::Digest
}

impl CRC32 {
    /// Generate new CRC32 struct
    pub fn new() -> Self {
        info!("New CRC32 checksum created");
        CRC32 {
            a: crc32::Digest::new(crc32::IEEE)
        }
    }
}

/// Use the new function for generating the default implementation
impl Default for CRC32 {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for CRC32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let csum = self.a.sum32();
        write!(f, "CRC32<{csum:#010X}>")
    }
}

/// Implementation of the Checksum trait for CRC32
impl Process for CRC32 {
    fn process(&mut self, source: &[u8], _: &mut Vec<u8>) -> std::io::Result<usize> {
        self.a.write(source);
        Ok(source.len())
    }
    fn finish(&mut self, _: &mut Vec<u8>) -> std::io::Result<usize> {
        Ok(0)
    }
}

impl Checksum for CRC32 {
    type Output = u32;

    fn checksum(&self) -> Self::Output {
        self.a.sum32()
    }
}

#[cfg(test)]
mod tests {
    use super::{Checksum, CRC32};
    use crate::core::Process;
    use std::fmt::{Debug, Display};

    fn assert_checksum<T: PartialEq + Debug, C: Default + Process + Checksum<Output = T>>(
        source: &[u8],
        expected: <C as Checksum>::Output,
    ) {
        let mut model: C = Default::default();
        let mut sink = Vec::<u8>::new();
        model.process(source, &mut sink).expect("Error");
        assert_eq!(model.checksum(), expected);
    }
    
    fn check_display_format<C: Default + Display>(expected: &str) {
        let m: C = Default::default();
        assert_eq!(format!("{m}"), expected)
    }

    #[test]
    fn crc32() {
        assert_checksum::<u32, CRC32>("Awesome-string-baby".as_bytes(), 0x7900b113);
        assert_checksum::<u32, CRC32>("Wikipedia".as_bytes(), 0xadaac02e);
        assert_checksum::<u32, CRC32>("This is great".as_bytes(), 0xc6314444);
        assert_checksum::<u32, CRC32>("sqsh".as_bytes(), 0x4a861156);
    }

    #[test]
    fn formatting() {
        check_display_format::<CRC32>("CRC32<0x00000000>");
    }

}
