//! The two analyzers the concept map reads words with. Each gives, token for
//! token, what tantivy 0.27 gives for the same pipeline of `SimpleTokenizer`,
//! `RemoveLongFilter`, `LowerCaser`, `StopWordFilter` and English `Stemmer`.

use frostem::{Algorithm, Stemmer};

/// Tokens of this many bytes or more are dropped from `STEMMED`, as URLs and
/// encoded blobs are noise.
const TOKEN_MAX_BYTES: usize = 40;

/// The English stopwords of tantivy's `StopWordFilter`, which are Lucene's.
const ENGLISH_STOPWORDS: [&str; 33] = [
    "a", "an", "and", "are", "as", "at", "be", "but", "by", "for", "if", "in", "into", "is", "it",
    "no", "not", "of", "on", "or", "such", "that", "the", "their", "then", "there", "these",
    "they", "this", "to", "was", "will", "with",
];

/// Question words, dropped from `STEMMED` along with the English stopwords,
/// which lack them.
const QUESTION_WORDS: [&str; 9] = [
    "how", "what", "when", "where", "which", "who", "whom", "whose", "why",
];

/// One of the two pipelines.
pub enum Analyzer {
    Stemmed,
    Key,
}

/// Lowercased, English-stemmed tokens without English stopwords, question
/// words or tokens of `TOKEN_MAX_BYTES` or more. Step 2 counts these words.
pub const STEMMED: Analyzer = Analyzer::Stemmed;

/// Lowercased, English-stemmed tokens with nothing dropped. It keys the
/// phrases of the concept map.
const KEY: Analyzer = Analyzer::Key;

/// A token as the analyzer emits it. The graph reads only its text.
pub struct Tok {
    pub text: String,
}

/// The tokens `analyzer` produces for `text`, in text order.
pub fn tokens(analyzer: Analyzer, text: &str) -> Vec<Tok> {
    let stemmer = Stemmer::new(Algorithm::English);
    let stem = |token: String| Tok {
        text: stemmer.stem(&token).into_owned(),
    };
    match analyzer {
        Analyzer::Stemmed => simple_tokens(text)
            .filter(|token| token.len() < TOKEN_MAX_BYTES)
            .map(lowercase)
            .filter(|token| !ENGLISH_STOPWORDS.contains(&token.as_str()))
            .filter(|token| !QUESTION_WORDS.contains(&token.as_str()))
            .map(stem)
            .collect(),
        Analyzer::Key => simple_tokens(text).map(lowercase).map(stem).collect(),
    }
}

/// `word` lowercased and stemmed with the English stemmer. A hyphenated word
/// is stemmed part by part, and its parts are joined with `-`.
pub fn stem_word(word: &str) -> String {
    tokens(KEY, word)
        .into_iter()
        .map(|token| token.text)
        .collect::<Vec<_>>()
        .join("-")
}

/// tantivy's `SimpleTokenizer`: the maximal runs of alphanumeric chars.
fn simple_tokens(text: &str) -> impl Iterator<Item = &str> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|token| !token.is_empty())
}

/// tantivy's `LowerCaser`: an ASCII token lowercased as ASCII, any other one
/// char by char. Unlike `str::to_lowercase`, this turns a capital sigma into
/// `σ` at the end of a word too.
fn lowercase(token: &str) -> String {
    if token.is_ascii() {
        token.to_ascii_lowercase()
    } else {
        token.chars().flat_map(char::to_lowercase).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stemmed_drops_stopwords_and_question_words() {
        let words: Vec<String> = tokens(STEMMED, "How the Queen ran to a tree, and why")
            .into_iter()
            .map(|t| t.text)
            .collect();
        assert_eq!(words, ["queen", "ran", "tree"]);
    }

    #[test]
    fn stem_word_stems_each_part_of_a_hyphenated_word() {
        assert_eq!(stem_word("Entropies"), stem_word("entropy"));
        assert_eq!(
            stem_word("labor-powers"),
            format!("{}-{}", stem_word("labor"), stem_word("power"))
        );
    }
}
