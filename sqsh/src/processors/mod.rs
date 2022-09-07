//! # Processors
//! 
//! Processors are consuming the data stream from the source and writing
//! some output to the sink. All submodules are implementing some kind of
//! processors which implement the `crate::core::Process` trait.
mod duplicate;

// Reexport processors on this level
pub use duplicate::Duplicate;
