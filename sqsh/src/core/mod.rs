//! # Core library
//!
//! This is the core library containing abstractions and traits.
//! While there are specific notes on each implementation, the core principles
//! are described in this module.
//!
//! The basic setup of the library takes a data source, processses the data,
//! produces new data and writes it into a data sink. This is valid for any
//! data processing in computer science. This module provides the necessary
//! abstractions and traits to implement these processes.
//!
//! The data source and data sinks are already provided by the Rust Standard
//! library and its `io` module. A data source should implement the
//! [`BufRead`](https://doc.rust-lang.org/std/io/trait.BufRead.html) trait and
//! a data sink implement the
//! [`Write`](https://doc.rust-lang.org/std/io/trait.Write.html) trait.
//! For the processing of the data the new Trait `Process` is being defined.
//!
//! These three components define the core of the data processing in the
//! library. The interaction of these components are organised by a `Stream`
//! object which coordinates the whole interaction.
//!
//! ## Terms
//!
//! - A *stream* is data traveling from a source to a sink.
//! - An *encoder* compresses a stream of data.
//! - A *decoder* decompresses a stream of data.
//! - A *codec* defines a pair of encoder and decoder.
//! - An processor operating in *streaming mode* processes each byte immediately.
//! - An processor operating in *block mode* processes the stream block by block and
//!   encodes them separately.
//! - The *compression factor* is the size of input stream / output stream. Higher is better.
//! - The *compression ratio* is the size of output stream / input stream. Lower is better.
//!
pub(crate) mod checksum;
pub(crate) mod process;
pub(crate) mod stream;

pub use checksum::Checksum;
pub use process::{Process, StreamProcess};
pub use stream::{Consume, Stream};
