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
//!   A state style changes paint only (see [`TransitionProp::Paint`]); a
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

use kivact_types::{Align, Cursor, ScrollAxes, TextAlign};

use super::color::Color;
use super::draw::{Draw, LocalPaint, Size};
use super::effect::{BlendMode, BorderStyle, CornerRadius, Filter, Shadow, Shape};
#[cfg(doc)]
use super::grid::Track;
use super::grid::{GridAutoFlow, GridPlacement, GridStyle, Tracks};
use super::length::Length;
use super::paint::{Background, Paint};
use super::text::{
    FontFeature, FontKerning, LineHeight, TextDecorationLine, TextExtras, TextLength, TextOverflow,
    TextTransform,
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

/// Property group animated by `Style::transition`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransitionProp {
    /// Paint: opacity, a solid background color, border and text colors, the
    /// transform (its translate, rotate, skew and scale parts and its
    /// origin: `rotate(0)` to `rotate(360)` turns once), the first two
    /// shadows (offset, blur, spread, color) and the filter's amounts
    /// (brightness .. opacity, blur, drop shadow). Gradients, images,
    /// shapes, the z-index and the filter's order change at once.
    Paint,
    /// Entrance and exit size along the selected axis.
    Size,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub(crate) struct Sides {
    pub(crate) top: Option<Length>,
    pub(crate) right: Option<Length>,
    pub(crate) bottom: Option<Length>,
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

/// A style value. Build with [`style()`].
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Style {
    pub(super) local_paint: Option<LocalPaint>,
    pub(super) padding: Sides,
    pub(super) margin: Sides,
    pub(super) inset: Sides,
    pub(super) row_gap: Option<Length>,
    pub(super) column_gap: Option<Length>,
    pub(super) width: Option<Length>,
    pub(super) height: Option<Length>,
    pub(super) min_width: Option<Length>,
    pub(super) min_height: Option<Length>,
    pub(super) max_width: Option<Length>,
    pub(super) max_height: Option<Length>,
    pub(super) grow: Option<f64>,
    pub(super) shrink: Option<f64>,
    pub(super) basis: Option<Length>,
    pub(super) aspect_ratio: Option<f64>,
    pub(super) direction: Option<Direction>,
    pub(super) align: Option<Align>,
    pub(super) align_self: Option<Align>,
    pub(super) justify: Option<Justify>,
    pub(super) align_content: Option<Justify>,
    /// Lays the children out as a grid (CSS `display: grid`; set by
    /// `grid(..)` only).
    pub(super) display: Option<kivact_types::Display>,
    /// Grid tracks and placement (rare: boxed).
    pub(super) grid: Option<Box<GridStyle>>,
    pub(super) wrap: Option<bool>,
    pub(super) position: Option<Position>,
    pub(super) overflow: Option<Overflow>,
    /// Scroll axes set by `scroll(..)`.
    pub(super) scroll_axes: Option<ScrollAxes>,
    pub(super) background: Option<Background>,
    /// Border widths and colors, top, right, bottom, left.
    pub(super) border_width: [Option<f64>; 4],
    pub(super) border_color: [Option<Color>; 4],
    pub(super) border_style: Option<BorderStyle>,
    pub(super) ring: Option<(f64, f64, Color)>,
    /// Corner radii, top left, top right, bottom right, bottom left.
    pub(super) radius: [Option<CornerRadius>; 4],
    pub(super) shadows: Option<Vec<Shadow>>,
    pub(super) opacity: Option<f64>,
    /// CSS `translate` (x, then y: each axis is set and merged on its own),
    /// `rotate`, `scale`, `transform`, `transform-origin`.
    pub(super) translate: [Option<Length>; 2],
    pub(super) rotate: Option<f64>,
    pub(super) scale: Option<(f64, f64)>,
    pub(super) transform: Option<Transform>,
    pub(super) transform_origin: Option<(Length, Length)>,
    pub(super) scrollbar: Option<Box<ScrollbarStyle>>,
    pub(super) cursor: Option<Cursor>,
    pub(super) z_index: Option<i32>,
    pub(super) clip: Option<bool>,
    pub(super) filter: Option<Filter>,
    pub(super) backdrop_filter: Option<Filter>,
    pub(super) blend: Option<BlendMode>,
    pub(super) mask: Option<Box<Paint>>,
    pub(super) clip_path: Option<Box<Shape>>,
    pub(super) font_size: Option<f64>,
    pub(super) weight: Option<u16>,
    pub(super) color: Option<Color>,
    pub(super) line_height: Option<LineHeight>,
    pub(super) text_wrap: Option<bool>,
    pub(super) text_align: Option<TextAlign>,
    pub(super) font_family: Option<String>,
    pub(super) italic: Option<bool>,
    /// The rarer text properties (letter spacing, decorations, features,
    /// overflow, clamps, case, shadows, input colors: boxed).
    pub(super) text: Option<Box<TextExtras>>,
    pub(super) hover: Option<Box<Style>>,
    pub(super) group: bool,
    pub(super) group_hover: Option<Box<Style>>,
    pub(super) group_focus: Option<Box<Style>>,
    pub(super) pressed: Option<Box<Style>>,
    pub(super) focus: Option<Box<Style>>,
    pub(super) focus_visible: Option<Box<Style>>,
    pub(super) disabled: Option<Box<Style>>,
    pub(super) selected: Option<Box<Style>>,
    pub(super) enter_size: Option<kivact_types::Orientation>,
    pub(super) exit_size: Option<kivact_types::Orientation>,
    pub(super) enter: Option<Box<Style>>,
    pub(super) exit: Option<Box<Style>>,
    pub(super) transitions: Vec<(TransitionProp, Transition)>,
    pub(super) duplicates: u32,
}

/// The transition of the paint props (`TransitionProp::Paint`), if `s`
/// sets one.
pub(crate) fn paint_transition(s: &Style) -> Option<Transition> {
    s.transitions
        .iter()
        .find(|(p, _)| *p == TransitionProp::Paint)
        .map(|(_, t)| t.clone())
}

/// Start a style chain.
pub fn style() -> Style {
    Style::default()
}

macro_rules! setter {
    ($(#[$m:meta])* $name:ident, $field:ident, $t:ty, $conv:expr) => {
        $(#[$m])*
        pub fn $name(mut self, v: $t) -> Self {
            if self.$field.is_some() {
                self.duplicates += 1;
            }
            #[allow(clippy::redundant_closure_call, reason = "`$conv` may be a closure literal called in place")]
            {
                self.$field = Some(($conv)(v.into()));
            }
            self
        }
    };
}

/// A setter of a color input, which takes any [`ThemeColor`].
macro_rules! color_setter {
    ($(#[$m:meta])* $name:ident, $field:ident) => {
        $(#[$m])*
        pub fn $name(mut self, v: impl ThemeColor) -> Self {
            if self.$field.is_some() {
                self.duplicates += 1;
            }
            self.$field = Some(Color::of(v));
            self
        }
    };
}

macro_rules! sides {
    ($(#[$m:meta])* $name:ident, $field:ident, [$($side:ident),+]) => {
        $(#[$m])*
        pub fn $name(mut self, v: impl Into<Length>) -> Self {
            let v = v.into();
            $(
                if self.$field.$side.is_some() {
                    self.duplicates += 1;
                }
                self.$field.$side = Some(v);
            )+
            self
        }
    };
}

impl Style {
    sides!(padding, padding, [top, right, bottom, left]);
    sides!(padding_x, padding, [left, right]);
    sides!(padding_y, padding, [top, bottom]);
    sides!(/// CSS `padding-top`; other edges remain unset.
        padding_top, padding, [top]);
    sides!(/// CSS `padding-right`; other edges remain unset.
        padding_right, padding, [right]);
    sides!(/// CSS `padding-bottom`; other edges remain unset.
        padding_bottom, padding, [bottom]);
    sides!(/// CSS `padding-left`; other edges remain unset.
        padding_left, padding, [left]);
    sides!(margin, margin, [top, right, bottom, left]);
    sides!(margin_x, margin, [left, right]);
    sides!(margin_y, margin, [top, bottom]);
    sides!(/// CSS `margin-top`; other edges remain unset.
        margin_top, margin, [top]);
    sides!(/// CSS `margin-right`; other edges remain unset.
        margin_right, margin, [right]);
    sides!(/// CSS `margin-bottom`; other edges remain unset.
        margin_bottom, margin, [bottom]);
    sides!(/// CSS `margin-left`; other edges remain unset.
        margin_left, margin, [left]);
    sides!(inset, inset, [top, right, bottom, left]);
    sides!(top, inset, [top]);
    sides!(right, inset, [right]);
    sides!(bottom, inset, [bottom]);
    sides!(left, inset, [left]);

    setter!(
        /// CSS `row-gap`: between the rows of a grid or wrapped flex lines (a
        /// column's children).
        row_gap, row_gap, impl Into<Length>, |v: Length| v);
    setter!(
        /// CSS `column-gap`: between the columns of a grid (a row's
        /// children).
        column_gap, column_gap, impl Into<Length>, |v: Length| v);
    setter!(
        /// CSS `align-content`: how a grid's rows (or a wrapping flex box's
        /// lines) share the free space along the cross axis.
        align_content, align_content, Justify, |v| v);
    setter!(width, width, impl Into<Length>, |v: Length| v);
    setter!(height, height, impl Into<Length>, |v: Length| v);
    setter!(min_width, min_width, impl Into<Length>, |v: Length| v);
    setter!(min_height, min_height, impl Into<Length>, |v: Length| v);
    setter!(max_width, max_width, impl Into<Length>, |v: Length| v);
    setter!(max_height, max_height, impl Into<Length>, |v: Length| v);
    setter!(basis, basis, impl Into<Length>, |v: Length| v);
    setter!(
        /// CSS `aspect-ratio`: width / height (`aspect_ratio(16.0 / 9.0)`). With
        /// one of width and height set (or stretched), the other follows.
        aspect_ratio, aspect_ratio, impl Into<f64>, |v: f64| v);
    setter!(grow, grow, impl Into<f64>, |v: f64| v);
    setter!(shrink, shrink, impl Into<f64>, |v: f64| v);
    setter!(direction, direction, Direction, |v| v);
    setter!(
        /// Cross-axis alignment of children.
        align_items, align, Align, |v| v);
    setter!(align_self, align_self, Align, |v| v);
    setter!(
        /// Main-axis distribution of children.
        justify_content, justify, Justify, |v| v);
    setter!(wrap, wrap, bool, |v| v);
    setter!(position, position, Position, |v| v);
    setter!(
        /// Layout overflow on both axes; Hidden also clips layout overflow.
        /// For an axis-specific accessible scroll container, use `scroll`.
        overflow, overflow, Overflow, |v| v);
    setter!(
        /// The background (CSS `background`): a color, a gradient, a
        /// [`Paint`] or a tuple of them stacked like CSS layers, the first
        /// on top (see [`Background`]). Layers are placed in the border box
        /// and clipped to its rounded shape.
        background,
        background,
        impl Into<Background>,
        |v: Background| v
    );
    setter!(
        /// The opacity of the node and its subtree, as one group (CSS
        /// `opacity`: overlapping children do not show through each
        /// other).
        opacity, opacity, impl Into<f64>, |v: f64| v.clamp(0.0, 1.0));
    setter!(
        /// CSS `filter`: color functions recolor everything the subtree
        /// draws; `blur` and `drop_shadow` filter the subtree's pixels.
        filter, filter, Filter, |v| v);
    setter!(
        /// CSS `backdrop-filter`: blur and recolor what lies behind the
        /// border box (glass), beneath the node's own background, which is
        /// then usually translucent.
        backdrop_filter, backdrop_filter, Filter, |v| v);
    setter!(
        /// CSS `mix-blend-mode`.
        mix_blend_mode, blend, BlendMode, |v| v);
    setter!(
        /// CSS `mask`: the node and its subtree keep the alpha of this
        /// paint (`mask(linear_gradient(180).colors([Color::BLACK,
        /// Color::TRANSPARENT]))` fades them out downwards).
        mask, mask, impl Into<Paint>, |v: Paint| Box::new(v));
    setter!(
        /// CSS `clip-path`: the node's own paint (background, border,
        /// shadows) and its subtree are clipped to `shape`, and so are
        /// hit testing and pointer input: a press outside the shape does
        /// not reach the node.
        clip_path, clip_path, Shape, Box::new);
    setter!(
        /// CSS `transform`: the functions of `t` in their order
        /// (`Transform::new().skew_x(-12).translate_y(2)`), after the
        /// `translate`, `rotate` and `scale` properties, about the
        /// `transform_origin` (the box's centre by default). It maps the
        /// box and its whole subtree, which is hit, pointed at
        /// (`PointerEvent::pos` stays in the untransformed local space),
        /// reported (`NodeRef::rect`: the bounds as drawn) and exposed
        /// where it is drawn; layout ignores it (siblings do not move).
        transform, transform, Transform, |v| v);
    setter!(
        /// CSS `rotate`: clockwise, in degrees, after `translate` and
        /// before `scale` and `transform` (CSS's order for the individual
        /// transform properties, whatever the call order).
        rotate, rotate, impl Into<f64>, |v: f64| v);
    setter!(
        /// CSS `scale` (uniform), applied after `translate` and `rotate`
        /// and before `transform`. A state style can set it alone:
        /// `.hover(style().scale(1.05))` keeps the node's rotation.
        scale, scale, impl Into<f64>, |v: f64| (v, v));
    setter!(
        /// CSS `z-index`: the order the node is painted and hit in among
        /// its siblings (higher on top; equal values keep the tree order).
        /// Every container isolates its children's z-indices (as CSS
        /// `isolation: isolate`): a negative one paints after the parent's
        /// own background, below its other children. Tab order stays the
        /// tree order.
        z_index, z_index, i32, |v| v);
    /// Set overlay scrollbar width; no layout space is reserved.
    pub fn scrollbar_width(mut self, width: impl Into<ScrollbarWidth>) -> Self {
        self.scrollbar.get_or_insert_with(Default::default).width = Some(width.into());
        self
    }
    /// Set thumb and track colors (CSS scrollbar-color).
    pub fn scrollbar_color(mut self, thumb: impl ThemeColor, track: impl ThemeColor) -> Self {
        self.scrollbar.get_or_insert_with(Default::default).colors =
            Some((Color::of(thumb), Color::of(track)));
        self
    }
    setter!(cursor, cursor, Cursor, |v| v);
    setter!(
        /// Clip drawing to the box without changing layout overflow.
        /// Use `overflow(Overflow::Hidden)` to affect layout overflow too.
        clip, clip, bool, |v| v);
    setter!(
        /// CSS `font-size` of the text in this node and below.
        font_size, font_size, impl Into<f64>, |v: f64| v);
    setter!(
        /// CSS `font-weight` (1 to 1000) of the text in this node and below.
        weight, weight, u16, |v| v);
    color_setter!(
        /// CSS `color` of the text in this node and below. As in CSS, each
        /// text property is inherited on its own: a descendant that sets
        /// only a `weight` or a `font_size` keeps this color. Without one
        /// on any ancestor, text takes the App theme's
        /// [`ColorToken::Text`](crate::ColorToken::Text); a
        /// `provide_theme` subtree gets its theme's by a
        /// `color(ColorToken::Text)` on its top node.
        color, color);
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
        line_height, line_height, impl Into<LineHeight>, |v: LineHeight| v);
    setter!(text_wrap, text_wrap, bool, |v| v);
    setter!(text_align, text_align, TextAlign, |v| v);
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
        font_family, font_family, impl Into<String>, |v: String| v);
    setter!(
        /// Italic text (the family's italic face when it has one).
        italic, italic, bool, |v| v);

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
        if self.translate[0].is_some() {
            self.duplicates += 1;
        }
        self.translate[0] = Some(x.into());
        self
    }

    /// The vertical part of [`translate`](Self::translate) alone
    /// (Tailwind's `translate-y-*`), merged on its own like
    /// [`translate_x`](Self::translate_x).
    pub fn translate_y(mut self, y: impl Into<Length>) -> Self {
        if self.translate[1].is_some() {
            self.duplicates += 1;
        }
        self.translate[1] = Some(y.into());
        self
    }

    /// CSS `scale: x y` (non-uniform).
    pub fn scale_xy(mut self, x: impl Into<f64>, y: impl Into<f64>) -> Self {
        if self.scale.is_some() {
            self.duplicates += 1;
        }
        self.scale = Some((x.into(), y.into()));
        self
    }

    /// CSS `transform-origin: x y`, the point the transform properties
    /// apply about: px or a percentage of the box (`left top` is
    /// `(0, 0)`, `right bottom` is `(pct(100), pct(100))`; the default is
    /// the centre, `(pct(50), pct(50))`).
    pub fn transform_origin(mut self, x: impl Into<Length>, y: impl Into<Length>) -> Self {
        if self.transform_origin.is_some() {
            self.duplicates += 1;
        }
        self.transform_origin = Some((x.into(), y.into()));
        self
    }

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
        if self.border_color.iter().any(Option::is_some) {
            self.duplicates += 1;
        }
        self.border_color = [Some(color); 4];
        self
    }

    /// How every side's border is drawn (CSS `border-style`).
    pub fn border_style(mut self, s: BorderStyle) -> Self {
        if self.border_style.is_some() {
            self.duplicates += 1;
        }
        self.border_style = Some(s);
        self
    }

    fn border_sides<const N: usize>(
        mut self,
        sides: [usize; N],
        width: f64,
        color: Option<Color>,
    ) -> Self {
        if sides.iter().any(|i| self.border_width[*i].is_some()) {
            self.duplicates += 1;
        }
        for i in sides {
            self.border_width[i] = Some(width);
            if let Some(c) = color {
                self.border_color[i] = Some(c);
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
        if which.iter().any(|i| self.radius[*i].is_some()) {
            self.duplicates += 1;
        }
        for (i, r) in which.into_iter().zip(radii) {
            self.radius[i] = Some(r);
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
        if self.ring.is_some() {
            self.duplicates += 1;
        }
        self.ring = Some((width.into(), offset.into(), Color::of(color)));
        self
    }

    /// Add a box shadow (CSS `box-shadow`); several stack, the first on
    /// top. It follows the corner radii, elliptical ones too.
    pub fn shadow(mut self, s: Shadow) -> Self {
        self.shadows.get_or_insert_with(Vec::new).push(s);
        self
    }

    /// The style while the pointer is over the node or a node below it
    /// (CSS `:hover`, which a hovered child keeps on its parents).
    ///
    /// A state style changes paint only (see [`TransitionProp::Paint`]); a
    /// layout or text property other than `color` set in it is reported as
    /// `Unsupported` and ignored. `selected` and `disabled` apply any
    /// property.
    pub fn hover(mut self, s: Style) -> Self {
        self.hover = Some(Box::new(s));
        self
    }

    /// Make the node a group for the [`group_hover`](Self::group_hover)
    /// and [`group_focus`](Self::group_focus) styles below it (Tailwind's
    /// `group`).
    pub fn group(mut self) -> Self {
        self.group = true;
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
        self.group_hover = Some(Box::new(s));
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
        self.group_focus = Some(Box::new(s));
        self
    }

    pub fn pressed(mut self, s: Style) -> Self {
        self.pressed = Some(Box::new(s));
        self
    }

    /// The style while the element has the focus, however it got it.
    pub fn focus(mut self, s: Style) -> Self {
        self.focus = Some(Box::new(s));
        self
    }

    /// The style while the element has the focus and the host asks for a
    /// focus indication (CSS `:focus-visible`): focus from the keyboard or
    /// from a program, not from a pointer press. It applies over
    /// [`focus`](Self::focus). A focusable element whose style has neither
    /// state draws the theme's focus ring here.
    pub fn focus_visible(mut self, s: Style) -> Self {
        self.focus_visible = Some(Box::new(s));
        self
    }

    /// No ring (CSS `ring-0`): in a [`focus_visible`](Self::focus_visible)
    /// style it turns the theme's focus ring off.
    pub fn ring_none(self) -> Self {
        self.ring(0, Color::TRANSPARENT)
    }

    pub fn disabled(mut self, s: Style) -> Self {
        self.disabled = Some(Box::new(s));
        self
    }

    pub fn selected(mut self, s: Style) -> Self {
        self.selected = Some(Box::new(s));
        self
    }

    /// The paint the node starts from when it is mounted (CSS
    /// `@starting-style`, Framer Motion's `initial`): its
    /// [`TransitionProp::Paint`] transition moves it from there to its
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
    ///     .transition(TransitionProp::Paint, Transition::tween(260, Easing::EaseOut));
    /// # let _ = toast;
    /// ```
    ///
    /// Without a paint transition, and under reduced motion with a
    /// decorative one, the node shows its paint at once.
    pub fn enter(mut self, s: Style) -> Self {
        self.enter = Some(Box::new(s));
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
        self.exit = Some(Box::new(s));
        self
    }

    /// Reveal the selected axis at entrance when Size has a transition.
    pub fn enter_size(mut self, axis: kivact_types::Orientation) -> Self {
        self.enter_size = Some(axis);
        self
    }

    /// Collapse the selected axis before removal when Size has a transition.
    pub fn exit_size(mut self, axis: kivact_types::Orientation) -> Self {
        self.exit_size = Some(axis);
        self
    }

    /// Animate changes of a property group with `t`.
    pub fn transition(mut self, prop: TransitionProp, t: Transition) -> Self {
        self.transitions.retain(|(p, _)| *p != prop);
        self.transitions.push((prop, t));
        self
    }

    /// Select flex, grid, or a transparent container that contributes no box.
    pub fn display(mut self, display: kivact_types::Display) -> Style {
        self.display = Some(display);
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
        self.local_paint = Some(LocalPaint::new(f));
        self
    }

    /// Overlay `o` on `self` (fields set in `o` win).
    pub fn merge(mut self, o: &Style) -> Style {
        macro_rules! take {
            ($($f:ident),*) => {$(
                if o.$f.is_some() {
                    self.$f = o.$f.clone();
                }
            )*};
        }
        self.padding.merge(&o.padding);
        if let Some(other) = &o.scrollbar {
            let mine = self.scrollbar.get_or_insert_with(Default::default);
            mine.width = other.width.or(mine.width);
            mine.colors = other.colors.or(mine.colors);
        }
        if let Some(g) = &o.grid {
            self.grid.get_or_insert_with(Default::default).merge(g);
        }
        if let Some(t) = &o.text {
            self.text.get_or_insert_with(Default::default).merge(t);
        }
        self.margin.merge(&o.margin);
        self.inset.merge(&o.inset);
        for (a, b) in self.border_width.iter_mut().zip(o.border_width) {
            *a = b.or(*a);
        }
        for (a, b) in self.border_color.iter_mut().zip(o.border_color) {
            *a = b.or(*a);
        }
        for (a, b) in self.radius.iter_mut().zip(o.radius) {
            *a = b.or(*a);
        }
        for (a, b) in self.translate.iter_mut().zip(o.translate) {
            *a = b.or(*a);
        }
        take!(
            enter_size,
            exit_size,
            local_paint,
            row_gap,
            column_gap,
            align_content,
            display,
            width,
            height,
            min_width,
            min_height,
            max_width,
            max_height,
            grow,
            shrink,
            basis,
            aspect_ratio,
            direction,
            align,
            align_self,
            justify,
            wrap,
            position,
            overflow,
            scroll_axes,
            background,
            border_style,
            ring,
            shadows,
            opacity,
            rotate,
            scale,
            transform,
            transform_origin,
            z_index,
            cursor,
            clip,
            filter,
            backdrop_filter,
            blend,
            mask,
            clip_path,
            font_size,
            weight,
            color,
            line_height,
            text_wrap,
            text_align,
            font_family,
            italic
        );
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
        for t in &o.transitions {
            self.transitions.retain(|(p, _)| *p != t.0);
            self.transitions.push(t.clone());
        }
        self.duplicates += o.duplicates;
        self
    }

    pub(super) fn has_text(&self) -> bool {
        self.font_size.is_some()
            || self.weight.is_some()
            || self.color.is_some()
            || self.line_height.is_some()
            || self.text_wrap.is_some()
            || self.text_align.is_some()
            || self.font_family.is_some()
            || self.italic.is_some()
            || self.text.is_some()
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
    pub(super) width: Option<ScrollbarWidth>,
    pub(super) colors: Option<(Color, Color)>,
}

macro_rules! text_color_setter {
    ($(#[$m:meta])* $name:ident, $field:ident) => {
        $(#[$m])*
        pub fn $name(mut self, v: impl ThemeColor) -> Self {
            let x = self.text.get_or_insert_with(Default::default);
            if x.$field.is_some() {
                self.duplicates += 1;
            }
            x.$field = Some(Color::of(v));
            self
        }
    };
}

macro_rules! text_setter {
    ($(#[$m:meta])* $name:ident, $field:ident, $t:ty, $conv:expr) => {
        $(#[$m])*
        pub fn $name(mut self, v: $t) -> Self {
            let x = self.text.get_or_insert_with(Default::default);
            if x.$field.is_some() {
                self.duplicates += 1;
            }
            #[allow(clippy::redundant_closure_call, reason = "`$conv` may be a closure literal called in place")]
            {
                x.$field = Some(($conv)(v.into()));
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
        let x = self.text.get_or_insert_with(Default::default);
        if x.features.is_some() {
            self.duplicates += 1;
        }
        x.features = Some(features.into_iter().map(Into::into).collect());
        self
    }

    /// CSS `font-variant-numeric: tabular-nums`: digits of one width (the
    /// font's `tnum` feature), so numbers in a column or a timer do not
    /// jiggle.
    pub fn tabular_nums(mut self) -> Self {
        let x = self.text.get_or_insert_with(Default::default);
        if x.tabular_nums.is_some() {
            self.duplicates += 1;
        }
        x.tabular_nums = Some(true);
        self
    }

    /// Add a text shadow (CSS `text-shadow: x y blur color`); several
    /// stack, the first on top. They shadow the glyphs and decoration
    /// lines (a shadow's spread and inset do not apply to text and are
    /// reported as unsupported).
    pub fn text_shadow(mut self, s: Shadow) -> Self {
        let x = self.text.get_or_insert_with(Default::default);
        x.shadows.get_or_insert_with(Vec::new).push(s);
        self
    }
}

macro_rules! grid_setter {
    ($(#[$m:meta])* $name:ident, $field:ident, $t:ty, $conv:expr) => {
        $(#[$m])*
        pub fn $name(mut self, v: $t) -> Self {
            let g = self.grid.get_or_insert_with(Default::default);
            if g.$field.is_some() {
                self.duplicates += 1;
            }
            #[allow(clippy::redundant_closure_call, reason = "`$conv` may be a closure literal called in place")]
            {
                g.$field = Some(($conv)(v.into()));
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
    style.scroll_axes = Some(axes);
    style
}
