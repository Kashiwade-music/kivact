//! Role: The item list builder, its child positions and its header rendering.

use crate::item_list::look::ItemListTheme;
use crate::item_list::look::{ItemListMarker, ItemPosition};
use crate::layer::{with_base_recipe_style, with_recipe_style};
use kivact_core::custom::part;
use kivact_core::state::{IntoValue, Value, provide};
use kivact_core::view::{Element, IntoChildren, IntoView, Node, Role, View, node, node_ref};

/// A semantic list; fixed direct children receive readonly positions at mount.
pub struct ItemList {
    root: Node,
    children: Vec<View>,
    header: Option<View>,
    marker: Value<ItemListMarker>,
    start: Value<u32>,
}
/// Group rows without introducing listbox selection or another interaction owner.
#[track_caller]
pub fn item_list(children: impl IntoChildren) -> ItemList {
    ItemList {
        root: node().role(Role::List),
        children: children.into_views(),
        header: None,
        marker: ItemListMarker::None.into_value(),
        start: 1u32.into_value(),
    }
}
impl ItemList {
    /// Supply visible content that labels the list.
    pub fn header(mut self, header: impl IntoView) -> Self {
        self.header = Some(header.into_view());
        self
    }
    /// Generate decorative markers from each actual row position.
    #[track_caller]
    pub fn marker(mut self, marker: impl IntoValue<ItemListMarker>) -> Self {
        self.marker = marker.into_value();
        self
    }
    /// First number for decimal markers; position semantics remain one based.
    #[track_caller]
    pub fn start(mut self, start: impl IntoValue<u32>) -> Self {
        self.start = start.into_value();
        self
    }
}
impl Element for ItemList {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for ItemList {
    fn into_view(mut self) -> View {
        let variants = self.root.authored().variants();
        let part = part(|s| s);
        let state = part.state();
        let header_ref = node_ref();
        let header = self.header.map(|header| {
            with_recipe_style(
                node().role(Role::Group).node_ref(header_ref).child(header),
                |r: &ItemListTheme| r.list_header_recipe(),
                &variants,
                move || state.get(),
            )
        });
        if header.is_some() {
            self.root = self.root.labelled_by(header_ref);
        }
        let count = self.children.len();
        let variants = variants.clone();
        let children = self
            .children
            .into_iter()
            .enumerate()
            .map(|(index, child)| {
                provide(
                    crate::item_list::context::ItemContext {
                        position: Some(ItemPosition { index, count }),
                        marker: self.marker,
                        start: self.start,
                        variants: variants.clone(),
                    },
                    move || child,
                )
            })
            .collect::<Vec<_>>();
        let list = with_base_recipe_style(
            self.root.part(part),
            |r: &ItemListTheme| r.root_recipe(),
            &variants,
            move || state.get(),
        )
        .child(children);
        match header {
            Some(header) => kivact_core::view::column((header, list)).into_view(),
            None => list.into_view(),
        }
    }
}
