//! # Checksums
//!
//! Checksums are used to check the integrity of the data after decompression.
//! Each Checksum has to implement the `Process` trait.
use super::Process;

/// Checksum trait for calculating the checksum from the internal state
pub trait Checksum: Process {
    type Output;

    /// Calculate the checksum from the inner state
    fn checksum(&self) -> Self::Output;
}

#[cfg(test)]
#[allow(dead_code)]
pub(crate) mod tests {
    use super::Checksum;
    use crate::core::Process;
    use std::fmt::{Debug, Display};

    pub(crate) fn assert_checksum<
        T: PartialEq + Debug,
        C: Default + Process + Checksum<Output = T>,
    >(
        source: &[u8],
        expected: <C as Checksum>::Output,
    ) {
        let mut model: C = Default::default();
        let mut sink = Vec::<u8>::new();
        model.process(source, &mut sink).expect("Error");
        assert_eq!(model.checksum(), expected);
    }

    pub(crate) fn check_debug_format<C: Default + Debug>(expected: &str) {
        let m: C = Default::default();
        assert_eq!(format!("{m:?}"), expected)
    }

    pub(crate) fn check_display_format<C: Default + Display>(expected: &str) {
        let m: C = Default::default();
        assert_eq!(format!("{m}"), expected)
    }
}
