#[allow(clippy::new_without_default)]
use rans::byte_decoder::{ByteRansDecSymbol, ByteRansDecoderMulti};
use rans::byte_encoder::{ByteRansEncSymbol, ByteRansEncoderMulti};
use rans::{RansDecSymbol, RansDecoderMulti, RansEncSymbol, RansEncoderMulti};

#[derive(Debug, Clone)]
pub struct Context {
    pub symbol_prob: Vec<f32>,
}

impl Context {
    pub fn new<I: Into<Vec<f32>>>(symbol_prob: I) -> Self {
        Self {
            symbol_prob: symbol_prob.into(),
        }
    }

    pub fn as_integer_cum_freqs(&self, scale_bits: u8) -> Vec<u32> {
        let total: u32 = 1 << scale_bits;

        let mut result = self
            .symbol_prob
            .iter()
            .map(|&x| x * total as f32)
            .scan(0.0_f32, |acc, x| {
                let val = *acc;
                *acc += x;
                Some(val)
            })
            .map(|x| x.round() as u32)
            .collect();

        Self::cum_freq_to_freq(&mut result, total);
        Self::fix_zero_freqs(&mut result);
        Self::freq_to_cum_freq(&mut result);

        result
    }

    fn fix_zero_freqs(result: &mut Vec<u32>) {
        let mut zero_count = 0;
        for freq in result.iter_mut() {
            if *freq == 0 {
                *freq = 1;
                zero_count += 1;
            }
        }

        let mut i: usize = 0;
        while zero_count > 0 {
            if result[i] > 1 {
                result[i] -= 1;
                zero_count -= 1;
            }

            i += 1;
            if i >= result.len() {
                i = 0;
            }
        }
    }

    pub fn cum_freq_to_freq(cum_freq: &mut Vec<u32>, total: u32) {
        for i in 0..cum_freq.len() - 1 {
            cum_freq[i] = cum_freq[i + 1] - cum_freq[i];
        }
        let last = cum_freq.last_mut().unwrap();
        *last = total - *last;
    }

    pub fn freq_to_cum_freq(freq: &mut Vec<u32>) {
        let mut acc: u32 = 0;
        for val in freq {
            let old_val = *val;
            *val = acc;
            acc += old_val;
        }
    }
}

#[derive(Debug, Clone)]
pub struct RansEncContext {
    symbols: Vec<ByteRansEncSymbol>,
}

impl RansEncContext {
    pub fn from_context(context: &Context, scale_bits: u8) -> Self {
        let cum_freqs = context.as_integer_cum_freqs(scale_bits);
        let mut freqs = cum_freqs.clone();
        Context::cum_freq_to_freq(&mut freqs, 1 << scale_bits);

        let symbols = cum_freqs
            .iter()
            .zip(freqs.iter())
            .map(|(&cum_freq, &freq)| ByteRansEncSymbol::new(cum_freq, freq, scale_bits as u32))
            .collect();

        Self { symbols }
    }
}

#[derive(Debug)]
pub struct DoubleCompressor {
    encoder: ByteRansEncoderMulti<2>,
}

impl DoubleCompressor {
    pub fn new() -> Self {
        Self {
            encoder: ByteRansEncoderMulti::new(1024),
        }
    }

    pub fn flush(&mut self) {
        self.encoder.flush_all();
    }

    pub fn data(&self) -> &[u8] {
        self.encoder.data()
    }

    pub fn put(
        &mut self,
        ctx_1: &RansEncContext,
        symbol_index_1: usize,
        ctx_2: &RansEncContext,
        symbol_index_2: usize,
    ) {
        self.encoder.put_at(0, &ctx_1.symbols[symbol_index_1]);
        self.encoder.put_at(1, &ctx_2.symbols[symbol_index_2]);
        println!("Encoded: {}, {}", symbol_index_1, symbol_index_2);
    }
}

#[derive(Debug, Clone)]
pub struct RansDecContext {
    symbols: Vec<ByteRansDecSymbol>,
    freq_to_symbol: Vec<usize>,
    scale_bits: u32,
}

impl RansDecContext {
    pub fn from_context(context: &Context, scale_bits: u8) -> Self {
        let total_freq = 1 << scale_bits;

        let cum_freqs = context.as_integer_cum_freqs(scale_bits);
        let mut freqs = cum_freqs.clone();
        Context::cum_freq_to_freq(&mut freqs, total_freq);

        let symbols = cum_freqs
            .iter()
            .zip(freqs.iter())
            .map(|(&cum_freq, &freq)| ByteRansDecSymbol::new(cum_freq, freq))
            .collect();

        let mut freq_to_symbol = Vec::with_capacity(total_freq as usize);
        for i in 0..cum_freqs.len() - 1 {
            freq_to_symbol.resize(cum_freqs[i + 1] as usize, i);
        }
        freq_to_symbol.resize(total_freq as usize, cum_freqs.len() - 1);

        Self {
            symbols,
            freq_to_symbol,
            scale_bits: scale_bits as u32,
        }
    }

    pub fn cum_freq_to_symbol_index(&self, cum_freq: u32) -> usize {
        self.freq_to_symbol[cum_freq as usize]
    }
}

pub struct DoubleDecompressor<'a> {
    decoder: ByteRansDecoderMulti<'a, 2>,
}

impl<'a> DoubleDecompressor<'a> {
    pub fn new(data: &'a mut [u8]) -> Self {
        Self {
            decoder: ByteRansDecoderMulti::new(data),
        }
    }

    pub fn get(&mut self, ctx_1: &RansDecContext, ctx_2: &RansDecContext) -> (usize, usize) {
        let cum_freq_2 = self.decoder.get_at(0, ctx_2.scale_bits);
        let cum_freq_1 = self.decoder.get_at(1, ctx_1.scale_bits);
        let symbol_index_2 = ctx_2.cum_freq_to_symbol_index(cum_freq_2);
        let symbol_index_1 = ctx_1.cum_freq_to_symbol_index(cum_freq_1);
        self.decoder
            .advance_step_at(0, &ctx_2.symbols[symbol_index_2], ctx_2.scale_bits);
        self.decoder
            .advance_step_at(1, &ctx_1.symbols[symbol_index_1], ctx_1.scale_bits);
        self.decoder.renorm_all();

        println!("Decoded: {}, {}", symbol_index_1, symbol_index_2);
        (symbol_index_1, symbol_index_2)
    }
}

fn main() {
    const SCALE_BITS: u8 = 6;

    let ctx1 = Context::new([0.25, 0.25, 0.25, 0.25]);
    let ctx2 = Context::new([0.125, 0.125, 0.125, 0.125, 0.125, 0.125, 0.125, 0.125]);
    let enc_ctx1 = RansEncContext::from_context(&ctx1, SCALE_BITS);
    let enc_ctx2 = RansEncContext::from_context(&ctx2, SCALE_BITS);
    let dec_ctx1 = RansDecContext::from_context(&ctx1, SCALE_BITS);
    let dec_ctx2 = RansDecContext::from_context(&ctx2, SCALE_BITS);

    let mut compressor = DoubleCompressor::new();
    compressor.put(&enc_ctx1, 0, &enc_ctx2, 1);
    compressor.put(&enc_ctx1, 1, &enc_ctx2, 3);
    compressor.put(&enc_ctx1, 2, &enc_ctx2, 5);
    compressor.put(&enc_ctx1, 3, &enc_ctx2, 7);
    compressor.flush();

    let mut compressed = compressor.data().to_owned();
    println!("\nCompressed data: {:?}\n", compressed);

    let mut decompressor = DoubleDecompressor::new(&mut compressed);
    assert_eq!(decompressor.get(&dec_ctx1, &dec_ctx2), (3, 7));
    assert_eq!(decompressor.get(&dec_ctx1, &dec_ctx2), (2, 5));
    assert_eq!(decompressor.get(&dec_ctx1, &dec_ctx2), (1, 3));
    assert_eq!(decompressor.get(&dec_ctx1, &dec_ctx2), (0, 1));
}
