use crate::core::process::StreamProcess;
use std::fmt::Display;

// Must be > 1
const LOSSY_RLE_THRESHOLD: usize = 2;

pub struct LossyRleEncoder {
    repetition: usize,
    threshold: usize,
    last_symbol: Option<u8>,
    loss_count: usize,
}

impl Display for LossyRleEncoder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LossyRLE<rep:{}, min:{}, sym:{:?}, lcount:{}>",
            self.repetition, self.threshold, self.last_symbol, self.loss_count
        )
    }
}

#[allow(dead_code, clippy::assertions_on_constants)]
impl LossyRleEncoder {
    pub fn new() -> Self {
        assert!(LOSSY_RLE_THRESHOLD > 1);
        LossyRleEncoder {
            repetition: 0,
            threshold: LOSSY_RLE_THRESHOLD,
            last_symbol: None,
            loss_count: 0,
        }
    }
    pub fn with_threshold(threshold: usize) -> Self {
        assert!(threshold > 1);
        LossyRleEncoder {
            repetition: 0,
            threshold,
            last_symbol: None,
            loss_count: 0,
        }
    }
    fn new_symbol(&mut self, byte: u8) {
        self.last_symbol = Some(byte);
        self.repetition = 1;
    }
    fn reset(&mut self) {
        self.last_symbol = None;
        self.repetition = 0;
    }
    fn write_to_sink(&mut self, sink: &mut Vec<u8>) -> std::io::Result<usize> {
        sink.push(self.last_symbol.unwrap());
        sink.push(self.repetition as u8 + self.loss_count as u8 - 1u8);
        self.loss_count = 0;
        Ok(1)
    }
}

impl Default for LossyRleEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamProcess for LossyRleEncoder {
    fn process_byte(&mut self, byte: &u8, sink: &mut Vec<u8>) -> std::io::Result<usize> {
        match self.last_symbol {
            None => self.new_symbol(*byte),
            Some(ls) => {
                if ls == *byte {
                    self.repetition += 1;
                } else if self.repetition < self.threshold {
                    self.loss_count += self.repetition;
                    self.new_symbol(*byte);
                } else {
                    self.write_to_sink(sink)?;
                    self.new_symbol(*byte);
                }
            }
        }
        Ok(1)
    }
    fn finish_byte(&mut self, sink: &mut Vec<u8>) -> std::io::Result<usize> {
        match self.last_symbol {
            None => Ok(0),
            Some(_) => {
                self.write_to_sink(sink)?;
                self.reset();
                Ok(1)
            }
        }
    }
}

pub struct LossyRleDecoder {
    last_symbol: Option<u8>,
}

impl Default for LossyRleDecoder {
    fn default() -> Self {
        LossyRleDecoder::new()
    }
}

impl LossyRleDecoder {
    pub fn new() -> Self {
        LossyRleDecoder { last_symbol: None }
    }
}

impl Display for LossyRleDecoder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LossyRLEDecoder<last_sym:{:?}>", self.last_symbol)
    }
}

impl StreamProcess for LossyRleDecoder {
    fn process_byte(&mut self, byte: &u8, sink: &mut Vec<u8>) -> std::io::Result<usize> {
        match self.last_symbol {
            None => {
                self.last_symbol = Some(*byte);
            }
            Some(ls) => {
                let mut output = [ls].repeat(*byte as usize + 1);
                sink.append(&mut output);
                self.last_symbol = None;
            }
        }
        Ok(1)
    }
    fn finish_byte(&mut self, sink: &mut Vec<u8>) -> std::io::Result<usize> {
        match self.last_symbol {
            None => Ok(0),
            Some(ls) => {
                sink.push(ls);
                Ok(1)
            }
        }
    }
}

impl From<LossyRleEncoder> for LossyRleDecoder {
    fn from(_: LossyRleEncoder) -> Self {
        LossyRleDecoder { last_symbol: None }
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::as_rle_bytes;
    use super::*;
    use crate::core::process::tests::{roundtrip_lossy, test_process};

    #[test]
    fn test_lossy_rle() {
        test_process::<LossyRleEncoder>("aaaabbvvvvv".as_bytes(), &as_rle_bytes("a3b1v4"));
        test_process::<LossyRleEncoder>("aaa".as_bytes(), &as_rle_bytes("a2"));
        test_process::<LossyRleEncoder>("aaabb".as_bytes(), &as_rle_bytes("a2b1"));
        test_process::<LossyRleEncoder>("aaabcde".as_bytes(), &as_rle_bytes("a2e3"));
        test_process::<LossyRleEncoder>("aaabcddde".as_bytes(), &as_rle_bytes("a2d4e0"));
        test_process::<LossyRleEncoder>("aabbccdd".as_bytes(), &as_rle_bytes("a1b1c1d1"));
        test_process::<LossyRleEncoder>("aabbcdd".as_bytes(), &as_rle_bytes("a1b1d2"));
    }

    #[test]
    fn test_lossy_roundtrip() {
        roundtrip_lossy::<LossyRleEncoder, LossyRleDecoder>(
            "aaaabbvvvvv".as_bytes(),
            "aaaabbvvvvv".as_bytes(),
        );
        roundtrip_lossy::<LossyRleEncoder, LossyRleDecoder>("aaa".as_bytes(), "aaa".as_bytes());
        roundtrip_lossy::<LossyRleEncoder, LossyRleDecoder>("aaabb".as_bytes(), "aaabb".as_bytes());
        roundtrip_lossy::<LossyRleEncoder, LossyRleDecoder>(
            "aaabcde".as_bytes(),
            "aaaeeee".as_bytes(),
        );
        roundtrip_lossy::<LossyRleEncoder, LossyRleDecoder>(
            "aaabcddde".as_bytes(),
            "aaaddddde".as_bytes(),
        );
        roundtrip_lossy::<LossyRleEncoder, LossyRleDecoder>(
            "aabbccdd".as_bytes(),
            "aabbccdd".as_bytes(),
        );
        roundtrip_lossy::<LossyRleEncoder, LossyRleDecoder>(
            "aabbcdd".as_bytes(),
            "aabbddd".as_bytes(),
        );
    }
}
