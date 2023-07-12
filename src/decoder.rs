/// Interleaved multi-stream rANS decoder interface.
pub trait RansDecoderMulti<const N: usize> {
    /// Type of a Symbol value that can be encoded using this decoder.
    type Symbol: RansDecSymbol;

    /// Gets the cumulative frequency for the current symbol at specified
    /// channel. Note that this does not advance the data position; for
    /// that, use [`Self::advance_at()`].
    ///
    /// # Examples
    /// ```
    /// use rans::byte_decoder::ByteRansDecoderMulti;
    /// use rans::RansDecoderMulti;
    ///
    /// let mut decoder = ByteRansDecoderMulti::<2>::new([2, 0, 0, 1, 0, 0, 0, 1]);
    /// assert_eq!(decoder.get_at(0, 2), 2);
    /// assert_eq!(decoder.get_at(1, 2), 0);
    /// ```
    #[must_use]
    fn get_at(&mut self, channel: usize, scale_bits: u32) -> u32;

    /// Advances the data position after reading a symbol at given channel.
    /// Equivalent to calling [`Self::advance_step_at()`] and
    /// [`Self::renorm_at()`].
    ///
    /// # Examples
    /// ```
    /// use rans::byte_decoder::{ByteRansDecSymbol, ByteRansDecoderMulti};
    /// use rans::{RansDecSymbol, RansDecoderMulti};
    ///
    /// let mut decoder = ByteRansDecoderMulti::<2>::new([2, 0, 0, 1, 0, 0, 0, 1]);
    /// let symbol_1 = ByteRansDecSymbol::new(0, 2);
    /// let symbol_2 = ByteRansDecSymbol::new(2, 2);
    /// assert_eq!(decoder.get_at(0, 2), 2);
    /// decoder.advance_at(0, &symbol_2, 2);
    /// assert_eq!(decoder.get_at(1, 2), 0);
    /// decoder.advance_at(1, &symbol_1, 2);
    /// assert_eq!(decoder.get_at(0, 2), 0);
    /// assert_eq!(decoder.get_at(1, 2), 0);
    /// ```
    fn advance_at(&mut self, channel: usize, symbol: &Self::Symbol, scale_bits: u32);

    /// Pops a single symbol from the internal state, without doing
    /// renormalization or modifying the internal buffer.
    ///
    /// # Examples
    /// ```
    /// use rans::byte_decoder::{ByteRansDecSymbol, ByteRansDecoderMulti};
    /// use rans::{RansDecSymbol, RansDecoderMulti};
    ///
    /// let mut decoder = ByteRansDecoderMulti::<2>::new([2, 0, 0, 1, 0, 0, 0, 1]);
    /// let symbol_1 = ByteRansDecSymbol::new(0, 2);
    /// let symbol_2 = ByteRansDecSymbol::new(2, 2);
    /// assert_eq!(decoder.get_at(0, 2), 2);
    /// decoder.advance_step_at(0, &symbol_2, 2);
    /// assert_eq!(decoder.get_at(1, 2), 0);
    /// decoder.advance_step_at(1, &symbol_1, 2);
    /// decoder.renorm_all();
    /// assert_eq!(decoder.get_at(0, 2), 0);
    /// assert_eq!(decoder.get_at(1, 2), 0);
    /// ```
    fn advance_step_at(&mut self, channel: usize, symbol: &Self::Symbol, scale_bits: u32);

    /// Renormalizes the data in the internal buffer after advancing a symbol.
    ///
    /// # Examples
    /// ```
    /// use rans::byte_decoder::{ByteRansDecSymbol, ByteRansDecoderMulti};
    /// use rans::{RansDecSymbol, RansDecoderMulti};
    ///
    /// let mut decoder = ByteRansDecoderMulti::<2>::new([2, 0, 0, 1, 0, 0, 0, 1]);
    /// let symbol_1 = ByteRansDecSymbol::new(0, 2);
    /// let symbol_2 = ByteRansDecSymbol::new(2, 2);
    /// assert_eq!(decoder.get_at(0, 2), 2);
    /// decoder.advance_step_at(0, &symbol_2, 2);
    /// assert_eq!(decoder.get_at(1, 2), 0);
    /// decoder.advance_step_at(1, &symbol_1, 2);
    /// decoder.renorm_at(0);
    /// decoder.renorm_at(1);
    /// assert_eq!(decoder.get_at(0, 2), 0);
    /// assert_eq!(decoder.get_at(1, 2), 0);
    /// ```
    fn renorm_at(&mut self, channel: usize);

    /// Renormalizes the data in all channels' internal buffers after advancing
    /// a symbol.
    ///
    /// # Examples
    /// ```
    /// use rans::byte_decoder::{ByteRansDecSymbol, ByteRansDecoderMulti};
    /// use rans::{RansDecSymbol, RansDecoderMulti};
    ///
    /// let mut decoder = ByteRansDecoderMulti::<2>::new([2, 0, 0, 1, 0, 0, 0, 1]);
    /// let symbol_1 = ByteRansDecSymbol::new(0, 2);
    /// let symbol_2 = ByteRansDecSymbol::new(2, 2);
    /// assert_eq!(decoder.get_at(0, 2), 2);
    /// decoder.advance_step_at(0, &symbol_2, 2);
    /// assert_eq!(decoder.get_at(1, 2), 0);
    /// decoder.advance_step_at(1, &symbol_1, 2);
    /// decoder.renorm_all();
    /// assert_eq!(decoder.get_at(0, 2), 0);
    /// assert_eq!(decoder.get_at(1, 2), 0);
    /// ```
    fn renorm_all(&mut self) {
        for i in 0..N {
            self.renorm_at(i);
        }
    }
}

/// Single-stream rANS decoder interface.
pub trait RansDecoder: RansDecoderMulti<1> {
    /// Gets the cumulative frequency for the current symbol. Note that this
    /// does not advance the data position; for that, use [`Self::advance()`].
    ///
    /// # Examples
    /// ```
    /// use rans::byte_decoder::{ByteRansDecSymbol, ByteRansDecoder};
    /// use rans::{RansDecSymbol, RansDecoder};
    ///
    /// let mut decoder = ByteRansDecoder::new([2, 0, 0, 2]);
    /// assert_eq!(decoder.get(4), 2);
    /// assert_eq!(decoder.get(4), 2);
    /// ```
    #[must_use]
    fn get(&mut self, scale_bits: u32) -> u32 {
        self.get_at(0, scale_bits)
    }

    /// Advances the data position after reading a symbol.
    ///
    /// # Examples
    /// ```
    /// use rans::byte_decoder::{ByteRansDecSymbol, ByteRansDecoder};
    /// use rans::{RansDecSymbol, RansDecoder};
    ///
    /// let mut decoder = ByteRansDecoder::new([2, 0, 0, 2]);
    /// let symbol = ByteRansDecSymbol::new(2, 2);
    /// assert_eq!(decoder.get(2), 2);
    /// decoder.advance(&symbol, 2);
    /// assert_eq!(decoder.get(2), 0);
    /// ```
    fn advance(&mut self, symbol: &Self::Symbol, scale_bits: u32) {
        self.advance_at(0, symbol, scale_bits);
    }
}

/// A symbol that can be decoded using a rANS decoder.
pub trait RansDecSymbol {
    /// Creates a new rANS decoder symbol instance.
    ///
    /// # Examples
    /// ```
    /// use rans::byte_decoder::ByteRansDecSymbol;
    /// use rans::RansDecSymbol;
    ///
    /// let _symbol = ByteRansDecSymbol::new(0, 2);
    /// ```
    #[must_use]
    fn new(cum_freq: u32, freq: u32) -> Self;

    /// Returns this symbol's cumulative frequency.
    ///
    /// # Examples
    /// ```
    /// use rans::byte_decoder::ByteRansDecSymbol;
    /// use rans::RansDecSymbol;
    ///
    /// let symbol = ByteRansDecSymbol::new(0, 2);
    /// assert_eq!(symbol.cum_freq(), 0);
    /// ```
    #[must_use]
    fn cum_freq(&self) -> u32;

    /// Returns this symbol's frequency.
    ///
    /// # Examples
    /// ```
    /// use rans::byte_decoder::ByteRansDecSymbol;
    /// use rans::RansDecSymbol;
    ///
    /// let symbol = ByteRansDecSymbol::new(0, 2);
    /// assert_eq!(symbol.freq(), 2);
    /// ```
    #[must_use]
    fn freq(&self) -> u32;
}

macro_rules! check_dec_pointer {
    ($self:ident) => {
        debug_assert!($self.is_ptr_valid(), "Data pointer is in an invalid state. Make sure you are not reading more symbols than originally encoded.");
    }
}
pub(crate) use check_dec_pointer;

#[cfg(test)]
pub(crate) mod tests {
    use crate::decoder::RansDecSymbol;
    use crate::{RansDecoder, RansDecoderMulti};

    pub(crate) fn test_decode_empty<T: RansDecoder>(mut decoder: T) {
        assert_eq!(decoder.get(2), 0);
    }

    pub(crate) fn test_decode_two_symbols<T: RansDecoder>(mut decoder: T) {
        let symbol1 = T::Symbol::new(0, 2);
        let symbol2 = T::Symbol::new(2, 2);

        let cum_freq = decoder.get(2);
        assert_eq!(cum_freq, 2);
        decoder.advance(&symbol2, 2);
        let cum_freq = decoder.get(2);
        assert_eq!(cum_freq, 0);
        decoder.advance(&symbol1, 2);
    }

    pub(crate) fn test_decode_symbols_clone<T>(mut decoder: T)
    where
        T: RansDecoder,
        T::Symbol: Clone,
    {
        let symbol1 = T::Symbol::new(0, 2);
        let symbol2 = T::Symbol::new(2, 2);

        let cum_freq = decoder.get(2);
        assert_eq!(cum_freq, 2);
        #[allow(clippy::redundant_clone)]
        decoder.advance(&symbol2.clone(), 2);
        let cum_freq = decoder.get(2);
        assert_eq!(cum_freq, 0);
        #[allow(clippy::redundant_clone)]
        decoder.advance(&symbol1.clone(), 2);
    }

    pub(crate) fn test_decode_more_data<T: RansDecoder>(mut decoder: T) {
        const SCALE_BITS: u32 = 8;
        let s1 = T::Symbol::new(0, 3);
        let s2 = T::Symbol::new(3, 10);
        let s3 = T::Symbol::new(13, 58);
        let s4 = T::Symbol::new(71, 34);
        let s5 = T::Symbol::new(105, 41);
        let s6 = T::Symbol::new(146, 17);
        let s7 = T::Symbol::new(163, 55);
        let s8 = T::Symbol::new(218, 38);
        let symbols = [&s1, &s2, &s3, &s4, &s5, &s6, &s7, &s8];

        let mut symbol_data = [
            &s1, &s2, &s3, &s4, &s5, &s6, &s7, &s8, &s3, &s3, &s3, &s3, &s3, &s5, &s4, &s3, &s4,
            &s3, &s7, &s8, &s8, &s6, &s5, &s3, &s4, &s7, &s6, &s7, &s7, &s3, &s4, &s5,
        ];
        symbol_data.reverse();

        for (i, expected_symbol) in symbol_data.iter().enumerate() {
            let cum_freq = decoder.get(SCALE_BITS);
            let actual_symbol = get_symbol(&symbols, cum_freq);
            assert_eq!(
                actual_symbol.cum_freq(),
                expected_symbol.cum_freq(),
                "Invalid symbol at position {} (decoded cumulative frequency: {})",
                i,
                cum_freq
            );
            decoder.advance(expected_symbol, SCALE_BITS);
        }
    }

    #[must_use]
    fn get_symbol<'a, T: RansDecSymbol>(symbols: &'a [&T], cum_freq: u32) -> &'a T {
        for symbol in symbols {
            if cum_freq < symbol.cum_freq() + symbol.freq() {
                return symbol;
            }
        }
        unreachable!("Invalid symbol frequency");
    }

    pub(crate) fn test_decode_interleaved<T: RansDecoderMulti<2>>(mut decoder: T) {
        const SCALE_BITS: u32 = 4;
        let symbol1 = T::Symbol::new(0, 4);
        let symbol2 = T::Symbol::new(4, 4);
        let symbol3 = T::Symbol::new(8, 4);
        let symbol4 = T::Symbol::new(12, 4);

        assert_eq!(decoder.get_at(0, SCALE_BITS), 12);
        assert_eq!(decoder.get_at(1, SCALE_BITS), 0);
        decoder.advance_step_at(0, &symbol4, SCALE_BITS);
        decoder.advance_step_at(1, &symbol1, SCALE_BITS);
        decoder.renorm_all();
        assert_eq!(decoder.get_at(0, SCALE_BITS), 8);
        assert_eq!(decoder.get_at(1, SCALE_BITS), 0);
        decoder.advance_step_at(0, &symbol3, SCALE_BITS);
        decoder.advance_step_at(1, &symbol1, SCALE_BITS);
        decoder.renorm_all();
        assert_eq!(decoder.get_at(0, SCALE_BITS), 4);
        assert_eq!(decoder.get_at(1, SCALE_BITS), 0);
        decoder.advance_step_at(0, &symbol2, SCALE_BITS);
        decoder.advance_step_at(1, &symbol1, SCALE_BITS);
        decoder.renorm_all();
        assert_eq!(decoder.get_at(0, SCALE_BITS), 0);
        assert_eq!(decoder.get_at(1, SCALE_BITS), 0);
        decoder.advance_step_at(0, &symbol1, SCALE_BITS);
        decoder.advance_step_at(1, &symbol1, SCALE_BITS);
        decoder.renorm_all();
    }
}
