//! jpdb.io pages: kanji details and vocabulary search results.

use ego_tree::NodeRef;
use percent_encoding::percent_decode_str;
use scraper::{ElementRef, Html, Node};

use super::{
    child_elements, has_class, next_element_sibling, non_empty_text, selector, style_contains, text,
};
use crate::model::{Component, Meaning, Reading};

/// Everything a jpdb kanji page says about one kanji.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct KanjiPage {
    pub keyword: String,
    pub frequency: Option<String>,
    pub kanji_type: Option<String>,
    pub kanken: Option<String>,
    pub heisig: Option<u32>,
    pub readings: Vec<Reading>,
    pub mnemonic: Option<String>,
    /// Kanji this one is visually composed of, in page order.
    pub components: Vec<String>,
}

/// The top vocabulary hit of a jpdb search.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct WordPage {
    pub word: String,
    pub reading: String,
    pub meanings: Vec<Meaning>,
    pub pitch_accent: Option<String>,
    pub frequency: Option<String>,
    pub composed_of: Vec<Component>,
    /// Kanji used in the word's spelling, in page order.
    pub kanji: Vec<String>,
}

pub fn parse_kanji(html: &str) -> KanjiPage {
    let doc = Html::parse_document(html);
    let root = doc.root_element();

    let mut page = KanjiPage {
        keyword: labeled_value(root, "Keyword").unwrap_or_default(),
        readings: readings(root),
        mnemonic: labeled_value(root, "Mnemonic"),
        components: kanji_links(root, "Composed of"),
        ..KanjiPage::default()
    };

    for row in root.select(selector!("table.cross-table tr")) {
        let mut cells = row.select(selector!("td"));
        let (Some(label), Some(value)) = (cells.next(), cells.next()) else {
            continue;
        };
        let value = text(value);
        match text(label).as_str() {
            "Frequency" => page.frequency = Some(value),
            // The cell ends with a "?" help link.
            "Type" => page.kanji_type = Some(value.trim_end_matches('?').trim().to_owned()),
            "Kanken" => page.kanken = Some(value),
            "Heisig" => page.heisig = leading_number(&value),
            _ => {}
        }
    }

    page
}

/// Parses the first vocabulary result of a search page, if there is one.
pub fn parse_word(html: &str) -> Option<WordPage> {
    let doc = Html::parse_document(html);
    let result = doc.select(selector!(".result.vocabulary")).next()?;

    let (word, reading) = spelling(result);
    Some(WordPage {
        word,
        reading,
        meanings: meanings(result),
        pitch_accent: pitch_accent(result),
        frequency: result
            .select(selector!(".tag"))
            .next()
            .and_then(non_empty_text),
        composed_of: composed_of(result),
        kanji: kanji_links(result, "Kanji used"),
    })
}

/// Text of the element following the `h6.subsection-label` reading `label`.
fn labeled_value(root: ElementRef, label: &str) -> Option<String> {
    root.select(selector!("h6.subsection-label"))
        .filter(|h| text(*h) == label)
        .find_map(next_element_sibling)
        .and_then(non_empty_text)
}

/// Kanji linked from the `.subsection-composed-of-kanji` section titled `label`.
fn kanji_links(root: ElementRef, label: &str) -> Vec<String> {
    root.select(selector!(".subsection-composed-of-kanji"))
        .filter(|section| {
            section
                .select(selector!("h6.subsection-label"))
                .next()
                .is_some_and(|h| text(h) == label)
        })
        .flat_map(|section| section.select(selector!("a.plain")))
        .filter_map(|link| {
            let path = link.attr("href")?.strip_prefix("/kanji/")?;
            let path = path.split('#').next().unwrap_or_default();
            let kanji = percent_decode_str(path).decode_utf8().ok()?;
            Some(kanji.into_owned()).filter(|k| !k.is_empty())
        })
        .collect()
}

fn readings(root: ElementRef) -> Vec<Reading> {
    let common = root
        .select(selector!("td.kanji-reading-list-common"))
        .flat_map(child_elements)
        .filter(|div| style_contains(*div, "display: flex"))
        .filter_map(|div| {
            let link = div.select(selector!("a")).next()?;
            let frequency = div
                .select(selector!("div[style*='margin-left']"))
                .next()
                .map(|f| text(f).replace(['(', ')'], ""));
            Some(Reading {
                reading: text(link),
                frequency,
            })
        });

    let rare = root
        .select(selector!("td.kanji-reading-list"))
        .flat_map(child_elements)
        .filter_map(|div| div.select(selector!("a")).next())
        .map(|link| Reading {
            reading: text(link),
            frequency: None,
        });

    common.chain(rare).collect()
}

fn spelling(result: ElementRef) -> (String, String) {
    let Some(spelling) = result
        .select(selector!(".primary-spelling .spelling"))
        .next()
    else {
        return Default::default();
    };
    match spelling.select(selector!("ruby")).next() {
        Some(ruby) => parse_ruby(ruby),
        None => (text(spelling), text(spelling)),
    }
}

/// Splits `<ruby>漢<rt>かん</rt>字<rt>じ</rt>です</ruby>` into the written form
/// and its full reading. Base text without furigana is read as written.
fn parse_ruby(ruby: ElementRef) -> (String, String) {
    let is_rt = |node: &NodeRef<Node>| node.value().as_element().is_some_and(|e| e.name() == "rt");

    let mut word = String::new();
    let mut reading = String::new();
    let mut nodes = ruby.children().peekable();
    while let Some(node) = nodes.next() {
        if is_rt(&node) {
            reading.extend(ElementRef::wrap(node).into_iter().flat_map(|rt| rt.text()));
        } else if let Some(t) = node.value().as_text() {
            word.push_str(t);
            if !nodes.peek().is_some_and(is_rt) {
                reading.push_str(t);
            }
        }
    }
    (word.trim().to_owned(), reading.trim().to_owned())
}

fn meanings(result: ElementRef) -> Vec<Meaning> {
    let Some(section) = result
        .select(selector!(".subsection-meanings .subsection"))
        .next()
    else {
        return Vec::new();
    };

    let mut meanings = Vec::new();
    let mut part_of_speech = String::new();
    for el in child_elements(section) {
        if has_class(el, "part-of-speech") {
            part_of_speech = el
                .select(selector!("div"))
                .map(text)
                .collect::<Vec<_>>()
                .join(", ");
        } else if has_class(el, "description") {
            let description = text(el);
            meanings.push(Meaning {
                part_of_speech: part_of_speech.clone(),
                definitions: strip_list_number(&description)
                    .split(';')
                    .map(str::trim)
                    .filter(|d| !d.is_empty())
                    .map(str::to_owned)
                    .collect(),
            });
        }
    }
    meanings
}

/// Renders the pitch pattern inline, e.g. `せ↑んげ↓ん`.
fn pitch_accent(result: ElementRef) -> Option<String> {
    let section = result
        .select(selector!(".subsection-pitch-accent .subsection"))
        .next()?;

    let mut accent = String::new();
    for part in section.select(selector!("div[style*='word-break'] > div")) {
        let part_text = text(part);
        let is_low = style_contains(part, "pitch-low") && !style_contains(part, "pitch-high");
        match (is_low, accent.is_empty()) {
            (true, true) => {
                accent.push_str(&part_text);
                accent.push('↑');
            }
            (true, false) => {
                accent.push('↓');
                accent.push_str(&part_text);
            }
            (false, _) => accent.push_str(&part_text),
        }
    }
    Some(accent).filter(|a| !a.is_empty())
}

fn composed_of(result: ElementRef) -> Vec<Component> {
    result
        .select(selector!(
            ".subsection-composed-of-vocabulary .subsection .composed-of"
        ))
        .filter_map(|item| {
            let jp = item.select(selector!(".jp")).next()?;
            let meaning = text(item.select(selector!(".description")).next()?);
            let (word, reading) = match jp.select(selector!("ruby")).next() {
                Some(ruby) => parse_ruby(ruby),
                None => (text(jp), text(jp)),
            };
            Some(Component {
                word,
                reading,
                meaning,
            })
        })
        .collect()
}

/// Strips a leading `"1. "` style list marker.
fn strip_list_number(s: &str) -> &str {
    let digits = s.len() - s.trim_start_matches(|c: char| c.is_ascii_digit()).len();
    match s[digits..].strip_prefix('.') {
        Some(rest) if digits > 0 => rest.trim_start(),
        _ => s,
    }
}

fn leading_number(s: &str) -> Option<u32> {
    let end = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
    s[..end].parse().ok().filter(|&n| n != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ruby(html: &str) -> (String, String) {
        let frag = Html::parse_fragment(html);
        parse_ruby(frag.select(selector!("ruby")).next().unwrap())
    }

    #[test]
    fn ruby_with_okurigana() {
        assert_eq!(
            ruby("<ruby>食<rt>た</rt>べる</ruby>"),
            ("食べる".to_owned(), "たべる".to_owned())
        );
    }

    #[test]
    fn list_number_is_stripped() {
        assert_eq!(strip_list_number("1.  declaration"), "declaration");
        assert_eq!(strip_list_number("12. x"), "x");
        assert_eq!(strip_list_number(".5 inches"), ".5 inches");
        assert_eq!(strip_list_number("plain"), "plain");
    }

    #[test]
    fn leading_number_is_lenient() {
        assert_eq!(leading_number("200"), Some(200));
        assert_eq!(leading_number("12a"), Some(12));
        assert_eq!(leading_number("n/a"), None);
    }

    #[test]
    fn heiban_pitch_accent() {
        let html = r#"<div class="subsection-pitch-accent"><div class="subsection">
            <div style="word-break: keep-all;">
              <div style="var(--pitch-low-s)">か</div>
              <div style="var(--pitch-high-s)">ぜ</div>
            </div></div></div>"#;
        let frag = Html::parse_fragment(html);
        assert_eq!(pitch_accent(frag.root_element()).as_deref(), Some("か↑ぜ"));
    }
}
