use gcd::Gcd;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rand::Rng;
use std::collections::HashMap;

#[derive(Copy, Clone, Debug)]
enum Token {
    Char(u8),
    Skip(char),
}

#[derive(Clone, Debug)]
struct Alphabet {
    range: (char, char),
    freqs: Vec<f32>,
}

impl Alphabet {
    fn tokenize(&self, text: &str) -> Vec<Token> {
        text.chars().map(|c| self.totok(c)).collect()
    }

    fn glue(&self, toks: &[Token]) -> String {
        toks.iter().map(|t| self.fromtok(*t)).collect()
    }

    fn range(&self) -> std::ops::RangeInclusive<char> {
        return self.range.0..=self.range.1;
    }

    fn rangesize(&self) -> usize {
        self.range().size_hint().0
    }

    fn totok(&self, c: char) -> Token {
        match self.range().contains(&c) {
            false => Token::Skip(c),
            true => Token::Char(self.range().position(|ic| ic == c).unwrap() as u8),
        }
    }

    fn fromtok(&self, tok: Token) -> char {
        match tok {
            Token::Skip(c) => c,
            Token::Char(idx) => self
                .range()
                .nth((idx % self.rangesize() as u8) as usize)
                .unwrap(),
        }
    }

    fn tooffsets(&self, w: &str) -> Vec<u8> {
        self.tokenize(w)
            .iter()
            .map(|t| if let Token::Char(i) = t { *i } else { 0u8 })
            .collect()
    }
}

struct VignereCipher<'a> {
    offsets: Vec<u8>,
    alphabet: &'a Alphabet,
}

impl<'a> VignereCipher<'a> {
    fn transform(&self, toks: &[Token], reverse: bool) -> Vec<Token> {
        let cycle = (0..self.offsets.len()).cycle();
        toks.iter()
            .zip(cycle)
            .map(|(t, ti)| match *t {
                skip @ Token::Skip(_) => skip,
                Token::Char(ci) => {
                    if !reverse {
                        Token::Char((ci + self.offsets[ti]) % self.alphabet.rangesize() as u8)
                    } else {
                        Token::Char(
                            (ci + (self.alphabet.rangesize() as u8) - self.offsets[ti])
                                % self.alphabet.rangesize() as u8,
                        )
                    }
                }
            })
            .collect()
    }
}

fn encrypt(text: &str, alphabet: &Alphabet, word: &str, reverse: bool) -> String {
    let mut tokens = alphabet.tokenize(text);
    let offsets = alphabet.tooffsets(word);
    alphabet.glue(
        &VignereCipher {
            offsets,
            alphabet: &alphabet,
        }
        .transform(&mut tokens, reverse),
    )
}

fn best_offset(alphabet: &Alphabet, freq: HashMap<u8, usize>) -> u8 {
    let total_count = freq.values().sum::<usize>() as f32;

    let (mut best, mut best_total) = (0u8, 100500f32);
    for offset in 0..26u8 {
        let sum: f32 = freq
            .iter()
            .map(|(ci, v)| {
                let f1 = 100f32 * *v as f32 / total_count;
                let f2 = alphabet.freqs[(offset + ci) as usize % alphabet.rangesize()];
                (f1 - f2).abs()
            })
            .sum();
        if sum < best_total {
            (best, best_total) = (offset, sum);
        }
    }
    26 - best
}

fn crack_vigere(text: &str, alphabet: &Alphabet, kwsize: usize) -> String {
    let tokens = alphabet.tokenize(text);
    let mut freqs: Vec<HashMap<u8, usize>> = vec![HashMap::new(); kwsize];

    tokens.iter().zip((0..kwsize).cycle()).for_each(|(t, ti)| {
        if let Token::Char(ci) = t {
            freqs[ti].entry(*ci).and_modify(|v| *v += 1).or_insert(1);
        }
    });

    let kw_toks: Vec<Token> = freqs
        .into_iter()
        .map(|f| Token::Char(best_offset(&alphabet, f)))
        .collect();
    alphabet.glue(&kw_toks)
}

fn estimate_kwsize(text: &str, alphabet: &Alphabet) -> usize {
    let tokens = alphabet.tokenize(text);

    let mut pos = HashMap::<String, Vec<usize>>::new();
    for l in [3, 4, 5, 6] {
        for i in 0..tokens.len() - l {
            if tokens[i..i + l].iter().any(|t| matches!(t, Token::Skip(_))) {
                continue;
            }

            pos.entry(alphabet.glue(&tokens[i..i + l]))
                .and_modify(|v| v.push(i))
                .or_insert(vec![i; 1]);
        }
    }

    let mut dists = Vec::<usize>::new();
    for (_, v) in pos {
        if v.len() < 3 {
            continue;
        }
        dists.extend(v.windows(2).map(|s| s[1] - s[0]))
    }

    dists.shuffle(&mut thread_rng());
    dists.resize((dists.len() as f32 * 0.01) as usize, 0);

    if dists.is_empty() {
        return 0;
    }

    let mut out: usize = dists[0];
    for d in dists {
        out = out.gcd(d)
    }

    out
}

fn cycle(word: &str) {
    let english: Alphabet = Alphabet {
        range: ('a', 'z'),
        freqs: vec![
            8.167, 1.492, 2.782, 4.253, 12.702, 2.228, 2.015, 6.094, 6.966, 0.153, 0.772, 4.025,
            2.406, 6.749, 7.507, 1.929, 0.095, 5.987, 6.327, 9.056, 2.758, 0.978, 2.360, 0.150,
            1.974, 0.074,
        ],
    };

    let mut success = Vec::<usize>::new();

    let data: String = std::fs::read_to_string("articles.csv")
        .unwrap()
        .chars()
        .filter(|c| c.is_ascii())
        .map(|c| c.to_ascii_lowercase())
        .collect();

    let mut texts = Vec::<String>::new();

    let mut rdr = csv::Reader::from_reader(data.trim().as_bytes());
    for result in rdr.records().take(5000) {
        let result = result.unwrap();
        let text = result.get(0).unwrap().trim();
        texts.push(text.to_owned());
    }

    texts.sort_by_key(|s| s.len());
    texts.reverse();

    for text in texts.iter().take(1200) {
        let encr_test = encrypt(&text, &english, word, false);

        let estimated_len = estimate_kwsize(&encr_test, &english);
        let found_word = crack_vigere(&encr_test, &english, estimated_len);

        if found_word == word {
            success.push(text.len())
        }
    }

    println!(
        "Sucess rate for word len {}: {} / {}, min text size decrypted ... {}",
        word.len(),
        success.len(),
        texts.len(),
        success.iter().min().unwrap()
    );
}

fn main() {
    for len in [1, 2, 3, 5, 10, 25, 50] {
        let mut rng = rand::thread_rng();
        let random_string: String = (0..len) // specify the length of the string
            .map(|_| (rng.gen_range(97..123) as u8) as char)
            .collect();
        cycle(&random_string);
    }
}
