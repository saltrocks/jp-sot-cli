//! Look up Japanese words and kanji on jpdb.io and kanjipedia.jp, producing
//! a JSON-friendly tree with a full recursive kanji decomposition.

pub mod fetch;
pub mod model;
pub mod page;
pub mod parse;
mod resolve;

pub use fetch::{Fetcher, HttpFetcher};
pub use resolve::Dictionary;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    #[error("failed to fetch {url}: {status}")]
    Status {
        url: url::Url,
        status: reqwest::StatusCode,
    },
    #[error("no vocabulary result found for \"{0}\"")]
    NoResult(String),
}

/// Whether `query` is a single CJK ideograph (looked up as a kanji rather
/// than as a word).
pub fn is_single_kanji(query: &str) -> bool {
    let mut chars = query.chars();
    matches!(
        (chars.next(), chars.next()),
        (
            Some(
                '\u{3400}'..='\u{4DBF}'
                | '\u{4E00}'..='\u{9FFF}'
                | '\u{F900}'..='\u{FAFF}'
                | '\u{20000}'..='\u{3134F}',
            ),
            None,
        )
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_kanji_detection() {
        assert!(is_single_kanji("宣"));
        assert!(is_single_kanji("宀"));
        assert!(is_single_kanji("𠮟"));
        assert!(!is_single_kanji("宣言"));
        assert!(!is_single_kanji("あ"));
        assert!(!is_single_kanji(""));
    }
}
