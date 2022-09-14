use crate::core::process::Process;
use std::io::{BufRead, Result as IOResult, Write};

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
                self.writer.write_all(&self.buffer)?;
                self.reader.consume(length);
                self.buffer.clear()
            } else {
                self.processor.finish(&mut self.buffer)?;
                self.writer.write_all(&self.buffer)?;
                self.writer.flush()?;
                break;
            }
        }
        Ok(consumed)
    }
}

impl<'a, B: BufRead, W: Write, P: Process> Iterator for &'a Stream<B, W, P> {
    type Item = &'a [u8];
    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}
