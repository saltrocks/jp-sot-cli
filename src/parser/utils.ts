import type { HTMLElement } from "node-html-parser";

export function extractLabeledValue(
  root: HTMLElement,
  labelText: string,
): string | undefined {
  const labels = root.querySelectorAll("h6.subsection-label");
  for (const label of labels) {
    if (label.text.trim() === labelText) {
      const sibling = label.nextElementSibling;
      if (sibling) return sibling.text.trim() || undefined;
    }
  }
  return undefined;
}

const KANJI_HREF_REGEX = /^\/kanji\/(.+?)(?:#|$)/;

export function extractKanjiFromLinks(
  root: HTMLElement,
  sectionLabel: string,
): string[] {
  const results: string[] = [];
  const sections = root.querySelectorAll(".subsection-composed-of-kanji");
  for (const section of sections) {
    const label = section.querySelector("h6.subsection-label");
    if (label && label.text.trim() === sectionLabel) {
      const links = section.querySelectorAll("a.plain");
      for (const link of links) {
        const href = link.getAttribute("href") || "";
        const match = href.match(KANJI_HREF_REGEX);
        if (match) {
          results.push(decodeURIComponent(match[1]));
        }
      }
    }
  }
  return results;
}

export function parseRuby(ruby: HTMLElement): { word: string; reading: string } {
  let word = "";
  let reading = "";
  const children = ruby.childNodes;
  for (let i = 0; i < children.length; i++) {
    const child = children[i] as HTMLElement;
    if (child.tagName === "RT") {
      reading += child.text;
    } else if (child.nodeType === 3) {
      const text = child.text || "";
      word += text;
      const next = children[i + 1] as HTMLElement | undefined;
      if (!next || next.tagName !== "RT") {
        reading += text;
      }
    }
  }
  return { word: word.trim(), reading: reading.trim() };
}
