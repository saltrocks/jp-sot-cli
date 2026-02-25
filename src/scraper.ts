const HEADERS = {
  "User-Agent":
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
  Accept: "text/html,application/xhtml+xml",
  "Accept-Language": "en-US,en;q=0.9",
};

const DELAY_MS = 300;

class ThrottleQueue {
  private lastFetch = 0;
  private pending: Promise<void> = Promise.resolve();

  async acquire(): Promise<void> {
    this.pending = this.pending.then(async () => {
      const now = Date.now();
      const elapsed = now - this.lastFetch;
      if (elapsed < DELAY_MS) {
        await new Promise((r) => setTimeout(r, DELAY_MS - elapsed));
      }
      this.lastFetch = Date.now();
    });
    return this.pending;
  }
}

const queues = new Map<string, ThrottleQueue>();

function queueForUrl(url: string): ThrottleQueue {
  const domain = new URL(url).hostname;
  let queue = queues.get(domain);
  if (!queue) {
    queue = new ThrottleQueue();
    queues.set(domain, queue);
  }
  return queue;
}

export async function fetchPage(url: string): Promise<string> {
  await queueForUrl(url).acquire();
  const res = await fetch(url, { headers: HEADERS });
  if (!res.ok) {
    throw new Error(`Failed to fetch ${url}: ${res.status} ${res.statusText}`);
  }
  return res.text();
}

export function wordSearchUrl(query: string): string {
  return `https://jpdb.io/search?q=${encodeURIComponent(query)}&lang=english#a`;
}

export function kanjiUrl(character: string): string {
  return `https://jpdb.io/kanji/${encodeURIComponent(character)}#a`;
}

export function kanjipediaSearchUrl(character: string): string {
  return `https://www.kanjipedia.jp/search?k=${encodeURIComponent(character)}&kt=1&sk=leftHand`;
}

export function kanjipediaKanjiUrl(id: string): string {
  return `https://www.kanjipedia.jp/kanji/${id}`;
}
