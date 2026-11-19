//! Role: Render accordion panels over one headless selection controller.
use kivact_core::state::IntoText;
use kivact_core::view::{Element, IntoView, Node, View, column};
use kivact_interaction::{DisclosureGroupApi, IntoSelection, use_disclosure_group};

/// A sequence of labelled disclosure panels sharing one selection.
pub struct DisclosureGroup<K: 'static> {
    root: Node,
    group: DisclosureGroupApi<K>,
    panels: Vec<crate::Disclosure>,
    variants: crate::variants::Variants,
    heading_level: u32,
}
/// Create a single-selection or multiple-selection accordion.
pub fn disclosure_group<K: Clone + PartialEq + 'static>(
    selected: impl IntoSelection<K>,
) -> DisclosureGroup<K> {
    DisclosureGroup {
        root: column(()),
        group: use_disclosure_group(selected),
        panels: Vec::new(),
        variants: crate::variants::Variants::default(),
        heading_level: 3,
    }
}
impl<K: Clone + PartialEq + 'static> DisclosureGroup<K> {
    /// Set the semantic level of all accordion headings (default 3).
    pub fn heading_level(mut self, level: u32) -> Self {
        self.heading_level = level;
        self
    }
    /// Apply a theme look to every panel, independent of builder call order.
    pub fn variant(mut self, variant: impl crate::Variant) -> Self {
        self.variants.add(variant);
        self
    }
    /// Append a labelled panel whose content is rebuilt on every opening.
    pub fn panel<V: IntoView>(
        mut self,
        key: K,
        title: impl IntoText,
        content: impl Fn() -> V + 'static,
    ) -> Self {
        self.panels
            .push(crate::disclosure(title, self.group.panel_open(key)).content(content));
        self
    }
}
impl<K: 'static> Element for DisclosureGroup<K> {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl<K: 'static> IntoView for DisclosureGroup<K> {
    fn into_view(self) -> View {
        let variants = self.variants;
        let level = self.heading_level;
        let count = self.panels.len();
        let panels = self
            .panels
            .into_iter()
            .enumerate()
            .map(|(index, panel)| {
                panel
                    .group_position(index, count)
                    .heading(level)
                    .group_variants(variants.clone())
                    .into_view()
            })
            .collect::<Vec<_>>();
        self.root.child(panels).into_view()
    }
}
