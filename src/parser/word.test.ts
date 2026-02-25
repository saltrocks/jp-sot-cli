import { describe, test, expect, beforeEach, mock } from "bun:test";
import { readFileSync } from "fs";
import { join } from "path";

const fixturesDir = join(import.meta.dir, "../__fixtures__");

function loadFixture(name: string): string {
  return readFileSync(join(fixturesDir, name), "utf-8");
}

const kanjiFixtures: Record<string, string> = {
  宣: loadFixture("kanji-sen.html"),
  宀: loadFixture("kanji-ukanmuri.html"),
  亘: loadFixture("kanji-wataru.html"),
  一: loadFixture("kanji-ichi.html"),
  旦: loadFixture("kanji-tan.html"),
  日: loadFixture("kanji-nichi.html"),
  言: loadFixture("kanji-iu.html"),
};

const wordFixtures: Record<string, string> = {
  宣言: loadFixture("word-sengen.html"),
};

const kanjipediaSearchFixtures: Record<string, string> = {
  宣: loadFixture("kanjipedia-search-sen.html"),
  宀: loadFixture("kanjipedia-search-ukanmuri.html"),
  亘: loadFixture("kanjipedia-search-wataru.html"),
  一: loadFixture("kanjipedia-search-ichi.html"),
  旦: loadFixture("kanjipedia-search-tan.html"),
  日: loadFixture("kanjipedia-search-nichi.html"),
  言: loadFixture("kanjipedia-search-iu.html"),
};

const kanjipediaKanjiFixtures: Record<string, string> = {
  "0004051500": loadFixture("kanjipedia-kanji-sen.html"),
  "0000209400": loadFixture("kanjipedia-kanji-ichi.html"),
  "0004598400": loadFixture("kanjipedia-kanji-tan.html"),
  "0005451300": loadFixture("kanjipedia-kanji-nichi.html"),
  "0001960600": loadFixture("kanjipedia-kanji-iu.html"),
};

mock.module("../scraper.js", () => ({
  fetchPage: mock(async (url: string) => {
    const jpdbKanjiMatch = url.match(/jpdb\.io\/kanji\/(.+?)(?:#|$)/);
    if (jpdbKanjiMatch) {
      const char = decodeURIComponent(jpdbKanjiMatch[1]);
      const html = kanjiFixtures[char];
      if (!html) throw new Error(`No fixture for kanji: ${char}`);
      return html;
    }

    const wordMatch = url.match(/jpdb\.io\/search\?q=([^&#]+)/);
    if (wordMatch) {
      const query = decodeURIComponent(wordMatch[1]);
      const html = wordFixtures[query];
      if (!html) throw new Error(`No fixture for word: ${query}`);
      return html;
    }

    const kanjipediaSearchMatch = url.match(/kanjipedia\.jp\/search\?k=([^&]+)/);
    if (kanjipediaSearchMatch) {
      const char = decodeURIComponent(kanjipediaSearchMatch[1]);
      const html = kanjipediaSearchFixtures[char];
      if (!html) throw new Error(`No kanjipedia search fixture for: ${char}`);
      return html;
    }

    const kanjipediaKanjiMatch = url.match(/kanjipedia\.jp\/kanji\/(\d+)/);
    if (kanjipediaKanjiMatch) {
      const id = kanjipediaKanjiMatch[1];
      const html = kanjipediaKanjiFixtures[id];
      if (!html) throw new Error(`No kanjipedia kanji fixture for id: ${id}`);
      return html;
    }

    throw new Error(`Unknown URL pattern: ${url}`);
  }),
  wordSearchUrl: (query: string) =>
    `https://jpdb.io/search?q=${encodeURIComponent(query)}&lang=english#a`,
  kanjiUrl: (character: string) =>
    `https://jpdb.io/kanji/${encodeURIComponent(character)}#a`,
  kanjipediaSearchUrl: (character: string) =>
    `https://www.kanjipedia.jp/search?k=${encodeURIComponent(character)}&kt=1&sk=leftHand`,
  kanjipediaKanjiUrl: (id: string) =>
    `https://www.kanjipedia.jp/kanji/${id}`,
}));

const { parseWord } = await import("./word.js");

describe("parseWord", () => {
  test("parses word spelling and reading", async () => {
    const result = await parseWord("宣言");

    expect(result.type).toBe("word");
    expect(result.word).toBe("宣言");
    expect(result.reading).toBe("せんげん");
  });

  test("parses meanings with part of speech", async () => {
    const result = await parseWord("宣言");

    expect(result.meanings).toHaveLength(1);
    expect(result.meanings[0].partOfSpeech).toBe("Noun, Verb (する)");
    expect(result.meanings[0].definitions).toEqual([
      "declaration",
      "proclamation",
      "announcement",
    ]);
  });

  test("parses pitch accent", async () => {
    const result = await parseWord("宣言");

    expect(result.pitchAccent).toBe("せ↑んげ↓ん");
  });

  test("parses frequency tag", async () => {
    const result = await parseWord("宣言");

    expect(result.frequency).toBe("Top 2000");
  });

  test("parses composed-of vocabulary", async () => {
    const result = await parseWord("宣言");

    expect(result.composedOf).toHaveLength(2);
    expect(result.composedOf[0]).toEqual({
      word: "宣",
      reading: "せん",
      meaning: "imperial order;  imperial decree",
    });
    expect(result.composedOf[1]).toEqual({
      word: "言",
      reading: "げん",
      meaning: "word;  remark;  statement",
    });
  });

  test("includes full kanji decomposition for each kanji used", async () => {
    const result = await parseWord("宣言");

    expect(result.kanji).toHaveLength(2);

    const sen = result.kanji[0];
    expect(sen.character).toBe("宣");
    expect(sen.keyword).toBe("formally declare");
    expect(sen.components).toHaveLength(2);

    const iu = result.kanji[1];
    expect(iu.character).toBe("言");
    expect(iu.keyword).toBe("say");
    expect(iu.mnemonic).toBe(
      "A cryptic pictograph of a face trying to say something."
    );
  });

  test("kanji decomposition is fully recursive", async () => {
    const result = await parseWord("宣言");

    const sen = result.kanji[0];
    const wataru = sen.components[1];
    expect(wataru.character).toBe("亘");

    const tan = wataru.components[1];
    expect(tan.character).toBe("旦");
    expect(tan.mnemonic).toBe(
      "A pictograph of a sun rising over the horizon which signifies the start of a day."
    );

    const nichi = tan.components[0];
    expect(nichi.character).toBe("日");
    expect(nichi.keyword).toBe("sun");
  });

  test("overall JSON structure is LLM-friendly", async () => {
    const result = await parseWord("宣言");

    expect(result).toMatchObject({
      type: "word",
      word: "宣言",
      reading: "せんげん",
      meanings: expect.arrayContaining([
        expect.objectContaining({
          partOfSpeech: expect.any(String),
          definitions: expect.any(Array),
        }),
      ]),
      kanji: expect.arrayContaining([
        expect.objectContaining({
          type: "kanji",
          character: expect.any(String),
          keyword: expect.any(String),
          readings: expect.any(Array),
          components: expect.any(Array),
        }),
      ]),
    });
  });
});
