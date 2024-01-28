mod compressor;
mod default_config;

use argh::FromArgs;
use crate::compressor::Compressor;
use crate::default_config as default;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(FromArgs, Debug, Default)]
/// Rust implementation of smaz2 algorithm
/// https://github.com/antirez/smaz2
struct Smaz2{
    /// text to be compressed
    #[argh(positional)]
    text: String,

    /// word table file, use default if not specified
    #[argh(option, short = 'w')]
    word_table: Option<String>,
}

fn main() {
    let smaz: Smaz2 = argh::from_env();
    let source = smaz.text;

    let words = match smaz.word_table {
        None => {
            default::WORDS.map(|s| String::from(s))
                .to_vec()
        }
        Some(file_path) => {
            let file = File::open(file_path).expect("no file found");
            let buf = BufReader::new(file);
            let lines: Vec<String> = buf.lines()
                .map(|line| line.expect("Could not parse line"))
                .collect();
            match lines.len() {
                ..=256 => lines
                , _ => panic!("word table file should not have more than 256 lines")
            }
        }
    };

    println!("input string: {:#?}", source);
    let compressor: Compressor = Compressor::new(
        &words,
        default::BI_GRAMS
    );
    let result = compressor.compress(&source);
    println!("original byte array length: {}, data: {:?}", source.as_bytes().len(), source.as_bytes().to_vec());
    println!("compressed result length:{}, data: {:?}", result.len(), result);
    println!("Compressed length: {:.02}%", (result.len() as f32 / source.as_bytes().len() as f32 ) * 100.0);
    let recovered = compressor.decompress(result);
    println!("recovered message: {:?}", recovered);
}
