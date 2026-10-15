mod common;

use common::{block_on, dictionary};
use jp_sot::model::{Kanji, Reading};

fn kanji(character: &str) -> Kanji {
    block_on(dictionary().kanji(character)).unwrap()
}

fn reading(reading: &str, frequency: Option<&str>) -> Reading {
    Reading {
        reading: reading.into(),
        frequency: frequency.map(Into::into),
    }
}

#[test]
fn parses_basic_info() {
    let k = kanji("宣");
    assert_eq!(k.character, "宣");
    assert_eq!(k.keyword, "formally declare");
    assert_eq!(k.frequency.as_deref(), Some("Top 1100-1200"));
    assert_eq!(k.kanji_type.as_deref(), Some("Kyōiku (6th grade)"));
    assert_eq!(k.kanken.as_deref(), Some("Level 5"));
    assert_eq!(k.heisig, Some(200));
}

#[test]
fn parses_common_and_rare_readings() {
    assert_eq!(
        kanji("宣").readings,
        [
            reading("せん", Some("97%")),
            reading("のたも", Some("1%")),
            reading("のたま", Some("1%")),
            reading("せ", None),
            reading("ぜん", None),
        ]
    );
}

#[test]
fn mnemonic_is_optional() {
    assert_eq!(
        kanji("宀").mnemonic.as_deref(),
        Some("A pictograph of a roof with a chimney.")
    );
    assert_eq!(kanji("宣").mnemonic, None);
}

#[test]
fn etymology_from_kanjipedia() {
    assert_eq!(
        kanji("宣").etymology.as_deref(),
        Some(
            "形声。宀と、音符亘(セン)とから成る。天子が暮らす正殿の意を表す。借りて、あきらかの意に用いる。"
        )
    );
    assert_eq!(
        kanji("一").etymology.as_deref(),
        Some("指事。横線一本で、「ひとつ」、ひいて、数のはじめの意を表す。")
    );
}

#[test]
fn etymology_missing_on_kanjipedia() {
    assert_eq!(kanji("宀").etymology, None);
}

#[test]
fn recursively_decomposes_components() {
    let k = kanji("宣");
    let [ukanmuri, wataru] = &k.components[..] else {
        panic!("{:?}", k.components)
    };
    assert_eq!(
        (ukanmuri.character.as_str(), ukanmuri.keyword.as_str()),
        ("宀", "roof with a chimney")
    );
    assert!(ukanmuri.components.is_empty());

    assert_eq!(
        (wataru.character.as_str(), wataru.keyword.as_str()),
        ("亘", "span")
    );
    let [ichi, tan] = &wataru.components[..] else {
        panic!("{:?}", wataru.components)
    };
    assert_eq!(
        (ichi.character.as_str(), ichi.keyword.as_str()),
        ("一", "one")
    );
    assert_eq!(
        (tan.character.as_str(), tan.keyword.as_str()),
        ("旦", "start of a day")
    );

    let [nichi, _] = &tan.components[..] else {
        panic!("{:?}", tan.components)
    };
    assert_eq!(
        (nichi.character.as_str(), nichi.keyword.as_str()),
        ("日", "sun")
    );
}

#[test]
fn repeated_components_are_expanded_once() {
    let k = kanji("宣");
    let tan = &k.components[1].components[1];
    assert_eq!(tan.components[1], Kanji::see_above("一"));
    assert!(tan.components[1].is_see_above());
}

#[test]
fn each_kanji_is_fetched_once() {
    let fetcher = common::FixtureFetcher::default();
    block_on(jp_sot::Dictionary::new(&fetcher).kanji("宣")).unwrap();
    let requests = fetcher.requests.borrow();
    let jpdb: Vec<_> = requests
        .iter()
        .filter(|p| matches!(p, jp_sot::page::Page::JpdbKanji(_)))
        .collect();
    assert_eq!(jpdb.len(), 6, "{jpdb:?}");
}

#[test]
fn primitive_kanji() {
    let k = kanji("一");
    assert_eq!(k.keyword, "one");
    assert_eq!(k.heisig, Some(1));
    assert_eq!(k.mnemonic.as_deref(), Some("A pictograph of a one line."));
    assert!(k.components.is_empty());
}

#[test]
fn kanji_without_type() {
    let k = kanji("宀");
    assert_eq!(k.kanji_type, None);
    assert_eq!(k.frequency.as_deref(), Some("Never used"));
}

#[test]
fn unknown_kanji_is_an_error() {
    let err = block_on(dictionary().kanji("龘")).unwrap_err();
    assert!(err.to_string().contains("404"), "{err}");
}
