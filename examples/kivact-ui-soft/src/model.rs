//! Role: Hold the catalog's selection, search and appearance signals.
use crate::pages::pages;
use kivact::state::{Signal, signal};
use kivact::style::ThemeMode;

#[derive(Clone, Copy)]
pub(super) struct CatalogState {
    // xtask-allow(state-fields): Independent caller-controlled appearance preference, unrelated to selected page or query.
    pub(super) dark: Signal<bool>,
    // xtask-allow(state-fields): The single selected-page index, independent of appearance and search text.
    pub(super) selected: Signal<usize>,
    pub(super) query: Signal<String>,
}

impl CatalogState {
    pub(super) fn new(mode: ThemeMode, slug: &str) -> Self {
        let first = pages()
            .iter()
            .position(|page| page.slug == slug)
            .unwrap_or_default();
        Self {
            dark: signal(mode == ThemeMode::Dark),
            selected: signal(first),
            query: signal(String::new()),
        }
    }
}
