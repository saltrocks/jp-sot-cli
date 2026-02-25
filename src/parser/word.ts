import { parse, type HTMLElement } from "node-html-parser";
import { fetchPage, wordSearchUrl } from "../scraper.js";
import { parseKanji, resetVisited } from "./kanji.js";
import type { KanjiResult, Meaning, WordResult } from "../types.js";
import { extractKanjiFromLinks, parseRuby } from "./utils.js";

export async function parseWord(query: string): Promise<WordResult> {
  const html = await fetchPage(wordSearchUrl(query));
  const root = parse(html);

  const result = root.querySelector(".result.vocabulary");
  if (!result) {
    throw new Error(`No vocabulary result found for "${query}"`);
  }

  const { word, reading } = extractSpelling(result);
  const meanings = extractMeanings(result);
  const pitchAccent = extractPitchAccent(result);
  const frequency = extractFrequency(result);
  const composedOf = extractComposedOf(result);
  const kanjiChars = extractKanjiFromLinks(result, "Kanji used");

  resetVisited();
  const kanji = await Promise.all(kanjiChars.map((ch) => parseKanji(ch)));

  return {
    type: "word",
    word,
    reading,
    meanings,
    ...(pitchAccent ? { pitchAccent } : {}),
    ...(frequency ? { frequency } : {}),
    composedOf,
    kanji,
  };
}

function extractSpelling(root: HTMLElement): {
  word: string;
  reading: string;
} {
  const spelling = root.querySelector(".primary-spelling .spelling");
  if (!spelling) return { word: "", reading: "" };

  const ruby = spelling.querySelector("ruby");
  if (!ruby) {
    return { word: spelling.text.trim(), reading: spelling.text.trim() };
  }

  return parseRuby(ruby);
}

function extractMeanings(root: HTMLElement): Meaning[] {
  const meanings: Meaning[] = [];
  const section = root.querySelector(".subsection-meanings .subsection");
  if (!section) return meanings;

  let currentPos: string[] = [];
  const children = section.childNodes;

  for (const child of children) {
    const el = child as HTMLElement;
    if (el.classList?.contains("part-of-speech")) {
      currentPos = el
        .querySelectorAll("div")
        .map((d: HTMLElement) => d.text.trim());
    } else if (el.classList?.contains("description")) {
      const defs = el.text
        .trim()
        .replace(/^\d+\.\s*/, "")
        .split(/;\s*/)
        .map((d: string) => d.trim())
        .filter(Boolean);
      meanings.push({
        partOfSpeech: currentPos.join(", "),
        definitions: defs,
      });
    }
  }

  return meanings;
}

function extractPitchAccent(root: HTMLElement): string | undefined {
  const section = root.querySelector(".subsection-pitch-accent .subsection");
  if (!section) return undefined;

  const parts = section.querySelectorAll(
    "div[style*='word-break'] > div"
  );
  if (parts.length === 0) return undefined;

  let accent = "";
  for (const part of parts) {
    const style = part.getAttribute("style") || "";
    const text = part.text.trim();
    const isHigh = style.includes("pitch-high");
    const isLow = style.includes("pitch-low");

    if (isLow && accent === "") {
      accent += text + "↑";
    } else if (isHigh) {
      accent += text;
    } else if (isLow && accent !== "") {
      accent += "↓" + text;
    } else {
      accent += text;
    }
  }

  return accent || undefined;
}

function extractFrequency(root: HTMLElement): string | undefined {
  const tag = root.querySelector(".tag");
  return tag?.text.trim() || undefined;
}

function extractComposedOf(
  root: HTMLElement
): { word: string; reading: string; meaning: string }[] {
  const results: { word: string; reading: string; meaning: string }[] = [];
  const section = root.querySelector(
    ".subsection-composed-of-vocabulary .subsection"
  );
  if (!section) return results;

  const items = section.querySelectorAll(".composed-of");
  for (const item of items) {
    const jp = item.querySelector(".jp");
    const desc = item.querySelector(".description");
    if (!jp || !desc) continue;

    const ruby = jp.querySelector("ruby");
    let word: string;
    let reading: string;
    if (ruby) {
      ({ word, reading } = parseRuby(ruby));
    } else {
      word = jp.text.trim();
      reading = jp.text.trim();
    }

    results.push({
      word,
      reading,
      meaning: desc.text.trim(),
    });
  }

  return results;
}
