//! # Telemetry RLE
//!
//! This RLE algorithm will first calculate the difference between
//! two consecutive symbols and based on the difference of both values
//! will encode the data differently. If it is above a certain threshold
//! the absolute value will be saved and if it is below it, the difference.
//!
//! The differentiation between the streams will be done using a infobyte
//! for the previous 8 values; or pairs of u8 values looking like a u16.
use crate::core::Process;
use std::fmt::Display;

const TELEMETRY_RLE_MAX_THRESHOLD: u8 = 10;

/// Telemetry with differences and infobytes following each byte block
pub struct TelemetryRleEncoder {
    max_threshold: u8,
    remainder: Option<Vec<u8>>,
    last_byte: u8,
}

#[allow(dead_code, clippy::assertions_on_constants)]
impl TelemetryRleEncoder {
    pub fn new() -> Self {
        assert!(TELEMETRY_RLE_MAX_THRESHOLD <= 128u8);
        TelemetryRleEncoder {
            max_threshold: TELEMETRY_RLE_MAX_THRESHOLD,
            remainder: None,
            last_byte: 0,
        }
    }
    pub fn with_threshold(threshold: u8) -> Self {
        assert!(threshold <= 128u8);
        TelemetryRleEncoder {
            max_threshold: threshold,
            remainder: None,
            last_byte: 0,
        }
    }
    fn process_chunk(&mut self, chunk: &[u8], sink: &mut Vec<u8>) {
        let mut infobyte = 0u8;
        for c in chunk.iter() {
            infobyte <<= 1;
            let diff = self.last_byte.max(*c) - self.last_byte.min(*c);
            if diff <= self.max_threshold {
                if self.last_byte > *c {
                    sink.push(128u8 - diff)
                } else {
                    sink.push(128u8 + diff)
                }
            } else {
                infobyte += 1;
                sink.push(*c)
            }
            self.last_byte = *c;
        }
        sink.push(infobyte);
    }
}

impl Default for TelemetryRleEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for TelemetryRleEncoder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TelemetryRleEncoder(max-threshold: {}, remainder: {:?}), last_byte: {}",
            self.max_threshold, self.remainder, self.last_byte
        )
    }
}

impl Process for TelemetryRleEncoder {
    fn process(&mut self, source: &[u8], sink: &mut Vec<u8>) -> std::io::Result<usize> {
        let mut count = 0usize;
        let chunks = source.chunks_exact(8); // TODO: Maybe use an array?
        let r = chunks.remainder();
        self.remainder = if r.is_empty() { None } else { Some(r.to_vec()) };
        for chunk in chunks {
            self.process_chunk(chunk, sink);
            count += 1;
        }
        Ok(count * 8)
    }
    fn finish(&mut self, sink: &mut Vec<u8>) -> std::io::Result<usize> {
        match &self.remainder {
            None => Ok(0),
            Some(data) => {
                let l = data.len();
                let mut infobyte = 0u8;
                for c in data {
                    infobyte <<= 1;
                    let diff = self.last_byte.max(*c) - self.last_byte.min(*c);
                    if diff <= self.max_threshold {
                        if self.last_byte > *c {
                            sink.push(128u8 - diff) // TODO: Switch to two's complement
                        } else {
                            sink.push(128u8 + diff)
                        }
                    } else {
                        infobyte += 1;
                        sink.push(*c)
                    }
                    self.last_byte = *c;
                }
                sink.push(infobyte);
                Ok(l)
            }
        }
    }
}

/// Telemetry with differences and infobytes following each byte block
pub struct TelemetryRleDecoder {
    remainder: Option<Vec<u8>>,
    last_byte: u8,
}

#[allow(dead_code)]
impl TelemetryRleDecoder {
    pub fn new() -> Self {
        TelemetryRleDecoder {
            remainder: None,
            last_byte: 0,
        }
    }
    fn process_chunk(&mut self, chunk: &[u8], sink: &mut Vec<u8>) {
        let mut infobyte = chunk[chunk.len() - 1];
        for byte in &chunk[..8] {
            if infobyte & 0b1000_0000 > 0 {
                sink.push(*byte);
                self.last_byte = *byte;
            } else {
                let diff = 128u8.max(*byte) - 128u8.min(*byte);
                if *byte >= 128u8 {
                    sink.push(self.last_byte + diff);
                    self.last_byte += diff;
                } else {
                    sink.push(self.last_byte - diff);
                    self.last_byte -= diff;
                }
            }
            infobyte <<= 1;
        }
    }
}

impl Default for TelemetryRleDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for TelemetryRleDecoder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TelemetryRleDecoder(remainder: {:?}), last_byte: {}",
            self.remainder, self.last_byte
        )
    }
}

impl Process for TelemetryRleDecoder {
    fn process(&mut self, source: &[u8], sink: &mut Vec<u8>) -> std::io::Result<usize> {
        let mut count = 0usize;
        let chunks = source.chunks_exact(9); // Maybe use an array
        let r = chunks.remainder();
        self.remainder = if r.is_empty() { None } else { Some(r.to_vec()) };
        for chunk in chunks {
            self.process_chunk(chunk, sink);
            count += 1;
        }
        Ok(count * 9)
    }
    fn finish(&mut self, sink: &mut Vec<u8>) -> std::io::Result<usize> {
        match &self.remainder {
            None => Ok(0),
            Some(data) => {
                // TODO: Merge with process_chunk (same with encoder)
                let mut infobyte = data[data.len() - 1];
                for byte in &data[..data.len() - 1] {
                    if infobyte & 0b1000_0000 > 0 {
                        sink.push(*byte);
                        self.last_byte = *byte;
                    } else {
                        let diff = 128u8.max(*byte) - 128u8.min(*byte);
                        if *byte >= 128u8 {
                            sink.push(self.last_byte + diff);
                            self.last_byte += diff;
                        } else {
                            sink.push(self.last_byte - diff);
                            self.last_byte -= diff;
                        }
                    }
                    infobyte <<= 1;
                }
                Ok(data.len())
            }
        }
    }
}

impl From<TelemetryRleDecoder> for TelemetryRleEncoder {
    fn from(_: TelemetryRleDecoder) -> Self {
        TelemetryRleEncoder::new()
    }
}

impl From<TelemetryRleEncoder> for TelemetryRleDecoder {
    fn from(_: TelemetryRleEncoder) -> Self {
        TelemetryRleDecoder::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::process::tests::{roundtrip, test_process};

    #[test]
    fn test_telemetry_compression() {
        let input: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let expect: Vec<u8> = vec![129, 129, 129, 129, 129, 129, 129, 129, 0];
        test_process::<TelemetryRleEncoder>(&input, &expect);

        let input: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
        let expect: Vec<u8> = vec![129, 129, 129, 129, 129, 129, 129, 129, 0, 129, 0];
        test_process::<TelemetryRleEncoder>(&input, &expect);

        let input: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 18];
        let expect: Vec<u8> = vec![129, 129, 129, 129, 129, 129, 129, 18, 0b0000_0001];
        test_process::<TelemetryRleEncoder>(&input, &expect);

        let input: Vec<u8> = vec![1, 2, 29, 4, 5, 6, 7, 18];
        let expect: Vec<u8> = vec![129, 129, 29, 4, 129, 129, 129, 18, 0b0011_0001];
        test_process::<TelemetryRleEncoder>(&input, &expect);

        let input: Vec<u8> = vec![14, 5, 29, 4, 5, 6, 7, 18];
        let expect: Vec<u8> = vec![14, 119, 29, 4, 129, 129, 129, 18, 0b1011_0001];
        test_process::<TelemetryRleEncoder>(&input, &expect);
    }

    #[test]
    fn test_telemetry_decompression() {
        let expect: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let input: Vec<u8> = vec![129, 129, 129, 129, 129, 129, 129, 129, 0];
        test_process::<TelemetryRleDecoder>(&input, &expect);

        let expect: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
        let input: Vec<u8> = vec![129, 129, 129, 129, 129, 129, 129, 129, 0, 129, 0];
        test_process::<TelemetryRleDecoder>(&input, &expect);

        let expect: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 18];
        let input: Vec<u8> = vec![129, 129, 129, 129, 129, 129, 129, 18, 0b0000_0001];
        test_process::<TelemetryRleDecoder>(&input, &expect);

        let expect: Vec<u8> = vec![1, 2, 29, 4, 5, 6, 7, 18];
        let input: Vec<u8> = vec![129, 129, 29, 4, 129, 129, 129, 18, 0b0011_0001];
        test_process::<TelemetryRleDecoder>(&input, &expect);

        let expect: Vec<u8> = vec![14, 5, 29, 4, 5, 6, 7, 18];
        let input: Vec<u8> = vec![14, 119, 29, 4, 129, 129, 129, 18, 0b1011_0001];
        test_process::<TelemetryRleDecoder>(&input, &expect);
    }

    #[test]
    fn test_roundtrip() {
        roundtrip::<TelemetryRleEncoder, TelemetryRleDecoder>(&vec![1, 2, 3, 4, 5, 6, 7, 8]);
        roundtrip::<TelemetryRleEncoder, TelemetryRleDecoder>(&vec![1, 2, 3, 4, 5, 6, 7, 9]);
        roundtrip::<TelemetryRleEncoder, TelemetryRleDecoder>(&vec![1, 2, 3, 4, 5, 6, 7, 18]);
        roundtrip::<TelemetryRleEncoder, TelemetryRleDecoder>(&vec![1, 2, 29, 4, 5, 6, 7, 18]);
        roundtrip::<TelemetryRleEncoder, TelemetryRleDecoder>(&vec![14, 5, 29, 4, 5, 6, 7, 18]);
    }
}
