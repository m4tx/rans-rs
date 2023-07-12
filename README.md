rans-rs
=======

[![Build Status](https://github.com/m4tx/rans-rs/workflows/Rust%20CI/badge.svg)](https://github.com/m4tx/rans-rs/actions)
[![crates.io](https://img.shields.io/crates/v/rans.svg)](https://crates.io/crates/rans)
[![Documentation](https://docs.rs/rans/badge.svg)](https://docs.rs/rans)
[![MIT licensed](https://img.shields.io/github/license/m4tx/rans-rs)](https://github.com/m4tx/rans-rs/blob/master/LICENSE)
[![codecov](https://codecov.io/gh/m4tx/rans-rs/branch/master/graph/badge.svg)](https://codecov.io/gh/m4tx/rans-rs)

Ranged Asymmetric Numeral Systems (rANS) encoder and decoder. Under the hood,
this is a high-level wrapper over
[ryg-rans-sys](https://github.com/m4tx/ryg-rans-sys/).

ANS is a family of modern entropy coding methods introduced by Jarek Duda from
Jagiellonian University. It serves as an alternative to arithmetic and Huffman
coding, combining the performance and compression ratio of both. Many recent
compression algorithms, such as Facebook’s Zstandard, Apple’s LZFSE, or JPEG XL,
use ANS under the hood.

See the [ryg_rans](https://github.com/rygorous/ryg_rans) repository for more
details about the underlying implementation.

## Usage

Add the following to your `Cargo.toml`:

```toml
[dependencies]
rans = "0.3.0"
```

## Examples

```rust
use rans::byte_decoder::{ByteRansDecSymbol, ByteRansDecoder};
use rans::byte_encoder::{ByteRansEncSymbol, ByteRansEncoder};
use rans::{RansDecSymbol, RansDecoder, RansEncSymbol, RansEncoder, RansEncoderMulti};

const SCALE_BITS: u32 = 2;

// Encode two symbols
let mut encoder = ByteRansEncoder::new(1024);
let symbol1 = ByteRansEncSymbol::new(0, 2, SCALE_BITS);
let symbol2 = ByteRansEncSymbol::new(2, 2, SCALE_BITS);

encoder.put(&symbol1);
encoder.put(&symbol2);
encoder.flush();

let mut data = encoder.data().to_owned();

// Decode the encoded data
let mut decoder = ByteRansDecoder::new(data);
let symbol1 = ByteRansDecSymbol::new(0, 2);
let symbol2 = ByteRansDecSymbol::new(2, 2);

// Please note that the data is being decoded in reverse
assert_eq!(decoder.get(SCALE_BITS), 2); // Decoder returns cumulative frequency
decoder.advance(&symbol2, SCALE_BITS);
assert_eq!(decoder.get(SCALE_BITS), 0);
decoder.advance(&symbol1, SCALE_BITS);
```

## License

The project is licensed under the [MIT license](LICENSE).

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the project by you shall be licensed as MIT, without any
additional terms or conditions.

## Developing
### `pre-commit`
We encourage contributors to use predefined [`pre-commit`](https://pre-commit.com/)
hooks --- to install them in your local repo, make sure you have `pre-commit`
installed and run
```shell
pre-commit install
```
