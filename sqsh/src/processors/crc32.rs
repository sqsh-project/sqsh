//! CRC32 checksum
//!
//! Implementation of the CRC32 checksum algorithm as described [here](https://en.wikipedia.org/wiki/Cyclic_redundancy_check).
use std::fmt::Display;

use crate::core::{Checksum, Process};
use crc::{crc32, Hasher32};
use log::info;

/// CRC32 struct to save inner Digest element from `crc32` crate
pub struct CRC32 {
    a: crc32::Digest,
}

impl CRC32 {
    /// Generate new CRC32 struct
    pub fn new() -> Self {
        info!("New CRC32 checksum created");
        CRC32 {
            a: crc32::Digest::new(crc32::IEEE),
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
    fn finish(&mut self, sink: &mut Vec<u8>) -> std::io::Result<usize> {
        let result = self.to_string();
        sink.extend(result.as_bytes());
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
    use super::*;
    use crate::core::checksum::tests::*;

    #[test]
    fn crc32() {
        assert_checksum::<u32, CRC32>("Awesome-string-baby".as_bytes(), 0x7900b113);
        assert_checksum::<u32, CRC32>("Wikipedia".as_bytes(), 0xadaac02e);
        assert_checksum::<u32, CRC32>("This is great".as_bytes(), 0xc6314444);
        assert_checksum::<u32, CRC32>("sqsh".as_bytes(), 0x4a861156);
        assert_checksum::<u32, CRC32>("".as_bytes(), 0x00000000);
    }

    #[test]
    fn formatting() {
        check_display_format::<CRC32>("CRC32<0x00000000>");
    }
}
