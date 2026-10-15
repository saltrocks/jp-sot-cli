# jp-sot

A command-line tool that looks up a Japanese word or kanji and prints its
full breakdown: meanings, readings, pitch accent, and a recursive
decomposition of every kanji into its components, down to the primitives.

Data is scraped from [jpdb.io](https://jpdb.io) (meanings, readings,
keywords, mnemonics, decomposition) and [kanjipedia.jp](https://www.kanjipedia.jp)
(etymology).

## Installation

Requires a Rust toolchain with edition 2024 support (Rust 1.85 or newer).

```sh
cargo install --path .
```

Or build without installing:

```sh
cargo build --release
./target/release/jp-sot 宣言
```

## Usage

```
jp-sot [OPTIONS] <QUERY>
```

A query that is a single kanji is looked up as a kanji; anything else is
looked up as a word.

```sh
jp-sot 宣言    # look up a word
jp-sot 宣      # look up a single kanji
```

| Option                | Description                                         |
| --------------------- | --------------------------------------------------- |
| `--format text\|json` | Output format (default: `text`)                     |
| `--compact`           | Print JSON on a single line (with `--format json`)  |

### Text output

The default format is an indented outline, with the kanji decomposition drawn
as a tree. Kanji that appear more than once are expanded only the first time.

```
$ jp-sot 宣言
宣言【せんげん】 · frequency: Top 2000 · pitch: せ↑んげ↓ん
Meanings:
  1. [Noun, Verb (する)] declaration; proclamation; announcement
Composed of:
  宣【せん】 imperial order; imperial decree
  言【げん】 word; remark; statement
Kanji:
  宣 "formally declare" · Kyōiku (6th grade) · Kanken 5 · Heisig #200 · frequency: Top 1100-1200
    │ readings: せん 97%, のたも 1%, のたま 1% · rare readings: せ, ぜん, の, ぜ, のり
    │ etymology: 形声。宀と、音符亘(セン)とから成る。天子が暮らす正殿の意を表す。借りて、あきらかの意に用いる。
    ├─ 宀 "roof with a chimney" · Kanken 1 · frequency: Never used
    │    rare readings: べん
    │    mnemonic: A pictograph of a roof with a chimney.
    └─ 亘 "span" · Jinmeiyō · Kanken Pre-1 · Heisig #32 · frequency: Top 2800-2900
       │ readings: わた 100% · rare readings: こう
       ├─ 一 "one" · Kyōiku (1st grade) · Kanken 10 · Heisig #1 · frequency: Top 100
       │    ...
       └─ 旦 "start of a day" · Jōyō · Kanken 2 · Heisig #30 · frequency: Top 1000-1100
          │ ...
          ├─ 日 "sun" · Kyōiku (1st grade) · Kanken 10 · Heisig #12 · frequency: Top 100
          │    ...
          └─ 一 (expanded above)
  言 "say" · Kyōiku (2nd grade) · Kanken 9 · Heisig #357 · frequency: Top 100
      ...
```

### JSON output

`--format json` emits the same data as a tree for use in scripts. Fields with
no value are omitted, and repeated kanji are replaced by a stub whose
`keyword` is `"(see above)"`.

```json
{
  "type": "word",
  "word": "宣言",
  "reading": "せんげん",
  "meanings": [
    {
      "partOfSpeech": "Noun, Verb (する)",
      "definitions": ["declaration", "proclamation", "announcement"]
    }
  ],
  "pitchAccent": "せ↑んげ↓ん",
  "frequency": "Top 2000",
  "composedOf": [
    { "word": "宣", "reading": "せん", "meaning": "imperial order;  imperial decree" }
  ],
  "kanji": [
    {
      "type": "kanji",
      "character": "宣",
      "keyword": "formally declare",
      "frequency": "Top 1100-1200",
      "kanjiType": "Kyōiku (6th grade)",
      "kanken": "Level 5",
      "heisig": 200,
      "readings": [{ "reading": "せん", "frequency": "97%" }],
      "etymology": "形声。…",
      "components": []
    }
  ]
}
```

(Abridged; arrays are truncated.)

## Development

```sh
cargo test
```

The tests run fully offline: they swap the HTTP fetcher for one that serves
saved pages from `tests/fixtures/`.

The crate is split into:

- `page`: the remote pages the tool reads and their URLs
- `fetch`: the `Fetcher` trait and a throttled HTTP implementation
- `parse`: HTML-to-data extraction for jpdb.io and kanjipedia.jp
- `resolve`: crawls all reachable kanji concurrently, then assembles the tree
- `model`: the output types, serialized as JSON
- `render`: the plain-text renderer
