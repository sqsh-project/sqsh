use crate::core::Process;
use std::io::Result as IOResult;

/// Duplicate all data from the source to the sink (copy).
pub struct Duplicate {}

impl Duplicate {
    pub fn new() -> Self {
        Duplicate {}
    }
}

impl Default for Duplicate {
    fn default() -> Self {
        Self::new()
    }
}

impl Process for Duplicate {
    fn process(&mut self, source: &[u8], sink: &mut Vec<u8>) -> IOResult<usize> {
        sink.extend(source);
        Ok(source.len())
    }
    fn finish(&mut self, _: &mut Vec<u8>) -> IOResult<usize> {
        Ok(0)
    }
    fn consume(&mut self, source: &[u8], sink: &mut impl std::io::Write) -> IOResult<usize> {
        // let mut result = Vec::new();
        // result.extend(source);
        sink.write_all(&source)?;
        Ok(source.len())
    }
    fn end(&mut self, _: &mut impl std::io::Write) -> IOResult<usize> {
        Ok(0)
    }
}
