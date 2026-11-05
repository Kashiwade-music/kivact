//! Role: Pair visible terms with arbitrary details in a semantic list.
use crate::layer::with_base_recipe_style;
use crate::variants::{Variant, Variants};
use kivact_core::state::{IntoText, Value};
use kivact_core::view::{
    IntoChildren, IntoView, Node, NodeBuilder, Role, View, node, node_ref, text,
};

/// A list of term-detail pairs.
pub struct DescriptionList {
    root: Node,
    variants: Variants,
}
/// Group descriptions with list semantics. Details may contain any view.
#[track_caller]
pub fn description_list(children: impl IntoChildren) -> DescriptionList {
    DescriptionList {
        root: node().role(Role::List).child(children.into_views()),
        variants: Variants::default(),
    }
}
impl DescriptionList {
    /// Add an appearance read by the list recipe.
    pub fn variant(self, value: impl Variant) -> Self {
        self.variants.add(value);
        self
    }
}
impl NodeBuilder for DescriptionList {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for DescriptionList {
    fn into_view(self) -> View {
        with_base_recipe_style(
            self.root,
            |r| r.description_list_recipes().root_recipe(),
            &self.variants,
            || (),
        )
        .into_view()
    }
}
/// One term and its associated detail, named by the visible term.
pub struct Description {
    root: Node,
    term: Value<String>,
    detail: View,
    term_content: Option<View>,
    variants: Variants,
}
/// Pair a reactive term with a detail. The detail group is labelled by the term.
#[track_caller]
pub fn description(term: impl IntoText, detail: impl IntoView) -> Description {
    Description {
        root: node().role(Role::ListItem),
        term: term.into_text_value(),
        detail: detail.into_view(),
        term_content: None,
        variants: Variants::default(),
    }
}
impl Description {
    /// Replace the visible term while retaining its canonical accessible name.
    pub fn term_content(mut self, value: impl IntoView) -> Self {
        self.term_content = Some(value.into_view());
        self
    }
    /// Add an appearance read by every recipe of this pair.
    pub fn variant(self, value: impl Variant) -> Self {
        self.variants.add(value);
        self
    }
}
impl NodeBuilder for Description {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Description {
    fn into_view(self) -> View {
        let label = node_ref();
        let term_content = match self.term_content {
            Some(view) => node().name(self.term).child(view),
            None => text(self.term).name(self.term),
        };
        let term = with_base_recipe_style(
            term_content.node_ref(label),
            |r| r.description_list_recipes().term_recipe(),
            &self.variants,
            || (),
        );
        let detail = with_base_recipe_style(
            node()
                .role(Role::Group)
                .name(self.term)
                .labelled_by(label)
                .child(self.detail),
            |r| r.description_list_recipes().detail_recipe(),
            &self.variants,
            || (),
        );
        with_base_recipe_style(
            self.root.name(self.term),
            |r| r.description_list_recipes().item_recipe(),
            &self.variants,
            || (),
        )
        .child((term, detail))
        .into_view()
    }
}
