//! The remote pages this tool reads, independent of how they are fetched.

use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Page {
    /// jpdb.io vocabulary search results for a query.
    JpdbSearch(String),
    /// jpdb.io kanji details page.
    JpdbKanji(String),
    /// kanjipedia.jp search results for a single kanji.
    KanjipediaSearch(String),
    /// kanjipedia.jp kanji details page, by kanjipedia's numeric id.
    KanjipediaKanji(String),
}

impl Page {
    pub fn url(&self) -> Url {
        match self {
            Page::JpdbSearch(query) => Url::parse_with_params(
                "https://jpdb.io/search",
                [("q", query.as_str()), ("lang", "english")],
            ),
            Page::JpdbKanji(character) => with_path("https://jpdb.io/kanji/", character),
            Page::KanjipediaSearch(character) => Url::parse_with_params(
                "https://www.kanjipedia.jp/search",
                [("k", character.as_str()), ("kt", "1"), ("sk", "leftHand")],
            ),
            Page::KanjipediaKanji(id) => with_path("https://www.kanjipedia.jp/kanji/", id),
        }
        .expect("static base URLs are valid")
    }
}

fn with_path(base: &str, segment: &str) -> Result<Url, url::ParseError> {
    let mut url = Url::parse(base)?;
    url.path_segments_mut()
        .expect("base URL has a path")
        .pop_if_empty()
        .push(segment);
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls_are_percent_encoded() {
        assert_eq!(
            Page::JpdbKanji("宣".into()).url().as_str(),
            "https://jpdb.io/kanji/%E5%AE%A3"
        );
        assert_eq!(
            Page::JpdbSearch("宣言".into()).url().as_str(),
            "https://jpdb.io/search?q=%E5%AE%A3%E8%A8%80&lang=english"
        );
        assert_eq!(
            Page::KanjipediaSearch("宣".into()).url().as_str(),
            "https://www.kanjipedia.jp/search?k=%E5%AE%A3&kt=1&sk=leftHand"
        );
        assert_eq!(
            Page::KanjipediaKanji("0004051500".into()).url().as_str(),
            "https://www.kanjipedia.jp/kanji/0004051500"
        );
    }
}
