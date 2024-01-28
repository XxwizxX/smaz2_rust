mod compressor;
mod default_config;

use argh::FromArgs;
use crate::compressor::Compressor;
use crate::default_config as default;

#[derive(FromArgs, Debug, Default)]
/// Rust implementation of smaz2 algorithm
/// https://github.com/antirez/smaz2
struct Smaz2{
    /// text to be compressed
    #[argh(positional)]
    text: String,
}

fn main() {
    let smaz: Smaz2 = argh::from_env();
    let source = smaz.text;
    println!("input string: {:#?}", source);
    let compressor: Compressor = Compressor::new(
        default::WORDS,
        default::BI_GRAMS
    );
    let result = compressor.compress(&source);
    println!("compressed result: {:?}", result);
    println!("Compressed length: {:.02}%", (result.len() as f32 / source.as_bytes().len() as f32 ) * 100.0);
    let recovered = compressor.decompress(result);
    println!("recovered message: {:?}", recovered);
}
