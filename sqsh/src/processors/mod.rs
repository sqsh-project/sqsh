//! # Processors
//!
//! Processors are consuming the data stream from the source and writing
//! some output to the sink. All submodules are implementing some kind of
//! processors which implement the `crate::core::Process` trait.
mod adler32;
mod crc32;
mod duplicate;
mod rle;

// Reexport processors on this level
pub use adler32::Adler32;
pub use crc32::CRC32;
pub use duplicate::Duplicate;
pub use rle::{RleClassicDecoder, RleClassicEncoder, TelemetryRleDecoder, TelemetryRleEncoder};
