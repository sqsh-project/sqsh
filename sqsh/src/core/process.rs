//! # Process
//!
//! This module defines the processing unit of the crate. It abstracts the
//! read and write of the data streams. It is the most integral part in the
//! library and shared by all components.
use std::io::Result as IOResult;

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

#[cfg(test)]
#[allow(dead_code)]
pub(crate) mod tests {
    use super::Process;

    pub(crate) fn test_buffered_process<P: Process + Default>(source: &[u8], expected: &[u8]) {
        let mut d: P = Default::default();
        let mut result: Vec<u8> = Vec::new();
        d.process(source, &mut result).expect("Error");
        let mut fin = Vec::<u8>::new();
        d.finish(&mut fin).expect("Error");
        result.append(&mut fin);
        assert_eq!(result, expected)
    }
}
