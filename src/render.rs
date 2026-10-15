//! Plain-text rendering: a compact, indented outline. The kanji
//! decomposition is drawn as a tree.

use std::fmt::{self, Write};

use crate::model::{Entry, Kanji, Reading, Word};

/// Renders `entry` as a text outline, one item per line.
pub fn text(entry: &Entry) -> String {
    let mut out = String::new();
    match entry {
        Entry::Kanji(kanji) => write_kanji(&mut out, kanji, "", "", ""),
        Entry::Word(word) => write_word(&mut out, word),
    }
    .expect("writing to a String cannot fail");
    out
}

fn write_word(out: &mut String, word: &Word) -> fmt::Result {
    let mut header = word.word.clone();
    if word.reading != word.word {
        write!(header, "【{}】", word.reading)?;
    }
    let parts = [
        Some(header),
        word.frequency.as_ref().map(|f| format!("frequency: {f}")),
        word.pitch_accent.as_ref().map(|p| format!("pitch: {p}")),
    ];
    writeln!(out, "{}", joined(parts.into_iter().flatten()))?;

    if !word.meanings.is_empty() {
        writeln!(out, "Meanings:")?;
        for (n, meaning) in word.meanings.iter().enumerate() {
            write!(out, "  {}. ", n + 1)?;
            if !meaning.part_of_speech.is_empty() {
                write!(out, "[{}] ", meaning.part_of_speech)?;
            }
            writeln!(out, "{}", one_line(&meaning.definitions.join("; ")))?;
        }
    }

    if !word.composed_of.is_empty() {
        writeln!(out, "Composed of:")?;
        for part in &word.composed_of {
            write!(out, "  {}", part.word)?;
            if part.reading != part.word {
                write!(out, "【{}】", part.reading)?;
            }
            writeln!(out, " {}", one_line(&part.meaning))?;
        }
    }

    if !word.kanji.is_empty() {
        writeln!(out, "Kanji:")?;
        for kanji in &word.kanji {
            write_kanji(out, kanji, "  ", "", "  ")?;
        }
    }
    Ok(())
}

/// Writes one kanji and its component subtree.
///
/// `connector` precedes the kanji's own line; `prefix` precedes everything,
/// and `indent` is what the kanji's details and children are nested under.
fn write_kanji(
    out: &mut String,
    kanji: &Kanji,
    prefix: &str,
    connector: &str,
    indent: &str,
) -> fmt::Result {
    write!(out, "{prefix}{connector}{}", kanji.character)?;
    if kanji.is_see_above() {
        return writeln!(out, " (expanded above)");
    }
    writeln!(out, " {}", header(kanji))?;

    let nested = format!("{prefix}{indent}");
    let gutter = if kanji.components.is_empty() {
        "  "
    } else {
        "│ "
    };
    for line in details(kanji) {
        writeln!(out, "{nested}{gutter}{line}")?;
    }

    let last = kanji.components.len().saturating_sub(1);
    for (i, component) in kanji.components.iter().enumerate() {
        let (connector, indent) = if i == last {
            ("└─ ", "   ")
        } else {
            ("├─ ", "│  ")
        };
        write_kanji(out, component, &nested, connector, indent)?;
    }
    Ok(())
}

/// `"keyword" · type · Kanken 5 · Heisig #200 · frequency: …`
fn header(kanji: &Kanji) -> String {
    let parts = [
        Some(format!("\"{}\"", kanji.keyword)),
        kanji.kanji_type.clone(),
        kanji
            .kanken
            .as_ref()
            .map(|k| format!("Kanken {}", k.strip_prefix("Level ").unwrap_or(k))),
        kanji.heisig.map(|h| format!("Heisig #{h}")),
        kanji.frequency.as_ref().map(|f| format!("frequency: {f}")),
    ];
    joined(parts.into_iter().flatten())
}

fn details(kanji: &Kanji) -> Vec<String> {
    let (common, rare): (Vec<&Reading>, Vec<&Reading>) =
        kanji.readings.iter().partition(|r| r.frequency.is_some());
    let readings = [
        (!common.is_empty()).then(|| format!("readings: {}", list(&common))),
        (!rare.is_empty()).then(|| format!("rare readings: {}", list(&rare))),
    ];

    [
        Some(joined(readings.into_iter().flatten())).filter(|r| !r.is_empty()),
        kanji
            .mnemonic
            .as_deref()
            .map(|m| format!("mnemonic: {}", one_line(m))),
        kanji
            .etymology
            .as_deref()
            .map(|e| format!("etymology: {}", one_line(e))),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn list(readings: &[&Reading]) -> String {
    readings
        .iter()
        .map(|r| match &r.frequency {
            Some(f) => format!("{} {f}", r.reading),
            None => r.reading.clone(),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn joined(parts: impl Iterator<Item = String>) -> String {
    parts.collect::<Vec<_>>().join(" · ")
}

/// Collapses all whitespace runs (including newlines) to single spaces.
fn one_line(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}
