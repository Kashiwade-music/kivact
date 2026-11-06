//! Role: Small Lyra primitives: label, separators, key groups, captions, crumb ellipsis.

use crate::ColorName;

use kivact::state::Value;
use kivact::view::{Node, NodeBuilder, icon, node, row, text};

use kivact::state::{IntoText, IntoValue};
use kivact::style::{Align, Direction, Justify, TextAlign, pct, style};
use kivact::view::{IntoView, NodeRef, Orientation, Role, View};

/// A form label (label.tsx): 12 px text without leading, centered with its children.
///
/// Created by [`label`]. With [`Label::target`], pressing the label
/// activates the target control as a click on it would.
pub struct Label {
    root: Node,
}

/// A label showing `text`.
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
///         text_input("Email", value).node_ref(field),
///     ))
///     .style(style().gap(8))
/// }
/// ```
#[track_caller]
pub fn label(text_value: impl IntoText) -> Label {
    let root = node()
        .style(
            style()
                .direction(Direction::Row)
                .align_items(Align::Center)
                .gap(8)
                .font_size(12)
                .line_height(1.0)
                .color(ColorName::Foreground)
                .disabled(crate::theme_access::disabled()),
        )
        .child(text(text_value));
    Label { root }
}

impl Label {
    /// Activate `target` when the label is pressed, as an HTML
    /// `<label for>`: a check box or switch toggles and takes the focus, a
    /// text input takes the focus. `target` may be bound to the control or
    /// to a node around it (the first control inside is activated).
    pub fn target(mut self, target: NodeRef) -> Self {
        // The label is a pointer-only forwarder: it has no role of its own.
        self.root = self
            .root
            .on_press(move || target.activate())
            .with_node(|root| root.role(Role::Generic));
        self
    }
}

impl NodeBuilder for Label {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Label {
    fn into_view(self) -> View {
        self.root.into_view()
    }
}

/// A 1 px line in the border color (separator.tsx), with `Role::Separator`.
///
/// Created by [`separator`]; horizontal unless [`Separator::orientation`] says
/// [`Orientation::Vertical`].
pub struct Separator {
    root: Node,
    orientation: Option<Value<Orientation>>,
}

/// A horizontal separator that spans its container's width.
#[track_caller]
pub fn separator() -> Separator {
    let root = node()
        .with_node(|root| root.role(Role::Separator))
        .orientation(Orientation::Horizontal);
    Separator {
        root,
        orientation: None,
    }
}

impl Separator {
    /// Draw a vertical line that stretches across the parent row's height
    /// ([`Orientation::Vertical`]), or a horizontal one; `Unspecified` is
    /// horizontal. Constant or reactive.
    #[track_caller]
    pub fn orientation(mut self, orientation: impl IntoValue<Orientation>) -> Self {
        let orientation = orientation.into_value();
        self.orientation = Some(orientation);
        self.root = self.root.orientation(orientation);
        self
    }
}

impl NodeBuilder for Separator {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Separator {
    fn into_view(self) -> View {
        let orientation = self.orientation;
        self.root
            .base_style(move || {
                let p = kivact::style::use_tokens::<crate::Tokens>();
                let s = style().shrink(0).background(p.border);
                if orientation.is_some_and(|o| o.get() == Orientation::Vertical) {
                    s.width(1).align_self(Align::Stretch)
                } else {
                    s.height(1).width(pct(100))
                }
            })
            .into_view()
    }
}

/// A horizontal rule between the rows of a `kivact::ui::item_list`, with
/// 8 px above and below.
#[track_caller]
pub fn item_separator() -> Node {
    node()
        .role(Role::Separator)
        .orientation(Orientation::Horizontal)
        .style(
            style()
                .width(pct(100))
                .height(1)
                .shrink(0)
                .margin_y(8)
                .background(ColorName::Border),
        )
}

/// Key caps of `kivact::ui::kbd` shown together, such as a shortcut
/// (kbd.tsx `KbdGroup`).
#[track_caller]
pub fn kbd_group(children: impl IntoView) -> Node {
    row(children).style(style().align_items(Align::Center).gap(4))
}

/// A caption under a `kivact::ui::table`, muted and centered, with 16 px
/// above it. Put it after the table: it is drawn where it is placed.
#[track_caller]
pub fn table_caption(text_value: impl IntoText) -> Node {
    text(text_value).style(
        style()
            .margin_top(16)
            .color(ColorName::MutedForeground)
            .text_align(TextAlign::Center),
    )
}

/// The "…" that stands for hidden crumbs in the `item` slot of a
/// `kivact::ui::breadcrumb`, read as the `basic.breadcrumb.more` message
/// ("More").
#[track_caller]
pub fn breadcrumb_ellipsis() -> Node {
    node()
        .role(Role::Text)
        .name(kivact::i18n::message("basic.breadcrumb.more", "More"))
        .style(
            style()
                .size(20, 20)
                .shrink(0)
                .direction(Direction::Row)
                .align_items(Align::Center)
                .justify_content(Justify::Center),
        )
        .child(icon("ellipsis"))
}
