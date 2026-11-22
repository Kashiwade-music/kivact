//! Role: Small Lyra primitives: label, separators, key groups, captions, crumb ellipsis.

use crate::color::{BORDER, FOREGROUND, MUTED_FOREGROUND};
use crate::recipe_parts::disabled;

use kivact::component;
use kivact::state::{IntoValue, Value};
use kivact::style::{Align, FlexDirection, Justify, TextAlign, pct, style, use_theme};
use kivact::view::{Element, NodeRef, Orientation, Role, View, icon, node, row, text};

/// A form label (label.tsx): 12 px text without leading, centered with its children.
///
/// A label showing `text_value`. With `.target(..)`, pressing the label
/// activates the target control as a click on it would.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact::view::{NodeRef};
/// use kivact_ui_basic as basic;
///
/// fn email() -> impl IntoView {
///     let field = node_ref();
///     let value = signal(String::new());
///     column((
///         basic::label("Email").target(field),
///         text_input(value).label("Email").node_ref(field),
///     ))
///     .style(style().gap(8))
/// }
/// ```
#[component]
pub fn label(text_value: Value<String>, target: Option<NodeRef>) -> impl Element {
    let root = node()
        .style(move || {
            style()
                .flex_direction(FlexDirection::Row)
                .align_items(Align::Center)
                .gap(8)
                .font_size(12)
                .line_height(1.0)
                .color(FOREGROUND)
                .disabled(disabled(&use_theme().get()))
        })
        .child(text(text_value));
    match target {
        // The label is a pointer-only forwarder: it has no role of its own.
        Some(target) => root
            .on_press(move || target.activate())
            .with_node(|root| root.role(Role::Generic)),
        None => root,
    }
}

/// A 1 px line in the border color (separator.tsx), with `Role::Separator`.
///
/// A horizontal separator that spans its container's width. Draw a vertical
/// line that stretches across the parent row's height with
/// `.orientation(Orientation::Vertical)`; horizontal is the default.
#[component]
pub fn separator(orientation: Option<Value<Orientation>>) -> impl Element {
    let orientation = orientation.unwrap_or_else(|| Orientation::Horizontal.into_value());
    node()
        .with_node(|root| root.role(Role::Separator))
        .orientation(orientation)
        .base_style(move || {
            let s = style().flex_shrink(0.0).background(BORDER);
            if orientation.get() == Orientation::Vertical {
                s.width(1).align_self(Align::Stretch)
            } else {
                s.height(1).width(pct(100))
            }
        })
}

/// A horizontal rule between the rows of a `kivact::ui::item_list`, with
/// 8 px above and below.
#[component]
pub fn item_separator() -> impl Element {
    node()
        .role(Role::Separator)
        .orientation(Orientation::Horizontal)
        .style(
            style()
                .width(pct(100))
                .height(1)
                .flex_shrink(0.0)
                .margin_y(8)
                .background(BORDER),
        )
}

/// Key caps of `kivact::ui::kbd` shown together, such as a shortcut
/// (kbd.tsx `KbdGroup`).
#[component]
pub fn kbd_group(children: View) -> impl Element {
    row(children).style(style().align_items(Align::Center).gap(4))
}

/// A caption under a `kivact::ui::table`, muted and centered, with 16 px
/// above it. Put it after the table: it is drawn where it is placed.
#[component]
pub fn table_caption(text_value: Value<String>) -> impl Element {
    text(text_value).style(
        style()
            .margin_top(16)
            .color(MUTED_FOREGROUND)
            .text_align(TextAlign::Center),
    )
}

/// The "…" that stands for hidden crumbs in the `item` slot of a
/// `kivact::ui::breadcrumb`, read as the `basic.breadcrumb.more` message
/// ("More").
#[component]
pub fn breadcrumb_ellipsis() -> impl Element {
    node()
        .role(Role::Text)
        .name(kivact::i18n::message("basic.breadcrumb.more", "More"))
        .style(
            style()
                .size(20, 20)
                .flex_shrink(0.0)
                .flex_direction(FlexDirection::Row)
                .align_items(Align::Center)
                .justify_content(Justify::Center),
        )
        .child(icon("ellipsis"))
}
