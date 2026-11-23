//! Role: `Style`: a sparse value chain and every setter of its properties.
//!
//! Setting a property twice keeps the last value; debug builds record the
//! duplicate in the diagnostics trace. The text, grid and scrollbar setters
//! are in the child modules too: they fill the boxed groups the style keeps
//! out of line.
//!
//! A [`Style`] is a flat value of `Option` fields, and a node's style is
//! the list of its `.style(..)` layers. A layer that uses the theme reads
//! `use_theme()` in its closure (tracked). Each node has one style
//! effect: it reads the theme and the disabled and selected states, merges
//! the layers in order and resolves them with `crate::internal::compile_style` into the
//! engine's typed props. The theme switching re-resolves the paint and
//! remounts nothing.
//!
//! Interaction states:
//! - `hover`, `active`, `focus`, `focus_visible`, `group_hover` and
//!   `group_focus` accept every property. The paint ones (background, border
//!   colors and line styles, outline, radii, opacity, shadows, transforms,
//!   z-index, scrollbar, cursor, filters, mask, clip path, text color) are
//!   deltas the host draws at once; the others (sizes, spacing, border
//!   widths, text properties, a paint callback) are compiled into the node's
//!   style while the host reports the state, so they lay out again like CSS.
//!   `enter` and `exit` are where a paint transition starts and ends (CSS
//!   `@starting-style`); properties a transition does not move have no effect
//!   there. `disabled` and `selected` are merged when the style is compiled.
//! - Hover is CSS `:hover`: the pointer is over the node or a descendant
//!   (the node is on the host's hover chain), so a hovered child keeps its
//!   parent hovered. There is no `hovered` style; `NodeRef::hovered` reads
//!   the same rule reactively, and `NodeRef::focus_within` the focus of
//!   the node or a descendant (`NodeRef::focused` the node alone).
//! - A descendant that changes with an ancestor's hover uses `group()` on
//!   the ancestor and `group_hover(s)` on itself (Tailwind's `group` and
//!   `group-hover:`): it applies under the node's own hover while the
//!   nearest group ancestor is hovered. `group_focus(s)` uses the same
//!   group and applies under the node's own focus while the focus is on the
//!   nearest group ancestor or below it (`:focus-within`). The engine
//!   evaluates both and repaints the affected nodes; neither reads a
//!   pointer event or a signal.
//! - A focus stop (focusable, or a native editor) gets a style effect even
//!   without a style, and resolve gives it the theme's defaults: without a
//!   `focus` or `focus_visible` state, `focus_visible` gets
//!   `outline(theme.focus_ring(), kivact::style::color::FOCUS)`; while disabled, the
//!   disabled state gets `theme.get(&kivact::style::control::DISABLED_OPACITY_VAR)` unless it sets an opacity,
//!   and hover and active are dropped. The defaults sit on the node that
//!   carries the disabled semantics and the focus, so drawing and meaning
//!   agree (G-07).
//!
//! Entry points that look alike but differ:
//! - Children are placed with `align_items` and `justify_content`, text
//!   with `text_align`; the `align` of an `Overlay` or a tooltip places the
//!   floating layer, and `TextStyle.align` places a `Draw` text against its
//!   anchor.
//! - `paint` is the recipes' and themes' drawing (kept by style merging);
//!   the caller uses `.draw` on the node, and `.draw_over` draws above the
//!   children.
//! - `clip(true)` clips drawing only; `overflow(Overflow::Hidden)` also
//!   changes layout overflow. `Overflow::Scroll` sets the overflow of both
//!   axes, while `scroll(child)` limits the axis (vertical by default,
//!   `.axes(ScrollAxes::Horizontal)` or `.axes(ScrollAxes::Both)` otherwise)
//!   and adds the `ScrollView` role.

mod decl;
mod edit_text;
mod folded;
mod kinds;
pub(crate) mod props;
mod shorthands;
mod styled_range;

#[doc(inline)]
pub use kivact_types::FlexDirection;

/// How content is distributed along an axis (CSS `justify-content`,
/// `align-content`).
///
/// `Stretch`: Grow auto grid tracks (or wrapped flex lines) to fill the
/// container (`align_content`); like `Start` for a flex `justify`.
#[doc(inline)]
pub use kivact_types::Justify;

/// How a box is positioned (CSS `position`), with the insets `top`,
/// `right`, `bottom`, `left` (`inset` for all four).
///
/// `Relative`: In flow (the default; insets do nothing).
///
/// `Absolute`: Out of flow, at its insets in its parent's box (CSS
/// `absolute` in a positioned parent).
///
/// `Sticky`: In flow, and kept at its insets from the edges of its nearest
/// scroll container's view while it scrolls (the window without one), but
/// never out of its parent: a table header with `top(0)` stays on top while
/// its rows scroll by, a column's first cell with `left(0)` while they
/// scroll sideways. It is drawn, hit, reported and exposed where it is
/// stuck, over its in-flow siblings of the same z-index.
///
/// `Fixed`: At its insets in the window (percentages of the window's size),
/// whatever scrolls or clips around it: a toast stack
/// (`bottom(16).right(16)`), an off-canvas sidebar and its scrim, a
/// maximized panel (`top(96).left(12).right(12).bottom(32)`). It is a layer
/// of its own over the window's content (over the content of the overlay it
/// is in), ordered with the other fixed boxes by z-index; its ancestors'
/// transforms, opacity and filters do not apply to it. It stays in the tree
/// (focus, Tab order, accessibility, state).
#[doc(inline)]
pub use kivact_types::Position;

#[doc(inline)]
pub use kivact_types::Overflow;

/// Property group animated by `Style::transition` (the wire enum).
#[doc(inline)]
pub use kivact_types::TransitionProp;

pub use kinds::ScrollbarWidth;
pub use props::{Style, StyleProperty};
pub use shorthands::{style, with_scroll_axes};
pub use styled_range::StyledRange;

pub(crate) use folded::{Groups, Sides, SizeFields, SpacingFields, TextFields, TransformFields};
pub(crate) use kinds::{PropClass, PropGroup};
