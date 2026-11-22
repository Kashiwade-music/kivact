//! Role: The avatar group component: overlapping avatars and an overflow count.

use crate::avatar_group::look::AvatarGroupTheme;
use kivact_core::custom::Caller;
use kivact_core::state::{Value, memo};
use kivact_core::style::style;
use kivact_core::view::{Node, Orientation, Role, View, node, row, show, text};
use kivact_macros::component;
use std::rc::Rc;

/// A row of overlapping avatars, showing the first `max` and a count of the
/// rest. `total` counts participants beyond the supplied children.
#[component]
pub fn avatar_group(
    children: Vec<View>,
    #[prop(default = usize::MAX)] max: Value<usize>,
    total: Option<Value<usize>>,
    overlap: Option<Value<f64>>,
    overflow_content: Option<Rc<dyn Fn(Value<usize>) -> View>>,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    let count = children.len();
    let hidden = memo(move || {
        total
            .map_or(count, |total| total.get().max(count))
            .saturating_sub(count.min(max.get()))
    });
    let direction = kivact_core::view::use_direction();
    let member_variants = variants.clone();
    let children = children
        .into_iter()
        .enumerate()
        .map(move |(index, child)| {
            crate::layer::with_base_recipe_style(
                node(),
                |r: &AvatarGroupTheme| r.member_recipe(),
                &member_variants,
                move || index,
            )
            .base_style(move || member_overlap(index, overlap, direction))
            .hidden(move || index >= max.get())
            .child(child)
        })
        .collect::<Vec<_>>();
    let overflow_variants = variants.clone();
    let root = row(())
        .role(Role::Group)
        .name(kivact_core::i18n::message("avatar_group.label", "Avatars"))
        .roving_focus(Orientation::Horizontal);
    crate::layer::with_base_recipe_style(
        root,
        |r: &AvatarGroupTheme| r.root_recipe(),
        &variants,
        || (),
    )
    .child(children)
    .child(show(
        move || hidden.get() > 0,
        move || overflow_view(overflow_content.as_ref(), hidden, &overflow_variants),
    ))
}

/// The omitted-count content: the caller's, or the count as text.
fn overflow_view(
    content: Option<&Rc<dyn Fn(Value<usize>) -> View>>,
    hidden: Value<usize>,
    variants: &kivact_core::view::Variants,
) -> Node {
    crate::layer::with_recipe_style(
        content.map_or_else(
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
        |r: &AvatarGroupTheme| r.overflow_recipe(),
        variants,
        move || hidden.get(),
    )
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
                kivact_core::style::use_theme()
                    .with(|theme| theme.get(&crate::avatar_group::look::OVERLAP_VAR))
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
