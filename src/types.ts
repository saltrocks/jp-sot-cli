export interface Reading {
  reading: string;
  frequency?: string;
}

export interface Meaning {
  partOfSpeech: string;
  definitions: string[];
}

export interface KanjiResult {
  type: "kanji";
  character: string;
  keyword: string;
  frequency?: string;
  kanjiType?: string;
  kanken?: string;
  heisig?: number;
  readings: Reading[];
  mnemonic?: string;
  etymology?: string;
  components: KanjiResult[];
}

export interface WordResult {
  type: "word";
  word: string;
  reading: string;
  meanings: Meaning[];
  pitchAccent?: string;
  frequency?: string;
  composedOf: { word: string; reading: string; meaning: string }[];
  kanji: KanjiResult[];
}

export type LookupResult = WordResult | KanjiResult;
