mod common;

use std::process::Command;

use common::{block_on, dictionary};
use jp_sot::model::{Component, Entry, Meaning, Word};

fn sengen() -> Word {
    block_on(dictionary().word("宣言")).unwrap()
}

#[test]
fn parses_spelling_and_reading() {
    let w = sengen();
    assert_eq!(w.word, "宣言");
    assert_eq!(w.reading, "せんげん");
}

#[test]
fn parses_meanings() {
    assert_eq!(
        sengen().meanings,
        [Meaning {
            part_of_speech: "Noun, Verb (する)".into(),
            definitions: vec![
                "declaration".into(),
                "proclamation".into(),
                "announcement".into()
            ],
        }]
    );
}

#[test]
fn parses_pitch_accent_and_frequency() {
    let w = sengen();
    assert_eq!(w.pitch_accent.as_deref(), Some("せ↑んげ↓ん"));
    assert_eq!(w.frequency.as_deref(), Some("Top 2000"));
}

#[test]
fn parses_composed_of() {
    let component = |word: &str, reading: &str, meaning: &str| Component {
        word: word.into(),
        reading: reading.into(),
        meaning: meaning.into(),
    };
    assert_eq!(
        sengen().composed_of,
        [
            component("宣", "せん", "imperial order;  imperial decree"),
            component("言", "げん", "word;  remark;  statement"),
        ]
    );
}

#[test]
fn includes_recursive_kanji_decomposition() {
    let w = sengen();
    let [sen, iu] = &w.kanji[..] else {
        panic!("{:?}", w.kanji)
    };
    assert_eq!(sen.keyword, "formally declare");
    assert_eq!(sen.components.len(), 2);
    assert_eq!(iu.keyword, "say");
    assert_eq!(
        iu.mnemonic.as_deref(),
        Some("A cryptic pictograph of a face trying to say something.")
    );

    let tan = &sen.components[1].components[1];
    assert_eq!(tan.character, "旦");
    assert_eq!(
        tan.mnemonic.as_deref(),
        Some("A pictograph of a sun rising over the horizon which signifies the start of a day.")
    );
    assert_eq!(tan.components[0].keyword, "sun");
}

#[test]
fn unknown_word_is_an_error() {
    let err = block_on(dictionary().word("ない")).unwrap_err();
    assert!(err.to_string().contains("404"), "{err}");
}

#[test]
fn json_output_shape() {
    let entry = block_on(dictionary().lookup("宣言")).unwrap();
    assert!(matches!(entry, Entry::Word(_)));
    let json = serde_json::to_value(&entry).unwrap();

    assert_eq!(json["type"], "word");
    assert_eq!(json["pitchAccent"], "せ↑んげ↓ん");
    assert_eq!(json["composedOf"][0]["word"], "宣");
    assert_eq!(json["meanings"][0]["partOfSpeech"], "Noun, Verb (する)");

    let sen = &json["kanji"][0];
    assert_eq!(sen["type"], "kanji");
    assert_eq!(sen["kanjiType"], "Kyōiku (6th grade)");
    assert_eq!(sen["readings"][3], serde_json::json!({ "reading": "せ" }));
    assert!(sen.get("mnemonic").is_none(), "absent fields are omitted");

    let see_above = &sen["components"][1]["components"][1]["components"][1];
    assert_eq!(
        *see_above,
        serde_json::json!({
            "type": "kanji",
            "character": "一",
            "keyword": "(see above)",
            "readings": [],
            "components": [],
        })
    );
}

#[test]
fn cli_without_query_prints_usage() {
    let output = Command::new(env!("CARGO_BIN_EXE_jp-sot")).output().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Usage:"));
}
