//! Pure HTML → data extraction. Nothing in here performs I/O.

pub mod jpdb;
pub mod kanjipedia;

use scraper::ElementRef;

/// A lazily compiled, process-wide CSS selector.
macro_rules! selector {
    ($css:literal) => {{
        static SELECTOR: std::sync::LazyLock<scraper::Selector> =
            std::sync::LazyLock::new(|| scraper::Selector::parse($css).expect("valid selector"));
        &*SELECTOR
    }};
}
pub(crate) use selector;

/// Trimmed text content of an element.
fn text(el: ElementRef) -> String {
    el.text().collect::<String>().trim().to_owned()
}

/// Trimmed text content, or `None` when empty.
fn non_empty_text(el: ElementRef) -> Option<String> {
    Some(text(el)).filter(|t| !t.is_empty())
}

fn next_element_sibling(el: ElementRef) -> Option<ElementRef> {
    el.next_siblings().find_map(ElementRef::wrap)
}

fn child_elements(el: ElementRef) -> impl Iterator<Item = ElementRef> {
    el.children().filter_map(ElementRef::wrap)
}

fn has_class(el: ElementRef, class: &str) -> bool {
    el.value().classes().any(|c| c == class)
}

fn style_contains(el: ElementRef, needle: &str) -> bool {
    el.attr("style").is_some_and(|s| s.contains(needle))
}
