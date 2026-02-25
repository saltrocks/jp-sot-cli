import { describe, test, expect, beforeEach, mock } from "bun:test";
import { readFileSync } from "fs";
import { join } from "path";

const fixturesDir = join(import.meta.dir, "__fixtures__");

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

mock.module("./scraper.js", () => ({
  fetchPage: mock(async (url: string) => {
    const kanjiMatch = url.match(/\/kanji\/(.+?)(?:#|$)/);
    if (kanjiMatch) {
      const char = decodeURIComponent(kanjiMatch[1]);
      const html = kanjiFixtures[char];
      if (!html) throw new Error(`No fixture for kanji: ${char}`);
      return html;
    }
    const wordMatch = url.match(/[?&]q=([^&#]+)/);
    if (wordMatch) {
      const query = decodeURIComponent(wordMatch[1]);
      const html = wordFixtures[query];
      if (!html) throw new Error(`No fixture for word: ${query}`);
      return html;
    }
    throw new Error(`Unknown URL pattern: ${url}`);
  }),
  wordSearchUrl: (query: string) =>
    `https://jpdb.io/search?q=${encodeURIComponent(query)}&lang=english#a`,
  kanjiUrl: (character: string) =>
    `https://jpdb.io/kanji/${encodeURIComponent(character)}#a`,
}));

describe("CLI integration", () => {
  test("kanji lookup via subprocess produces valid JSON", async () => {
    const proc = Bun.spawn(["bun", "run", "src/index.ts", "宣"], {
      cwd: import.meta.dir + "/..",
      stdout: "pipe",
      stderr: "pipe",
    });

    const stdout = await new Response(proc.stdout).text();
    const exitCode = await proc.exited;

    if (exitCode !== 0) {
      const stderr = await new Response(proc.stderr).text();
      console.error("stderr:", stderr);
    }

    expect(exitCode).toBe(0);

    const result = JSON.parse(stdout);
    expect(result.type).toBe("kanji");
    expect(result.character).toBe("宣");
    expect(result.keyword).toBe("formally declare");
    expect(result.components.length).toBeGreaterThan(0);
  }, 15000);

  test("word lookup via subprocess produces valid JSON", async () => {
    const proc = Bun.spawn(["bun", "run", "src/index.ts", "宣言"], {
      cwd: import.meta.dir + "/..",
      stdout: "pipe",
      stderr: "pipe",
    });

    const stdout = await new Response(proc.stdout).text();
    const exitCode = await proc.exited;

    if (exitCode !== 0) {
      const stderr = await new Response(proc.stderr).text();
      console.error("stderr:", stderr);
    }

    expect(exitCode).toBe(0);

    const result = JSON.parse(stdout);
    expect(result.type).toBe("word");
    expect(result.word).toBe("宣言");
    expect(result.reading).toBe("せんげん");
    expect(result.kanji.length).toBeGreaterThan(0);
  }, 30000);

  test("no argument shows usage and exits with code 1", async () => {
    const proc = Bun.spawn(["bun", "run", "src/index.ts"], {
      cwd: import.meta.dir + "/..",
      stdout: "pipe",
      stderr: "pipe",
    });

    const stderr = await new Response(proc.stderr).text();
    const exitCode = await proc.exited;

    expect(exitCode).toBe(1);
    expect(stderr).toContain("Usage:");
  }, 10000);
});
