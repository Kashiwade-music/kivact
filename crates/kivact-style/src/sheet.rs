//! Role: `Style`: a flat value-type chain and every setter of its properties.
//!
//! Setting a property twice keeps the last value; debug builds record the
//! duplicate in the diagnostics trace. The text, grid and scrollbar setters
//! are here too: they fill the boxed groups the style keeps out of line.
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
//! - `hover`, `pressed`, `focus` and `focus_visible` become `StateStyles`
//!   deltas the host evaluates (`focus_visible` lies over `focus` while the
//!   focus is visible); `enter` and `exit` are deltas of the same kind, on
//!   top right after mounting and while leaving (`kivact::motion`).
//!   `disabled` and `selected` are merged when the style is compiled.
//!   A state style changes paint only (see [`TransitionProp::All`]); a
//!   layout or text property other than `color` set in a delta state is
//!   reported as `Unsupported` and ignored. `selected` and `disabled` apply
//!   any property.
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
//!   `ring(theme.focus_ring(), ColorToken::Focus)`; while disabled, the
//!   disabled state gets `theme.disabled_opacity()` unless it sets an opacity,
//!   and hover and pressed are dropped. The defaults sit on the node that
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
//!
//! Layout vocabulary:
//! - `node()` and `column` are vertical flex boxes, `row` horizontal; `gap`
//!   sets `row_gap` and `column_gap`. Children are aligned with
//!   `align_items` and `justify_content`; `align_content` is a `Justify`
//!   (with `Stretch`). There are `aspect_ratio`, the positions `Relative`,
//!   `Absolute`, `Sticky` and `Fixed` with `top`/`right`/`bottom`/`left`
//!   and `inset`. A justification left unset is `None` (CSS `normal`: the
//!   start of a flex box, stretched auto tracks of a grid).
//! - In a row, `Align::Baseline` aligns first baselines: those of texts,
//!   native editors and a primitive's `Measurement`; a leaf without one
//!   uses its box's bottom, a container its first child's baseline.
//! - `grid(children)` is the one way to make a grid box (`Display` is not
//!   public); see `crate::grid` for tracks and placement.
//! - A [`Length`] in px, percent or calc works throughout layout; the
//!   content sizes work only for widths, heights, the basis and tracks
//!   (elsewhere they are `auto`, reported once per node as `Unsupported`).
//!   A component prop taking `IntoValue<Length>` takes a number as logical
//!   px. `platform::window_size` is tracked, so media queries and `vw`,
//!   `vh` or `min()` equivalents are computed from it.
//! - `overlay(anchor, placement, child)` has an `align` (start, center or
//!   end of the side; `Stretch` and `Baseline` mean start) and an `offset`
//!   in px; `overlay_at` takes a window point (`PointerEvent::window_pos`).
//!   A floating part with an anchor, flipping, dismissal or a modal is an
//!   overlay; a fixed place in the window is `Fixed`.
//! - Padding and margin take all sides, `x`/`y`, and each side alone
//!   (`padding_top`, ..., `margin_left`), merged side by side with the same
//!   `Length` and duplicate diagnostics; one side never sets another, so an
//!   asymmetric layout needs only the public `Style`.

use kivact_types::{
    Align, BackfaceVisibility, Cursor, Isolation, ScrollAxes, TextAlign, TransformStyle, WillChange,
};

use super::color::Color;
use super::draw::{Draw, LocalPaint, Size};
use super::effect::{BlendMode, BorderStyle, CornerRadius, Filter, Shadow, Shape};
#[cfg(doc)]
use super::grid::Track;
use super::grid::{GridAutoFlow, GridPlacement, GridStyle, Tracks};
use super::length::Length;
use super::paint::{Background, BorderImage, Mask};
use super::text::{
    FontFeature, FontKerning, FontVariation, LineHeight, PaintOrder, TextDecorationLine,
    TextExtras, TextLength, TextOverflow, TextTransform,
};
use super::theme::value::ThemeColor;
use crate::transform::Transform;
use kivact_runtime::Transition;

#[doc(inline)]
pub use kivact_types::FlexDirection as Direction;

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

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub(crate) struct Sides {
    // xtask-allow(state-fields): passive record - top
    pub(crate) top: Option<Length>,
    // xtask-allow(state-fields): passive record - right
    pub(crate) right: Option<Length>,
    // xtask-allow(state-fields): passive record - bottom
    pub(crate) bottom: Option<Length>,
    // xtask-allow(state-fields): passive record - left
    pub(crate) left: Option<Length>,
}

impl Sides {
    fn merge(&mut self, o: &Sides) {
        self.top = o.top.or(self.top);
        self.right = o.right.or(self.right);
        self.bottom = o.bottom.or(self.bottom);
        self.left = o.left.or(self.left);
    }
}

/// The `size` group of a [`Style`]: the box's size and flex sizing.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct SizeFields {
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) width: Option<Length>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) height: Option<Length>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) min_width: Option<Length>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) min_height: Option<Length>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) max_width: Option<Length>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) max_height: Option<Length>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) basis: Option<Length>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) aspect_ratio: Option<f64>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) grow: Option<f64>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) shrink: Option<f64>,
}

impl SizeFields {
    fn merge(&mut self, o: &Self) {
        macro_rules! take {
            ($($f:ident),*) => {$(
                if o.$f.is_some() {
                    self.$f = o.$f;
                }
            )*};
        }
        take!(
            width,
            height,
            min_width,
            min_height,
            max_width,
            max_height,
            basis,
            aspect_ratio,
            grow,
            shrink
        );
    }
}

/// The `spacing` group of a [`Style`]: padding, margin, insets and gaps.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct SpacingFields {
    pub(crate) padding: Sides,
    pub(crate) margin: Sides,
    pub(crate) inset: Sides,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) row_gap: Option<Length>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) column_gap: Option<Length>,
}

impl SpacingFields {
    fn merge(&mut self, o: &Self) {
        self.padding.merge(&o.padding);
        self.margin.merge(&o.margin);
        self.inset.merge(&o.inset);
        if o.row_gap.is_some() {
            self.row_gap = o.row_gap;
        }
        if o.column_gap.is_some() {
            self.column_gap = o.column_gap;
        }
    }
}

/// The `flow` group of a [`Style`]: flex, grid, position and overflow.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct FlowFields {
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) direction: Option<Direction>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) align: Option<Align>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) align_self: Option<Align>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) justify: Option<Justify>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) align_content: Option<Justify>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    /// Lays the children out as a grid (CSS `display: grid`; set by
    /// `grid(..)` only).
    pub(crate) display: Option<kivact_types::Display>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) visibility: Option<kivact_types::Visibility>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    /// Grid tracks and placement (rare: boxed).
    pub(crate) grid: Option<Box<GridStyle>>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) wrap: Option<bool>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) position: Option<Position>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) overflow: Option<Overflow>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    /// Scroll axes set by `scroll(..)`.
    pub(crate) scroll_axes: Option<ScrollAxes>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) scroll_snap: Option<kivact_types::ScrollSnap>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) z_index: Option<i32>,
}

impl FlowFields {
    fn merge(&mut self, o: &Self) {
        macro_rules! take {
            ($($f:ident),*) => {$(
                if o.$f.is_some() {
                    self.$f = o.$f.clone();
                }
            )*};
        }
        take!(
            direction,
            align,
            align_self,
            justify,
            align_content,
            display,
            visibility,
            wrap,
            position,
            overflow,
            scroll_axes,
            scroll_snap,
            z_index
        );
        if let Some(g) = &o.grid {
            self.grid.get_or_insert_with(Default::default).merge(g);
        }
    }
}

/// The `paint` group of a [`Style`]: backgrounds, borders, effects and
/// the ways a box is drawn.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct PaintFields {
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) local_paint: Option<LocalPaint>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) background: Option<Background>,
    /// Border widths and colors, top, right, bottom, left.
    pub(crate) border_width: [Option<f64>; 4],
    pub(crate) border_color: [Option<Color>; 4],
    /// Border line styles, top, right, bottom, left.
    pub(crate) border_style: [Option<BorderStyle>; 4],
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) ring: Option<(f64, f64, Color)>,
    /// Corner radii, top left, top right, bottom right, bottom left.
    pub(crate) radius: [Option<CornerRadius>; 4],
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) shadows: Option<Vec<Shadow>>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) opacity: Option<f64>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) isolation: Option<Isolation>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) border_image: Option<Box<BorderImage>>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) scrollbar: Option<Box<ScrollbarStyle>>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) cursor: Option<Cursor>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) clip: Option<bool>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) filter: Option<Filter>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) backdrop_filter: Option<Filter>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) blend: Option<BlendMode>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) mask: Option<Box<Mask>>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) clip_path: Option<Box<Shape>>,
}

impl PaintFields {
    fn merge(&mut self, o: &Self) {
        macro_rules! take {
            ($($f:ident),*) => {$(
                if o.$f.is_some() {
                    self.$f = o.$f.clone();
                }
            )*};
        }
        take!(
            local_paint,
            background,
            ring,
            shadows,
            opacity,
            isolation,
            border_image,
            cursor,
            clip,
            filter,
            backdrop_filter,
            blend,
            mask,
            clip_path
        );
        if let Some(other) = &o.scrollbar {
            let mine = self.scrollbar.get_or_insert_with(Default::default);
            mine.width = other.width.or(mine.width);
            mine.colors = other.colors.or(mine.colors);
        }
        for (a, b) in self.border_width.iter_mut().zip(o.border_width) {
            *a = b.or(*a);
        }
        for (a, b) in self.border_style.iter_mut().zip(o.border_style) {
            *a = b.or(*a);
        }
        for (a, b) in self.border_color.iter_mut().zip(o.border_color) {
            *a = b.or(*a);
        }
        for (a, b) in self.radius.iter_mut().zip(o.radius) {
            *a = b.or(*a);
        }
    }
}

/// The `transform` group of a [`Style`]: the transform properties.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct TransformFields {
    /// CSS `translate` (x, then y: each axis is set and merged on its own),
    /// `rotate`, `scale`, `transform`, `transform-origin`.
    pub(crate) translate: [Option<Length>; 2],
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) rotate: Option<f64>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) scale: Option<(f64, f64)>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) transform: Option<Transform>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) transform_origin: Option<(Length, Length)>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) translate_z: Option<f64>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) rotate_x: Option<f64>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) rotate_y: Option<f64>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) transform_origin_z: Option<f64>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) perspective: Option<f64>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) perspective_origin: Option<(Length, Length)>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) transform_style: Option<TransformStyle>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) backface: Option<BackfaceVisibility>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) will_change: Option<WillChange>,
}

impl TransformFields {
    fn merge(&mut self, o: &Self) {
        macro_rules! take {
            ($($f:ident),*) => {$(
                if o.$f.is_some() {
                    self.$f = o.$f.clone();
                }
            )*};
        }
        take!(
            rotate,
            scale,
            transform,
            transform_origin,
            translate_z,
            rotate_x,
            rotate_y,
            transform_origin_z,
            perspective,
            perspective_origin,
            transform_style,
            backface,
            will_change
        );
        for (a, b) in self.translate.iter_mut().zip(o.translate) {
            *a = b.or(*a);
        }
    }
}

/// The `text` group of a [`Style`]: the text properties of a node and its
/// subtree.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct TextFields {
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) font_size: Option<f64>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) weight: Option<u16>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) color: Option<Color>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) line_height: Option<LineHeight>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) text_wrap: Option<bool>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) text_align: Option<TextAlign>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) font_family: Option<String>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) italic: Option<bool>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    /// The rarer text properties (letter spacing, decorations, features,
    /// overflow, clamps, case, shadows, input colors: boxed).
    pub(crate) text: Option<Box<TextExtras>>,
}

impl TextFields {
    fn merge(&mut self, o: &Self) {
        macro_rules! take {
            ($($f:ident),*) => {$(
                if o.$f.is_some() {
                    self.$f = o.$f.clone();
                }
            )*};
        }
        take!(
            font_size,
            weight,
            color,
            line_height,
            text_wrap,
            text_align,
            font_family,
            italic
        );
        if let Some(t) = &o.text {
            self.text.get_or_insert_with(Default::default).merge(t);
        }
    }
}

/// The `states` group of a [`Style`]: the interaction styles and the
/// nesting of a group.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct StateFields {
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) hover: Option<Box<Style>>,
    pub(crate) group: bool,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) group_hover: Option<Box<Style>>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) group_focus: Option<Box<Style>>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) pressed: Option<Box<Style>>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) focus: Option<Box<Style>>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) focus_visible: Option<Box<Style>>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) disabled: Option<Box<Style>>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) selected: Option<Box<Style>>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) enter: Option<Box<Style>>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) exit: Option<Box<Style>>,
}

impl StateFields {
    fn merge(&mut self, o: &Self) {
        self.group |= o.group;
        for (a, b) in [
            (&mut self.hover, &o.hover),
            (&mut self.group_hover, &o.group_hover),
            (&mut self.group_focus, &o.group_focus),
            (&mut self.pressed, &o.pressed),
            (&mut self.focus, &o.focus),
            (&mut self.focus_visible, &o.focus_visible),
            (&mut self.disabled, &o.disabled),
            (&mut self.selected, &o.selected),
            (&mut self.enter, &o.enter),
            (&mut self.exit, &o.exit),
        ] {
            if let Some(b) = b {
                *a = Some(Box::new(match a.take() {
                    Some(x) => (*x).merge(b),
                    None => (**b).clone(),
                }));
            }
        }
    }
}

/// The `motion` group of a [`Style`]: the enter/exit sizes and the
/// transitions.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct MotionFields {
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) enter_size: Option<kivact_types::Orientation>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) exit_size: Option<kivact_types::Orientation>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) layout_transition: Option<Transition>,
    pub(crate) transitions: Vec<(TransitionProp, Transition)>,
}

impl MotionFields {
    fn merge(&mut self, o: &Self) {
        if o.enter_size.is_some() {
            self.enter_size = o.enter_size;
        }
        if o.exit_size.is_some() {
            self.exit_size = o.exit_size;
        }
        if o.layout_transition.is_some() {
            self.layout_transition = o.layout_transition.clone();
        }
        for t in &o.transitions {
            self.transitions.retain(|(p, _)| *p != t.0);
            self.transitions.push(t.clone());
        }
    }
}

/// A style value. Build with [`style()`].
///
/// An authored style: each group of properties is allocated only when one
/// of its properties is set. Values and merge rules are the dense style's.
#[derive(Clone, Debug, Default)]
pub struct Style {
    // xtask-allow(state-fields): option: one group of authored properties; absent means the group's defaults
    pub(crate) size: Option<Box<SizeFields>>,
    // xtask-allow(state-fields): option: one group of authored properties; absent means the group's defaults
    pub(crate) spacing: Option<Box<SpacingFields>>,
    // xtask-allow(state-fields): option: one group of authored properties; absent means the group's defaults
    pub(crate) flow: Option<Box<FlowFields>>,
    // xtask-allow(state-fields): option: one group of authored properties; absent means the group's defaults
    pub(crate) paint: Option<Box<PaintFields>>,
    // xtask-allow(state-fields): option: one group of authored properties; absent means the group's defaults
    pub(crate) transform: Option<Box<TransformFields>>,
    // xtask-allow(state-fields): option: one group of authored properties; absent means the group's defaults
    pub(crate) text: Option<Box<TextFields>>,
    // xtask-allow(state-fields): option: one group of authored properties; absent means the group's defaults
    pub(crate) states: Option<Box<StateFields>>,
    // xtask-allow(state-fields): option: one group of authored properties; absent means the group's defaults
    pub(crate) motion: Option<Box<MotionFields>>,
    /// Setter calls that replaced an earlier value (the diagnostics count).
    pub(crate) duplicates: u32,
}

const _: () = assert!(
    std::mem::size_of::<Style>() <= 72,
    "Style keeps its groups boxed"
);

impl PartialEq for Style {
    fn eq(&self, other: &Self) -> bool {
        self.size.as_deref().cloned().unwrap_or_default()
            == other.size.as_deref().cloned().unwrap_or_default()
            && self.spacing.as_deref().cloned().unwrap_or_default()
                == other.spacing.as_deref().cloned().unwrap_or_default()
            && self.flow.as_deref().cloned().unwrap_or_default()
                == other.flow.as_deref().cloned().unwrap_or_default()
            && self.paint.as_deref().cloned().unwrap_or_default()
                == other.paint.as_deref().cloned().unwrap_or_default()
            && self.transform.as_deref().cloned().unwrap_or_default()
                == other.transform.as_deref().cloned().unwrap_or_default()
            && self.text.as_deref().cloned().unwrap_or_default()
                == other.text.as_deref().cloned().unwrap_or_default()
            && self.states.as_deref().cloned().unwrap_or_default()
                == other.states.as_deref().cloned().unwrap_or_default()
            && self.motion.as_deref().cloned().unwrap_or_default()
                == other.motion.as_deref().cloned().unwrap_or_default()
            && self.duplicates == other.duplicates
    }
}

/// The group of `style` named, or the group's defaults when it is unset:
/// `let size = group_or_default!(style.size, SizeFields);`.
macro_rules! group_or_default {
    ($group:expr, $ty:ident) => {
        match $group.as_deref() {
            Some(group) => std::borrow::Cow::Borrowed(group),
            None => std::borrow::Cow::Owned($ty::default()),
        }
    };
}

pub(crate) use group_or_default;

impl Style {
    pub(crate) fn size_mut(&mut self) -> &mut SizeFields {
        self.size.get_or_insert_with(Default::default)
    }
    pub(crate) fn spacing_mut(&mut self) -> &mut SpacingFields {
        self.spacing.get_or_insert_with(Default::default)
    }
    pub(crate) fn flow_mut(&mut self) -> &mut FlowFields {
        self.flow.get_or_insert_with(Default::default)
    }
    pub(crate) fn paint_mut(&mut self) -> &mut PaintFields {
        self.paint.get_or_insert_with(Default::default)
    }
    pub(crate) fn transform_mut(&mut self) -> &mut TransformFields {
        self.transform.get_or_insert_with(Default::default)
    }
    pub(crate) fn text_mut(&mut self) -> &mut TextFields {
        self.text.get_or_insert_with(Default::default)
    }
    pub(crate) fn states_mut(&mut self) -> &mut StateFields {
        self.states.get_or_insert_with(Default::default)
    }
    pub(crate) fn motion_mut(&mut self) -> &mut MotionFields {
        self.motion.get_or_insert_with(Default::default)
    }
}

/// Start a style chain.
pub fn style() -> Style {
    Style::default()
}

macro_rules! setter {
    ($(#[$m:meta])* $name:ident, $group:ident, $field:ident, $t:ty, $conv:expr) => {
        $(#[$m])*
        pub fn $name(mut self, v: $t) -> Self {
            if self.$group.as_deref().is_some_and(|g| g.$field.is_some()) {
                self.duplicates += 1;
            }
            #[allow(clippy::redundant_closure_call, reason = "`$conv` may be a closure literal called in place")]
            {
                self.$group.get_or_insert_with(Default::default).$field = Some(($conv)(v.into()));
            }
            self
        }
    };
}

/// A setter of a color input, which takes any [`ThemeColor`].
macro_rules! color_setter {
    ($(#[$m:meta])* $name:ident, $group:ident, $field:ident) => {
        $(#[$m])*
        pub fn $name(mut self, v: impl ThemeColor) -> Self {
            if self.$group.as_deref().is_some_and(|g| g.$field.is_some()) {
                self.duplicates += 1;
            }
            self.$group.get_or_insert_with(Default::default).$field = Some(Color::of(v));
            self
        }
    };
}

macro_rules! sides {
    ($(#[$m:meta])* $name:ident, $group:ident, $field:ident, [$($side:ident),+]) => {
        $(#[$m])*
        pub fn $name(mut self, v: impl Into<Length>) -> Self {
            let v = v.into();
            $(
                if self.$group.as_deref().is_some_and(|g| g.$field.$side.is_some()) {
                    self.duplicates += 1;
                }
                self.$group.get_or_insert_with(Default::default).$field.$side = Some(v);
            )+
            self
        }
    };
}

impl Style {
    /// Align direct in-flow children to scroll stops after touch scrolling.
    pub fn scroll_snap(mut self, snap: kivact_types::ScrollSnap) -> Self {
        self.flow_mut().scroll_snap = Some(snap);
        self
    }
    sides!(padding, spacing, padding, [top, right, bottom, left]);
    sides!(padding_x, spacing, padding, [left, right]);
    sides!(padding_y, spacing, padding, [top, bottom]);
    sides!(/// CSS `padding-top`; other edges remain unset.
        padding_top, spacing, padding, [top]);
    sides!(/// CSS `padding-right`; other edges remain unset.
        padding_right, spacing, padding, [right]);
    sides!(/// CSS `padding-bottom`; other edges remain unset.
        padding_bottom, spacing, padding, [bottom]);
    sides!(/// CSS `padding-left`; other edges remain unset.
        padding_left, spacing, padding, [left]);
    sides!(margin, spacing, margin, [top, right, bottom, left]);
    sides!(margin_x, spacing, margin, [left, right]);
    sides!(margin_y, spacing, margin, [top, bottom]);
    sides!(/// CSS `margin-top`; other edges remain unset.
        margin_top, spacing, margin, [top]);
    sides!(/// CSS `margin-right`; other edges remain unset.
        margin_right, spacing, margin, [right]);
    sides!(/// CSS `margin-bottom`; other edges remain unset.
        margin_bottom, spacing, margin, [bottom]);
    sides!(/// CSS `margin-left`; other edges remain unset.
        margin_left, spacing, margin, [left]);
    sides!(inset, spacing, inset, [top, right, bottom, left]);
    sides!(top, spacing, inset, [top]);
    sides!(right, spacing, inset, [right]);
    sides!(bottom, spacing, inset, [bottom]);
    sides!(left, spacing, inset, [left]);

    setter!(
        /// CSS `row-gap`: between the rows of a grid or wrapped flex lines (a
        /// column's children).
        row_gap, spacing, row_gap, impl Into<Length>, |v: Length| v);
    setter!(
        /// CSS `column-gap`: between the columns of a grid (a row's
        /// children).
        column_gap, spacing, column_gap, impl Into<Length>, |v: Length| v);
    setter!(
        /// CSS `align-content`: how a grid's rows (or a wrapping flex box's
        /// lines) share the free space along the cross axis.
        align_content, flow, align_content, Justify, |v| v);
    setter!(width, size, width, impl Into<Length>, |v: Length| v);
    setter!(height, size, height, impl Into<Length>, |v: Length| v);
    setter!(min_width, size, min_width, impl Into<Length>, |v: Length| v);
    setter!(
        min_height,
        size,
        min_height,
        impl Into<Length>,
        |v: Length| v
    );
    setter!(max_width, size, max_width, impl Into<Length>, |v: Length| v);
    setter!(
        max_height,
        size,
        max_height,
        impl Into<Length>,
        |v: Length| v
    );
    setter!(basis, size, basis, impl Into<Length>, |v: Length| v);
    setter!(
        /// CSS `aspect-ratio`: width / height (`aspect_ratio(16.0 / 9.0)`). With
        /// one of width and height set (or stretched), the other follows.
        aspect_ratio, size, aspect_ratio, impl Into<f64>, |v: f64| v);
    setter!(grow, size, grow, impl Into<f64>, |v: f64| v);
    setter!(shrink, size, shrink, impl Into<f64>, |v: f64| v);
    setter!(direction, flow, direction, Direction, |v| v);
    setter!(
        /// Cross-axis alignment of children.
        align_items, flow, align, Align, |v| v);
    setter!(align_self, flow, align_self, Align, |v| v);
    setter!(
        /// Main-axis distribution of children.
        justify_content, flow, justify, Justify, |v| v);
    setter!(wrap, flow, wrap, bool, |v| v);
    setter!(position, flow, position, Position, |v| v);
    setter!(
        /// Layout overflow on both axes; Hidden also clips layout overflow.
        /// For an axis-specific accessible scroll container, use `scroll`.
        overflow, flow, overflow, Overflow, |v| v);
    setter!(
        /// The background (CSS `background`): a color, a gradient, a
        /// [`Paint`] or a tuple of them stacked like CSS layers, the first
        /// on top (see [`Background`]). Layers are placed in the border box
        /// and clipped to its rounded shape.
        background,
        paint,
        background,
        impl Into<Background>,
        |v: Background| v
    );
    setter!(
        /// The opacity of the node and its subtree, as one group (CSS
        /// `opacity`: overlapping children do not show through each
        /// other).
        opacity, paint, opacity, impl Into<f64>, |v: f64| v.clamp(0.0, 1.0));
    setter!(
        /// CSS `filter`: color functions recolor everything the subtree
        /// draws; `blur` and `drop_shadow` filter the subtree's pixels.
        filter, paint, filter, Filter, |v| v);
    setter!(
        /// CSS `backdrop-filter`: blur and recolor what lies behind the
        /// border box (glass), beneath the node's own background, which is
        /// then usually translucent.
        backdrop_filter, paint, backdrop_filter, Filter, |v| v);
    setter!(
        /// CSS `mix-blend-mode`.
        mix_blend_mode, paint, blend, BlendMode, |v| v);
    setter!(
        /// CSS `mask`: the node and its subtree keep the alpha of these
        /// layers (`mask(linear_gradient(180).colors([Color::BLACK,
        /// Color::TRANSPARENT]))` fades them out downwards).
        mask, paint, mask, impl Into<Mask>, |v: Mask| Box::new(v));
    setter!(
        /// CSS `clip-path`: the node's own paint (background, border,
        /// shadows) and its subtree are clipped to `shape`, and so are
        /// hit testing and pointer input: a press outside the shape does
        /// not reach the node.
        clip_path, paint, clip_path, Shape, Box::new);
    setter!(
        /// CSS `transform`: the functions of `t` in their order
        /// (`Transform::new().skew_x(-12).translate_y(2)`), after the
        /// `translate`, `rotate` and `scale` properties, about the
        /// `transform_origin` (the box's centre by default). It maps the
        /// box and its whole subtree, which is hit, pointed at
        /// (`PointerEvent::pos` stays in the untransformed local space),
        /// reported (`NodeRef::rect`: the bounds as drawn) and exposed
        /// where it is drawn; layout ignores it (siblings do not move).
        transform, transform, transform, Transform, |v| v);
    setter!(
        /// CSS `rotate`: clockwise, in degrees, after `translate` and
        /// before `scale` and `transform` (CSS's order for the individual
        /// transform properties, whatever the call order).
        rotate, transform, rotate, impl Into<f64>, |v: f64| v);
    setter!(
        /// CSS `scale` (uniform), applied after `translate` and `rotate`
        /// and before `transform`. A state style can set it alone:
        /// `.hover(style().scale(1.05))` keeps the node's rotation.
        scale, transform, scale, impl Into<f64>, |v: f64| (v, v));
    setter!(
        /// CSS `z-index`: the order the node is painted and hit in among
        /// its siblings (higher on top; equal values keep the tree order).
        /// Every container isolates its children's z-indices (as CSS
        /// `isolation: isolate`): a negative one paints after the parent's
        /// own background, below its other children. Tab order stays the
        /// tree order.
        z_index, flow, z_index, i32, |v| v);
    /// Set overlay scrollbar width; no layout space is reserved.
    pub fn scrollbar_width(mut self, width: impl Into<ScrollbarWidth>) -> Self {
        self.paint_mut()
            .scrollbar
            .get_or_insert_with(Default::default)
            .width = Some(width.into());
        self
    }
    /// Set thumb and track colors (CSS scrollbar-color).
    pub fn scrollbar_color(mut self, thumb: impl ThemeColor, track: impl ThemeColor) -> Self {
        self.paint_mut()
            .scrollbar
            .get_or_insert_with(Default::default)
            .colors = Some((Color::of(thumb), Color::of(track)));
        self
    }
    setter!(cursor, paint, cursor, Cursor, |v| v);
    setter!(
        /// Clip drawing to the box without changing layout overflow.
        /// Use `overflow(Overflow::Hidden)` to affect layout overflow too.
        clip, paint, clip, bool, |v| v);
    setter!(
        /// CSS `font-size` of the text in this node and below.
        font_size, text, font_size, impl Into<f64>, |v: f64| v);
    setter!(
        /// CSS `font-weight` (1 to 1000) of the text in this node and below.
        weight, text, weight, u16, |v| v);
    color_setter!(
        /// CSS `color` of the text in this node and below. As in CSS, each
        /// text property is inherited on its own: a descendant that sets
        /// only a `weight` or a `font_size` keeps this color. Without one
        /// on any ancestor, text takes the App theme's
        /// [`ColorToken::Text`](crate::ColorToken::Text); a
        /// `provide_theme` subtree gets its theme's by a
        /// `color(ColorToken::Text)` on its top node.
        color, text, color);
    setter!(
        /// CSS `line-height` of the text in this node and below. A number is
        /// a multiple of the font size, as CSS's unitless value:
        /// `line_height(1.5)` is `line-height: 1.5`. A length in px is
        /// `line_height(LineHeight::px(20))` (`line-height: 20px`), and
        /// `LineHeight::Normal` is `normal`: the font size times 1.3,
        /// rounded (a value like the others: it does not inherit the
        /// theme's). Text that sets none inherits the nearest ancestor's,
        /// else `Theme.text.line_height`. Not finite or not above zero is
        /// `normal`.
        line_height, text, line_height, impl Into<LineHeight>, |v: LineHeight| v);
    setter!(text_wrap, text, text_wrap, bool, |v| v);
    setter!(text_align, text, text_align, TextAlign, |v| v);
    setter!(
        /// Font family of the text in this node and below (CSS
        /// `font-family`): a family name (`"Inter"`, a font
        /// registered with `App::font` or installed on the system) or a
        /// generic family (`"system-ui"`, `"sans-serif"`, `"serif"`,
        /// `"monospace"`). Descendants inherit it, also those that set
        /// other text properties. A family that is not installed falls back
        /// to the system UI font and is reported once as `ErrorKind::Asset`.
        /// An ordered comma-separated list (such as
        /// `"Inter, 'Noto Sans JP', sans-serif"`) selects a font per glyph
        /// before system fallback; unavailable entries are skipped without
        /// diagnostics. Quote names containing commas. Malformed lists use
        /// system fallback. Without one, text uses `Theme.text.family`.
        font_family, text, font_family, impl Into<String>, |v: String| v);
    setter!(
        /// Italic text (the family's italic face when it has one).
        italic, text, italic, bool, |v| v);

    /// CSS `gap`: between rows and between columns (`row_gap` and
    /// `column_gap` at once; CSS `gap: 6px 16px` is
    /// `.row_gap(6).column_gap(16)`).
    pub fn gap(self, v: impl Into<Length>) -> Self {
        let v = v.into();
        self.row_gap(v).column_gap(v)
    }

    /// A label that stays on one line and ends in "…" when its box is too
    /// narrow. In CSS: `white-space: nowrap; overflow: hidden;
    /// text-overflow: ellipsis; min-width: 0` (so that a text in a flex row
    /// can shrink below its content width). It is
    /// `text_wrap(false)`, `text_overflow(TextOverflow::Ellipsis)`,
    /// `min_width(0)` and `overflow(Overflow::Hidden)`, which clips both
    /// axes of the text's own box. The accessible name stays the whole text.
    ///
    /// A parent that is a flex row also needs `min_width(0)` for the text to
    /// shrink.
    pub fn single_line(self) -> Self {
        self.text_wrap(false)
            .text_overflow(TextOverflow::Ellipsis)
            .min_width(0)
            .overflow(Overflow::Hidden)
    }

    /// Both width and height.
    pub fn size(self, w: impl Into<Length>, h: impl Into<Length>) -> Self {
        self.width(w).height(h)
    }

    /// CSS `translate: x y`: a move in px or a percentage of the node's
    /// own box (`translate(pct(-50), pct(-50))` centres it on its
    /// position), applied first of the transform properties. It sets both
    /// axes, like [`translate_x`](Self::translate_x) and
    /// [`translate_y`](Self::translate_y) together.
    pub fn translate(self, x: impl Into<Length>, y: impl Into<Length>) -> Self {
        self.translate_x(x).translate_y(y)
    }

    /// The horizontal part of [`translate`](Self::translate) alone
    /// (Tailwind's `translate-x-*`). Each axis is a property of its own: a
    /// style merged over this one (a later layer, or a `hover`, `pressed`
    /// or `focus` state) that sets only the other axis keeps this one, so a
    /// press that nudges a node down keeps its horizontal centring by
    /// translate. A layer or state that sets this axis replaces it, as in
    /// CSS.
    pub fn translate_x(mut self, x: impl Into<Length>) -> Self {
        if self
            .transform
            .as_deref()
            .is_some_and(|g| g.translate[0].is_some())
        {
            self.duplicates += 1;
        }
        self.transform_mut().translate[0] = Some(x.into());
        self
    }

    /// The vertical part of [`translate`](Self::translate) alone
    /// (Tailwind's `translate-y-*`), merged on its own like
    /// [`translate_x`](Self::translate_x).
    pub fn translate_y(mut self, y: impl Into<Length>) -> Self {
        if self
            .transform
            .as_deref()
            .is_some_and(|g| g.translate[1].is_some())
        {
            self.duplicates += 1;
        }
        self.transform_mut().translate[1] = Some(y.into());
        self
    }

    /// CSS `scale: x y` (non-uniform).
    pub fn scale_xy(mut self, x: impl Into<f64>, y: impl Into<f64>) -> Self {
        if self.transform.as_deref().is_some_and(|g| g.scale.is_some()) {
            self.duplicates += 1;
        }
        self.transform_mut().scale = Some((x.into(), y.into()));
        self
    }

    /// CSS `transform-origin: x y`, the point the transform properties
    /// apply about: px or a percentage of the box (`left top` is
    /// `(0, 0)`, `right bottom` is `(pct(100), pct(100))`; the default is
    /// the centre, `(pct(50), pct(50))`).
    pub fn transform_origin(mut self, x: impl Into<Length>, y: impl Into<Length>) -> Self {
        if self
            .transform
            .as_deref()
            .is_some_and(|g| g.transform_origin.is_some())
        {
            self.duplicates += 1;
        }
        self.transform_mut().transform_origin = Some((x.into(), y.into()));
        self
    }

    setter!(
        /// CSS `translate` on the z axis, in px (`translateZ`).
        translate_z, transform, translate_z, impl Into<f64>, |v: f64| v);

    setter!(
        /// CSS `rotate` about the x axis, in degrees (`rotateX`).
        rotate_x, transform, rotate_x, impl Into<f64>, |v: f64| v);

    setter!(
        /// CSS `rotate` about the y axis, in degrees (`rotateY`).
        rotate_y, transform, rotate_y, impl Into<f64>, |v: f64| v);

    setter!(
        /// The z of the transform origin, in px (the third value of CSS `transform-origin`).
        transform_origin_z, transform, transform_origin_z, impl Into<f64>, |v: f64| v);

    setter!(
        /// CSS `perspective` of this node's children, in px: the distance of
        /// the viewer. The origin is the center of the box unless
        /// [`perspective_origin`](Self::perspective_origin) says otherwise.
        perspective, transform, perspective, impl Into<f64>, |v: f64| v);

    /// CSS `perspective-origin`: where the viewer looks at, against the box.
    pub fn perspective_origin(mut self, x: impl Into<Length>, y: impl Into<Length>) -> Self {
        if self
            .transform
            .as_deref()
            .is_some_and(|g| g.perspective_origin.is_some())
        {
            self.duplicates += 1;
        }
        self.transform_mut().perspective_origin = Some((x.into(), y.into()));
        self
    }

    setter!(
        /// CSS `transform-style`: `Preserve3d` keeps the children in one 3D space.
        transform_style, transform, transform_style, TransformStyle, |v| v);

    setter!(
        /// CSS `backface-visibility`.
        backface_visibility, transform, backface, BackfaceVisibility, |v| v);

    setter!(
        /// CSS `will-change`.
        will_change, transform, will_change, WillChange, |v| v);

    setter!(
        /// CSS `isolation`.
        isolation, paint, isolation, Isolation, |v| v);

    setter!(
        /// CSS `border-image`: a picture or a gradient cut into nine pieces
        /// that fill the border.
        border_image, paint, border_image, BorderImage, Box::new);

    /// A border on every side (CSS `border: w solid c`). Like padding, it
    /// takes room inside the box (CSS `box-sizing: border-box`): the
    /// children and the text sit inside the border and the padding, and a
    /// width or height is the border box's. A change of its color or style
    /// alone does not lay out again. In a state style (`hover`, `focus`,
    /// ...) a width is drawn but does not move the content: states change
    /// the paint only.
    pub fn border(self, width: impl Into<f64>, color: impl ThemeColor) -> Self {
        self.border_sides([0, 1, 2, 3], width.into(), Some(Color::of(color)))
    }

    /// The top border (CSS `border-top: w solid c`).
    pub fn border_top(self, width: impl Into<f64>, color: impl ThemeColor) -> Self {
        self.border_sides([0], width.into(), Some(Color::of(color)))
    }

    /// The right border (CSS `border-right`).
    pub fn border_right(self, width: impl Into<f64>, color: impl ThemeColor) -> Self {
        self.border_sides([1], width.into(), Some(Color::of(color)))
    }

    /// The bottom border (CSS `border-bottom`).
    pub fn border_bottom(self, width: impl Into<f64>, color: impl ThemeColor) -> Self {
        self.border_sides([2], width.into(), Some(Color::of(color)))
    }

    /// The left border (CSS `border-left`).
    pub fn border_left(self, width: impl Into<f64>, color: impl ThemeColor) -> Self {
        self.border_sides([3], width.into(), Some(Color::of(color)))
    }

    /// Every side's border width (CSS `border-width`); a side without a
    /// color takes `ColorToken::Border`.
    pub fn border_width(self, width: impl Into<f64>) -> Self {
        self.border_sides([0, 1, 2, 3], width.into(), None)
    }

    /// Every side's border color (CSS `border-color`), keeping the widths
    /// (handy in state styles: `.hover(style().border_color(..))`).
    pub fn border_color(mut self, color: impl ThemeColor) -> Self {
        let color = Color::of(color);
        if self
            .paint
            .as_deref()
            .is_some_and(|g| g.border_color.iter().any(Option::is_some))
        {
            self.duplicates += 1;
        }
        self.paint_mut().border_color = [Some(color); 4];
        self
    }

    /// How every side's border is drawn (CSS `border-style`).
    pub fn border_style(mut self, s: BorderStyle) -> Self {
        if self
            .paint
            .as_deref()
            .is_some_and(|g| g.border_style.iter().any(Option::is_some))
        {
            self.duplicates += 1;
        }
        self.paint_mut().border_style = [Some(s); 4];
        self
    }

    /// The top border's line style alone (CSS `border-top-style`).
    pub fn border_top_style(self, s: BorderStyle) -> Self {
        self.border_side_style(0, s)
    }

    /// The right border's line style alone (CSS `border-right-style`).
    pub fn border_right_style(self, s: BorderStyle) -> Self {
        self.border_side_style(1, s)
    }

    /// The bottom border's line style alone (CSS `border-bottom-style`).
    pub fn border_bottom_style(self, s: BorderStyle) -> Self {
        self.border_side_style(2, s)
    }

    /// The left border's line style alone (CSS `border-left-style`).
    pub fn border_left_style(self, s: BorderStyle) -> Self {
        self.border_side_style(3, s)
    }

    fn border_side_style(mut self, side: usize, s: BorderStyle) -> Self {
        if self
            .paint
            .as_deref()
            .is_some_and(|g| g.border_style[side].is_some())
        {
            self.duplicates += 1;
        }
        self.paint_mut().border_style[side] = Some(s);
        self
    }

    fn border_sides<const N: usize>(
        mut self,
        sides: [usize; N],
        width: f64,
        color: Option<Color>,
    ) -> Self {
        if sides.iter().any(|i| {
            self.paint
                .as_deref()
                .is_some_and(|g| g.border_width[*i].is_some())
        }) {
            self.duplicates += 1;
        }
        let paint = self.paint_mut();
        for i in sides {
            paint.border_width[i] = Some(width);
            if let Some(c) = color {
                paint.border_color[i] = Some(c);
            }
        }
        self
    }

    /// Round every corner (CSS `border-radius`): `8`, `pct(50)` (an
    /// ellipse on a box that is not square) or `(20, 10)` (elliptical).
    pub fn radius(self, r: impl Into<CornerRadius>) -> Self {
        let r = r.into();
        self.corners([0, 1, 2, 3], [r; 4])
    }

    /// Round each corner: top left, top right, bottom right, bottom left
    /// (CSS `border-radius: tl tr br bl`).
    pub fn radius_corners(
        self,
        top_left: impl Into<CornerRadius>,
        top_right: impl Into<CornerRadius>,
        bottom_right: impl Into<CornerRadius>,
        bottom_left: impl Into<CornerRadius>,
    ) -> Self {
        self.corners(
            [0, 1, 2, 3],
            [
                top_left.into(),
                top_right.into(),
                bottom_right.into(),
                bottom_left.into(),
            ],
        )
    }

    /// CSS `border-top-left-radius`.
    pub fn radius_top_left(self, r: impl Into<CornerRadius>) -> Self {
        self.corners([0], [r.into()])
    }

    /// CSS `border-top-right-radius`.
    pub fn radius_top_right(self, r: impl Into<CornerRadius>) -> Self {
        self.corners([1], [r.into()])
    }

    /// CSS `border-bottom-right-radius`.
    pub fn radius_bottom_right(self, r: impl Into<CornerRadius>) -> Self {
        self.corners([2], [r.into()])
    }

    /// CSS `border-bottom-left-radius`.
    pub fn radius_bottom_left(self, r: impl Into<CornerRadius>) -> Self {
        self.corners([3], [r.into()])
    }

    fn corners<const N: usize>(mut self, which: [usize; N], radii: [CornerRadius; N]) -> Self {
        if which.iter().any(|i| {
            self.paint
                .as_deref()
                .is_some_and(|g| g.radius[*i].is_some())
        }) {
            self.duplicates += 1;
        }
        let paint = self.paint_mut();
        for (i, r) in which.into_iter().zip(radii) {
            paint.radius[i] = Some(r);
        }
        self
    }

    /// CSS `backdrop-filter: blur(px)`.
    pub fn backdrop_blur(self, px: impl Into<f64>) -> Self {
        self.backdrop_filter(Filter::new().blur(px))
    }

    /// Focus ring (drawn outside the border box); usually inside
    /// `focus_visible(..)`. A focusable element without a focus look of its
    /// own draws the theme's ([`Theme::focus_ring`](crate::Theme::focus_ring)).
    pub fn ring(self, width: impl Into<f64>, color: impl ThemeColor) -> Self {
        self.ring_with_offset(width, 0, color)
    }

    /// Draw a focus ring with a gap in logical pixels from the border box.
    /// A positive offset follows the border box even if the node has a clip path.
    pub fn ring_with_offset(
        mut self,
        width: impl Into<f64>,
        offset: impl Into<f64>,
        color: impl ThemeColor,
    ) -> Self {
        if self.paint.as_deref().is_some_and(|g| g.ring.is_some()) {
            self.duplicates += 1;
        }
        self.paint_mut().ring = Some((width.into(), offset.into(), Color::of(color)));
        self
    }

    /// Add a box shadow (CSS `box-shadow`); several stack, the first on
    /// top. It follows the corner radii, elliptical ones too.
    pub fn shadow(mut self, s: Shadow) -> Self {
        self.paint_mut()
            .shadows
            .get_or_insert_with(Vec::new)
            .push(s);
        self
    }

    /// The style while the pointer is over the node or a node below it
    /// (CSS `:hover`, which a hovered child keeps on its parents).
    ///
    /// A state style changes paint only (see [`TransitionProp::All`]); a
    /// layout or text property other than `color` set in it is reported as
    /// `Unsupported` and ignored. `selected` and `disabled` apply any
    /// property.
    pub fn hover(mut self, s: Style) -> Self {
        self.states_mut().hover = Some(Box::new(s));
        self
    }

    /// Make the node a group for the [`group_hover`](Self::group_hover)
    /// and [`group_focus`](Self::group_focus) styles below it (Tailwind's
    /// `group`).
    pub fn group(mut self) -> Self {
        self.states_mut().group = true;
        self
    }

    /// The style while the nearest ancestor made a [`group`](Self::group)
    /// is hovered (Tailwind's `group-hover:`), under the node's own
    /// [`hover`](Self::hover): a table row's actions that show while the
    /// pointer is anywhere over the row.
    ///
    /// ```
    /// use kivact::style::style;
    /// let row = style().group();
    /// let actions = style().opacity(0.0).group_hover(style().opacity(1.0));
    /// # let _ = (row, actions);
    /// ```
    pub fn group_hover(mut self, s: Style) -> Self {
        self.states_mut().group_hover = Some(Box::new(s));
        self
    }

    /// The style while the keyboard focus is on the nearest ancestor made a
    /// [`group`](Self::group) or on a node below it (Tailwind's
    /// `group-focus-within:`, CSS `:focus-within` read from the group),
    /// under the node's own [`focus`](Self::focus): a table row's actions
    /// that show while a control in the row has the focus.
    ///
    /// ```
    /// use kivact::style::style;
    /// let row = style().group();
    /// let actions = style().opacity(0.0).group_focus(style().opacity(1.0));
    /// # let _ = (row, actions);
    /// ```
    pub fn group_focus(mut self, s: Style) -> Self {
        self.states_mut().group_focus = Some(Box::new(s));
        self
    }

    pub fn pressed(mut self, s: Style) -> Self {
        self.states_mut().pressed = Some(Box::new(s));
        self
    }

    /// The style while the element has the focus, however it got it.
    pub fn focus(mut self, s: Style) -> Self {
        self.states_mut().focus = Some(Box::new(s));
        self
    }

    /// The style while the element has the focus and the host asks for a
    /// focus indication (CSS `:focus-visible`): focus from the keyboard or
    /// from a program, not from a pointer press. It applies over
    /// [`focus`](Self::focus). A focusable element whose style has neither
    /// state draws the theme's focus ring here.
    pub fn focus_visible(mut self, s: Style) -> Self {
        self.states_mut().focus_visible = Some(Box::new(s));
        self
    }

    /// No ring (CSS `ring-0`): in a [`focus_visible`](Self::focus_visible)
    /// style it turns the theme's focus ring off.
    pub fn ring_none(self) -> Self {
        self.ring(0, Color::TRANSPARENT)
    }

    pub fn disabled(mut self, s: Style) -> Self {
        self.states_mut().disabled = Some(Box::new(s));
        self
    }

    pub fn selected(mut self, s: Style) -> Self {
        self.states_mut().selected = Some(Box::new(s));
        self
    }

    /// The paint the node starts from when it is mounted (CSS
    /// `@starting-style`, Framer Motion's `initial`): its
    /// [`TransitionProp::All`] transition moves it from there to its
    /// paint. Only the paint properties the transition animates move; the
    /// others take their value at once.
    ///
    /// ```
    /// use kivact::motion::{Easing, Transition};
    /// use kivact::style::{TransitionProp, style};
    /// // A toast slides in from the right and fades in.
    /// let toast = style()
    ///     .enter(style().translate_x(40).opacity(0.0))
    ///     .exit(style().translate_x(40).opacity(0.0))
    ///     .transition(TransitionProp::All, Transition::tween(260, Easing::EaseOut));
    /// # let _ = toast;
    /// ```
    ///
    /// Without a paint transition, and under reduced motion with a
    /// decorative one, the node shows its paint at once.
    pub fn enter(mut self, s: Style) -> Self {
        self.states_mut().enter = Some(Box::new(s));
        self
    }

    /// The paint a removed node moves to before it goes (CSS exit
    /// transitions, Framer Motion's `exit`): when the node or an ancestor
    /// is removed, the removed subtree is drawn where it was until every
    /// such transition in it ends, then it goes. From the removal on, it
    /// takes no pointer, focus or accessibility node: it is gone for
    /// everything but the eye.
    ///
    /// A leaving node keeps its place in the layout until it goes, as with
    /// a CSS exit transition or Framer Motion's `AnimatePresence`: the
    /// nodes after it move into its place when it goes, not when it is
    /// removed. The node builder's `exit_layout(ExitLayout::Pop)` modifier
    /// releases that space at removal while retaining its last local box.
    ///
    /// Without a paint transition, and under reduced motion with a
    /// decorative one, the node goes at once.
    pub fn exit(mut self, s: Style) -> Self {
        self.states_mut().exit = Some(Box::new(s));
        self
    }

    /// Reveal the selected axis at entrance when Size has a transition.
    pub fn enter_size(mut self, axis: kivact_types::Orientation) -> Self {
        self.motion_mut().enter_size = Some(axis);
        self
    }

    /// Collapse the selected axis before removal when Size has a transition.
    pub fn exit_size(mut self, axis: kivact_types::Orientation) -> Self {
        self.motion_mut().exit_size = Some(axis);
        self
    }

    /// Animate position and size changes after layout, using a visual transform.
    /// Layout is computed once; painting interpolates from the previous rectangle.
    pub fn layout_transition(mut self, transition: Transition) -> Self {
        self.motion_mut().layout_transition = Some(transition);
        self
    }

    /// Animate changes of a property group with `t`.
    pub fn transition(mut self, prop: TransitionProp, t: Transition) -> Self {
        let motion = self.motion_mut();
        motion.transitions.retain(|(p, _)| *p != prop);
        motion.transitions.push((prop, t));
        self
    }

    /// Keep layout space while hiding this subtree from painting and interaction.
    pub fn visibility(mut self, visibility: kivact_types::Visibility) -> Self {
        self.flow_mut().visibility = Some(visibility);
        self
    }

    /// Set the text decoration pattern, tracking reactive inputs.
    pub fn text_decoration_style(
        mut self,
        value: impl kivact_runtime::IntoValue<crate::text::TextDecorationStyle>,
    ) -> Self {
        let set = self
            .text
            .as_deref()
            .and_then(|text| text.text.as_deref())
            .is_some_and(|x| x.decoration_style.is_some());
        if set {
            self.duplicates += 1;
        }
        self.text_mut()
            .text
            .get_or_insert_with(Default::default)
            .decoration_style = Some(crate::text::DecorationStyleValue::new(value));
        self
    }

    /// Select flex, grid, or a transparent container that contributes no box.
    pub fn display(mut self, display: kivact_types::Display) -> Style {
        self.flow_mut().display = Some(display);
        self
    }

    /// Paint locally after the node's drawing and before its children.
    /// Recipes and themes use this layer; callers use the node's `draw` layer
    /// so style merging retains the recipe's paint.
    ///
    /// The callback follows size, theme and signals it reads. A later paint
    /// replaces this callback when styles merge; unspecified paint retains it.
    /// This does not replace the node's `draw` or `draw_over` callbacks.
    /// Painting requires a box, such as `node` or `canvas`. On text, image or
    /// text-input leaves it is ignored and reported as unsupported; paint
    /// their containing box instead.
    /// Paint belongs on the base style. Paint inside hover, group hover, group focus,
    /// pressed, focus, disabled, selected, enter or exit styles is ignored
    /// and reported as unsupported;
    /// return a base paint from a reactive style to follow interaction state.
    pub fn paint(mut self, f: impl Fn(&mut Draw, Size) + 'static) -> Self {
        self.paint_mut().local_paint = Some(LocalPaint::new(f));
        self
    }

    /// Overlay `o` on `self` (fields set in `o` win).
    pub fn merge(mut self, o: &Style) -> Style {
        if let Some(group) = o.size.as_deref() {
            self.size_mut().merge(group);
        }
        if let Some(group) = o.spacing.as_deref() {
            self.spacing_mut().merge(group);
        }
        if let Some(group) = o.flow.as_deref() {
            self.flow_mut().merge(group);
        }
        if let Some(group) = o.paint.as_deref() {
            self.paint_mut().merge(group);
        }
        if let Some(group) = o.transform.as_deref() {
            self.transform_mut().merge(group);
        }
        if let Some(group) = o.text.as_deref() {
            self.text_mut().merge(group);
        }
        if let Some(group) = o.states.as_deref() {
            self.states_mut().merge(group);
        }
        if let Some(group) = o.motion.as_deref() {
            self.motion_mut().merge(group);
        }
        self.duplicates += o.duplicates;
        self
    }

    pub(super) fn has_text(&self) -> bool {
        self.text.as_deref().is_some_and(|text| {
            text.font_size.is_some()
                || text.weight.is_some()
                || text.color.is_some()
                || text.line_height.is_some()
                || text.text_wrap.is_some()
                || text.text_align.is_some()
                || text.font_family.is_some()
                || text.italic.is_some()
                || text.text.is_some()
        })
    }
}

kivact_runtime::constant_value!(Style);

/// Overlay scrollbar width.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ScrollbarWidth {
    /// Thin default (6 logical pixels).
    #[default]
    Auto,
    /// Thin (6 logical pixels).
    Thin,
    /// Hidden; scrolling still works.
    None,
    /// Explicit logical pixel width.
    Px(f64),
}
impl From<f64> for ScrollbarWidth {
    fn from(px: f64) -> Self {
        Self::Px(px)
    }
}
impl From<i32> for ScrollbarWidth {
    fn from(px: i32) -> Self {
        Self::Px(px as f64)
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct ScrollbarStyle {
    // xtask-allow(state-fields): passive record - width
    pub(super) width: Option<ScrollbarWidth>,
    // xtask-allow(state-fields): passive record - colors
    pub(super) colors: Option<(Color, Color)>,
}

macro_rules! text_color_setter {
    ($(#[$m:meta])* $name:ident, $field:ident) => {
        $(#[$m])*
        pub fn $name(mut self, v: impl ThemeColor) -> Self {
            let set = self
                .text
                .as_deref()
                .and_then(|text| text.text.as_deref())
                .is_some_and(|x| x.$field.is_some());
            if set {
                self.duplicates += 1;
            }
            self.text_mut().text.get_or_insert_with(Default::default).$field = Some(Color::of(v));
            self
        }
    };
}

macro_rules! text_setter {
    ($(#[$m:meta])* $name:ident, $field:ident, $t:ty, $conv:expr) => {
        $(#[$m])*
        pub fn $name(mut self, v: $t) -> Self {
            let set = self
                .text
                .as_deref()
                .and_then(|text| text.text.as_deref())
                .is_some_and(|x| x.$field.is_some());
            if set {
                self.duplicates += 1;
            }
            #[allow(clippy::redundant_closure_call, reason = "`$conv` may be a closure literal called in place")]
            {
                self.text_mut().text.get_or_insert_with(Default::default).$field = Some(($conv)(v.into()));
            }
            self
        }
    };
}

impl Style {
    text_setter!(
        /// CSS `letter-spacing`: extra space after every character, in px
        /// (`letter_spacing(1)`) or em (`letter_spacing(em(0.08))`, of the
        /// text's font size; negative tightens).
        letter_spacing, letter_spacing, impl Into<TextLength>, |v: TextLength| v);
    text_setter!(
        /// CSS `word-spacing`: extra space after every space, in px
        /// (`word_spacing(4)`) or em (`word_spacing(em(0.25))`, of the
        /// text's font size; negative tightens). It adds to
        /// `text_align(TextAlign::Justify)`'s widening.
        word_spacing, word_spacing, impl Into<TextLength>, |v: TextLength| v);
    text_setter!(
        /// CSS `font-kerning`: `FontKerning::None` turns the OpenType `kern`
        /// feature off, `Normal` on, `Auto` (the default) leaves it to the
        /// font. A `("kern", ..)` in `font_feature_settings` wins.
        font_kerning, kerning, FontKerning, |v| v);
    text_setter!(
        /// CSS `text-decoration-line` (and the `text-decoration: underline`
        /// shorthand): `text_decoration(TextDecorationLine::UNDERLINE)`.
        /// The lines are drawn in the text's color unless
        /// `text_decoration_color` says otherwise; a span of rich text can
        /// set its own.
        text_decoration, decoration_line, TextDecorationLine, |v| v);
    text_color_setter!(
        /// CSS `text-decoration-color` (default: the text's color).
        text_decoration_color, decoration_color);
    text_setter!(
        /// CSS `text-decoration-thickness` in px or em (default: the font's).
        text_decoration_thickness,
        decoration_thickness,
        impl Into<TextLength>,
        |v: TextLength| v
    );
    text_setter!(
        /// CSS `text-underline-offset`: the underline this much further
        /// from the text than the font puts it (`text_underline_offset(3)`).
        text_underline_offset, underline_offset, impl Into<TextLength>, |v: TextLength| v);
    text_setter!(
        /// CSS `text-overflow`: `TextOverflow::Ellipsis` cuts a line that
        /// does not fit its box and ends it in "…" (with
        /// `text_wrap(false)`, CSS's `white-space: nowrap`; such a text can
        /// shrink down to the ellipsis in a row). The accessible name is
        /// the whole text.
        text_overflow, overflow, TextOverflow, |v| v);
    text_setter!(
        /// CSS `line-clamp`: show at most `n` lines, the last ending in
        /// "…" when more follow (the box is as high as the lines shown).
        line_clamp, line_clamp, u32, |v| v);
    text_setter!(
        /// Trim the first line to the measured cap height and the last to its baseline.
        text_capsize, capsize, bool, |v| v);
    text_setter!(
        /// CSS `text-transform`: draw the text in upper case, lower case
        /// or capitalized; its accessible name keeps it as written.
        text_transform, transform, TextTransform, |v| v);
    /// CSS `-webkit-text-stroke`: the outline of the glyphs, in px or em, in
    /// `color`, drawn over or under the fill as `paint_order` says.
    pub fn text_stroke(mut self, width: impl Into<TextLength>, color: impl ThemeColor) -> Self {
        let set = self
            .text
            .as_deref()
            .and_then(|text| text.text.as_deref())
            .is_some_and(|x| x.stroke.is_some());
        if set {
            self.duplicates += 1;
        }
        self.text_mut()
            .text
            .get_or_insert_with(Default::default)
            .stroke = Some((width.into(), Color::of(color)));
        self
    }

    text_setter!(
        /// CSS `paint-order`: whether the fill or the stroke of the glyphs is drawn first.
        paint_order, paint_order, PaintOrder, |v| v);

    /// CSS `font-variation-settings`: axis settings such as `wght`.
    pub fn font_variation_settings(
        mut self,
        settings: impl IntoIterator<Item = impl Into<FontVariation>>,
    ) -> Self {
        let set = self
            .text
            .as_deref()
            .and_then(|text| text.text.as_deref())
            .is_some_and(|x| x.variations.is_some());
        if set {
            self.duplicates += 1;
        }
        self.text_mut()
            .text
            .get_or_insert_with(Default::default)
            .variations = Some(settings.into_iter().map(Into::into).collect());
        self
    }

    text_setter!(
        /// CSS `font-optical-sizing`: whether the optical size axis follows the font size.
        font_optical_sizing, optical_sizing, bool, |v| v);

    text_setter!(
        /// CSS `font-stretch` as a percentage of the normal width (50 to 200).
        font_width, font_width, impl Into<f64>, |v: f64| v);

    text_color_setter!(
        /// CSS `caret-color` of a text field.
        caret_color, caret_color);
    text_color_setter!(
        /// The band behind a text field's selected text (CSS
        /// `::selection { background }`).
        selection_color, selection_color);
    text_color_setter!(
        /// A text field's placeholder color (CSS `::placeholder { color }`;
        /// default: the text color at half its alpha).
        placeholder_color, placeholder_color);

    /// CSS `font-feature-settings`: OpenType features of the font,
    /// `font_feature_settings([("tnum", 1), ("ss01", 1)])` (a tag alone,
    /// `"cv11"`, is on). The font must have them; others do nothing.
    pub fn font_feature_settings(
        mut self,
        features: impl IntoIterator<Item = impl Into<FontFeature>>,
    ) -> Self {
        let set = self
            .text
            .as_deref()
            .and_then(|text| text.text.as_deref())
            .is_some_and(|x| x.features.is_some());
        if set {
            self.duplicates += 1;
        }
        self.text_mut()
            .text
            .get_or_insert_with(Default::default)
            .features = Some(features.into_iter().map(Into::into).collect());
        self
    }

    /// CSS `font-variant-numeric: tabular-nums`: digits of one width (the
    /// font's `tnum` feature), so numbers in a column or a timer do not
    /// jiggle.
    pub fn tabular_nums(mut self) -> Self {
        let set = self
            .text
            .as_deref()
            .and_then(|text| text.text.as_deref())
            .is_some_and(|x| x.tabular_nums.is_some());
        if set {
            self.duplicates += 1;
        }
        self.text_mut()
            .text
            .get_or_insert_with(Default::default)
            .tabular_nums = Some(true);
        self
    }

    /// Add a text shadow (CSS `text-shadow: x y blur color`); several
    /// stack, the first on top. They shadow the glyphs and decoration
    /// lines (a shadow's spread and inset do not apply to text and are
    /// reported as unsupported).
    pub fn text_shadow(mut self, s: Shadow) -> Self {
        let x = self.text_mut().text.get_or_insert_with(Default::default);
        x.shadows.get_or_insert_with(Vec::new).push(s);
        self
    }
}

macro_rules! grid_setter {
    ($(#[$m:meta])* $name:ident, $field:ident, $t:ty, $conv:expr) => {
        $(#[$m])*
        pub fn $name(mut self, v: $t) -> Self {
            let set = self
                .flow
                .as_deref()
                .and_then(|flow| flow.grid.as_deref())
                .is_some_and(|g| g.$field.is_some());
            if set {
                self.duplicates += 1;
            }
            #[allow(clippy::redundant_closure_call, reason = "`$conv` may be a closure literal called in place")]
            {
                self.flow_mut().grid.get_or_insert_with(Default::default).$field = Some(($conv)(v.into()));
            }
            self
        }
    };
}

impl Style {
    grid_setter!(
        /// CSS `grid-template-columns`: the grid's columns, a [`Track`] or a
        /// tuple of them: `(248, Track::fr(1))`,
        /// `Track::repeat_auto_fill(Track::minmax(330, Track::fr(1)))`,
        /// `(Track::minmax(70, pct(38)), Track::fr(1), 20)`. For a `grid`
        /// box.
        grid_template_columns, columns, impl Into<Tracks>, |v: Tracks| v);
    grid_setter!(
        /// CSS `grid-template-rows`.
        grid_template_rows, rows, impl Into<Tracks>, |v: Tracks| v);
    grid_setter!(
        /// CSS `grid-auto-columns`: the size of the columns auto-placed
        /// items make beyond the template (cycling through the list).
        grid_auto_columns, auto_columns, impl Into<Tracks>, |v: Tracks| v);
    grid_setter!(
        /// CSS `grid-auto-rows`: the size of the rows beyond the template
        /// (`grid_auto_rows(120)` makes every new row 120 px high).
        grid_auto_rows, auto_rows, impl Into<Tracks>, |v: Tracks| v);
    grid_setter!(
        /// CSS `grid-auto-flow`.
        grid_auto_flow, flow, GridAutoFlow, |v| v);
    grid_setter!(
        /// CSS `grid-column` of a grid item: `GridPlacement::span(4)`,
        /// `(1, -1)` (the whole row), `2`, `(2, GridPlacement::span(3))`.
        grid_column, column, impl Into<GridPlacement>, |v: GridPlacement| v);
    grid_setter!(
        /// CSS `grid-row` of a grid item.
        grid_row, row, impl Into<GridPlacement>, |v: GridPlacement| v);
    grid_setter!(
        /// CSS `justify-items`: how a grid's items sit in their areas along
        /// the row (default: stretch).
        justify_items, justify_items, Align, |v| v);
    grid_setter!(
        /// CSS `justify-self`: how this grid item sits in its area along
        /// the row.
        justify_self, justify_self, Align, |v| v);

    /// CSS `place-items`: `align_items` and `justify_items` at once
    /// (`place_items(Align::Center)` centers a grid's items in their
    /// areas).
    pub fn place_items(self, a: Align) -> Self {
        self.align_items(a).justify_items(a)
    }
}

/// Make this a scroll container along `axes`: content overflowing a
/// scrolling axis scrolls, the other axis clips (the base style of the
/// `scroll(..)` container).
pub fn with_scroll_axes(mut style: Style, axes: ScrollAxes) -> Style {
    style.flow_mut().scroll_axes = Some(axes);
    style
}
