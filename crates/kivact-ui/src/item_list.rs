//! Role: Compose semantic static lists with fixed child positions and header content.
use crate::layer::{with_base_recipe_style, with_recipe_style};
use crate::variants::{Variant, Variants};
use crate::{ItemListMarker, ItemPosition};
use kivact_core::custom::part;
use kivact_core::state::{IntoValue, Value, provide};
use kivact_core::view::{IntoChildren, IntoView, Node, NodeBuilder, Role, View, node, node_ref};
/// A semantic list; fixed direct children receive readonly positions at mount.
pub struct ItemList {
    root: Node,
    children: Vec<View>,
    header: Option<View>,
    marker: Value<ItemListMarker>,
    start: Value<u32>,
    variants: Variants,
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
        variants: Variants::default(),
    }
}
impl ItemList {
    /// Supply visible content that labels the list.
    pub fn header(mut self, header: impl IntoView) -> Self {
        self.header = Some(header.into_view());
        self
    }
    /// Generate decorative markers from each actual row position.
    pub fn marker(mut self, marker: impl IntoValue<ItemListMarker>) -> Self {
        self.marker = marker.into_value();
        self
    }
    /// First number for decimal markers; position semantics remain one based.
    pub fn start(mut self, start: impl IntoValue<u32>) -> Self {
        self.start = start.into_value();
        self
    }
    pub fn variant(self, variant: impl Variant) -> Self {
        self.variants.add(variant);
        self
    }
}
impl NodeBuilder for ItemList {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for ItemList {
    fn into_view(mut self) -> View {
        let part = part(|s| s);
        let state = part.state();
        let header_ref = node_ref();
        let header = self.header.map(|header| {
            with_recipe_style(
                node().role(Role::Group).node_ref(header_ref).child(header),
                |r| r.item_list_recipes().list_header_recipe(),
                &self.variants,
                move || state.get(),
            )
        });
        if header.is_some() {
            self.root = self.root.labelled_by(header_ref);
        }
        let count = self.children.len();
        let variants = self.variants.clone();
        let children = self
            .children
            .into_iter()
            .enumerate()
            .map(|(index, child)| {
                provide(
                    crate::item_context::ItemContext {
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
            |r| r.item_list_recipes().root_recipe(),
            &self.variants,
            move || state.get(),
        )
        .child(children);
        match header {
            Some(header) => kivact_core::view::column((header, list)).into_view(),
            None => list.into_view(),
        }
    }
}
