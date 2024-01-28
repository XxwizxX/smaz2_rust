use std::mem::replace;
use std::str;

#[derive(Debug, Copy, Clone)]
pub struct Compressor<'a> {
    pub words: &'a Vec<String>,
    pub bi_grams: &'a str,
}

impl<'a> Compressor<'a> {
    pub fn new(words: &'a Vec<String>, bi_grams: &'a str) -> Compressor<'a> {
        Self { words, bi_grams }
    }

    pub fn compress(self: Self, original: &str) -> Vec<u8> {
        let mut original = original.as_bytes().to_vec();
        let mut compressed: Vec<u8> = Vec::new();
        let mut verbatim_len: usize = 0;

        let bi_gram_vec = &self.bi_grams.as_bytes().to_vec();

        while !original.is_empty() {
            // try to find a matching word
            let word_match = self.words.into_iter().position(|word| {
                let word_len = word.len();
                let has_space_in_front = original[0].is_ascii_whitespace();
                let space_delta = has_space_in_front as usize;
                original.len() > (word_len + space_delta) && original[space_delta..word_len + space_delta].eq(word.as_bytes())
            });
            if let Some(i) = word_match {
                let matched_word = self.words.get(i).expect("failed to get word in table");
                let matched_word_len = matched_word.len();
                if original[0].is_ascii_whitespace() {
                    compressed.push(8);
                    compressed.push(u8::try_from(i).unwrap());
                    original = original[(1 + matched_word_len)..].to_vec();
                } else if original.len() > matched_word_len
                    && original[matched_word_len].is_ascii_whitespace() {
                    compressed.push(7);
                    compressed.push(u8::try_from(i).unwrap());
                    original = original[(1 + matched_word_len)..].to_vec();
                } else {
                    compressed.push(6);
                    compressed.push(u8::try_from(i).unwrap());
                    original = original[matched_word_len..].to_vec();
                }
                verbatim_len = 0;
                continue;
            }

            if original.len() > 2 {
                for (i, _) in bi_gram_vec.into_iter().enumerate().step_by(2) {
                    let bi_gram = &original[0..=1];
                    let slice = &bi_gram_vec[i..=i+1];
                    if bi_gram[0] == slice[0] && bi_gram[1] == slice[1] {
                        break
                    }
                }
                let bi_gram_match = bi_gram_vec.into_iter().enumerate().step_by(2).position(|(i, _)| {
                    let bi_gram = &original[0..=1];
                    let slice = &bi_gram_vec[i..=i+1];
                    bi_gram.eq(slice)
                });
                if let Some(i) = bi_gram_match {
                    compressed.push(1 << 7 | i as u8);
                    original = original[2..].to_vec();
                    verbatim_len = 0;
                    continue;
                }
            }

            if !(0 < original[0] && original[0] < 9) && original[0] < 128 {
                compressed.push(original[0]);
                original = original[1..].to_vec();
                verbatim_len = 0;
                continue;
            }

            verbatim_len += 1;
            match verbatim_len {
                1 => {
                    compressed.extend(vec![verbatim_len as u8, original[0]]);
                }
                2..=5 => {
                    let current_len = compressed.len();
                    let _ = replace(&mut compressed[current_len - verbatim_len], verbatim_len as u8);
                    compressed.push(original[0]);
                    verbatim_len %= 5;
                }
                _ => {
                    panic!("should not reach here!")
                }
            }
            original = original[1..].to_vec();
        }
        compressed
    }

    pub fn decompress(self: Self, encoded: Vec<u8>) -> String {
        let mut decoded: Vec<u8> = Vec::new();
        let mut i: usize = 0;
        while i < encoded.len() {
            let code = *encoded.get(i).unwrap();
            match code {
                128..=255 => {
                    let bi_gram_idx = ((code & 127) * 2) as usize;
                    let bi_gram = &self.bi_grams[bi_gram_idx..bi_gram_idx + 2];
                    decoded.extend(bi_gram.as_bytes());
                    i += 1;
                }
                1..=5 => {
                    let verbatim_len = code as usize;
                    i += 1;
                    let verbatim_arr = &encoded[i..i + verbatim_len];
                    decoded.extend(verbatim_arr);
                    i += verbatim_len
                }
                6..=8 => {
                    if code == 8 {
                        decoded.push(32u8);
                    }
                    i += 1;
                    let word_idx = encoded[i] as usize;
                    decoded.extend(self.words.get(word_idx).unwrap().as_bytes());
                    if code == 7 {
                        decoded.push(32u8);
                    }
                    i += 1;
                }
                _ => {
                    decoded.push(*encoded.get(i).unwrap());
                    i += 1;
                }
            }
        }

        match String::from_utf8(decoded) {
            Ok(result) => {
                result
            }
            Err(e) => {
                panic!("failed to decode message, error message: {:#?}", e)
            }
        }
    }
}