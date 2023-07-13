use std::mem::MaybeUninit;

use crate::encoder::{check_enc_pointer, RansEncSymbol, RansEncoder, RansEncoderMulti};

/// Multi-stream interleaved rANS encoder - byte-aligned version.
#[derive(Debug)]
pub struct ByteRansEncoderMulti<const N: usize> {
    states: [ryg_rans_sys::rans_byte::RansState; N],
    dst: Vec<u8>,
    ptr: *mut u8,
}

/// Single-stream rANS encoder - byte-aligned version.
pub type ByteRansEncoder = ByteRansEncoderMulti<1>;

impl<const N: usize> ByteRansEncoderMulti<N> {
    /// Creates a new `ByteMultiRansEncoder` instance that can contain `max_len`
    /// bytes in the internal buffer.
    ///
    /// Note that most of the API is inside the [`RansEncoderMulti`] trait, so
    /// you probably want to `use rans::RansEncoderMulti`.
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
    pub fn new(max_len: usize) -> Self {
        debug_assert!(N > 0);

        unsafe {
            #[allow(clippy::uninit_assumed_init)]
            let states: [ryg_rans_sys::rans_byte::RansState; N] =
                MaybeUninit::uninit().assume_init();
            let dst = vec![0; max_len];

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
        range_inclusive.contains(&(self.ptr as *const u8))
    }
}

impl<const N: usize> RansEncoderMulti<N> for ByteRansEncoderMulti<N> {
    type Symbol = ByteRansEncSymbol;

    fn reset(&mut self) {
        unsafe {
            for state in &mut self.states {
                ryg_rans_sys::rans_byte::rans_enc_init(state);
            }

            let mut ptr: *mut u8 = self.dst.as_mut_ptr();
            ptr = ptr.add(self.dst.len());
            self.ptr = ptr;
        }
    }

    #[inline]
    fn put_at(&mut self, channel: usize, symbol: &Self::Symbol) {
        debug_assert!(channel <= N);

        unsafe {
            ryg_rans_sys::rans_byte::rans_enc_put_symbol(
                &mut self.states[channel],
                &mut self.ptr,
                &symbol.symbol,
            );
        }

        check_enc_pointer!(self);
    }

    #[inline]
    fn flush_at(&mut self, channel: usize) {
        debug_assert!(channel <= N);

        unsafe {
            ryg_rans_sys::rans_byte::rans_enc_flush(&mut self.states[channel], &mut self.ptr);
        }

        check_enc_pointer!(self);
    }

    #[inline]
    fn data(&self) -> &[u8] {
        unsafe {
            let begin_ptr = self.dst.as_ptr();
            let start_index = self.ptr.offset_from(begin_ptr) as usize;
            &self.dst[start_index..]
        }
    }
}

impl RansEncoder for ByteRansEncoderMulti<1> {}

/// rANS encoder symbol - byte-aligned version.
#[derive(Debug, Clone)]
pub struct ByteRansEncSymbol {
    symbol: ryg_rans_sys::rans_byte::RansEncSymbol,
}

impl RansEncSymbol for ByteRansEncSymbol {
    #[inline]
    fn new(cum_freq: u32, freq: u32, scale_bits: u32) -> Self {
        unsafe {
            let mut symbol = MaybeUninit::uninit();
            ryg_rans_sys::rans_byte::rans_enc_symbol_init(
                symbol.as_mut_ptr(),
                cum_freq,
                freq,
                scale_bits,
            );

            Self {
                symbol: symbol.assume_init(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::byte_encoder::{ByteRansEncoder, ByteRansEncoderMulti};
    use crate::encoder::tests as enc_tests;

    #[test]
    fn test_encode_nothing() {
        let encoder = ByteRansEncoder::new(1024);

        enc_tests::test_encode_nothing(encoder);
    }

    #[test]
    fn test_encode_empty_data() {
        let encoder = ByteRansEncoder::new(1024);
        let data = [0, 0, 128, 0];

        enc_tests::test_encode_empty_data(encoder, &data);
    }

    #[test]
    fn test_encode_and_reset() {
        let encoder = ByteRansEncoder::new(1024);
        let data1 = [0, 0, 0, 1];
        let data2 = [2, 0, 0, 1];

        enc_tests::test_encode_and_reset(encoder, &data1, &data2);
    }

    #[test]
    fn test_encode_two_symbols() {
        let encoder = ByteRansEncoder::new(1024);
        let data = [2, 0, 0, 2];

        enc_tests::test_encode_two_symbols(encoder, &data)
    }

    #[test]
    fn test_encode_symbols_clone() {
        let encoder = ByteRansEncoder::new(1024);
        let data = [2, 0, 0, 2];

        enc_tests::test_encode_symbols_clone(encoder, &data)
    }

    #[test]
    fn test_encode_more_data() {
        let encoder = ByteRansEncoder::new(1024);
        let data = [
            106, 184, 212, 0, 84, 205, 93, 162, 171, 34, 28, 50, 161, 66, 2,
        ];

        enc_tests::test_encode_more_data(encoder, &data);
    }

    #[test]
    fn test_encode_interleaved() {
        let encoder = ByteRansEncoderMulti::<2>::new(1024);
        let data = [12, 0, 128, 0, 0, 0, 128, 0, 24, 0];

        enc_tests::encode_interleaved(encoder, &data);
    }

    #[test]
    fn test_has_debug_output() {
        let encoder = ByteRansEncoder::new(1024);
        enc_tests::test_has_debug_output(encoder);
    }
}
