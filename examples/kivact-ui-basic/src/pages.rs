//! Role: Every catalog page and its examples, as the catalog tests read them.

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
    crate::docs::PAGES
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
