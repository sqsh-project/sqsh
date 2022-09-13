//! # Adler32
//!
//! Implementation of the Adler32 checksum algorithm as described
//! [here](https://en.wikipedia.org/wiki/Adler-32).
use crate::core::Process;
use log::{debug, info};
use std::fmt::Display;

/// Adler32 struct to save normal and aggregated sum
#[derive(Debug)]
pub struct Adler32 {
    a: u16,
    b: u16,
}

pub trait Checksum {
    type Output;

    fn checksum(&self) -> Self::Output;
}

impl Adler32 {
    /// Generate new Adler32 struct
    pub fn new() -> Self {
        info!("New Adler32 checksum");
        Adler32 { a: 1, b: 0 }
    }
}

impl Checksum for Adler32 {
    type Output = u32;

    fn checksum(&self) -> u32 {
        let result = ((self.b as u32) << 16) | self.a as u32;
        info!("Adler32 Checksum: {}", result);
        result
    }
}

/// Use the new function for generating the default implementation
impl Default for Adler32 {
    fn default() -> Self {
        Self::new()
    }
}

/// Printing should display the checksum
impl Display for Adler32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let csum = self.checksum();
        write!(f, "Adler32<{csum:#010X}>")
    }
}

/// Implementation of the Process trait for Adler32
impl Process for Adler32 {
    fn process(&mut self, source: &[u8], _: &mut Vec<u8>) -> std::io::Result<usize> {
        for byte in source.iter() {
            self.a += *byte as u16 % u16::MAX;
            self.b += self.a % u16::MAX;
            debug!("Adler32 Update: {}, New State: {:?}", byte, self)
        }
        Ok(source.len())
    }
    fn finish(&mut self, _: &mut Vec<u8>) -> std::io::Result<usize> {
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::{Adler32, Checksum};
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

    fn check_debug_format<C: Default + Debug>(expected: &str) {
        let m: C = Default::default();
        assert_eq!(format!("{m:?}"), expected)
    }

    fn check_display_format<C: Default + Display>(expected: &str) {
        let m: C = Default::default();
        assert_eq!(format!("{m}"), expected)
    }

    #[test]
    fn adler32() {
        assert_checksum::<u32, Adler32>("Wikipedia".as_bytes(), 0x11E60398);
        assert_checksum::<u32, Adler32>("Awesome-string-baby".as_bytes(), 0x49D50761);
        assert_checksum::<u32, Adler32>("This is great".as_bytes(), 0x20AF04C8);
    }

    #[test]
    fn formatting() {
        check_debug_format::<Adler32>("Adler32 { a: 1, b: 0 }");
        check_display_format::<Adler32>("Adler32<0x00000001>");
    }
}
