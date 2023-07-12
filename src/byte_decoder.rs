use std::mem::MaybeUninit;

use crate::decoder::check_dec_pointer;
use crate::mut_cow::MutCow;
use crate::{RansDecSymbol, RansDecoder, RansDecoderMulti};

/// Multi-stream interleaved rANS decoder - byte-aligned version.
#[derive(Debug)]
pub struct ByteRansDecoderMulti<'a, const N: usize> {
    states: [ryg_rans_sys::rans_byte::RansState; N],
    data: MutCow<'a, [u8]>,
    ptr: *mut u8,
}

/// Single-stream rANS decoder - byte-aligned version.
pub type ByteRansDecoder<'a> = ByteRansDecoderMulti<'a, 1>;

impl<'a, const N: usize> ByteRansDecoderMulti<'a, N> {
    /// Creates a new `ByteMultiRansDecoder` instance with given `data`.
    ///
    /// Note that most of the API is inside the [`RansDecoderMulti`] trait, so
    /// you probably want to `use rans::RansDecoderMulti`.
    ///
    /// # Examples
    /// ```
    /// use rans::byte_decoder::ByteRansDecoderMulti;
    /// use rans::RansDecoderMulti;
    ///
    /// let mut decoder = ByteRansDecoderMulti::<1>::new(vec![0]);
    /// assert_eq!(decoder.get_at(0, 4), 0);
    /// ```
    #[must_use]
    pub fn new<T: Into<MutCow<'a, [u8]>>>(data: T) -> Self {
        let mut data = data.into();
        assert!(!data.is_empty());

        unsafe {
            let mut ptr = data.as_mut_ptr();

            #[allow(clippy::uninit_assumed_init)]
            let mut states: [ryg_rans_sys::rans_byte::RansState; N] =
                MaybeUninit::uninit().assume_init();
            for state in &mut states {
                ryg_rans_sys::rans_byte::rans_dec_init(state, &mut ptr);
            }

            Self { states, data, ptr }
        }
    }

    #[inline]
    fn is_ptr_valid(&self) -> bool {
        let range = self.data.as_ptr_range();
        let range_inclusive = range.start..=range.end;
        range_inclusive.contains(&(self.ptr as *const u8))
    }
}

impl<'a, const N: usize> RansDecoderMulti<N> for ByteRansDecoderMulti<'a, N> {
    type Symbol = ByteRansDecSymbol;

    #[inline]
    fn get_at(&mut self, channel: usize, scale_bits: u32) -> u32 {
        debug_assert!(channel <= N);

        unsafe { ryg_rans_sys::rans_byte::rans_dec_get(&mut self.states[channel], scale_bits) }
    }

    #[inline]
    fn advance_at(&mut self, channel: usize, symbol: &Self::Symbol, scale_bits: u32) {
        debug_assert!(channel <= N);

        unsafe {
            ryg_rans_sys::rans_byte::rans_dec_advance_symbol(
                &mut self.states[channel],
                &mut self.ptr,
                &symbol.symbol,
                scale_bits,
            );
        }

        check_dec_pointer!(self);
    }

    #[inline]
    fn advance_step_at(&mut self, channel: usize, symbol: &Self::Symbol, scale_bits: u32) {
        debug_assert!(channel <= N);

        unsafe {
            ryg_rans_sys::rans_byte::rans_dec_advance_symbol_step(
                &mut self.states[channel],
                &symbol.symbol,
                scale_bits,
            );
        }
    }

    #[inline]
    fn renorm_at(&mut self, channel: usize) {
        debug_assert!(channel <= N);

        unsafe {
            ryg_rans_sys::rans_byte::rans_dec_renorm(&mut self.states[channel], &mut self.ptr);
        }

        check_dec_pointer!(self);
    }
}

impl<'a> RansDecoder for ByteRansDecoderMulti<'a, 1> {}

/// rANS decoder symbol - byte-aligned version.
#[derive(Debug, Clone)]
pub struct ByteRansDecSymbol {
    symbol: ryg_rans_sys::rans_byte::RansDecSymbol,
}

impl RansDecSymbol for ByteRansDecSymbol {
    #[inline]
    fn new(cum_freq: u32, freq: u32) -> Self {
        unsafe {
            let mut symbol = MaybeUninit::uninit();
            ryg_rans_sys::rans_byte::rans_dec_symbol_init(symbol.as_mut_ptr(), cum_freq, freq);

            Self {
                symbol: symbol.assume_init(),
            }
        }
    }

    #[inline]
    fn cum_freq(&self) -> u32 {
        self.symbol.start as u32
    }

    #[inline]
    fn freq(&self) -> u32 {
        self.symbol.freq as u32
    }
}

#[cfg(test)]
mod tests {
    use crate::byte_decoder::{ByteRansDecoder, ByteRansDecoderMulti};
    use crate::decoder::tests as dec_tests;

    #[test]
    fn test_decode_empty() {
        let decoder = ByteRansDecoder::new([0, 0, 128, 0]);

        dec_tests::test_decode_empty(decoder);
    }

    #[test]
    fn test_decode_two_symbols() {
        let decoder = ByteRansDecoder::new([2, 0, 0, 2]);

        dec_tests::test_decode_two_symbols(decoder);
    }

    #[test]
    fn test_decode_symbols_clone() {
        let decoder = ByteRansDecoder::new([2, 0, 0, 2]);

        dec_tests::test_decode_symbols_clone(decoder);
    }

    #[test]
    fn test_decode_more_data() {
        let mut data = [
            106, 184, 212, 0, 84, 205, 93, 162, 171, 34, 28, 50, 161, 66, 2,
        ];
        let decoder = ByteRansDecoder::new(&mut data);

        dec_tests::test_decode_more_data(decoder);
    }

    #[test]
    fn test_decode_interleaved() {
        let mut data = [12, 0, 128, 0, 0, 0, 128, 0, 24, 0];
        let decoder = ByteRansDecoderMulti::<2>::new(&mut data);

        dec_tests::test_decode_interleaved(decoder);
    }
}
