import { parse, type HTMLElement } from "node-html-parser";
import { fetchPage, kanjiUrl, kanjipediaSearchUrl, kanjipediaKanjiUrl } from "../scraper.js";
import type { KanjiResult, Reading } from "../types.js";
import { extractLabeledValue, extractKanjiFromLinks } from "./utils.js";

const visited = new Set<string>();

export function resetVisited(): void {
  visited.clear();
}

export async function parseKanji(character: string): Promise<KanjiResult> {
  if (visited.has(character)) {
    return {
      type: "kanji",
      character,
      keyword: "(see above)",
      readings: [],
      components: [],
    };
  }
  visited.add(character);

  const html = await fetchPage(kanjiUrl(character));
  const root = parse(html);

  const keyword = extractLabeledValue(root, "Keyword") || "";
  const info = extractInfo(root);
  const readings = extractReadings(root);
  const mnemonic = extractLabeledValue(root, "Mnemonic");
  const componentChars = extractKanjiFromLinks(root, "Composed of");

  const [etymology, ...components] = await Promise.all([
    fetchEtymology(character),
    ...componentChars.map((c) => parseKanji(c)),
  ]);

  return {
    type: "kanji",
    character,
    keyword,
    ...info,
    readings,
    ...(mnemonic ? { mnemonic } : {}),
    ...(etymology ? { etymology } : {}),
    components,
  };
}

function extractInfo(root: HTMLElement): {
  frequency?: string;
  kanjiType?: string;
  kanken?: string;
  heisig?: number;
} {
  const result: {
    frequency?: string;
    kanjiType?: string;
    kanken?: string;
    heisig?: number;
  } = {};

  const rows = root.querySelectorAll("table.cross-table tr");
  for (const row of rows) {
    const cells = row.querySelectorAll("td");
    if (cells.length < 2) continue;
    const label = cells[0].text.trim();
    const value = cells[1].text.trim();

    switch (label) {
      case "Frequency":
        result.frequency = value;
        break;
      case "Type":
        result.kanjiType = value.replace(/\s*\?$/, "").trim();
        break;
      case "Kanken":
        result.kanken = value;
        break;
      case "Heisig":
        result.heisig = parseInt(value, 10) || undefined;
        break;
    }
  }

  return result;
}

function extractReadings(root: HTMLElement): Reading[] {
  const readings: Reading[] = [];

  const commonList = root.querySelector("td.kanji-reading-list-common");
  if (commonList) {
    const divs = commonList.querySelectorAll(
      ":scope > div[style*='display: flex']"
    );
    for (const div of divs) {
      const link = div.querySelector("a");
      const freqDiv = div.querySelector("div[style*='margin-left']");
      if (link) {
        readings.push({
          reading: link.text.trim(),
          frequency: freqDiv?.text.trim().replace(/[()]/g, ""),
        });
      }
    }
  }

  const rareList = root.querySelector("td.kanji-reading-list");
  if (rareList) {
    const divs = rareList.querySelectorAll(":scope > div");
    for (const div of divs) {
      const link = div.querySelector("a");
      if (link) {
        readings.push({ reading: link.text.trim() });
      }
    }
  }

  return readings;
}

async function fetchEtymology(character: string): Promise<string | undefined> {
  try {
    const searchHtml = await fetchPage(kanjipediaSearchUrl(character));
    const searchRoot = parse(searchHtml);
    const kanjiId = extractKanjipediaId(searchRoot, character);
    if (!kanjiId) return undefined;

    const kanjiHtml = await fetchPage(kanjipediaKanjiUrl(kanjiId));
    const kanjiRoot = parse(kanjiHtml);
    return extractNaritachi(kanjiRoot);
  } catch {
    return undefined;
  }
}

function extractKanjipediaId(root: HTMLElement, character: string): string | undefined {
  const links = root.querySelectorAll("#resultKanjiList a");
  for (const link of links) {
    const href = link.getAttribute("href") || "";
    const text = link.text.trim();
    if (text === character) {
      const match = href.match(/\/kanji\/(\d+)/);
      if (match) return match[1];
    }
  }
  return undefined;
}

function extractNaritachi(root: HTMLElement): string | undefined {
  const naritachi = root.querySelector("li.naritachi");
  if (!naritachi) return undefined;
  const div = naritachi.querySelectorAll("div");
  const contentDiv = div.length >= 2 ? div[1] : undefined;
  if (!contentDiv) return undefined;
  const text = contentDiv.text.trim();
  return text || undefined;
}
