//! Turns queries into result trees.
//!
//! Resolution happens in two phases so that output is deterministic while
//! fetching stays concurrent:
//!
//! 1. **Crawl**: fetch every kanji reachable from the roots (each one once),
//!    concurrently, into a flat map.
//! 2. **Assemble**: walk the decomposition depth-first in page order. The
//!    first occurrence of a kanji is expanded; later ones become
//!    "(see above)" stubs.

use std::collections::{HashMap, HashSet};

use futures::future::LocalBoxFuture;
use futures::stream::{FuturesUnordered, StreamExt};

use crate::fetch::Fetcher;
use crate::model::{Entry, Kanji, Word};
use crate::page::Page;
use crate::parse::{jpdb, kanjipedia};
use crate::{Error, is_single_kanji};

pub struct Dictionary<F> {
    fetcher: F,
}

impl<F: Fetcher> Dictionary<F> {
    pub fn new(fetcher: F) -> Self {
        Self { fetcher }
    }

    /// Looks `query` up as a kanji if it is a single one, otherwise as a word.
    pub async fn lookup(&self, query: &str) -> Result<Entry, Error> {
        if is_single_kanji(query) {
            self.kanji(query).await.map(Entry::Kanji)
        } else {
            self.word(query).await.map(Entry::Word)
        }
    }

    pub async fn kanji(&self, character: &str) -> Result<Kanji, Error> {
        let crawled = self.crawl([character.to_owned()]).await?;
        Ok(Assembler::new(&crawled).kanji(character))
    }

    pub async fn word(&self, query: &str) -> Result<Word, Error> {
        let html = self
            .fetcher
            .fetch(&Page::JpdbSearch(query.to_owned()))
            .await?;
        let page = jpdb::parse_word(&html).ok_or_else(|| Error::NoResult(query.to_owned()))?;

        let crawled = self.crawl(page.kanji.iter().cloned()).await?;
        let mut assembler = Assembler::new(&crawled);
        let kanji = page.kanji.iter().map(|k| assembler.kanji(k)).collect();

        Ok(Word {
            kind: (),
            word: page.word,
            reading: page.reading,
            meanings: page.meanings,
            pitch_accent: page.pitch_accent,
            frequency: page.frequency,
            composed_of: page.composed_of,
            kanji,
        })
    }

    /// Fetches every kanji reachable from `roots` through their components.
    ///
    /// jpdb pages and kanjipedia etymologies are fetched as independent tasks
    /// so component discovery never waits on the (slower) etymology lookups.
    async fn crawl(&self, roots: impl IntoIterator<Item = String>) -> Result<Crawled, Error> {
        enum Fetched {
            Page(String, Result<jpdb::KanjiPage, Error>),
            Etymology(String, Option<String>),
        }

        let mut tasks: FuturesUnordered<LocalBoxFuture<'_, Fetched>> = FuturesUnordered::new();
        let mut seen = HashSet::new();
        let mut schedule = |character: String, tasks: &mut FuturesUnordered<_>| {
            if !seen.insert(character.clone()) {
                return;
            }
            let c = character.clone();
            tasks.push(Box::pin(async move {
                let page = self.fetcher.fetch(&Page::JpdbKanji(c.clone())).await;
                Fetched::Page(c, page.map(|html| jpdb::parse_kanji(&html)))
            }) as LocalBoxFuture<'_, _>);
            tasks.push(Box::pin(async move {
                let etymology = self.etymology(&character).await;
                Fetched::Etymology(character, etymology)
            }));
        };

        for root in roots {
            schedule(root, &mut tasks);
        }

        let mut crawled = Crawled::default();
        while let Some(fetched) = tasks.next().await {
            match fetched {
                Fetched::Page(character, page) => {
                    let page = page?;
                    for component in &page.components {
                        schedule(component.clone(), &mut tasks);
                    }
                    crawled.pages.insert(character, page);
                }
                Fetched::Etymology(character, Some(etymology)) => {
                    crawled.etymologies.insert(character, etymology);
                }
                Fetched::Etymology(_, None) => {}
            }
        }
        Ok(crawled)
    }

    /// Best-effort なりたち lookup; any failure just means no etymology.
    async fn etymology(&self, character: &str) -> Option<String> {
        let search = self
            .fetcher
            .fetch(&Page::KanjipediaSearch(character.to_owned()))
            .await
            .ok()?;
        let id = kanjipedia::parse_search(&search, character)?;
        let page = self.fetcher.fetch(&Page::KanjipediaKanji(id)).await.ok()?;
        kanjipedia::parse_etymology(&page)
    }
}

#[derive(Default)]
struct Crawled {
    pages: HashMap<String, jpdb::KanjiPage>,
    etymologies: HashMap<String, String>,
}

/// Builds kanji trees from crawled data, expanding each kanji only once
/// across every tree it builds.
struct Assembler<'a> {
    crawled: &'a Crawled,
    expanded: HashSet<&'a str>,
}

impl<'a> Assembler<'a> {
    fn new(crawled: &'a Crawled) -> Self {
        Self {
            crawled,
            expanded: HashSet::new(),
        }
    }

    fn kanji(&mut self, character: &str) -> Kanji {
        // A successful crawl contains every kanji reachable from its roots.
        let (character, page) = self
            .crawled
            .pages
            .get_key_value(character)
            .expect("kanji was crawled");
        if !self.expanded.insert(character) {
            return Kanji::see_above(character.as_str());
        }

        Kanji {
            kind: (),
            character: character.clone(),
            keyword: page.keyword.clone(),
            frequency: page.frequency.clone(),
            kanji_type: page.kanji_type.clone(),
            kanken: page.kanken.clone(),
            heisig: page.heisig,
            readings: page.readings.clone(),
            mnemonic: page.mnemonic.clone(),
            etymology: self.crawled.etymologies.get(character).cloned(),
            components: page.components.iter().map(|c| self.kanji(c)).collect(),
        }
    }
}
