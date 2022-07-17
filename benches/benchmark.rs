use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use lazy_static::lazy_static;
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;
use rans::b64_decoder::{B64RansDecoder, B64RansDecoderMulti};
use rans::b64_encoder::{B64RansEncoder, B64RansEncoderMulti};
use rans::byte_decoder::{ByteRansDecoder, ByteRansDecoderMulti};
use rans::byte_encoder::{ByteRansEncoder, ByteRansEncoderMulti};
use rans::{
    RansDecSymbol, RansDecoder, RansDecoderMulti, RansEncSymbol, RansEncoder, RansEncoderMulti,
};

const TEST_DATA_LEN: usize = 256;
const SCALE_BITS: u32 = 8;

lazy_static! {
    static ref BYTE_DATA_SINGLE: Vec<u8> = gen_data_single(ByteRansEncoder::new(1024));
    static ref BYTE_DATA_INTERLEAVED: Vec<u8> =
        gen_data_interleaved(ByteRansEncoderMulti::new(1024));
    static ref B64_DATA_SINGLE: Vec<u8> = gen_data_single(B64RansEncoder::new(1024));
    static ref B64_DATA_INTERLEAVED: Vec<u8> = gen_data_interleaved(B64RansEncoderMulti::new(1024));
}

fn byte_encoder_encode_single(c: &mut Criterion) {
    encoder_encode_single(c, "Byte", move || ByteRansEncoder::new(1024));
}

fn b64_encoder_encode_single(c: &mut Criterion) {
    encoder_encode_single(c, "64b", move || B64RansEncoder::new(1024));
}

fn encoder_encode_single<T, F>(c: &mut Criterion, name: &str, f: F)
where
    T: RansEncoder,
    F: Fn() -> T + Copy,
{
    let symbols = get_enc_symbols();
    let symbol_vals = get_symbols_vals();

    c.bench_function(&format!("{} encode {}syms", name, TEST_DATA_LEN), |b| {
        b.iter_batched_ref(
            f,
            |encoder| {
                for i in 0..TEST_DATA_LEN {
                    encoder.put(&symbols[symbol_vals[i]]);
                }
            },
            BatchSize::LargeInput,
        )
    });
}

fn byte_encoder_encode_interleaved(c: &mut Criterion) {
    encoder_encode_interleaved(c, "Byte", move || ByteRansEncoderMulti::<2>::new(1024));
}

fn b64_encoder_encode_interleaved(c: &mut Criterion) {
    encoder_encode_interleaved(c, "64b", move || B64RansEncoderMulti::<2>::new(1024));
}

fn encoder_encode_interleaved<T, F>(c: &mut Criterion, name: &str, f: F)
where
    T: RansEncoderMulti<2>,
    F: Fn() -> T + Copy,
{
    let symbols = get_enc_symbols();
    let symbol_vals = get_symbols_vals();

    c.bench_function(
        &format!("{} encode {}syms interleaved", name, TEST_DATA_LEN),
        |b| {
            b.iter_batched_ref(
                f,
                |encoder| {
                    for i in 0..TEST_DATA_LEN {
                        encoder.put_at(i & 1, &symbols[symbol_vals[i]]);
                    }
                },
                BatchSize::LargeInput,
            )
        },
    );
}

fn get_enc_symbols<T: RansEncSymbol>() -> [T; 8] {
    let s1 = T::new(0, 3, SCALE_BITS);
    let s2 = T::new(3, 10, SCALE_BITS);
    let s3 = T::new(13, 58, SCALE_BITS);
    let s4 = T::new(71, 34, SCALE_BITS);
    let s5 = T::new(105, 41, SCALE_BITS);
    let s6 = T::new(146, 17, SCALE_BITS);
    let s7 = T::new(163, 55, SCALE_BITS);
    let s8 = T::new(218, 38, SCALE_BITS);

    [s1, s2, s3, s4, s5, s6, s7, s8]
}

fn get_dec_symbols<T: RansDecSymbol>() -> [T; 8] {
    let s1 = T::new(0, 3);
    let s2 = T::new(3, 10);
    let s3 = T::new(13, 58);
    let s4 = T::new(71, 34);
    let s5 = T::new(105, 41);
    let s6 = T::new(146, 17);
    let s7 = T::new(163, 55);
    let s8 = T::new(218, 38);

    [s1, s2, s3, s4, s5, s6, s7, s8]
}

fn byte_decoder_decode_single(c: &mut Criterion) {
    decoder_decode_single(c, "Byte", move || {
        ByteRansDecoder::new(BYTE_DATA_SINGLE.clone())
    });
}

fn b64_decoder_decode_single(c: &mut Criterion) {
    decoder_decode_single(c, "64b", move || {
        B64RansDecoder::new(B64_DATA_SINGLE.clone())
    });
}

fn decoder_decode_single<T, F>(c: &mut Criterion, name: &str, f: F)
where
    T: RansDecoder,
    F: Fn() -> T + Copy,
{
    let symbols = get_dec_symbols();
    let mut symbol_vals = get_symbols_vals();
    symbol_vals.reverse();

    c.bench_function(&format!("{} decode {}syms", name, TEST_DATA_LEN), |b| {
        b.iter_batched_ref(
            f,
            |decoder| {
                for i in 0..TEST_DATA_LEN {
                    let _ = decoder.get(SCALE_BITS);
                    decoder.advance(&symbols[symbol_vals[i]], 2);
                }
            },
            BatchSize::LargeInput,
        )
    });
}

fn byte_decoder_decode_interleaved(c: &mut Criterion) {
    decoder_decode_interleaved(c, "Byte", move || {
        ByteRansDecoderMulti::new(BYTE_DATA_INTERLEAVED.clone())
    });
}

fn b64_decoder_decode_interleaved(c: &mut Criterion) {
    decoder_decode_interleaved(c, "64b", move || {
        B64RansDecoderMulti::new(B64_DATA_INTERLEAVED.clone())
    });
}

fn decoder_decode_interleaved<T, F>(c: &mut Criterion, name: &str, f: F)
where
    T: RansDecoderMulti<2>,
    F: Fn() -> T + Copy,
{
    let symbols = get_dec_symbols();
    let mut symbol_vals = get_symbols_vals();
    symbol_vals.reverse();

    c.bench_function(
        &format!("{} decode {}syms interleaved", name, TEST_DATA_LEN),
        |b| {
            b.iter_batched_ref(
                f,
                |decoder| {
                    for i in (0..TEST_DATA_LEN).step_by(2) {
                        let _ = decoder.get_at(0, SCALE_BITS);
                        let _ = decoder.get_at(1, SCALE_BITS);
                        decoder.advance_step_at(0, &symbols[symbol_vals[i]], SCALE_BITS);
                        decoder.advance_step_at(1, &symbols[symbol_vals[i + 1]], SCALE_BITS);
                        decoder.renorm_all();
                    }
                },
                BatchSize::LargeInput,
            )
        },
    );
}

fn get_symbols_vals() -> [usize; TEST_DATA_LEN] {
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(1337);

    let mut data = [0; TEST_DATA_LEN];
    for item in data.iter_mut() {
        *item = rng.gen_range(0..8);
    }

    data
}

fn gen_data_single<T: RansEncoder>(mut encoder: T) -> Vec<u8> {
    let enc_symbols = get_enc_symbols();
    for sym in get_symbols_vals() {
        encoder.put(&enc_symbols[sym]);
    }

    encoder.data().to_owned()
}

fn gen_data_interleaved<T: RansEncoderMulti<2>>(mut encoder: T) -> Vec<u8> {
    let enc_symbols = get_enc_symbols();
    for sym in get_symbols_vals() {
        encoder.put_at(sym & 1, &enc_symbols[sym]);
    }

    encoder.data().to_owned()
}

criterion_group!(
    benches,
    byte_encoder_encode_single,
    b64_encoder_encode_single,
    byte_encoder_encode_interleaved,
    b64_encoder_encode_interleaved,
    byte_decoder_decode_single,
    b64_decoder_decode_single,
    byte_decoder_decode_interleaved,
    b64_decoder_decode_interleaved,
);
criterion_main!(benches);
