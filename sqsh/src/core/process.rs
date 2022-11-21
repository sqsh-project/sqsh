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

/// The `StreamProcess` trait allows processing of bytes individually.
pub trait StreamProcess {
    fn process_byte(&mut self, byte: &u8, sink: &mut Vec<u8>) -> IOResult<usize>;
    fn finish_byte(&mut self, sink: &mut Vec<u8>) -> IOResult<usize>;
}

/// Blank implementation of `Process` for objects implementing
/// the `StreamProcess` trait.
impl<S: StreamProcess> Process for S {
    fn process(&mut self, source: &[u8], sink: &mut Vec<u8>) -> IOResult<usize> {
        for byte in source {
            S::process_byte(self, byte, sink)?;
        }
        Ok(source.len())
    }
    fn finish(&mut self, sink: &mut Vec<u8>) -> IOResult<usize> {
        S::finish_byte(self, sink)
    }
}

#[cfg(test)]
#[allow(dead_code)]
pub(crate) mod tests {
    use super::Process;

    pub(crate) fn test_process<P: Process + Default>(source: &[u8], expected: &[u8]) {
        let mut d: P = Default::default();
        let mut result: Vec<u8> = Vec::new();
        d.process(source, &mut result).expect("Error");
        let mut fin = Vec::<u8>::new();
        d.finish(&mut fin).expect("Error");
        result.append(&mut fin);
        assert_eq!(result, expected)
    }

    pub(crate) fn roundtrip<E: Process + Default, D: From<E> + Process>(source: &[u8]) {
        let mut enc: E = Default::default();
        let mut encoded: Vec<u8> = Vec::new();
        enc.process(source, &mut encoded).expect("Error");
        let mut fin = Vec::<u8>::new();
        enc.finish(&mut fin).expect("Error");
        encoded.append(&mut fin);

        let mut dec: D = D::from(enc);
        let mut decoded: Vec<u8> = Vec::new();
        dec.process(&encoded[..], &mut decoded).expect("Error");
        let mut fin = Vec::<u8>::new();
        dec.finish(&mut fin).expect("Error");
        decoded.append(&mut fin);

        assert_eq!(source, decoded)
    }

    pub(crate) fn roundtrip_lossy<E: Process + Default, D: From<E> + Process>(
        source: &[u8],
        expected: &[u8],
    ) {
        let mut enc: E = Default::default();
        let mut encoded: Vec<u8> = Vec::new();
        enc.process(source, &mut encoded).expect("Error");
        let mut fin = Vec::<u8>::new();
        enc.finish(&mut fin).expect("Error");
        encoded.append(&mut fin);

        let mut dec: D = D::from(enc);
        let mut decoded: Vec<u8> = Vec::new();
        dec.process(&encoded[..], &mut decoded).expect("Error");
        let mut fin = Vec::<u8>::new();
        dec.finish(&mut fin).expect("Error");
        decoded.append(&mut fin);

        assert_eq!(decoded, expected)
    }
}
