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
use std::io::{BufRead, Result as IOResult, Write};

/// The `Process` trait allows processing bytes from a source and
/// writing the results to a sink.
///
/// Implementor of the `Process` trait are called `processors`.
///
/// This is an abstraction of any computational process. The bytes from the
/// source will be read. The processor decides based on the read bytes what to
/// write to the sink. It returns the number of bytes processed.
pub trait Process {
    /// Process the data from the source and write output to the sink
    fn process(&mut self, source: &[u8], sink: &mut Vec<u8>) -> IOResult<usize>;
    /// Finish the processing by outputing possible further data
    fn finish(&mut self, sink: &mut Vec<u8>) -> IOResult<usize>;
}

/// Default buffer size for the write buffer
const WRITE_BUFFER_SIZE: usize = 4_096;

/// Stream consumes the source and writes the output of the
/// processor to the sink.
///
/// The main task of the `Stream` is to consume the source. The only deciding
/// property is the buffer size. After that no property is being changed. The
/// `consume` method **fully** consumes the source.
pub struct Stream<B, W, P> {
    reader: B,
    writer: W,
    processor: P,
    buffer: Vec<u8>,
}

impl<B: BufRead, W: Write, P: Process> Stream<B, W, P> {
    /// Create a new Stream object with default buffer size
    pub fn new(reader: B, writer: W, processor: P) -> Self {
        let buffer = Vec::with_capacity(WRITE_BUFFER_SIZE);
        Stream {
            reader,
            writer,
            processor,
            buffer,
        }
    }
    /// Create a new Stream object with custom buffer size
    pub fn with_capacity(reader: B, writer: W, processor: P, capacity: usize) -> Self {
        let buffer = Vec::with_capacity(capacity);
        Stream {
            reader,
            writer,
            processor,
            buffer,
        }
    }
    /// Consume the source and fill the sink
    pub fn consume(&mut self) -> IOResult<usize> {
        let mut consumed: usize = 0;
        loop {
            let data = self.reader.fill_buf()?;
            let length = data.len();
            consumed += length;
            if length > 0 {
                self.processor.process(data, &mut self.buffer)?;
                self.writer.write(&self.buffer)?;
                self.reader.consume(length);
                self.buffer.clear()
            } else {
                self.processor.finish(&mut self.buffer)?;
                self.writer.write(&self.buffer)?;
                self.writer.flush()?;
                break;
            }
        }
        Ok(consumed)
    }
}
