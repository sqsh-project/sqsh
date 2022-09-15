use crate::core::process::StreamProcess;
use std::fmt::Display;

/// Must be > 1
const CLASSIC_RLE_THRESHOLD: usize = 2;

/// Classic RLE encoder implementation using special key defined by MNP5 algorithm
///
/// Classic RLE encoder with `repetition` representing the number of occurences
/// of the last seen symbol, `max_threshold` the number of repetition which will
/// be replaced by the encoder (must be at least 2; efficient encoding only happens
/// with max_threshold + 1 repetitions), and the last_symbol.
pub struct RleClassicEncoder {
    repetition: usize,
    max_threshold: usize,
    last_symbol: Option<u8>,
}

impl Display for RleClassicEncoder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ClassicRLE< reps:{} max:{} sym:{:#?} >",
            self.repetition, self.max_threshold, self.last_symbol
        )
    }
}

#[allow(dead_code)]
impl RleClassicEncoder {
    /// Create a new classic RLE Encoder with default threshold
    pub fn new() -> Self {
        assert!(CLASSIC_RLE_THRESHOLD > 1);
        RleClassicEncoder {
            repetition: 0,
            max_threshold: CLASSIC_RLE_THRESHOLD,
            last_symbol: None,
        }
    }
    /// Create a new classic RLE Encoder with custom threshold
    pub fn with_threshold(max_threshold: usize) -> Self {
        assert!(max_threshold > 1);
        RleClassicEncoder {
            repetition: 0,
            max_threshold,
            last_symbol: None,
        }
    }

    /// Reset Encoder
    pub fn reset(&mut self) {
        self.repetition = 0;
        self.last_symbol = None;
    }

    /// Write last symbol and if necessary the number of occurences to sink
    fn write_to_sink(&mut self, sink: &mut Vec<u8>) {
        let last_symbol = self.last_symbol.unwrap();
        if self.repetition >= self.max_threshold {
            let diff = self.repetition - self.max_threshold;
            assert!(diff <= u8::MAX as usize);

            let mut output = [last_symbol].repeat(self.max_threshold);
            output.push(diff as u8);
            sink.append(&mut output);
        } else {
            let mut output = [last_symbol].repeat(self.repetition);
            sink.append(&mut output);
        }
    }

    /// Setup new symbol
    fn new_symbol(&mut self, byte: u8) {
        self.repetition = 1;
        self.last_symbol = Some(byte);
    }
}

impl Default for RleClassicEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamProcess for RleClassicEncoder {
    fn process_byte(&mut self, byte: &u8, sink: &mut Vec<u8>) -> std::io::Result<usize> {
        match self.last_symbol {
            Some(ls) => {
                if ls == *byte {
                    self.repetition += 1;
                } else {
                    self.write_to_sink(sink);
                    self.new_symbol(*byte);
                }
            }
            None => self.new_symbol(*byte),
        }
        Ok(1)
    }

    fn finish_byte(&mut self, sink: &mut Vec<u8>) -> std::io::Result<usize> {
        match self.last_symbol {
            Some(_) => {
                self.write_to_sink(sink);
                self.reset();
                Ok(1)
            }
            None => Ok(0),
        }
    }
}

pub struct RleClassicDecoder {
    repetition: usize,
    max_threshold: usize,
    last_symbol: Option<u8>,
}

impl Display for RleClassicDecoder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RLE Classic Decoder < reps:{} max:{} sym:{:#?} >",
            self.repetition, self.max_threshold, self.last_symbol
        )
    }
}

#[allow(dead_code)]
impl RleClassicDecoder {
    /// Create a new classic RLE Decoder with default threshold
    pub fn new() -> Self {
        assert!(CLASSIC_RLE_THRESHOLD > 1);
        RleClassicDecoder {
            repetition: 0,
            max_threshold: CLASSIC_RLE_THRESHOLD,
            last_symbol: None,
        }
    }
    /// Create a new classic RLE Decoder with custom threshold
    pub fn with_threshold(max_threshold: usize) -> Self {
        assert!(max_threshold > 1);
        RleClassicDecoder {
            repetition: 0,
            max_threshold,
            last_symbol: None,
        }
    }
    /// Reset Decoder
    pub fn reset(&mut self) {
        self.repetition = 0;
        self.last_symbol = None;
    }
}

impl Default for RleClassicDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamProcess for RleClassicDecoder {
    fn process_byte(&mut self, byte: &u8, sink: &mut Vec<u8>) -> std::io::Result<usize> {
        match self.last_symbol {
            None => {
                self.last_symbol = Some(*byte);
                self.repetition = 1;
                sink.push(*byte);
                Ok(1)
            }
            Some(ls) => {
                if *byte == ls {
                    self.repetition += 1;
                    sink.push(*byte);
                } else if self.repetition == self.max_threshold {
                    let mut v = [ls].repeat(*byte as usize);
                    sink.append(&mut v);
                    self.reset();
                } else {
                    self.repetition = 1;
                    self.last_symbol = Some(*byte);
                    sink.push(*byte)
                }
                Ok(1)
            }
        }
    }
    fn finish_byte(&mut self, _: &mut Vec<u8>) -> std::io::Result<usize> {
        self.reset();
        Ok(0)
    }
}

impl From<RleClassicEncoder> for RleClassicDecoder {
    fn from(rle: RleClassicEncoder) -> Self {
        RleClassicDecoder::with_threshold(rle.max_threshold)
    }
}

impl From<RleClassicDecoder> for RleClassicEncoder {
    fn from(rle: RleClassicDecoder) -> Self {
        RleClassicEncoder::with_threshold(rle.max_threshold)
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::as_rle_bytes;
    use super::*;
    use crate::core::{
        process::tests::{roundtrip, test_process},
        Process,
    };

    #[test]
    fn test_init_new() {
        let rle = RleClassicEncoder::new();

        assert_eq!(rle.max_threshold, CLASSIC_RLE_THRESHOLD);
        assert_eq!(rle.repetition, 0);
        assert_eq!(rle.last_symbol, None);
    }

    #[test]
    fn test_init_custom_threshold() {
        let max_threshold: usize = CLASSIC_RLE_THRESHOLD + 2;
        let rle = RleClassicEncoder::with_threshold(max_threshold);

        assert_eq!(rle.max_threshold, CLASSIC_RLE_THRESHOLD + 2);
        assert_eq!(rle.repetition, 0);
        assert_eq!(rle.last_symbol, None);
    }

    #[test]
    fn test_reset() {
        let mut rle = RleClassicEncoder::new();
        let source = "Wikipedia".as_bytes();
        let mut v = Vec::new();

        rle.process(source, &mut v).unwrap();
        assert_eq!(rle.repetition, 1);
        assert_eq!(rle.last_symbol, Some(97));

        rle.reset();
        assert_eq!(rle.repetition, 0);
        assert_eq!(rle.last_symbol, None);
    }

    #[test]
    fn test_format() {
        let rle = RleClassicEncoder::new();
        let expected: String = format!("ClassicRLE< reps:0 max:2 sym:None >");

        assert_eq!(rle.to_string(), expected);
    }

    #[test]
    fn test_classic_rle() {
        test_process::<RleClassicEncoder>("Awesome".as_bytes(), "Awesome".as_bytes());
        test_process::<RleClassicEncoder>("Aweeeeee".as_bytes(), &as_rle_bytes("Awee4"));
        test_process::<RleClassicEncoder>("eeeeeeeeeee".as_bytes(), &as_rle_bytes("ee9"));
        test_process::<RleClassicEncoder>("eeeeeeeeeeee".as_bytes(), &as_rle_bytes("ee10"));
        test_process::<RleClassicEncoder>("eeeeeeeeeeeez".as_bytes(), &as_rle_bytes("ee10z"));
        test_process::<RleClassicEncoder>(
            "eeezzzzzzzopsaa".as_bytes(),
            &as_rle_bytes("ee1zz5opsaa0"),
        );
    }

    #[test]
    fn test_roundtrip() {
        roundtrip::<RleClassicEncoder, RleClassicDecoder>("Wikipedia".as_bytes());
        roundtrip::<RleClassicEncoder, RleClassicDecoder>("eeeeeeeee".as_bytes());
        roundtrip::<RleClassicEncoder, RleClassicDecoder>("Awesome".as_bytes());
        roundtrip::<RleClassicEncoder, RleClassicDecoder>("Aweeeeee".as_bytes());
        roundtrip::<RleClassicEncoder, RleClassicDecoder>("eeeeeeeeeee".as_bytes());
        roundtrip::<RleClassicEncoder, RleClassicDecoder>("eeeeeeeeeeee".as_bytes());
        roundtrip::<RleClassicEncoder, RleClassicDecoder>("eeeeeeeeeeeez".as_bytes());
        roundtrip::<RleClassicEncoder, RleClassicDecoder>("eeezzzzzzzopsaa".as_bytes());
    }

    #[test]
    fn test_enc_to_dec() {}

    #[test]
    fn test_dec_to_enc() {
        let dec = RleClassicDecoder::new();
        let v = dec.max_threshold;
        let enc: RleClassicEncoder = RleClassicDecoder::into(dec);

        assert_eq!(v, enc.max_threshold)
    }
}
