import { describe, test, expect, beforeEach, mock } from "bun:test";
import { readFileSync } from "fs";
import { join } from "path";

const fixturesDir = join(import.meta.dir, "../__fixtures__");

function loadFixture(name: string): string {
  return readFileSync(join(fixturesDir, name), "utf-8");
}

const jpdbFixtures: Record<string, string> = {
  宣: loadFixture("kanji-sen.html"),
  宀: loadFixture("kanji-ukanmuri.html"),
  亘: loadFixture("kanji-wataru.html"),
  一: loadFixture("kanji-ichi.html"),
  旦: loadFixture("kanji-tan.html"),
  日: loadFixture("kanji-nichi.html"),
  言: loadFixture("kanji-iu.html"),
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
    const jpdbMatch = url.match(/jpdb\.io\/kanji\/(.+?)(?:#|$)/);
    if (jpdbMatch) {
      const char = decodeURIComponent(jpdbMatch[1]);
      const html = jpdbFixtures[char];
      if (!html) throw new Error(`No fixture for kanji: ${char} (url: ${url})`);
      return html;
    }

    const kanjipediaSearchMatch = url.match(/kanjipedia\.jp\/search\?k=([^&]+)/);
    if (kanjipediaSearchMatch) {
      const char = decodeURIComponent(kanjipediaSearchMatch[1]);
      const html = kanjipediaSearchFixtures[char];
      if (!html) throw new Error(`No kanjipedia search fixture for: ${char} (url: ${url})`);
      return html;
    }

    const kanjipediaKanjiMatch = url.match(/kanjipedia\.jp\/kanji\/(\d+)/);
    if (kanjipediaKanjiMatch) {
      const id = kanjipediaKanjiMatch[1];
      const html = kanjipediaKanjiFixtures[id];
      if (!html) throw new Error(`No kanjipedia kanji fixture for id: ${id} (url: ${url})`);
      return html;
    }

    throw new Error(`No fixture for url: ${url}`);
  }),
  kanjiUrl: (character: string) =>
    `https://jpdb.io/kanji/${encodeURIComponent(character)}#a`,
  wordSearchUrl: (query: string) =>
    `https://jpdb.io/search?q=${encodeURIComponent(query)}&lang=english#a`,
  kanjipediaSearchUrl: (character: string) =>
    `https://www.kanjipedia.jp/search?k=${encodeURIComponent(character)}&kt=1&sk=leftHand`,
  kanjipediaKanjiUrl: (id: string) =>
    `https://www.kanjipedia.jp/kanji/${id}`,
}));

const { parseKanji, resetVisited } = await import("./kanji.js");

describe("parseKanji", () => {
  beforeEach(() => {
    resetVisited();
  });

  test("parses basic kanji info for 宣", async () => {
    const result = await parseKanji("宣");

    expect(result.type).toBe("kanji");
    expect(result.character).toBe("宣");
    expect(result.keyword).toBe("formally declare");
    expect(result.frequency).toBe("Top 1100-1200");
    expect(result.kanjiType).toBe("Kyōiku (6th grade)");
    expect(result.kanken).toBe("Level 5");
    expect(result.heisig).toBe(200);
  });

  test("extracts common readings with frequency", async () => {
    const result = await parseKanji("宣");

    expect(result.readings).toContainEqual({
      reading: "せん",
      frequency: "97%",
    });
    expect(result.readings).toContainEqual({
      reading: "のたも",
      frequency: "1%",
    });
    expect(result.readings).toContainEqual({
      reading: "のたま",
      frequency: "1%",
    });
  });

  test("extracts rare readings without frequency", async () => {
    const result = await parseKanji("宣");

    expect(result.readings).toContainEqual({ reading: "せ" });
    expect(result.readings).toContainEqual({ reading: "ぜん" });
  });

  test("extracts mnemonic when present", async () => {
    resetVisited();
    const result = await parseKanji("宀");
    expect(result.mnemonic).toBe("A pictograph of a roof with a chimney.");
  });

  test("omits mnemonic when absent", async () => {
    const result = await parseKanji("宣");
    expect(result.mnemonic).toBeUndefined();
  });

  test("extracts etymology from kanjipedia", async () => {
    const result = await parseKanji("宣");
    expect(result.etymology).toBe(
      "形声。宀と、音符亘(セン)とから成る。天子が暮らす正殿の意を表す。借りて、あきらかの意に用いる。"
    );
  });

  test("extracts etymology for primitive kanji", async () => {
    const result = await parseKanji("一");
    expect(result.etymology).toBe(
      "指事。横線一本で、「ひとつ」、ひいて、数のはじめの意を表す。"
    );
  });

  test("omits etymology when not found on kanjipedia", async () => {
    resetVisited();
    const result = await parseKanji("宀");
    expect(result.etymology).toBeUndefined();
  });

  test("recursively decomposes components", async () => {
    const result = await parseKanji("宣");

    expect(result.components).toHaveLength(2);
    expect(result.components[0].character).toBe("宀");
    expect(result.components[0].keyword).toBe("roof with a chimney");
    expect(result.components[0].components).toHaveLength(0);

    expect(result.components[1].character).toBe("亘");
    expect(result.components[1].keyword).toBe("span");
    expect(result.components[1].components).toHaveLength(2);

    const ichi = result.components[1].components[0];
    expect(ichi.character).toBe("一");
    expect(ichi.keyword).toBe("one");

    const tan = result.components[1].components[1];
    expect(tan.character).toBe("旦");
    expect(tan.keyword).toBe("start of a day");
    expect(tan.components).toHaveLength(2);
    expect(tan.components[0].character).toBe("日");
    expect(tan.components[0].keyword).toBe("sun");
  });

  test("handles cycle detection for repeated components", async () => {
    const result = await parseKanji("宣");

    const tan = result.components[1].components[1];
    const secondIchi = tan.components[1];
    expect(secondIchi.character).toBe("一");
    expect(secondIchi.keyword).toBe("(see above)");
    expect(secondIchi.readings).toHaveLength(0);
    expect(secondIchi.components).toHaveLength(0);
  });

  test("parses a primitive kanji with no components", async () => {
    const result = await parseKanji("一");

    expect(result.character).toBe("一");
    expect(result.keyword).toBe("one");
    expect(result.heisig).toBe(1);
    expect(result.mnemonic).toBe("A pictograph of a one line.");
    expect(result.components).toHaveLength(0);
  });

  test("parses kanji without type field", async () => {
    const result = await parseKanji("宀");

    expect(result.kanjiType).toBeUndefined();
    expect(result.frequency).toBe("Never used");
  });
});
