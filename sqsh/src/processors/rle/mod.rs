//! # Run-Length Encoding
//!
//! This module implements Run-Length Encoding (RLE) in different modes.
//! In RLE the byte stream is compressed by coding consecutive bytes of the same
//! value together. This is usually done by writing the byte followed by a
//! repetition count. To differentiate between a value and count byte a special
//! character is introduced. This character indicates that the next character
//! is a count. Therefore, a repetition is replaced by three characters: one
//! for the character being replaced, the special character and the count.
//!
//! One challenge is to decide what character to use as the special character
//! indicator. Since every character can itself occur in the source. The
//! Microcom Network Protocol 5 (MNP5) algorithm solved this issue by not using
//! a dedicated special character but repeating the original value. If two
//! consecutive bytes are coded, then the next byte will be the count eg.
//! `aaa` -> `aa1` where the count represents the number of occurences
//! additionally to the two repetitions in the count.
//!
//! The next challenge is that a repetition of two bytes is replaced by
//! three bytes which results in an expansion. Since double repetitions are
//! more common then triple or more, the MNP5 algorithm avoids this expension by
//! using RLE for repetitions of more than three.
//!
//! ## Compression Factor calculation
//! The Compression Factor (CF) for a RLE algorithm can be calculated using
//! the following formula: `N / (N-M(L-k))`
//! where N represents the number of characters in the source, M the number
//! of repetitions, L the average length of the repetitions and k the number
//! of characters used by the RLE algorithm for replacment.
//!
//! > Hint: Instead of encoding the information bytes and actual source together, there
//! > is always the option to split up the encoding in two streams and process
//! > them in the latter steps separately. This might enable a better CF at the end.
//! > The most common way though is to encode the information bytes within the
//! > encoded stream
//!
//! ## Special forms of RLE
//!
//! There are special modus operandi for RLE which change their behaviour based
//! on the data at hand.
//!
//! - [Digram RLE](#digram-encoding)
//! - [Pattern Substition RLE](#pattern-substitution)
//! - [Relative RLE](#relative-encoding)
//!     - [Single Relative RLE](#single-encoding)
//!     - [Pair Relative RLE](#pair-encoding)
//! - [Lossy RLE](#lossy-rle)
//! - [Conditional RLE](#conditionalcontext-based-rle)
//!
//! ### Digram encoding
//!
//! Should the replacement of characters not be related to single bytes,
//! one specs of ngram encoding eg. digram encoding for two characters. Here,
//! instead of single bytes, pairs of bytes are replaced by the RLE.
//!
//! ### Pattern substitution
//!
//! If the source has a clear cut vocabulary than it can be done that each word
//! is replaced by a control character. Should there be even more such words,
//! then the same principle as above, a special character followed by the control
//! character, can be used.
//!
//! ### Relative Encoding
//!
//! Sometimes also called *difference encoding*. If the source represents numerical
//! data such telemetry or measurement, then these data can be encoded using
//! a stream of differences. Differences can be positive or negative.
//! Should the difference be large, the actual value is send. But now, one needs
//! to diffenentiate between a difference and an actual data point. This is often
//! down by sending extra bytes containing the information about the type of
//! the next bytes in binary, where 1 represents a difference and 0 an actual value.
//! There are two methods: single or pair encoding.
//!
//! #### Single Encoding
//!
//! Here, the information byte is send before or after a block of eight bytes.
//! Therefore increasing the number of bytes by 1/8 of N and hoping to increase
//! CF by better compression of the whole bytes.
//!
//! #### Pair Encoding
//!
//! Here, pairs of values are being build, such that either a 16 bit value is send
//! to the sink or a pair of differences with eigth bit resolution. Similar to the
//! above method, after 16 bytes, a information byte is being send to the sink.
//! This increases the source stream by 1/16 of N and allows for higher resolution
//! values. An actual value can be between 0 - 32k, where a difference can be between
//! [-128, 128]. Should there be no need for a second difference value, a special
//! byte value (like `0b1111_1111`) can be defined, such that the decoder
//! acknowledges the missing difference.
//!
//! ### Lossy RLE
//!
//! A special lossy version of RLE can be implemented where runs of short lengths
//! are merged with neighbouring runs. The easiest implementation would merge these
//! bytes with the following byte. A more sophisticated algorithm could consider
//! the latter and former byte and extend the one with the smallest difference. For
//! the sophisticated approach the writing of the bytes need to be delayed until
//! it is clear where the short runs could be merged.
//!
//! ### Conditional/Context-based RLE
//!
//! A conditional RLE maps the whole source stream to different values that
//! are then easier to RLE. For this initial mapping, the most common bytes
//! will be replaced by bytes with few runs. The mapping is generated using an
//! Variable-Order Markov (VOM) model. The VOM model defines the probability `P` of
//! a value `x` given some condition `c`:  `P( x | c )`. Here, the condition `c`
//! can either consist of a single state (first order VOM), two (second order VOM)
//! or more states (n-order VOM).
//!
//! Given four bit values these are sorted to
//! following groups:
//!
//! - Group 1: `0b0000`, `0b1111`
//! - Group 2: `0b0001`, `0b0011`, `0b0111`, `0b1110`, `0b1100`, `0b1000`
//! - Group 3: `0b0100`, `0b0010`, `0b0110`, `0b1011`, `0b1101`, `0b1001`
//! - Group 4: `0b0101`, `0b1010`
//!
//! where group *i* consists of *i* runs. The groups are build in such a way that
//! the second half of the group is the complement of the first half eg.
//! `0b0001` -> `0b1110` or `0b1010` -> `0b0101`.
//!
//! The mapping is handled in two steps. The VOM returns for each condition the
//! probability distribution. First, the value with the highest probability will get the
//! symbol mapped with the least runs (Group 1, Group 2, ...). After the mapping
//! is selected the previous mapping is looked at. If the least-significant bit (LSB)
//! is a `1` than the complement of the code is choosen. Therefore, decreasing the
//! number of runs ie. increasing the length of the last run.
mod classic;
mod lossy;
mod telemetry;

pub use classic::{RleClassicDecoder, RleClassicEncoder};
pub use lossy::{LossyRleDecoder, LossyRleEncoder};
pub use telemetry::{TelemetryRleDecoder, TelemetryRleEncoder};

#[cfg(test)]
mod tests {

    pub(crate) fn as_rle_bytes(word: &str) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        let mut tmp = 0u8;
        let mut tmp_filled = false;

        for val in word.as_bytes() {
            if *val < 48 || *val > 57 {
                // val is not a number
                if tmp_filled {
                    result.push(tmp);
                    tmp = 0;
                    tmp_filled = false;
                }
                result.push(*val);
            } else {
                // val is a number
                if tmp_filled {
                    tmp *= 10;
                    tmp += val - 48;
                } else {
                    tmp_filled = true;
                    tmp += val - 48;
                }
            }
        }
        if tmp_filled {
            result.push(tmp)
        }
        result
    }
}
