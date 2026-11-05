//! Role: Lay out a bounded facepile with the shared roving-focus owner.
use crate::variants::{Variant, Variants};
use kivact_core::state::{IntoValue, Value};
use kivact_core::style::style;
use kivact_core::view::{
    IntoChildren, IntoView, Node, NodeBuilder, Orientation, Role, View, node, row, show, text,
};
pub struct AvatarGroup {
    root: Node,
    children: Vec<View>,
    max: Value<usize>,
    variants: Variants,
}
pub fn avatar_group(children: impl IntoChildren) -> AvatarGroup {
    AvatarGroup {
        root: row(())
            .role(Role::Group)
            .name(kivact_core::i18n::message("avatar_group.label", "Avatars"))
            .roving_focus(Orientation::Horizontal),
        children: children.into_views(),
        max: usize::MAX.into_value(),
        variants: Variants::default(),
    }
}
impl AvatarGroup {
    pub fn max(mut self, max: impl IntoValue<usize>) -> Self {
        self.max = max.into_value();
        self
    }
    pub fn variant(self, variant: impl Variant) -> Self {
        self.variants.add(variant);
        self
    }
}
impl NodeBuilder for AvatarGroup {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for AvatarGroup {
    fn into_view(self) -> View {
        let max = self.max;
        let count = self.children.len();
        let children = self
            .children
            .into_iter()
            .enumerate()
            .map(move |(index, child)| {
                node()
                    .base_style(move || style().margin_left(if index == 0 { 0 } else { -8 }))
                    .hidden(move || index >= max.get())
                    .child(child)
            })
            .collect::<Vec<_>>();
        crate::layer::with_base_recipe_style(
            self.root,
            |r| r.avatar_group_recipe(),
            &self.variants,
            || (),
        )
        .child(children)
        .child(show(
            move || count > max.get(),
            move || text(move || format!("+{}", count.saturating_sub(max.get()))),
        ))
        .into_view()
    }
}
