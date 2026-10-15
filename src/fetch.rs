//! Fetching pages: the [`Fetcher`] abstraction and its HTTP implementation.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::time::Instant;

use crate::Error;
use crate::page::Page;

/// Source of raw HTML for a [`Page`]. Swapped for fixtures in tests.
#[allow(async_fn_in_trait)]
pub trait Fetcher {
    async fn fetch(&self, page: &Page) -> Result<String, Error>;
}

impl<F: Fetcher + ?Sized> Fetcher for &F {
    async fn fetch(&self, page: &Page) -> Result<String, Error> {
        (**self).fetch(page).await
    }
}

const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

/// Fetches pages over HTTP, spacing requests to the same host by `delay`.
pub struct HttpFetcher {
    client: reqwest::Client,
    throttle: Throttle,
}

impl HttpFetcher {
    pub const DEFAULT_DELAY: Duration = Duration::from_millis(300);

    pub fn new(delay: Duration) -> Result<Self, Error> {
        use reqwest::header::{ACCEPT, ACCEPT_LANGUAGE, HeaderMap, HeaderValue};

        let headers = HeaderMap::from_iter([
            (
                ACCEPT,
                HeaderValue::from_static("text/html,application/xhtml+xml"),
            ),
            (ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9")),
        ]);
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .build()?;
        Ok(Self {
            client,
            throttle: Throttle::new(delay),
        })
    }
}

impl Fetcher for HttpFetcher {
    async fn fetch(&self, page: &Page) -> Result<String, Error> {
        let url = page.url();
        self.throttle.wait(url.host_str().unwrap_or_default()).await;
        let response = self.client.get(url.clone()).send().await?;
        let status = response.status();
        if !status.is_success() {
            return Err(Error::Status { url, status });
        }
        Ok(response.text().await?)
    }
}

/// Per-host rate limiter: callers for the same host are released one at a
/// time, at least `delay` apart.
struct Throttle {
    delay: Duration,
    hosts: Mutex<HashMap<String, Arc<tokio::sync::Mutex<Option<Instant>>>>>,
}

impl Throttle {
    fn new(delay: Duration) -> Self {
        Self {
            delay,
            hosts: Mutex::default(),
        }
    }

    async fn wait(&self, host: &str) {
        let slot = {
            let mut hosts = self.hosts.lock().unwrap_or_else(|e| e.into_inner());
            Arc::clone(hosts.entry(host.to_owned()).or_default())
        };
        let mut last = slot.lock().await;
        if let Some(last) = *last {
            tokio::time::sleep_until(last + self.delay).await;
        }
        *last = Some(Instant::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(start_paused = true)]
    async fn throttle_spaces_same_host_only() {
        let throttle = Throttle::new(Duration::from_millis(300));
        let start = Instant::now();

        futures::join!(throttle.wait("a"), throttle.wait("a"), throttle.wait("b"));
        assert_eq!(start.elapsed(), Duration::from_millis(300));

        throttle.wait("a").await;
        assert_eq!(start.elapsed(), Duration::from_millis(600));
    }
}
