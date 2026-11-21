//! Role: The item list component, its child positions and its header rendering.

use crate::item_list::look::ItemListTheme;
use crate::item_list::look::{ItemListMarker, ItemPosition};
use crate::layer::{with_base_recipe_style, with_recipe_style};
use kivact_core::custom::{Caller, part};
use kivact_core::state::{Value, provide};
use kivact_core::view::{Children, Role, View, column, node, node_ref};
use kivact_macros::component;

/// Group rows without introducing listbox selection or another interaction owner.
///
/// A semantic list; fixed direct children receive readonly positions at mount.
/// `header(..)` supplies visible content that labels the list. `marker(..)`
/// generates decorative markers from each actual row position, and `start(..)`
/// is the first number for decimal markers (position semantics stay one based).
#[component]
pub fn item_list(
    children: Children,
    header: Option<View>,
    #[prop(default = ItemListMarker::None)] marker: Value<ItemListMarker>,
    #[prop(default = 1)] start: Value<u32>,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    let part = part(|s| s);
    let state = part.state();
    let header_ref = node_ref();
    let header = header.map(|header| {
        with_recipe_style(
            node().role(Role::Group).node_ref(header_ref).child(header),
            |r: &ItemListTheme| r.list_header_recipe(),
            &variants,
            move || state.get(),
        )
    });
    let mut root = node().role(Role::List);
    if header.is_some() {
        root = root.labelled_by(header_ref);
    }
    let children = children.into_views();
    let count = children.len();
    let child_variants = variants.clone();
    let children = children
        .into_iter()
        .enumerate()
        .map(|(index, child)| {
            provide(
                crate::item_list::context::ItemContext {
                    position: Some(ItemPosition { index, count }),
                    marker,
                    start,
                    variants: child_variants.clone(),
                },
                move || child,
            )
        })
        .collect::<Vec<_>>();
    // The recipe is the base style, so the caller's styles win.
    let list = with_base_recipe_style(
        root.part(part),
        |r: &ItemListTheme| r.root_recipe(),
        &variants,
        move || state.get(),
    )
    .child(children);
    match header {
        Some(header) => column((header, list)),
        None => list,
    }
}
