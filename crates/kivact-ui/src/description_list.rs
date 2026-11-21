//! Role: Pair visible terms with arbitrary details in a semantic list.
pub(crate) mod look;

use crate::description_list::look::DescriptionListTheme;
use crate::layer::with_base_recipe_style;
use kivact_core::custom::Caller;
use kivact_core::state::Value;
use kivact_core::view::{Role, View, node, node_ref, text};
use kivact_macros::component;

/// A list of term-detail pairs. Details may contain any view.
#[component]
pub fn description_list(children: Vec<View>, #[caller] caller: Caller) -> impl Element {
    let variants = caller.variants();
    with_base_recipe_style(
        node().role(Role::List).child(children),
        |r: &DescriptionListTheme| r.root_recipe(),
        &variants,
        || (),
    )
}

/// One term and its associated detail, named by the visible term.
/// `term_content` replaces the visible term and keeps its accessible name.
/// The detail group is labelled by the term.
#[component]
pub fn description(
    term: Value<String>,
    detail: View,
    term_content: Option<View>,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    let label = node_ref();
    let term_node = match term_content {
        Some(view) => node().role(Role::Group).name(term).child(view),
        None => text(term).name(term),
    };
    let term_view = with_base_recipe_style(
        term_node.node_ref(label),
        |r: &DescriptionListTheme| r.term_recipe(),
        &variants,
        || (),
    );
    let detail_view = with_base_recipe_style(
        node()
            .role(Role::Group)
            .name(term)
            .labelled_by(label)
            .child(detail),
        |r: &DescriptionListTheme| r.detail_recipe(),
        &variants,
        || (),
    );
    with_base_recipe_style(
        node().role(Role::ListItem).name(term),
        |r: &DescriptionListTheme| r.item_recipe(),
        &variants,
        || (),
    )
    .child((term_view, detail_view))
}
