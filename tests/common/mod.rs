use std::cell::RefCell;
use std::path::PathBuf;

use jp_sot::page::Page;
use jp_sot::{Dictionary, Error, Fetcher};

/// Serves saved pages from `tests/fixtures`, recording every request.
#[derive(Default)]
pub struct FixtureFetcher {
    pub requests: RefCell<Vec<Page>>,
}

impl FixtureFetcher {
    fn fixture(page: &Page) -> Option<&'static str> {
        Some(match page {
            Page::JpdbSearch(q) if q == "宣言" => "word-sengen",
            Page::JpdbSearch(_) => return None,
            Page::JpdbKanji(k) => match k.as_str() {
                "宣" => "kanji-sen",
                "宀" => "kanji-ukanmuri",
                "亘" => "kanji-wataru",
                "一" => "kanji-ichi",
                "旦" => "kanji-tan",
                "日" => "kanji-nichi",
                "言" => "kanji-iu",
                _ => return None,
            },
            Page::KanjipediaSearch(k) => match k.as_str() {
                "宣" => "kanjipedia-search-sen",
                "宀" => "kanjipedia-search-ukanmuri",
                "亘" => "kanjipedia-search-wataru",
                "一" => "kanjipedia-search-ichi",
                "旦" => "kanjipedia-search-tan",
                "日" => "kanjipedia-search-nichi",
                "言" => "kanjipedia-search-iu",
                _ => return None,
            },
            Page::KanjipediaKanji(id) => match id.as_str() {
                "0004051500" => "kanjipedia-kanji-sen",
                "0000209400" => "kanjipedia-kanji-ichi",
                "0004598400" => "kanjipedia-kanji-tan",
                "0005451300" => "kanjipedia-kanji-nichi",
                "0001960600" => "kanjipedia-kanji-iu",
                _ => return None,
            },
        })
    }
}

impl Fetcher for FixtureFetcher {
    async fn fetch(&self, page: &Page) -> Result<String, Error> {
        self.requests.borrow_mut().push(page.clone());
        let name = Self::fixture(page).ok_or_else(|| Error::Status {
            url: page.url(),
            status: reqwest::StatusCode::NOT_FOUND,
        })?;
        let path: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "tests",
            "fixtures",
            &format!("{name}.html"),
        ]
        .iter()
        .collect();
        Ok(std::fs::read_to_string(path).expect("fixture is readable"))
    }
}

pub fn dictionary() -> Dictionary<FixtureFetcher> {
    Dictionary::new(FixtureFetcher::default())
}

pub fn block_on<T>(f: impl Future<Output = T>) -> T {
    tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap()
        .block_on(f)
}
