//! Output model. Serializes to the same JSON shape the original CLI emitted.

use serde::{Serialize, Serializer};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Entry {
    Kanji(Kanji),
    Word(Word),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Kanji {
    #[serde(rename = "type", serialize_with = "tag_kanji")]
    pub kind: (),
    pub character: String,
    pub keyword: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kanji_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kanken: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heisig: Option<u32>,
    pub readings: Vec<Reading>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mnemonic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub etymology: Option<String>,
    pub components: Vec<Kanji>,
}

impl Kanji {
    pub const SEE_ABOVE: &str = "(see above)";

    /// Placeholder for a kanji already expanded earlier in the same tree.
    pub fn see_above(character: impl Into<String>) -> Self {
        Self {
            kind: (),
            character: character.into(),
            keyword: Self::SEE_ABOVE.to_owned(),
            frequency: None,
            kanji_type: None,
            kanken: None,
            heisig: None,
            readings: Vec::new(),
            mnemonic: None,
            etymology: None,
            components: Vec::new(),
        }
    }

    pub fn is_see_above(&self) -> bool {
        self.keyword == Self::SEE_ABOVE && self.readings.is_empty() && self.components.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Reading {
    pub reading: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Word {
    #[serde(rename = "type", serialize_with = "tag_word")]
    pub kind: (),
    pub word: String,
    pub reading: String,
    pub meanings: Vec<Meaning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pitch_accent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency: Option<String>,
    pub composed_of: Vec<Component>,
    pub kanji: Vec<Kanji>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Meaning {
    pub part_of_speech: String,
    pub definitions: Vec<String>,
}

/// A vocabulary item a word is composed of.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Component {
    pub word: String,
    pub reading: String,
    pub meaning: String,
}

fn tag_kanji<S: Serializer>(_: &(), s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str("kanji")
}

fn tag_word<S: Serializer>(_: &(), s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str("word")
}
