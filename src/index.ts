#!/usr/bin/env bun

import { parseKanji, resetVisited } from "./parser/kanji.js";
import { parseWord } from "./parser/word.js";

const KANJI_REGEX = /^[\u4e00-\u9fff\u3400-\u4dbf]$/;

async function main() {
  const input = process.argv[2];

  if (!input) {
    console.error("Usage: jp-sot <word-or-kanji>");
    console.error("  jp-sot 宣言    # look up a word");
    console.error("  jp-sot 宣      # look up a single kanji");
    process.exit(1);
  }

  try {
    let result;
    if (input.length === 1 && KANJI_REGEX.test(input)) {
      resetVisited();
      result = await parseKanji(input);
    } else {
      result = await parseWord(input);
    }
    console.log(JSON.stringify(result, null, 2));
  } catch (err) {
    console.error(
      `Error: ${err instanceof Error ? err.message : String(err)}`
    );
    process.exit(1);
  }
}

main();
