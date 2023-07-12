//! Ranged Asymmetric Numeral Systems (rANS) encoder and decoder. Under the
//! hood, this is a high-level wrapper over
//! [ryg-rans-sys](https://github.com/m4tx/ryg-rans-sys/).
//!
//! ANS is a family of modern entropy coding methods introduced by Jarek Duda
//! from Jagiellonian University. It serves as an alternative to arithmetic and
//! Huffman coding, combining the performance and compression ratio of both.
//! Many recent compression algorithms, such as Facebook’s Zstandard, Apple’s
//! LZFSE, or JPEG XL, use ANS under the hood.
//!
//! # Types of encoders/decoders in this crate
//! * [`byte_encoder`]/[`byte_decoder`] has a byte-aligned rANS encoder/decoder.
//! * [`b64_encoder`]/[`b64_decoder`] is a 64-bit version that emits entire
//!   32-bit words at a time. It might be faster than byte-aligned variant on
//!   64-bit architectures, and also makes for a very precise arithmetic coder
//!   (i.e. it gets quite close to entropy). The trade-off is that this version
//!   will be slower on 32-bit machines, and the output bitstream is not
//!   endian-neutral.
//!
//! See the [ryg_rans](https://github.com/rygorous/ryg_rans) repository for more details.
//!
//! # See also
//! * [rANS on Wikipedia](https://en.wikipedia.org/wiki/Asymmetric_numeral_systems#Range_variants_(rANS)_and_streaming)
//!
//! # Examples
//! ```
//! use rans::byte_decoder::{ByteRansDecSymbol, ByteRansDecoder};
//! use rans::byte_encoder::{ByteRansEncSymbol, ByteRansEncoder};
//! use rans::{RansDecSymbol, RansDecoder, RansEncSymbol, RansEncoder, RansEncoderMulti};
//!
//! const SCALE_BITS: u32 = 2;
//!
//! // Encode two symbols
//! let mut encoder = ByteRansEncoder::new(1024);
//! let symbol1 = ByteRansEncSymbol::new(0, 2, SCALE_BITS);
//! let symbol2 = ByteRansEncSymbol::new(2, 2, SCALE_BITS);
//!
//! encoder.put(&symbol1);
//! encoder.put(&symbol2);
//! encoder.flush();
//!
//! let mut data = encoder.data().to_owned();
//!
//! // Decode the encoded data
//! let mut decoder = ByteRansDecoder::new(data);
//! let symbol1 = ByteRansDecSymbol::new(0, 2);
//! let symbol2 = ByteRansDecSymbol::new(2, 2);
//!
//! // Please note that the data is being decoded in reverse
//! assert_eq!(decoder.get(SCALE_BITS), 2); // Decoder returns cumulative frequency
//! decoder.advance(&symbol2, SCALE_BITS);
//! assert_eq!(decoder.get(SCALE_BITS), 0);
//! decoder.advance(&symbol1, SCALE_BITS);
//! ```

#![deny(
    missing_docs,
    missing_debug_implementations,
    trivial_numeric_casts,
    unreachable_pub,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

pub use decoder::*;
pub use encoder::*;

/// 64-bit rANS decoder.
pub mod b64_decoder;
/// 64-bit rANS encoder.
pub mod b64_encoder;
/// Byte-aligned rANS decoder.
pub mod byte_decoder;
/// Byte-aligned rANS encoder.
pub mod byte_encoder;
mod decoder;
mod encoder;
/// `MutCow` smart pointer to work with mutably-borrowed/owned data in a
/// unified way.
pub mod mut_cow;
