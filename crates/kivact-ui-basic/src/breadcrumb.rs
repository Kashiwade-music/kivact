//! Role: The Lyra breadcrumb trail with links, current page, separators and ellipsis.

use crate::Color;
use crate::parts::{pressable, sized_icon};
use crate::theme_access::tokens;
use kivact::control::icon;
use kivact::control::{Node, NodeBuilder, node, row, text};
use kivact::primitive::NodeExt;
use kivact::style::{Align, Cursor, Justify, style};
use kivact::view::HandlerResult;
use kivact::view::{IntoText, IntoValue, IntoView, Role, View, callback};
use kivact::view::{ReadingDirection, current_direction};
use kivact::{Callback, Value};

/// A breadcrumb trail (breadcrumb.tsx): a `Role::Navigation` landmark named
/// "Breadcrumb" (the `breadcrumb.label` message).
///
/// Pass [`breadcrumb_link`], [`breadcrumb_page`], [`breadcrumb_separator`]
/// and [`breadcrumb_ellipsis`] nodes as children. In a
/// [`direction`](kivact::view::provide_direction) of [`ReadingDirection::Rtl`] the trail runs from
/// the right and the separators point left.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn trail() -> impl IntoView {
///     basic::breadcrumb_content((
///         basic::breadcrumb_link("Home").on_press(|| {}),
///         basic::breadcrumb_separator(),
///         basic::breadcrumb_link("Components"),
///         basic::breadcrumb_separator(),
///         basic::breadcrumb_page("Breadcrumb"),
///     ))
/// }
/// ```
#[track_caller]
pub fn breadcrumb_content(children: impl IntoView) -> Node {
    let rtl = current_direction() == ReadingDirection::Rtl;
    row(children)
        .role(Role::Navigation)
        .name(kivact::i18n::message(
            "basic.breadcrumb.label",
            "Breadcrumb",
        ))
        .style(move || {
            style()
                .direction(if rtl {
                    kivact::style::Direction::RowReverse
                } else {
                    kivact::style::Direction::Row
                })
                .wrap(true)
                .align_items(Align::Center)
                .gap(6)
                .font_size(12)
                .color(tokens().muted_foreground)
        })
}

/// A link of a [`breadcrumb`]. Created by [`breadcrumb_link`].
pub struct BreadcrumbLink {
    root: Node,
    label: Value<String>,
    on_press: Option<Callback<()>>,
}

/// A breadcrumb link showing `text`; it turns to the foreground color on hover.
#[track_caller]
pub fn breadcrumb_link(text_value: impl IntoText) -> BreadcrumbLink {
    let label = text_value.into_text_value();
    BreadcrumbLink {
        root: node().name(label),
        label,
        on_press: None,
    }
}

impl BreadcrumbLink {
    /// Report presses (click, Enter, Space) to `f`; without it the link takes no focus.
    pub fn on_press<R: HandlerResult>(mut self, f: impl Fn() -> R + 'static) -> Self {
        self.on_press = Some(callback(move |_| f()));
        self
    }
}

impl NodeBuilder for BreadcrumbLink {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for BreadcrumbLink {
    fn into_view(self) -> View {
        let actionable = self.on_press.is_some();
        let (root, _) = pressable(self.root, Role::Link, self.on_press);
        root.base_style(move || {
            let p = tokens();
            // A pressable link is a focus stop: the theme's ring shows while
            // the focus is visible.
            let s = style()
                .text_wrap(false)
                .color(p.muted_foreground)
                .hover(style().color(p.foreground));
            if actionable {
                s.cursor(Cursor::Pointer)
            } else {
                s
            }
        })
        .child(text(self.label))
        .into_view()
    }
}

/// The current page of a [`breadcrumb`]: foreground text, a disabled link.
#[track_caller]
pub fn breadcrumb_page(text_value: impl IntoText) -> Node {
    let label = text_value.into_text_value();
    node()
        .role(Role::Link)
        .name(label)
        .disabled(true)
        .style(style().color(Color::Foreground).text_wrap(false))
        .child(text(label))
}

/// The chevron between two crumbs; it points left in a right-to-left direction.
#[track_caller]
pub fn breadcrumb_separator() -> Node {
    let name = if current_direction() == ReadingDirection::Rtl {
        "chevron-left"
    } else {
        "chevron-right"
    };
    node()
        .style(style().shrink(0))
        .child(sized_icon(icon(name), 14.0))
}

/// The "…" that stands for hidden crumbs, read as the text "More".
#[track_caller]
pub fn breadcrumb_ellipsis() -> Node {
    node()
        .role(Role::Text)
        .name(kivact::i18n::message("basic.breadcrumb.more", "More"))
        .style(
            style()
                .size(20, 20)
                .shrink(0)
                .direction(kivact::style::Direction::Row)
                .align_items(Align::Center)
                .justify_content(Justify::Center),
        )
        .child(icon("ellipsis"))
}

/// Reactive text breadcrumb items. Use [`breadcrumb_content`] for rich content.
#[track_caller]
pub fn breadcrumb(items: impl IntoValue<Vec<String>>) -> Node {
    let items = items.into_value();
    breadcrumb_content(kivact::view::dynamic(items, |items| {
        let count = items.len();
        let mut nodes = Vec::new();
        for (i, item) in items.into_iter().enumerate() {
            if i > 0 {
                nodes.push(breadcrumb_separator().into_view());
            }
            nodes.push(if i + 1 == count {
                breadcrumb_page(item).into_view()
            } else {
                breadcrumb_link(item).into_view()
            });
        }
        nodes
    }))
}
