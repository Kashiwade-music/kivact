//! Role: Hold the catalog's selection, search and appearance signals.
use crate::pages::pages;
use kivact::style::ThemeMode;
use kivact::{Signal, signal};

#[derive(Clone, Copy)]
pub(super) struct CatalogState {
    pub(super) dark: Signal<bool>,
    pub(super) selected: Signal<usize>,
    pub(super) query: Signal<String>,
}

impl CatalogState {
    pub(super) fn new(mode: ThemeMode) -> Self {
        let first = pages()
            .iter()
            .position(|page| page.slug == "button")
            .unwrap_or_default();
        Self {
            dark: signal(mode == ThemeMode::Dark),
            selected: signal(first),
            query: signal(String::new()),
        }
    }
}
