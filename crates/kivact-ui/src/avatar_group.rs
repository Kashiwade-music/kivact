//! Role: Lay out a bounded facepile with the shared roving-focus owner.
use crate::variants::{Variant, Variants};
use kivact_core::state::{IntoValue, Value, memo};
use kivact_core::style::style;
use kivact_core::view::{
    IntoChildren, IntoView, Node, NodeBuilder, Orientation, Role, View, node, row, show, text,
};
use std::rc::Rc;
// xtask-allow(state-fields): Independent immutable group limits, overlap input and presentation factory; roving focus remains the sole interaction owner.
pub struct AvatarGroup {
    root: Node,
    children: Vec<View>,
    max: Value<usize>,
    overlap: Option<Value<f64>>,
    overflow: Option<Rc<dyn Fn(Value<usize>) -> View>>,
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
        overlap: None,
        overflow: None,
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
    /// Override the theme overlap in logical pixels, clamped to nonnegative values.
    pub fn overlap(mut self, overlap: impl IntoValue<f64>) -> Self {
        self.overlap = Some(overlap.into_value());
        self
    }
    /// Build the actual omitted-count content, including an optional action target.
    /// The source follows `total`, mounted children, and `max`; no placeholder avatars are needed.
    pub fn overflow_content<V: IntoView>(
        mut self,
        content: impl Fn(Value<usize>) -> V + 'static,
    ) -> Self {
        self.overflow = Some(Rc::new(move |count| content(count).into_view()));
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
        let hidden = memo(move || {
            total
                .map_or(count, |total| total.get().max(count))
                .saturating_sub(count.min(max.get()))
        });
        let variants = self.variants.clone();
        let overlap = self.overlap;
        let direction = kivact_core::view::use_direction();
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
                .base_style(move || member_overlap(index, overlap, direction.get()))
                .hidden(move || index >= max.get())
                .child(child)
            })
            .collect::<Vec<_>>();
        let overflow_variants = self.variants.clone();
        let overflow = self.overflow;
        crate::layer::with_base_recipe_style(
            self.root,
            |r| r.avatar_group_recipe(),
            &self.variants,
            || (),
        )
        .child(children)
        .child(show(
            move || hidden.get() > 0,
            move || {
                crate::layer::with_recipe_style(
                    overflow.as_ref().map_or_else(
                        || {
                            node()
                                .role(Role::Image)
                                .name(kivact_core::i18n::message_with(
                                    "avatar_group.overflow",
                                    "{count} more",
                                    move || [("count", hidden.get().to_string())],
                                ))
                                .child(text(move || format!("+{}", hidden.get())))
                        },
                        |content| node().child(content(hidden)),
                    ),
                    |r| r.avatar_group_overflow_recipe(),
                    &overflow_variants,
                    move || hidden.get(),
                )
            },
        ))
        .into_view()
    }
}

/// Derive logical overlap from the one caller source or the theme token.
fn member_overlap(
    index: usize,
    overlap: Option<Value<f64>>,
    direction: kivact_core::view::ReadingDirection,
) -> kivact_core::style::Style {
    let amount = overlap
        .map_or_else(
            || {
                kivact_core::style::use_tokens::<crate::UiTokens>()
                    .indicators
                    .avatar_overlap
            },
            |overlap| overlap.get(),
        )
        .max(0.0);
    let margin = if index == 0 { 0.0 } else { -amount };
    if direction == kivact_core::view::ReadingDirection::Rtl {
        style().margin_right(margin)
    } else {
        style().margin_left(margin)
    }
}
