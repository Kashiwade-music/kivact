//! Role: The Lyra marker: a muted inline note with optional rules and icon.

use crate::recipe_parts::sized_icon;
use crate::variants::MarkerVariant;
use crate::{BORDER, MUTED_FOREGROUND};
use kivact::component;
use kivact::custom::Caller;
use kivact::style::{Align, FlexDirection, pct, style};
use kivact::view::{Node, View, node, row};

/// A 1 px line that takes the free space of the marker's row.
fn rule(margin_right: f64, margin_left: f64) -> Node {
    node().style(
        style()
            .flex_grow(1.0)
            .min_width(0)
            .height(1)
            .margin_right(margin_right)
            .margin_left(margin_left)
            .background(BORDER),
    )
}

/// A muted note such as a date divider in a conversation.
///
/// A marker showing `content`. Choose the decoration with
/// `.variant(MarkerVariant::Separator)`, and show an icon before the content
/// at 14 px with `.icon(..)`.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn day() -> impl IntoView {
///     basic::marker("Today").variant(basic::MarkerVariant::Separator)
/// }
/// ```
#[component]
pub fn marker(content: View, icon: Option<View>, #[caller] caller: Caller) -> impl Element {
    let variant = caller.variants().get::<MarkerVariant>().unwrap_or_default();
    let separated = variant == MarkerVariant::Separator;
    let content = row(content).style(
        style()
            .min_width(0)
            .align_items(Align::Center)
            .flex_shrink(if separated { 0.0 } else { 1.0 }),
    );
    node()
        .base_style(move || {
            let s = style()
                .flex_direction(FlexDirection::Row)
                .align_items(Align::Center)
                .gap(8)
                .width(pct(100))
                .min_height(16)
                .font_size(12)
                .color(MUTED_FOREGROUND);
            if variant == MarkerVariant::Border {
                s.border_bottom(1, BORDER).padding_bottom(8)
            } else {
                s
            }
        })
        .child(separated.then(|| rule(4.0, 0.0)))
        .child(icon.map(|icon| sized_icon(icon, 14.0)))
        .child(content)
        .child(separated.then(|| rule(0.0, 4.0)))
}
