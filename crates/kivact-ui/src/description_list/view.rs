//! Role: The description list and description builders and their rendering.

use crate::description_list::look::DescriptionListTheme;
use crate::layer::with_base_recipe_style;
use kivact_core::state::{IntoText, Value};
use kivact_core::view::{Element, IntoChildren, IntoView, Node, Role, View, node, node_ref, text};

/// A list of term-detail pairs.
pub struct DescriptionList {
    root: Node,
}
/// Group descriptions with list semantics. Details may contain any view.
#[track_caller]
pub fn description_list(children: impl IntoChildren) -> DescriptionList {
    DescriptionList {
        root: node().role(Role::List).child(children.into_views()),
    }
}
impl DescriptionList {}
impl Element for DescriptionList {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for DescriptionList {
    fn into_view(self) -> View {
        let variants = self.root.authored().variants();
        with_base_recipe_style(
            self.root,
            |r: &DescriptionListTheme| r.root_recipe(),
            &variants,
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
}
/// Pair a reactive term with a detail. The detail group is labelled by the term.
#[track_caller]
pub fn description(term: impl IntoText, detail: impl IntoView) -> Description {
    Description {
        root: node().role(Role::ListItem),
        term: term.into_text_value(),
        detail: detail.into_view(),
        term_content: None,
    }
}
impl Description {
    /// Replace the visible term while retaining its canonical accessible name.
    pub fn term_content(mut self, value: impl IntoView) -> Self {
        self.term_content = Some(value.into_view());
        self
    }
}
impl Element for Description {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Description {
    fn into_view(self) -> View {
        let variants = self.root.authored().variants();
        let label = node_ref();
        let term_content = match self.term_content {
            Some(view) => node().role(Role::Group).name(self.term).child(view),
            None => text(self.term).name(self.term),
        };
        let term = with_base_recipe_style(
            term_content.node_ref(label),
            |r: &DescriptionListTheme| r.term_recipe(),
            &variants,
            || (),
        );
        let detail = with_base_recipe_style(
            node()
                .role(Role::Group)
                .name(self.term)
                .labelled_by(label)
                .child(self.detail),
            |r: &DescriptionListTheme| r.detail_recipe(),
            &variants,
            || (),
        );
        with_base_recipe_style(
            self.root.name(self.term),
            |r: &DescriptionListTheme| r.item_recipe(),
            &variants,
            || (),
        )
        .child((term, detail))
        .into_view()
    }
}
