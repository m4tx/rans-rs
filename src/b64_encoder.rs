use std::mem::MaybeUninit;
use std::slice;

use crate::encoder::{check_enc_pointer, RansEncSymbol, RansEncoder, RansEncoderMulti};

/// Multi-stream interleaved rANS encoder - 64-bit version.
#[derive(Debug)]
pub struct B64RansEncoderMulti<const N: usize> {
    states: [ryg_rans_sys::rans_64::Rans64State; N],
    dst: Vec<u32>,
    ptr: *mut u32,
}

/// Single-stream rANS encoder - 64-bit version.
pub type B64RansEncoder = B64RansEncoderMulti<1>;

impl<const N: usize> B64RansEncoderMulti<N> {
    /// Creates a new `B64MultiRansEncoder` instance that can contain `max_len`
    /// bytes in the internal buffer.
    ///
    /// Note that most of the API is inside the [`RansEncoderMulti`] trait, so
    /// you probably want to `use rans::RansEncoderMulti`.
    ///
    /// # Examples
    /// ```
    /// use rans::b64_encoder::B64RansEncoderMulti;
    /// use rans::RansEncoderMulti;
    ///
    /// let encoder = B64RansEncoderMulti::<2>::new(1024);
    /// assert_eq!(encoder.data(), []);
    /// ```
    #[must_use]
    pub fn new(max_len: usize) -> Self {
        debug_assert!(N > 0);

        unsafe {
            #[allow(clippy::uninit_assumed_init)]
            let states: [ryg_rans_sys::rans_64::Rans64State; N] =
                MaybeUninit::uninit().assume_init();

            let dst = vec![0; max_len / 4];

            let mut encoder = Self {
                states,
                dst,
                ptr: std::ptr::null_mut(),
            };
            encoder.reset();
            encoder
        }
    }

    #[inline]
    fn is_ptr_valid(&self) -> bool {
        let range = self.dst.as_ptr_range();
        let range_inclusive = range.start..=range.end;
        range_inclusive.contains(&(self.ptr as *const u32))
    }
}

impl<const N: usize> RansEncoderMulti<N> for B64RansEncoderMulti<N> {
    type Symbol = B64RansEncSymbol;

    fn reset(&mut self) {
        unsafe {
            for state in &mut self.states {
                ryg_rans_sys::rans_64::rans_64_enc_init(state);
            }

            let mut ptr: *mut u32 = self.dst.as_mut_ptr();
            ptr = ptr.add(self.dst.capacity());
            self.ptr = ptr;
        }
    }

    #[inline]
    fn put_at(&mut self, channel: usize, symbol: &Self::Symbol) {
        debug_assert!(channel <= N);

        unsafe {
            ryg_rans_sys::rans_64::rans_64_enc_put_symbol(
                &mut self.states[channel],
                &mut self.ptr,
                &symbol.symbol,
                symbol.scale_bits,
            );
        }

        check_enc_pointer!(self);
    }

    #[inline]
    fn flush_at(&mut self, channel: usize) {
        debug_assert!(channel <= N);

        unsafe {
            ryg_rans_sys::rans_64::rans_64_enc_flush(&mut self.states[channel], &mut self.ptr);
        }

        check_enc_pointer!(self);
    }

    #[inline]
    fn data(&self) -> &[u8] {
        unsafe {
            let begin_ptr = self.dst.as_ptr();
            let start_index = self.ptr.offset_from(begin_ptr) as usize;
            let len = self.dst.capacity() - start_index;
            slice::from_raw_parts(self.ptr as *const u8, len * 4)
        }
    }
}

impl RansEncoder for B64RansEncoderMulti<1> {}

/// rANS encoder symbol - 64-bit version.
#[derive(Debug, Clone)]
pub struct B64RansEncSymbol {
    symbol: ryg_rans_sys::rans_64::Rans64EncSymbol,
    scale_bits: u32,
}

impl RansEncSymbol for B64RansEncSymbol {
    #[inline]
    #[must_use]
    fn new(cum_freq: u32, freq: u32, scale_bits: u32) -> Self {
        unsafe {
            let mut symbol = MaybeUninit::uninit();
            ryg_rans_sys::rans_64::rans_64_enc_symbol_init(
                symbol.as_mut_ptr(),
                cum_freq,
                freq,
                scale_bits,
            );

            Self {
                symbol: symbol.assume_init(),
                scale_bits,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::b64_encoder::{B64RansEncoder, B64RansEncoderMulti};
    use crate::encoder::tests as enc_tests;

    #[test]
    fn test_encode_nothing() {
        let encoder = B64RansEncoder::new(1024);

        enc_tests::test_encode_nothing(encoder);
    }

    #[test]
    fn test_encode_empty_data() {
        let encoder = B64RansEncoder::new(1024);
        let data = [0, 0, 0, 128, 0, 0, 0, 0];

        enc_tests::test_encode_empty_data(encoder, &data);
    }

    #[test]
    fn test_encode_and_reset() {
        let encoder = B64RansEncoder::new(1024);
        let data1 = [0, 0, 0, 0, 1, 0, 0, 0];
        let data2 = [2, 0, 0, 0, 1, 0, 0, 0];

        enc_tests::test_encode_and_reset(encoder, &data1, &data2);
    }

    #[test]
    fn test_encode_two_symbols() {
        let encoder = B64RansEncoder::new(1024);
        let data = [2, 0, 0, 0, 2, 0, 0, 0];

        enc_tests::test_encode_two_symbols(encoder, &data)
    }

    #[test]
    fn test_encode_symbols_clone() {
        let encoder = B64RansEncoder::new(1024);
        let data = [2, 0, 0, 0, 2, 0, 0, 0];

        enc_tests::test_encode_symbols_clone(encoder, &data)
    }

    #[test]
    fn test_encode_more_data() {
        let encoder = B64RansEncoder::new(1024);
        let data = [
            122, 27, 118, 146, 40, 184, 212, 0, 147, 60, 144, 230, 24, 137, 205, 128,
        ];

        enc_tests::test_encode_more_data(encoder, &data);
    }

    #[test]
    fn test_encode_interleaved() {
        let encoder = B64RansEncoderMulti::<2>::new(1024);
        let data = [108, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 128, 0, 0, 0];

        enc_tests::encode_interleaved(encoder, &data);
    }
}
