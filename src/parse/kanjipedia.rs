//! kanjipedia.jp pages: kanji search results and the なりたち (etymology) section.

use scraper::Html;

use super::{non_empty_text, selector, text};

/// Finds kanjipedia's numeric id for `character` in its search results.
pub fn parse_search(html: &str, character: &str) -> Option<String> {
    let doc = Html::parse_document(html);
    doc.select(selector!("#resultKanjiList a"))
        .filter(|link| text(*link) == character)
        .filter_map(|link| link.attr("href"))
        .find_map(|href| {
            let rest = &href[href.find("/kanji/")? + "/kanji/".len()..];
            let id: String = rest.chars().take_while(char::is_ascii_digit).collect();
            Some(id).filter(|id| !id.is_empty())
        })
}

/// Extracts the なりたち (character formation) text from a kanji page.
pub fn parse_etymology(html: &str) -> Option<String> {
    let doc = Html::parse_document(html);
    let naritachi = doc.select(selector!("li.naritachi")).next()?;
    // The first div is the section heading; the second holds the content.
    let content = naritachi.select(selector!("div")).nth(1)?;
    non_empty_text(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_matches_exact_character() {
        let html = r#"<ul id="resultKanjiList">
            <li><a href="/kanji/0000000001">亘</a></li>
            <li><a href="/kanji/0004051500">宣</a></li>
        </ul>"#;
        assert_eq!(parse_search(html, "宣").as_deref(), Some("0004051500"));
        assert_eq!(parse_search(html, "言"), None);
    }

    #[test]
    fn etymology_missing_section() {
        assert_eq!(parse_etymology("<ul><li>nothing</li></ul>"), None);
    }
}
