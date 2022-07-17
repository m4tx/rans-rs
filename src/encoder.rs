/// Interleaved multi-stream rANS encoder interface.
pub trait RansEncoderMulti<const N: usize> {
    /// Type of a Symbol value that can be encoded using this encoder.
    type Symbol: RansEncSymbol;

    /// Resets this encoder's internal state, so that the internal buffer
    /// becomes empty.
    ///
    /// # Examples
    /// ```
    /// use rans::byte_encoder::{ByteRansEncSymbol, ByteRansEncoderMulti};
    /// use rans::{RansEncSymbol, RansEncoderMulti};
    ///
    /// let mut encoder = ByteRansEncoderMulti::<2>::new(1024);
    /// let symbol = ByteRansEncSymbol::new(0, 1, 4);
    /// encoder.put_at(0, &symbol);
    /// encoder.flush_all();
    /// encoder.reset();
    /// assert_eq!(encoder.data(), []);
    /// ```
    fn reset(&mut self);

    /// Puts a symbol into the specified channel.
    ///
    /// # Examples
    /// ```
    /// use rans::byte_encoder::{ByteRansEncSymbol, ByteRansEncoderMulti};
    /// use rans::{RansEncSymbol, RansEncoderMulti};
    ///
    /// let mut encoder = ByteRansEncoderMulti::<2>::new(1024);
    /// let symbol = ByteRansEncSymbol::new(0, 1, 4);
    /// encoder.put_at(0, &symbol);
    /// encoder.put_at(1, &symbol);
    /// encoder.flush_all();
    /// assert_eq!(encoder.data(), [0, 0, 0, 8, 0, 0, 0, 8]);
    /// ```
    fn put_at(&mut self, channel: usize, symbol: &Self::Symbol);

    /// Flushes the encoder's intermediate data at given channel into the
    /// buffer.
    ///
    /// # Examples
    /// ```
    /// use rans::byte_encoder::ByteRansEncoderMulti;
    /// use rans::RansEncoderMulti;
    ///
    /// let mut encoder = ByteRansEncoderMulti::<2>::new(1024);
    /// assert_eq!(encoder.data(), []);
    /// encoder.flush_at(0);
    /// assert_eq!(encoder.data(), [0, 0, 128, 0]);
    /// ```
    fn flush_at(&mut self, channel: usize);

    /// Flushes the encoder's intermediate data at all channels into the buffer.
    ///
    /// # Examples
    /// ```
    /// use rans::byte_encoder::ByteRansEncoderMulti;
    /// use rans::RansEncoderMulti;
    ///
    /// let mut encoder = ByteRansEncoderMulti::<2>::new(1024);
    /// assert_eq!(encoder.data(), []);
    /// encoder.flush_all();
    /// assert_eq!(encoder.data(), [0, 0, 128, 0, 0, 0, 128, 0]);
    /// ```
    fn flush_all(&mut self) {
        for i in 0..N {
            self.flush_at(i);
        }
    }

    /// Returns this encoder's internal buffer content.
    ///
    /// # Examples
    /// ```
    /// use rans::byte_encoder::ByteRansEncoderMulti;
    /// use rans::RansEncoderMulti;
    ///
    /// let encoder = ByteRansEncoderMulti::<2>::new(1024);
    /// assert_eq!(encoder.data(), []);
    /// ```
    #[must_use]
    fn data(&self) -> &[u8];

    /// Returns this encoder's internal buffer current length in bytes.
    ///
    /// # Examples
    /// ```
    /// use rans::byte_encoder::ByteRansEncoderMulti;
    /// use rans::RansEncoderMulti;
    ///
    /// let encoder = ByteRansEncoderMulti::<2>::new(1024);
    /// assert_eq!(encoder.len(), 0);
    /// ```
    #[must_use]
    #[inline]
    fn len(&self) -> usize {
        self.data().len()
    }

    /// Returns whether this encoder's internal buffer is empty.
    ///
    /// # Examples
    /// ```
    /// use rans::byte_encoder::ByteRansEncoderMulti;
    /// use rans::RansEncoderMulti;
    ///
    /// let encoder = ByteRansEncoderMulti::<2>::new(1024);
    /// assert!(encoder.is_empty());
    /// ```
    #[must_use]
    #[inline]
    fn is_empty(&self) -> bool {
        self.data().is_empty()
    }
}

/// Single-stream rANS encoder interface.
pub trait RansEncoder: RansEncoderMulti<1> {
    /// Puts the specified symbol into this encoder.
    ///
    /// # Examples
    /// ```
    /// use rans::byte_encoder::{ByteRansEncSymbol, ByteRansEncoder};
    /// use rans::{RansEncSymbol, RansEncoder, RansEncoderMulti};
    ///
    /// let mut encoder = ByteRansEncoder::new(1024);
    /// let symbol = ByteRansEncSymbol::new(0, 1, 4);
    /// encoder.put(&symbol);
    /// encoder.flush();
    /// assert_eq!(encoder.data(), [0, 0, 0, 8]);
    /// ```
    fn put(&mut self, symbol: &Self::Symbol) {
        self.put_at(0, symbol);
    }

    /// Flushes the encoder's intermediate data into the buffer.
    ///
    /// # Examples
    /// ```
    /// use rans::byte_encoder::ByteRansEncoder;
    /// use rans::{RansEncoder, RansEncoderMulti};
    ///
    /// let mut encoder = ByteRansEncoder::new(1024);
    /// assert_eq!(encoder.data(), []);
    /// encoder.flush();
    /// assert_eq!(encoder.data(), [0, 0, 128, 0]);
    /// ```
    fn flush(&mut self) {
        self.flush_at(0);
    }
}

/// A symbol that can be encoded using a rANS encoder.
pub trait RansEncSymbol {
    /// Creates a new rANS encoder symbol instance.
    ///
    /// # Examples
    /// ```
    /// use rans::byte_encoder::ByteRansEncSymbol;
    /// use rans::RansEncSymbol;
    ///
    /// let _symbol = ByteRansEncSymbol::new(0, 1, 4);
    /// ```
    #[must_use]
    fn new(cum_freq: u32, freq: u32, scale_bits: u32) -> Self;
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::encoder::{RansEncSymbol, RansEncoder, RansEncoderMulti};

    pub(crate) fn test_encode_nothing<T: RansEncoder>(encoder: T) {
        assert!(encoder.data().is_empty());
    }

    pub(crate) fn test_encode_empty_data<T: RansEncoder>(mut encoder: T, data: &[u8]) {
        encoder.flush();

        assert_eq!(encoder.data(), data);
    }

    pub(crate) fn test_encode_two_symbols<T: RansEncoder>(mut encoder: T, data: &[u8]) {
        const SCALE_BITS: u32 = 2;
        let symbol1 = T::Symbol::new(0, 2, SCALE_BITS);
        let symbol2 = T::Symbol::new(2, 2, SCALE_BITS);

        encoder.put(&symbol1);
        encoder.put(&symbol2);
        encoder.flush();

        assert_eq!(encoder.data(), data);
    }

    pub(crate) fn test_encode_symbols_clone<T>(mut encoder: T, data: &[u8])
    where
        T: RansEncoder,
        T::Symbol: Clone,
    {
        const SCALE_BITS: u32 = 2;
        let symbol1 = T::Symbol::new(0, 2, SCALE_BITS);
        let symbol2 = T::Symbol::new(2, 2, SCALE_BITS);

        #[allow(clippy::redundant_clone)]
        encoder.put(&symbol1.clone());
        #[allow(clippy::redundant_clone)]
        encoder.put(&symbol2.clone());
        encoder.flush();

        assert_eq!(encoder.data(), data);
    }

    pub(crate) fn test_encode_and_reset<T: RansEncoder>(
        mut encoder: T,
        data1: &[u8],
        data2: &[u8],
    ) {
        const SCALE_BITS: u32 = 2;
        let symbol1 = T::Symbol::new(0, 2, SCALE_BITS);
        let symbol2 = T::Symbol::new(2, 2, SCALE_BITS);

        encoder.put(&symbol1);
        encoder.flush();
        assert_eq!(encoder.data(), data1);

        encoder.reset();

        encoder.put(&symbol2);
        encoder.flush();
        assert_eq!(encoder.data(), data2);
    }

    pub(crate) fn test_encode_more_data<T: RansEncoder>(mut encoder: T, data: &[u8]) {
        const SCALE_BITS: u32 = 8;
        let s1 = T::Symbol::new(0, 3, SCALE_BITS);
        let s2 = T::Symbol::new(3, 10, SCALE_BITS);
        let s3 = T::Symbol::new(13, 58, SCALE_BITS);
        let s4 = T::Symbol::new(71, 34, SCALE_BITS);
        let s5 = T::Symbol::new(105, 41, SCALE_BITS);
        let s6 = T::Symbol::new(146, 17, SCALE_BITS);
        let s7 = T::Symbol::new(163, 55, SCALE_BITS);
        let s8 = T::Symbol::new(218, 38, SCALE_BITS);

        let symbols = [
            &s1, &s2, &s3, &s4, &s5, &s6, &s7, &s8, &s3, &s3, &s3, &s3, &s3, &s5, &s4, &s3, &s4,
            &s3, &s7, &s8, &s8, &s6, &s5, &s3, &s4, &s7, &s6, &s7, &s7, &s3, &s4, &s5,
        ];
        for symbol in symbols {
            encoder.put(symbol);
        }
        encoder.flush();

        assert_eq!(encoder.data(), data);
    }

    pub(crate) fn encode_interleaved<T: RansEncoderMulti<2>>(mut encoder: T, data: &[u8]) {
        const SCALE_BITS: u32 = 4;
        let symbol1 = T::Symbol::new(0, 4, SCALE_BITS);
        let symbol2 = T::Symbol::new(4, 4, SCALE_BITS);
        let symbol3 = T::Symbol::new(8, 4, SCALE_BITS);
        let symbol4 = T::Symbol::new(12, 4, SCALE_BITS);

        encoder.put_at(0, &symbol1);
        encoder.put_at(1, &symbol1);
        encoder.put_at(0, &symbol1);
        encoder.put_at(1, &symbol2);
        encoder.put_at(0, &symbol1);
        encoder.put_at(1, &symbol3);
        encoder.put_at(0, &symbol1);
        encoder.put_at(1, &symbol4);
        encoder.flush_all();

        assert_eq!(encoder.data(), data);
    }
}
