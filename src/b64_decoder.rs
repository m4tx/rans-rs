use std::mem::MaybeUninit;

use crate::decoder::check_dec_pointer;
use crate::mut_cow::MutCow;
use crate::{RansDecSymbol, RansDecoder, RansDecoderMulti};

/// Multi-stream interleaved rANS decoder - 64-bit version.
#[derive(Debug)]
pub struct B64RansDecoderMulti<'a, const N: usize> {
    states: [ryg_rans_sys::rans_64::Rans64State; N],
    data: MutCow<'a, [u8]>,
    ptr: *mut u32,
}

/// Single-stream rANS decoder - 64-bit version.
pub type B64RansDecoder<'a> = B64RansDecoderMulti<'a, 1>;

impl<'a, const N: usize> B64RansDecoderMulti<'a, N> {
    /// Creates a new `B64MultiRansDecoder` instance with given `data`.
    ///
    /// Note that most of the API is inside the [`RansDecoderMulti`] trait, so
    /// you probably want to `use rans::RansDecoderMulti`.
    ///
    /// # Examples
    /// ```
    /// use rans::b64_decoder::B64RansDecoderMulti;
    /// use rans::RansDecoderMulti;
    ///
    /// let mut decoder = B64RansDecoderMulti::<1>::new(vec![0]);
    /// assert_eq!(decoder.get_at(0, 4), 0);
    /// ```
    #[must_use]
    pub fn new<T: Into<MutCow<'a, [u8]>>>(data: T) -> Self {
        let mut data = data.into();
        assert!(!data.is_empty());

        unsafe {
            let mut ptr = data.as_mut_ptr() as *mut u32;

            #[allow(clippy::uninit_assumed_init)]
            let mut states: [ryg_rans_sys::rans_64::Rans64State; N] =
                MaybeUninit::uninit().assume_init();
            for state in &mut states {
                ryg_rans_sys::rans_64::rans_64_dec_init(state, &mut ptr);
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

impl<const N: usize> RansDecoderMulti<N> for B64RansDecoderMulti<'_, N> {
    type Symbol = B64RansDecSymbol;

    #[inline]
    fn get_at(&mut self, channel: usize, scale_bits: u32) -> u32 {
        debug_assert!(channel <= N);

        unsafe { ryg_rans_sys::rans_64::rans_64_dec_get(&mut self.states[channel], scale_bits) }
    }

    #[inline]
    fn advance_at(&mut self, channel: usize, symbol: &Self::Symbol, scale_bits: u32) {
        debug_assert!(channel <= N);

        unsafe {
            ryg_rans_sys::rans_64::rans_64_dec_advance_symbol(
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
            ryg_rans_sys::rans_64::rans_64_dec_advance_symbol_step(
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
            ryg_rans_sys::rans_64::rans_64_dec_renorm(&mut self.states[channel], &mut self.ptr);
        }

        check_dec_pointer!(self);
    }
}

impl RansDecoder for B64RansDecoderMulti<'_, 1> {}

/// rANS decoder symbol - 64-bit version.
#[derive(Debug, Clone)]
pub struct B64RansDecSymbol {
    symbol: ryg_rans_sys::rans_64::Rans64DecSymbol,
}

impl RansDecSymbol for B64RansDecSymbol {
    #[inline]
    fn new(cum_freq: u32, freq: u32) -> Self {
        unsafe {
            let mut symbol = MaybeUninit::uninit();
            ryg_rans_sys::rans_64::rans_64_dec_symbol_init(symbol.as_mut_ptr(), cum_freq, freq);

            Self {
                symbol: symbol.assume_init(),
            }
        }
    }

    #[inline]
    fn cum_freq(&self) -> u32 {
        self.symbol.start
    }

    #[inline]
    fn freq(&self) -> u32 {
        self.symbol.freq
    }
}

#[cfg(test)]
mod tests {
    use crate::b64_decoder::{B64RansDecoder, B64RansDecoderMulti};
    use crate::decoder::tests as dec_tests;

    #[test]
    fn test_decode_empty() {
        let decoder = B64RansDecoder::new([0, 0, 0, 128, 0, 0, 0, 0]);

        dec_tests::test_decode_empty(decoder);
    }

    #[test]
    fn test_decode_two_symbols() {
        let decoder = B64RansDecoder::new([2, 0, 0, 0, 2, 0, 0, 0]);

        dec_tests::test_decode_two_symbols(decoder);
    }

    #[test]
    fn test_decode_symbols_clone() {
        let decoder = B64RansDecoder::new([2, 0, 0, 0, 2, 0, 0, 0]);

        dec_tests::test_decode_symbols_clone(decoder);
    }

    #[test]
    fn test_decode_more_data() {
        let mut data = [
            122, 27, 118, 146, 40, 184, 212, 0, 147, 60, 144, 230, 24, 137, 205, 128,
        ];
        let decoder = B64RansDecoder::new(data);

        dec_tests::test_decode_more_data(decoder);
    }

    #[test]
    fn test_decode_interleaved() {
        let mut data = [108, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 128, 0, 0, 0];
        let decoder = B64RansDecoderMulti::<2>::new(data);

        dec_tests::test_decode_interleaved(decoder);
    }

    #[test]
    fn test_has_debug_output() {
        let decoder = B64RansDecoder::new([0, 0, 0, 128, 0, 0, 0, 0]);
        dec_tests::test_has_debug_output(decoder);
    }
}
