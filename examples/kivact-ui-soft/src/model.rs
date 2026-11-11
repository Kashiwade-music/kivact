//! Role: Hold the catalog's selection, search and appearance signals.
use crate::pages::pages;
use kivact::state::{Signal, signal};
use kivact::style::ThemeMode;

#[derive(Clone, Copy)]
pub(super) struct CatalogState {
    // xtask-allow(state-fields): independent: the appearance, seeded from the caller's ThemeMode and flipped by the Toggle appearance button; it only picks the theme, so every page and query can be shown in light or dark
    pub(super) dark: Signal<bool>,
    // xtask-allow(state-fields): independent: the shown page, seeded from the caller's slug and set by the navigation buttons; the theme toggle never writes it, so any page shows in light or dark
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
