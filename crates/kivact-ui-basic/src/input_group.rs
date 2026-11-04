//! Role: Input group: a bordered row of addons around one borderless control.

use crate::theme_access::{field_fill, tokens};
use kivact::state::memo;
use kivact::state::{Binding, IntoValue, Value};
use kivact::view::{Node, NodeBuilder, column, node, row, text};

use kivact::state::{IntoText, provide, try_use_context};
use kivact::style::{Align, Color, Direction, Justify, Style, pct, style};
use kivact::ui::{Button, ButtonVariant, ControlSize, TextInput, text_area, text_input};
use kivact::view::{IntoView, NodeRef, Role, View, node_ref};
use kivact::view::{ReadingDirection, use_direction};
use std::any::Any;

/// The sizes of an [`input_group_button`] (`InputGroupButton`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum InputGroupButtonSize {
    /// `h-6 px-1.5` with a 14 px icon.
    #[default]
    Xs,
    /// The standard button height.
    Sm,
    /// A 24 px square for an icon.
    IconXs,
    /// A 28 px square for an icon.
    IconSm,
}

/// The block addons around the control, which the group provides to it
/// (`has-[>[data-align=block-start]]`, `has-[>[data-align=block-end]]`).
#[derive(Clone, Copy)]
struct Around {
    block_start: bool,
    block_end: bool,
}

/// An addon and whether it is a button (`has-[>button]`).
struct Addon {
    view: View,
    button: bool,
}

impl Addon {
    fn new<V: IntoView + 'static>(addon: V) -> Self {
        let any: &dyn Any = &addon;
        let button = any.is::<InputGroupButton>() || any.is::<Button>();
        Addon {
            view: addon.into_view(),
            button,
        }
    }
}

/// A bordered group of one control and its addons (`InputGroup`).
///
/// Created by [`input_group`]. The group draws the border and the focus ring
/// (it shows while any control inside has the focus). Compose the control with
/// [`input_group_input`] or [`input_group_textarea`]. Pressing an addon
/// outside its buttons focuses the control. An inline addon that is an
/// [`input_group_button`] (or a `kivact::ui::Button`) sits 0.3 rem closer to
/// the border. In a right-to-left [`direction`](kivact::view::provide_direction) boundary the
/// inline addons swap sides.
///
/// Give `.disabled(..)` the same flag as the control to draw the group
/// disabled (half opacity, `input/50` fill; `has-disabled`). It only paints:
/// disable the control and the buttons inside as well. `.invalid(..)` draws
/// the destructive border and ring.
pub struct InputGroup {
    root: Node,
    input: View,
    start: Option<Addon>,
    end: Option<Addon>,
    block_start: Option<kivact::view::View>,
    block_end: Option<kivact::view::View>,
    anchor: Option<NodeRef>,
    /// The reading direction when the group was made.
    rtl: bool,
}

/// An input group around `input`.
///
/// ```no_run
/// use kivact::view::{icon};
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn search() -> impl IntoView {
///     let query = signal(String::new());
///     basic::input_group(basic::input_group_input("Search", query).placeholder("Search..."))
///         .start(icon("search"))
///         .end(basic::input_group_text("12 results"))
/// }
/// ```
#[track_caller]
pub fn input_group(input: impl IntoView) -> InputGroup {
    InputGroup {
        root: node().with_node(|root| root.role(Role::Group)),
        input: input.into_view(),
        start: None,
        end: None,
        block_start: None,
        block_end: None,
        anchor: None,
        rtl: use_direction().get() == ReadingDirection::Rtl,
    }
}

impl InputGroup {
    /// An addon before the control (`align="inline-start"`).
    pub fn start<V: IntoView + 'static>(mut self, addon: V) -> Self {
        self.start = Some(Addon::new(addon));
        self
    }

    /// An addon after the control (`align="inline-end"`).
    pub fn end<V: IntoView + 'static>(mut self, addon: V) -> Self {
        self.end = Some(Addon::new(addon));
        self
    }

    /// An addon row above the control (`align="block-start"`).
    pub fn block_start(mut self, addon: impl IntoView) -> Self {
        self.block_start = Some(addon.into_view());
        self
    }

    /// An addon row below the control (`align="block-end"`).
    pub fn block_end(mut self, addon: impl IntoView) -> Self {
        self.block_end = Some(addon.into_view());
        self
    }
}

impl InputGroup {
    /// Bind `anchor` to the group's box, for a popup that opens below it.
    pub(crate) fn anchored(mut self, anchor: NodeRef) -> Self {
        self.anchor = Some(anchor);
        self
    }
}

impl NodeBuilder for InputGroup {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

/// An inline addon: `text-xs font-medium text-muted-foreground` centered items.
/// Pressing it outside a button activates the group's control (`input`).
/// Its `py-1.5` is left out: in the `h-8` group a 24 px button and that
/// padding overflow the 30 px row without growing it, and here they would
/// grow it to 36 px.
fn inline_addon(addon: Addon, padding_start: bool, input: NodeRef) -> Node {
    let button = addon.button;
    node()
        .child(addon.view)
        .on_press(move || input.activate())
        .with_node(|root| root.role(Role::Generic))
        .style(move || {
            let s = style()
                .direction(Direction::Row)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .gap(8)
                .shrink(0)
                .font_size(12)
                .weight(500)
                .color(tokens().muted_foreground);
            // `pl-2 has-[>button]:ml-[-0.3rem]`, mirrored at the end.
            match (padding_start, button) {
                (true, false) => s.padding_left(8),
                (true, true) => s.padding_left(8).margin_left(-4.8),
                (false, false) => s.padding_right(8),
                (false, true) => s.padding_right(8).margin_right(-4.8),
            }
        })
}

fn block_addon(view: View, top: bool, input: NodeRef) -> Node {
    node()
        .child(view)
        .on_press(move || input.activate())
        .with_node(|root| root.role(Role::Generic))
        .style(move || {
            let s = style()
                .direction(Direction::Row)
                .align_items(Align::Center)
                .justify_content(Justify::Start)
                .gap(8)
                .width(pct(100))
                .padding_x(10)
                .font_size(12)
                .weight(500)
                .color(tokens().muted_foreground);
            // `py-1.5`, with `pt-2` before the control and `pb-2` after it.
            if top {
                s.padding_top(8).padding_bottom(6)
            } else {
                s.padding_top(6).padding_bottom(8)
            }
        })
}

impl IntoView for InputGroup {
    fn into_view(self) -> View {
        let InputGroup {
            mut root,
            input,
            start,
            end,
            block_start,
            block_end,
            anchor,
            rtl,
        } = self;
        let invalid = root.accessible_invalid();
        let disabled = root.accessible_disabled();
        let anchor = anchor.unwrap_or_else(node_ref);
        let focused = anchor.focus_within();
        let bad = memo(move || invalid.as_ref().is_some_and(|read| read.get()));
        let off = memo(move || disabled.as_ref().is_some_and(|read| read.get()));
        root = root.base_style(move || {
            let p = tokens();
            let dark = p.is_dark();
            let mut s = style()
                .direction(Direction::Column)
                .width(pct(100))
                .min_width(0)
                .min_height(32)
                .shrink(0)
                .radius(0)
                .background(field_fill(&p))
                .border(1, p.input);
            if off.get() {
                // `has-disabled:bg-input/50 has-disabled:opacity-50`, `/80` in the dark.
                s = s
                    .merge(&crate::theme_access::disabled())
                    .background(p.input.with_alpha(if dark { 0.8 } else { 0.5 }));
            }
            if bad.get() {
                let ring = p.destructive.with_alpha(if dark { 0.4 } else { 0.2 });
                s.border_color(p.destructive).ring(1, ring)
            } else if focused.get() {
                // The control inside has the focus: the group is not a focus
                // stop, so it follows the control's focus.
                s.merge(&crate::theme_access::focus_ring(&p))
            } else {
                s
            }
        });
        // `[&>input]:pl-1.5` and `pr-1.5`: the input sits closer to an inline addon.
        let (near_start, near_end) = (start.is_some(), end.is_some());
        let control = node_ref();
        let inline = row((
            start.map(|v| inline_addon(v, true, control)),
            node().child(input).node_ref(control).style(
                style()
                    .grow(1)
                    .min_width(0)
                    .direction(Direction::Row)
                    .margin_left(if near_start { -4 } else { 0 })
                    .margin_right(if near_end { -4 } else { 0 }),
            ),
            end.map(|v| inline_addon(v, false, control)),
        ))
        .style(
            style()
                .direction(if rtl {
                    Direction::RowReverse
                } else {
                    Direction::Row
                })
                .align_items(Align::Center)
                .width(pct(100))
                .min_width(0),
        );
        let around = Around {
            block_start: block_start.is_some(),
            block_end: block_end.is_some(),
        };
        provide(around, move || {
            root.node_ref(anchor).child(column((
                block_start.map(|v| block_addon(v.into_view(), true, control)),
                inline,
                block_end.map(|v| block_addon(v.into_view(), false, control)),
            )))
        })
    }
}

/// An addon text with `text-xs text-muted-foreground` (`InputGroupText`).
#[track_caller]
pub fn input_group_text(label: impl IntoText) -> impl kivact::view::Component {
    row(text(label)).style(
        style()
            .align_items(Align::Center)
            .gap(8)
            .font_size(12)
            .color(crate::Color::MutedForeground),
    )
}

/// A button inside an input group (`InputGroupButton`): ghost, extra small.
///
/// Created by [`input_group_button`].
pub struct InputGroupButton {
    button: Button,
    size: Value<InputGroupButtonSize>,
}

/// The measures of one size of the button (`inputGroupButtonVariants`).
fn button_style(size: InputGroupButtonSize) -> Style {
    match size {
        InputGroupButtonSize::Xs => style().height(24).padding_x(6).gap(4),
        InputGroupButtonSize::Sm => style().height(32).padding_x(10).gap(4),
        InputGroupButtonSize::IconXs => style().size(24, 24).padding(0),
        InputGroupButtonSize::IconSm => style().size(28, 28).padding(0),
    }
}

/// A ghost button named `label` for an input group addon.
#[track_caller]
pub fn input_group_button(label: impl IntoText) -> InputGroupButton {
    let button = kivact::ui::button(label).variant(ButtonVariant::Ghost);
    InputGroupButton {
        button,
        size: InputGroupButtonSize::Xs.into_value(),
    }
    .size(InputGroupButtonSize::Xs)
}

impl InputGroupButton {
    /// The size (default [`InputGroupButtonSize::Xs`]).
    pub fn size(mut self, size: impl IntoValue<InputGroupButtonSize>) -> Self {
        let size = size.into_value();
        self.size = size;
        self.button = self
            .button
            .size(move || match size.get() {
                InputGroupButtonSize::Xs | InputGroupButtonSize::IconXs => ControlSize::Small,
                InputGroupButtonSize::Sm | InputGroupButtonSize::IconSm => ControlSize::Medium,
            })
            .hide_label(move || {
                matches!(
                    size.get(),
                    InputGroupButtonSize::IconXs | InputGroupButtonSize::IconSm
                )
            });
        self
    }

    /// The button variant (default Ghost).
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.button = self.button.variant(variant);
        self
    }

    /// A leading icon.
    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.button = self.button.icon(icon);
        self
    }
}

impl NodeBuilder for InputGroupButton {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        self.button.node_mut()
    }
}

impl IntoView for InputGroupButton {
    fn into_view(self) -> View {
        let size = self.size;
        self.button
            .with_node(|node| node.base_style(move || button_style(size.get())))
            .into_view()
    }
}

/// The borderless look of `InputGroupInput` and `InputGroupTextarea`.
///
/// A single-line input (`height`) also follows the block addons of the group
/// it is in: with an addon row below it, `pt-3`; with one above it, `pb-3`;
/// and either way the full 32 px of `h-8`, since the group is `h-auto` then.
pub(crate) fn borderless(input: TextInput, height: Option<f64>) -> TextInput {
    input.style(move || {
        let s = style()
            .grow(1)
            .min_width(0)
            .border(0, Color::TRANSPARENT)
            .ring(0, Color::TRANSPARENT)
            .background(Color::TRANSPARENT);
        // `disabled:bg-transparent`: the group dims itself, so the control must
        // not dim a second time. `focus-visible:ring-0`: the group draws the
        // focus, so the field draws no box of its own inside it.
        let s = s
            .disabled(style().opacity(1.0).background(Color::TRANSPARENT))
            .focus(style().border_color(Color::TRANSPARENT).ring_none())
            .focus_visible(style().border_color(Color::TRANSPARENT).ring_none());
        let Some(height) = height else { return s };
        let around = try_use_context::<Around>();
        let below = around.is_some_and(|around| around.block_end);
        let above = around.is_some_and(|around| around.block_start);
        let s = s.height(if below || above { 32.0 } else { height });
        let s = if below { s.padding_top(12) } else { s };
        if above { s.padding_bottom(12) } else { s }
    })
}

/// The text input of an input group (`InputGroupInput`): no border, no ring.
#[track_caller]
pub fn input_group_input(
    label: impl IntoText,
    value: impl kivact::state::IntoBinding<String>,
) -> TextInput {
    borderless(text_input(label, value), Some(30.0))
}

/// The text area of an input group (`InputGroupTextarea`).
#[track_caller]
pub fn input_group_textarea(
    label: impl IntoText,
    value: impl kivact::state::IntoBinding<String>,
) -> TextInput {
    borderless(text_area(label, value), None).style(style().padding_y(8))
}

kivact::state::constant_value!(InputGroupButtonSize);
