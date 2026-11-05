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
    total: Option<Value<usize>>,
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
        total: None,
        variants: Variants::default(),
    }
}
impl AvatarGroup {
    pub fn max(mut self, max: impl IntoValue<usize>) -> Self {
        self.max = max.into_value();
        self
    }
    /// Total participants, including all supplied avatars, for partially loaded groups.
    pub fn total(mut self, total: impl IntoValue<usize>) -> Self {
        self.total = Some(total.into_value());
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
        let total = self.total;
        let hidden = move || {
            total
                .map_or(count, |total| total.get().max(count))
                .saturating_sub(count.min(max.get()))
        };
        let variants = self.variants.clone();
        let children = self
            .children
            .into_iter()
            .enumerate()
            .map(move |(index, child)| {
                crate::layer::with_base_recipe_style(
                    node(),
                    |r| r.avatar_group_member_recipe(),
                    &variants,
                    move || index,
                )
                    .base_style(move || {
                        let overlap = kivact_core::style::use_tokens::<crate::UiTokens>()
                            .indicators
                            .avatar_overlap;
                        style().margin_left(if index == 0 { 0.0 } else { -overlap })
                    })
                    .hidden(move || index >= max.get())
                    .child(child)
            })
            .collect::<Vec<_>>();
        let overflow_variants = self.variants.clone();
        crate::layer::with_base_recipe_style(
            self.root,
            |r| r.avatar_group_recipe(),
            &self.variants,
            || (),
        )
        .child(children)
        .child(show(
            move || hidden() > 0,
            move || {
                crate::layer::with_recipe_style(
                    text(move || format!("+{}", hidden())),
                    |r| r.avatar_group_overflow_recipe(),
                    &overflow_variants,
                    move || hidden(),
                )
            },
        ))
        .into_view()
    }
}
