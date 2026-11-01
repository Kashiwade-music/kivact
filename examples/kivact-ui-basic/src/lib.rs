//! Role: Document-style component catalog for kivact-ui-basic.
//!
//! A top bar with the theme toggle, a searchable component list and one
//! documentation page per component. Every code block shows the source of
//! the demo above it.
#![warn(redundant_imports)]
mod docs;
mod look;
mod model;
mod page;
mod shell;

use kivact::style::ThemeMode;
use kivact::view::IntoView;

/// Mount the complete catalog through public component APIs.
pub fn catalog() -> impl IntoView {
    catalog_with_mode(ThemeMode::Light)
}

/// The catalog with an explicit initial theme mode.
pub fn catalog_with_mode(mode: ThemeMode) -> impl IntoView {
    shell::catalog(mode)
}

/// Application-owned, offline OFL font bytes for the catalog and its tests.
pub fn fonts() -> &'static [&'static [u8]] {
    &[
        include_bytes!("../assets/fonts/inter/Inter[opsz,wght].ttf"),
        include_bytes!("../assets/fonts/geistmono/GeistMono[wght].ttf"),
        include_bytes!("../assets/fonts/notosansjp/NotoSansJP[wght].ttf"),
    ]
}

/// One example of a catalog page, as the tests read it.
#[derive(Clone, Debug)]
pub struct ExampleInfo {
    /// The example id, as in `example-<slug>-<id>-preview`.
    pub id: &'static str,
    /// The dedented source between the example markers of the page file.
    pub snippet: String,
}

/// One catalog page, as the tests read it.
#[derive(Clone, Debug)]
pub struct PageInfo {
    /// The slug of the page, as in `catalog-nav-<slug>`.
    pub slug: &'static str,
    /// The English component name shown as the page title.
    pub name: &'static str,
    /// The whole source of the page file.
    pub source: &'static str,
    /// The examples in page order.
    pub examples: Vec<ExampleInfo>,
}

/// Every page of the catalog in navigation order.
pub fn pages() -> Vec<PageInfo> {
    docs::PAGES
        .iter()
        .map(|page| PageInfo {
            slug: page.slug,
            name: page.name,
            source: page.source,
            examples: page
                .examples
                .iter()
                .map(|example| ExampleInfo {
                    id: example.id,
                    snippet: page.snippet(example.id),
                })
                .collect(),
        })
        .collect()
}
