use crate::core::Process;
use crate::stats::ProbTable;
use std::collections::HashMap;
use std::fmt::Display;

mod rlesortedu8;

type CtxProbTable<T> = HashMap<Vec<T>, ProbTable<T>>;

#[derive(Debug)]
pub struct ConditionalRleEncoder {
    order: usize,
    ctx_tables: CtxProbTable<u8>,
    code_table: rlesortedu8::RLEU8,
}

impl Default for ConditionalRleEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for ConditionalRleEncoder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ConditionalRleEncoder {
    /// Create an empty `ConditionalRleEncoder`
    ///
    /// # Examples
    ///
    /// ```
    /// use sqsh::processors::ConditionalRleEncoder;
    ///
    /// let rle = ConditionalRleEncoder::new();
    /// assert!(rle.is_empty());
    /// ```
    pub fn new() -> Self {
        ConditionalRleEncoder {
            ctx_tables: CtxProbTable::<u8>::new(),
            order: 1,
            code_table: rlesortedu8::RLEU8::Bit8,
        }
    }
    /// Create an empty `ConditionalRleEncoder` with fixed bit length
    ///
    /// # Examples
    ///
    /// ```
    /// use sqsh::processors::ConditionalRleEncoder;
    ///
    /// let rle = ConditionalRleEncoder::with_bitlength(3);
    /// assert_eq!(rle.bitlength(), 3);
    /// ```
    pub fn with_bitlength(length: usize) -> Self {
        assert!(length > 0 && length <= 8);
        ConditionalRleEncoder {
            ctx_tables: CtxProbTable::<u8>::new(),
            order: 1,
            code_table: rlesortedu8::RLEU8::with_bitlength(length),
        }
    }
    /// Create an empty `ConditionalRleEncoder` of fixed order
    ///
    /// # Examples
    ///
    /// ```
    /// use sqsh::processors::ConditionalRleEncoder;
    ///
    /// let rle = ConditionalRleEncoder::with_order(2);
    /// assert_eq!(rle.order(), 2);
    /// ```
    pub fn with_order(order: usize) -> Self {
        ConditionalRleEncoder {
            ctx_tables: CtxProbTable::<u8>::new(),
            order,
            code_table: rlesortedu8::RLEU8::Bit8,
        }
    }
    /// Create an empty `ConditionalRleEncoder` of fixed order and defined code length
    ///
    /// # Examples
    ///
    /// ```
    /// use sqsh::processors::ConditionalRleEncoder;
    ///
    /// let rle = ConditionalRleEncoder::with_order_with_bitlength(2, 7);
    /// assert_eq!(rle.order(), 2);
    /// assert_eq!(rle.bitlength(), 7);
    /// ```
    pub fn with_order_with_bitlength(order: usize, length: usize) -> Self {
        assert!(length > 0 && length <= 8);
        ConditionalRleEncoder {
            ctx_tables: CtxProbTable::<u8>::new(),
            order,
            code_table: rlesortedu8::RLEU8::with_bitlength(length),
        }
    }
    /// Return the code length of the `ConditionalRleEncoder`
    ///
    /// # Examples
    ///
    /// ```
    /// use sqsh::processors::ConditionalRleEncoder;
    ///
    /// let rle = ConditionalRleEncoder::with_bitlength(7);
    /// assert_eq!(rle.bitlength(), 7);
    /// ```
    pub fn bitlength(&self) -> usize {
        self.code_table.bitlength()
    }
    /// Return the capacity of the `ConditionalRleEncoder`
    ///
    /// # Examples
    ///
    /// ```
    /// use sqsh::processors::ConditionalRleEncoder;
    ///
    /// let rle = ConditionalRleEncoder::new();
    /// assert_eq!(rle.capacity(), 0);
    /// ```
    pub fn capacity(&self) -> usize {
        self.ctx_tables.capacity()
    }
    /// Return the order of the `ConditionalRleEncoder`
    ///
    /// # Examples
    ///
    /// ```
    /// use sqsh::processors::ConditionalRleEncoder;
    ///
    /// let rle = ConditionalRleEncoder::with_order(7);
    /// assert_eq!(rle.order(), 7);
    /// ```
    pub fn order(&self) -> usize {
        self.order
    }
    /// Return if `ConditionalRleEncoder` is empty
    ///
    /// # Examples
    ///
    /// ```
    /// use sqsh::processors::ConditionalRleEncoder;
    ///
    /// let rle = ConditionalRleEncoder::with_order(7);
    /// assert_eq!(rle.is_empty(), true);
    /// ```
    pub fn is_empty(&self) -> bool {
        self.ctx_tables.is_empty()
    }

    fn encode(&mut self, cx: &[u8], next: u8, sink: &mut Vec<u8>) -> std::io::Result<usize> {
        // TODO: This encoding method is faulty.
        // REASON: If the rank of a value is not
        // found (i.e. symbol was never seen) then currently the symbol itself
        // will be used as the rank. Which leads to the fact that there is a misshapen.
        // If the symbol is small (e.g. 2) and there are >2 seen symbols (e.g. [4:6,1:3,6:3])
        // then the symbol will be encoded like the one at rank 'symbol' e.g. 2 will be encoded
        // like a 6.
        let encoded = self
            .ctx_tables
            .get(cx)
            .and_then(|t| t.rank(&next))
            .and_then(|rank| self.code_table.encode(rank))
            .unwrap_or(&next);
        sink.push(*encoded);
        Ok(1)
    }

    fn single_update(&mut self, cx: &[u8], val: u8) -> std::io::Result<usize> {
        let updated = self.ctx_tables.get_mut(cx).and_then(|t| {
            let v = t.insert(val);
            Some(v)
        });
        match updated {
            Some(_) => Ok(1),
            None => {
                let mut t = ProbTable::<u8>::new();
                let v: Vec<u8> = (0..=u8::MAX).collect();
                t.feed(&v);
                t.insert(val);
                self.ctx_tables.insert(cx.to_vec(), t);
                Ok(1)
            }
        }
    }

    fn full_update(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        println!("Current state of encoder is {:?}", self.ctx_tables);
        println!("Update w/ {:?}", bytes);
        let mut result = 0usize;
        let mut v = Vec::<u8>::new();
        for val in bytes.iter().take(self.order) {
            self.single_update(&v, *val)?;
            v.push(*val);
            result += 1;
        }
        for window in bytes.windows(self.order + 1) {
            let cx = &window[..self.order];
            let val = window[self.order];
            self.single_update(cx, val)?;
            result += 1;
        }
        println!("New state of encoder is {:?}", self.ctx_tables);
        Ok(result)
    }
}

impl Process for ConditionalRleEncoder {
    fn process(&mut self, bytes: &[u8], sink: &mut Vec<u8>) -> std::io::Result<usize> {
        let mut result = 0usize;
        let mut v = Vec::<u8>::new();
        for val in bytes.iter().take(self.order) {
            self.encode(&v, *val, sink)?;
            v.push(*val);
            result += 1;
        }
        for window in bytes.windows(self.order + 1) {
            let cx = &window[..self.order];
            let val = window[self.order];
            self.encode(cx, val, sink)?;
            result += 1;
        }
        self.full_update(&bytes[..result])?;
        Ok(result)
    }

    fn finish(&mut self, _sink: &mut Vec<u8>) -> std::io::Result<usize> {
        Ok(0)
    }
}

#[derive(Debug)]
pub struct ConditionalRleDecoder {
    tables: CtxProbTable<u8>,
    order: usize,
    code: rlesortedu8::RLEU8,
}

impl Default for ConditionalRleDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for ConditionalRleDecoder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ConditionalRleDecoder {
    pub fn new() -> Self {
        ConditionalRleDecoder {
            tables: CtxProbTable::<u8>::new(),
            order: 1,
            code: rlesortedu8::RLEU8::Bit8,
        }
    }
    pub fn with_bitlength(length: usize) -> Self {
        assert!(length > 0 && length <= 8);
        ConditionalRleDecoder {
            tables: CtxProbTable::<u8>::new(),
            order: 1,
            code: rlesortedu8::RLEU8::with_bitlength(length),
        }
    }
    pub fn with_order(order: usize) -> Self {
        ConditionalRleDecoder {
            tables: CtxProbTable::<u8>::new(),
            order,
            code: rlesortedu8::RLEU8::Bit8,
        }
    }
    pub fn with_order_with_bitlength(order: usize, length: usize) -> Self {
        assert!(length > 0 && length <= 8);
        ConditionalRleDecoder {
            tables: CtxProbTable::<u8>::new(),
            order,
            code: rlesortedu8::RLEU8::with_bitlength(length),
        }
    }
    pub fn bitlength(&self) -> usize {
        self.code.bitlength()
    }
    pub fn capacity(&self) -> usize {
        self.tables.capacity()
    }
    pub fn order(&self) -> usize {
        self.order
    }
    pub fn is_empty(&self) -> bool {
        self.tables.is_empty()
    }
    /// Decode a value based on context and write to sink
    ///
    /// 1. Get table, 2. Get ranking, and 3. Get code
    fn decode(&mut self, cx: &[u8], val: u8, sink: &mut Vec<u8>) -> std::io::Result<u8> {
        let decoded_val = self.code.decode(val).unwrap();
        let decoded = self
            .tables
            .get(cx)
            .and_then(|t| t.position(decoded_val))
            .unwrap_or(*self.code.encode(decoded_val).unwrap());
        sink.push(decoded);
        Ok(decoded)
    }
    fn single_update(&mut self, cx: &[u8], val: u8) -> std::io::Result<usize> {
        let updated = self.tables.get_mut(cx).and_then(|t| {
            let v = t.insert(val);
            Some(v)
        });
        match updated {
            Some(_) => Ok(1),
            None => {
                let mut t = ProbTable::<u8>::new();
                let v: Vec<u8> = (0..=u8::MAX).collect();
                t.feed(&v);
                t.insert(val);
                self.tables.insert(cx.to_vec(), t);
                Ok(1)
            }
        }
    }

    fn full_update(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        println!("Current state of decoder is {:?}", self.tables);
        println!("Update w/ {:?}", bytes);
        let mut result = 0usize;
        let mut v = Vec::<u8>::new();
        for val in bytes.iter().take(self.order) {
            self.single_update(&v, *val)?;
            v.push(*val);
            result += 1;
        }
        for window in bytes.windows(self.order + 1) {
            let cx = &window[..self.order];
            let val = window[self.order];
            self.single_update(cx, val)?;
            result += 1;
        }
        println!("New state of decoder is {:?}", self.tables);
        Ok(result)
    }
}

impl Process for ConditionalRleDecoder {
    fn process(&mut self, byte: &[u8], sink: &mut Vec<u8>) -> std::io::Result<usize> {
        let mut result = 0usize;
        let mut update_vector = Vec::<u8>::new();
        let mut v = Vec::<u8>::new();
        for val in byte.iter().take(self.order) {
            let decoded = self.decode(&v, *val, sink)?;
            update_vector.push(decoded);
            v.push(decoded);
            result += 1;
        }
        for val in byte.iter().skip(self.order) {
            let decoded = self.decode(&v, *val, sink)?;
            update_vector.push(decoded);
            if self.order > 0 {
                v.rotate_left(1);
                v[self.order - 1] = decoded;
            }
            result += 1;
        }
        self.full_update(&update_vector)?;
        Ok(result)
    }

    fn finish(&mut self, _sink: &mut Vec<u8>) -> std::io::Result<usize> {
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let enc = ConditionalRleEncoder::new();
        assert_eq!(enc.order, 1);
        assert!(enc.ctx_tables.is_empty());
    }

    #[test]
    fn new_with_high_order() {
        let order = 0;
        let enc = ConditionalRleEncoder::with_order(order);
        assert_eq!(enc.order, order);
        assert!(enc.ctx_tables.is_empty());
    }

    #[test]
    fn encoding_easy_process() {
        let order = 4;
        let mut enc = ConditionalRleEncoder::with_order(order);
        let data = vec![2u8, 2, 2, 2, 2, 2, 2, 2];

        // Encode once
        let mut encoded = Vec::<u8>::new();
        enc.process(&data, &mut encoded).unwrap();
        let mut expected = vec![2u8, 2, 2, 2, 2, 2, 2, 2];
        assert_eq!(expected, encoded);

        // Encode twice
        enc.process(&data, &mut encoded).unwrap();
        expected.append(&mut vec![0u8, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(expected, encoded);
    }

    #[test]
    fn encoder() {
        let order = 0;
        let mut enc = ConditionalRleEncoder::with_order(order);
        let source: Vec<u8> = vec![3, 4, 3, 3, 4, 5, 5, 5, 7, 7, 7, 7, 7, 7, 7, 2, 1];
        let mut sink: Vec<u8> = Vec::new();
        enc.process(&source, &mut sink).unwrap();

        assert_eq!(source, sink);
    }

    #[test]
    fn roundtrip_single_encode() {
        // Roundtrip with a single encoding process
        for order in 0..5 {
            let source: Vec<u8> = vec![3, 4, 3, 3, 4, 5, 5, 5, 7, 7, 7, 7, 7, 7, 7, 2, 1];
            let mut enc = ConditionalRleEncoder::with_order(order);
            let mut encoded: Vec<u8> = Vec::new();
            enc.process(&source, &mut encoded).unwrap();

            let mut decoded: Vec<u8> = Vec::new();
            let mut dec = ConditionalRleDecoder::with_order(order);
            dec.process(&encoded, &mut decoded).unwrap();

            println!("{:?}", order);
            assert_eq!(source, decoded);
        }
    }
    #[test]
    fn roundtrip_multi_encode() {
        // Roundtrip with a multiple encoding process
        for order in 0..5 {
            let source: Vec<u8> = vec![3, 4, 3, 3, 4, 5, 5, 5, 7, 7, 7, 7, 7, 7, 7, 2, 1];
            let split = 10usize;

            let mut encoded: Vec<u8> = Vec::new();
            let mut enc = ConditionalRleEncoder::with_order(order);
            enc.process(&source[..split], &mut encoded).unwrap();
            println!("Encoding 1: {:?}", encoded);
            enc.process(&source[split..], &mut encoded).unwrap();
            println!("Encoding 2: {:?}", encoded);

            let mut decoded: Vec<u8> = Vec::new();
            let mut dec = ConditionalRleDecoder::with_order(order);
            dec.process(&encoded[..split], &mut decoded).unwrap();
            println!("Decoding 1: {:?}", decoded);
            dec.process(&encoded[split..], &mut decoded).unwrap();
            println!("Decoding 2: {:?}", decoded);

            println!("Error w/ order: {:?}", order);
            assert_eq!(source, decoded);
        }
    }
}
