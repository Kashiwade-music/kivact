//! Role: Small Lyra primitives: label, separator, kbd, spinner, skeleton and more.

use crate::Color;
use crate::theme_access::tokens;
use kivact::Value;
use kivact::control::icon;
use kivact::control::{Node, NodeBuilder, node, row, text};
use kivact::motion::{Animated, Easing, Repeat, Transition};
use kivact::primitive::NodeExt;
use kivact::style::{Align, Direction, Justify, pct, style};
use kivact::view::{IntoText, IntoValue, IntoView, NodeRef, Orientation, Role, View};

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
/// use kivact::view::NodeRef;
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
                .color(Color::Foreground)
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
            .role(Role::Generic);
        self
    }
}

impl NodeBuilder for Label {
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
        .role(Role::Separator)
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
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Separator {
    fn into_view(self) -> View {
        let orientation = self.orientation;
        self.root
            .base_style(move || {
                let p = tokens();
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

/// A keyboard key cap (kbd.tsx): muted, 20 px high, 12 px medium text.
#[track_caller]
pub fn kbd(text_value: impl IntoText) -> Node {
    node()
        .style(
            style()
                .direction(Direction::Row)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .gap(4)
                .height(20)
                .min_width(20)
                .padding_x(4)
                .radius(0)
                .background(Color::Muted)
                .color(Color::MutedForeground)
                .font_size(12)
                .weight(500)
                .text_wrap(false),
        )
        .child(text(text_value))
}

/// Key caps shown together, such as a shortcut (`KbdGroup`).
#[track_caller]
pub fn kbd_group(children: impl IntoView) -> Node {
    row(children).style(style().align_items(Align::Center).gap(4))
}

/// A turning Lucide `loader-circle` (spinner.tsx) with `Role::Status`.
///
/// Its accessible name is the `spinner.loading` message ("Loading"). The
/// rotation is essential motion (`Transition::decorative(false)`): it says
/// that work is going on, so it keeps turning under reduced motion, as the
/// platforms' own spinners do.
#[track_caller]
pub fn spinner() -> Node {
    let turn = Animated::from(
        0.0f32,
        360.0f32,
        Transition::tween(1000, Easing::Linear)
            .repeat(Repeat::Forever)
            .decorative(false),
    );
    node()
        .role(Role::Status)
        .name(kivact::i18n::message("basic.spinner.loading", "Loading"))
        .style(move || style().size(16, 16).shrink(0).rotate(turn.get()))
        .child(icon("loader-circle"))
}

/// A pulsing muted placeholder (skeleton.tsx); size it with `.style(..)`.
///
/// The pulse is decorative motion: under reduced motion the block is opaque
/// and still.
#[track_caller]
pub fn skeleton() -> Skeleton {
    Skeleton {
        root: kivact::control::column(()),
        rows: 1usize.into_value(),
    }
}
/// Presentation builder returned by [`skeleton`].
pub struct Skeleton {
    root: Node,
    rows: kivact::Value<usize>,
}
impl Skeleton {
    /// Reactive row count; one by default.
    pub fn rows(mut self, rows: impl IntoValue<usize>) -> Self {
        self.rows = rows.into_value();
        self
    }
}
impl NodeBuilder for Skeleton {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Skeleton {
    fn into_view(self) -> View {
        let rows = self.rows;
        let pulse = Animated::from(
            1.0f32,
            0.5f32,
            Transition::tween(1000, Easing::EaseInOut)
                .repeat(Repeat::Forever)
                .alternate(),
        );
        self.root
            .base_style(style().gap(6).clip(true))
            .child(kivact::view::for_index(
                move || (0..rows.get()).collect::<Vec<_>>(),
                move |_, _| {
                    node().style(move || {
                        style()
                            .grow(1)
                            .min_height(16)
                            .width(pct(100))
                            .radius(0)
                            .background(crate::theme_access::tokens().muted)
                            .opacity(if kivact::platform::reduced_motion() {
                                1.
                            } else {
                                pulse.get()
                            })
                    })
                },
            ))
            .into_view()
    }
}

/// A box of the full available width whose height follows `ratio` (width / height).
///
/// The child is stretched to the box's width; give it `height(pct(100))`
/// to fill the box.
#[track_caller]
pub fn aspect_ratio(ratio: f32, child: impl IntoView) -> Node {
    node()
        .style(
            style()
                .direction(Direction::Column)
                .width(pct(100))
                .aspect_ratio(ratio)
                .overflow(kivact::style::Overflow::Hidden),
        )
        .child(child)
}
