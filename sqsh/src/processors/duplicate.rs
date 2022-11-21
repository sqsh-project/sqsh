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
}

#[cfg(test)]
mod tests {
    use super::Duplicate;
    use crate::core::process::tests::*;

    #[test]
    fn test_duplication() {
        test_process::<Duplicate>("Wikipedia".as_bytes(), "Wikipedia".as_bytes());
        test_process::<Duplicate>(
            "Awesome-string-baby".as_bytes(),
            "Awesome-string-baby".as_bytes(),
        );
        test_process::<Duplicate>("This is great".as_bytes(), "This is great".as_bytes());
    }
}
