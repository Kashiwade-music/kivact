//! Role: Style property resolution against the theme into node props.
//!
//! Style + theme → the engine's node props (`wire`): layout, text and
//! paint (backgrounds, borders, radii, shadows, effects, the transform and
//! the z-index), and the paint deltas of the interaction states. The
//! values a style holds resolve themselves (`Color::resolve`,
//! `paint::fill_layer`, `effect::filter_ops`, `text::resolve_extras`), so a
//! `Draw` resolves them the same way without a `Style`.
//!
//! Invariants:
//! - Every color position (gradient stops, each border side, shadows, drop
//!   shadows, the mask) resolves its theme token here, once.
//! - A solid background over the whole box is an allocation-free
//!   `Fill::Solid`; only a node that uses effects holds an
//!   `Option<Box<Effects>>`.
//! - A state style is resolved over the base by groups (background,
//!   border, radius, shadow, ring, effects, transform, z-index), so a state
//!   that sets only `border_color` keeps the base width, and one that sets
//!   only `scale` keeps the base rotation.
//! - A transition over the paint properties (`TransitionProp::All`) works the same for a
//!   signal or style change and for the host's hover, pressed and focus
//!   switches: it interpolates the opacity, a solid background, each
//!   border color, the canonical transform and its origin, the first two
//!   shadows and the filter amounts; gradients, images, shapes, the
//!   z-index and the order of filters switch to the target at once.
//! - A dynamic style that drops a state writes empty `StateStyles`, and one
//!   that drops a text property writes `None`, through the typed setter, so
//!   the node goes back to what it inherits.

use kivact_runtime::{Transition, UiError};
use kivact_types as p;
use kivact_types::{
    Border, Corners, Edges, Effects, ElementTransform, Fill, LayoutProps, PaintDelta, PaintProps,
    Perspective, Repeat, Ring, ScrollAxes, StateStyles, TextStyle,
};

use super::color::ColorToken;
use super::draw::{Draw, LocalPaint, Size};
use super::effect::{Shadow, clip_shape, filter_ops, filter_problems};
use super::geometry::corner;
use super::length::{Length, pct};
use super::paint::{Background, fill_layer};
use super::sheet::{Direction, Overflow, Position, ScrollbarWidth, Sides, Style};
use super::theme::value::Theme;
use super::transform::Transform;

// xtask-allow(state-fields): This compilation result independently contains optional text, state deltas and retained paint.
/// A [`Style`] compiled against a theme: the engine's node props and the
/// diagnostics of the compilation, made by [`crate::internal::compile_style`].
///
/// The retained base paint callback (`Style::paint`) is not a node prop. It
/// runs while the node's drawing is recorded: ask [`has_retained_paint`]
/// (whether there is one), [`same_retained_paint`] (whether another compiled
/// style retains the same callback) and call [`paint_retained`].
///
/// [`has_retained_paint`]: CompiledStyle::has_retained_paint
/// [`same_retained_paint`]: CompiledStyle::same_retained_paint
/// [`paint_retained`]: CompiledStyle::paint_retained
#[derive(Clone)]
pub struct CompiledStyle {
    retained_paint: Option<LocalPaint>,
    /// Layout props.
    pub layout: LayoutProps,
    /// Paint props.
    pub paint: PaintProps,
    /// Text props: when the style sets any (only what it sets: the rest
    /// is inherited property by property), and always for the root, which
    /// carries the theme's text defaults that every node inherits.
    pub text: Option<TextStyle>,
    /// Hover/pressed/focus/disabled overrides, if any.
    pub states: Option<StateStyles>,
    /// Properties set more than once (reported as diagnostics).
    pub duplicates: u32,
    /// Content sizes (`Length::MaxContent` ..) where only sizes take them (they
    /// are `auto` there), reported as `Unsupported`.
    pub misplaced_content: u32,
    /// Text values the host cannot draw (a feature tag that is not four
    /// ASCII characters, a text shadow's spread or inset), dropped and
    /// reported as `Unsupported`.
    pub unsupported_text: u32,
    /// Properties set in a state style that the state cannot change, as
    /// `state.property` (`"hover.width"`, `"disabled.hover.font_size"`):
    /// ignored and reported as `Unsupported`.
    pub unsupported_in_states: Vec<String>,
    /// Problems the style reports on compilation: spring transitions that
    /// cannot run, a layout transition that repeats, shader values and
    /// filter graphs that do not validate, transform functions the form
    /// cannot keep.
    pub reports: Vec<UiError>,
}

fn dim(l: Option<Length>, default: p::Dimension) -> p::Dimension {
    l.map(Length::to_wire).unwrap_or(default)
}

fn edges(s: &Sides, default: p::Dimension) -> Edges<p::Dimension> {
    Edges {
        top: dim(s.top, default),
        right: dim(s.right, default),
        bottom: dim(s.bottom, default),
        left: dim(s.left, default),
    }
}

fn layout(s: &Style) -> LayoutProps {
    let zero = p::Dimension::Px(0.0);
    let ov = s.overflow.unwrap_or(p::Overflow::Visible);
    let (ov_x, ov_y) = match s.scroll_axes {
        Some(ScrollAxes::Vertical) => (p::Overflow::Hidden, p::Overflow::Scroll),
        Some(ScrollAxes::Horizontal) => (p::Overflow::Scroll, p::Overflow::Hidden),
        Some(ScrollAxes::Both) => (p::Overflow::Scroll, p::Overflow::Scroll),
        None => (ov, ov),
    };
    LayoutProps {
        scroll_snap: s.scroll_snap.unwrap_or_default(),
        enter_size: s.enter_size,
        exit_size: s.exit_size,
        layout_transition: s.layout_transition.clone(),
        size_transition: s
            .transitions
            .iter()
            .find(|(p, _)| *p == super::sheet::TransitionProp::Size)
            .map(|(_, t)| t.clone()),
        direction: s.direction.unwrap_or(Direction::Column),
        align_items: s.align,
        align_self: s.align_self,
        justify: s.justify,
        align_content: s.align_content,
        wrap: s.wrap.unwrap_or(false),
        row_gap: dim(s.row_gap, zero),
        column_gap: dim(s.column_gap, zero),
        padding: edges(&s.padding, zero),
        border: border_widths(s),
        margin: edges(&s.margin, zero),
        width: dim(s.width, p::Dimension::Auto),
        height: dim(s.height, p::Dimension::Auto),
        min_width: dim(s.min_width, p::Dimension::Auto),
        min_height: dim(s.min_height, p::Dimension::Auto),
        max_width: dim(s.max_width, p::Dimension::Auto),
        max_height: dim(s.max_height, p::Dimension::Auto),
        grow: s.grow.unwrap_or(0.0),
        shrink: s.shrink.unwrap_or(1.0),
        basis: dim(s.basis, p::Dimension::Auto),
        aspect_ratio: s.aspect_ratio,
        position: s.position.unwrap_or(Position::Relative),
        inset: edges(&s.inset, p::Dimension::Auto),
        overflow_x: ov_x,
        overflow_y: ov_y,
        hidden: false,
        visibility: s.visibility.unwrap_or_default(),
        display: s.display.unwrap_or_default(),
        grid: super::grid::resolve(s.grid.as_deref(), s.display == Some(p::Display::Grid)),
    }
}

/// The number of content sizes (`Length::MinContent`, `Length::MaxContent`,
/// `Length::FitContent`) among the lengths of `s` that are not sizes (width,
/// height, the flex basis): minimums, maximums, padding, margins, insets
/// and gaps, where they are `auto`.
fn misplaced_content(s: &Style) -> u32 {
    let sides = |x: &Sides| [x.top, x.right, x.bottom, x.left];
    [
        s.row_gap,
        s.column_gap,
        s.min_width,
        s.min_height,
        s.max_width,
        s.max_height,
    ]
    .into_iter()
    .chain(sides(&s.padding))
    .chain(sides(&s.margin))
    .chain(sides(&s.inset))
    .flatten()
    .filter(|l| l.is_content())
    .count() as u32
}

/// Text props of a style: the properties it sets, and how many of its
/// values were dropped (reported as `Unsupported`:
/// `style::text::resolve_extras`). The host takes each unset one from the
/// nearest ancestor that sets it, and the root carries the theme's
/// ([`resolve`]).
fn text(s: &Style, theme: &Theme) -> (TextStyle, u32) {
    let mut t = TextStyle {
        font_size: s.font_size,
        weight: s.weight,
        color: s.color.map(|c| c.resolve(theme)),
        line_height: s.line_height,
        wrap: s.text_wrap.map(|wrap| {
            if wrap {
                p::TextWrap::Wrap
            } else {
                p::TextWrap::NoWrap
            }
        }),
        align: s.text_align,
        family: s.font_family.clone(),
        italic: s.italic,
        ..TextStyle::default()
    };
    let dropped = match &s.text {
        Some(x) => super::text::resolve_extras(&mut t, x, theme),
        None => 0,
    };
    (t, dropped)
}

/// Interactive states a tool forces on a node (DevTools), as a set.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ForcedStates(u8);
impl ForcedStates {
    /// Hovered.
    pub const HOVER: Self = Self(1);
    /// Pressed.
    pub const PRESSED: Self = Self(2);
    /// Focused.
    pub const FOCUS: Self = Self(4);
    /// Focused with a visible focus ring.
    pub const FOCUS_VISIBLE: Self = Self(8);
    /// Disabled.
    pub const DISABLED: Self = Self(16);
    /// Selected.
    pub const SELECTED: Self = Self(32);
    /// Whether every state of `s` is in the set.
    pub fn contains(self, s: Self) -> bool {
        self.0 & s.0 == s.0
    }
    /// Add the states of `s`.
    pub fn insert(&mut self, s: Self) {
        self.0 |= s.0;
    }
    /// Whether no state is forced.
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
}

// xtask-allow(state-fields): Independent input facts select compiled style defaults; this value owns no transitions.
/// How a node takes part in the theme's defaults for interactive elements,
/// the input of [`crate::internal::compile_style`] next to the style and the theme.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Interactive {
    // xtask-allow(state-fields): passive record (compile input) - The node is a focus stop (focusable, or a native editor): it gets the theme's focus ring and disabled opacity unless its style sets its own.
    /// The node is a focus stop (focusable, or a native editor): it gets
    /// the theme's focus ring and disabled opacity unless its style sets
    /// its own.
    pub focusable: bool,
    // xtask-allow(state-fields): passive record (compile input) - Apply the style's `disabled` state style, and take no hover or pressed look.
    /// Apply the style's `disabled` state style, and take no hover or pressed
    /// look.
    pub disabled: bool,
    // xtask-allow(state-fields): passive record (compile input) - Apply the style's `selected` state style.
    /// Apply the style's `selected` state style.
    pub selected: bool,
    // xtask-allow(state-fields): passive record (compile input) - The node is the root of the App: its text props always carry the theme's text defaults, which every node inherits.
    /// The node is the root of the App: its text props always carry the
    /// theme's text defaults, which every node inherits.
    pub root: bool,
    /// States a tool forces on the node; empty for an App.
    pub forced: ForcedStates,
}

/// The theme's defaults under a focusable node's style: a ring of
/// [`Theme::focus_ring`] in `ColorToken::Focus` while the focus is visible,
/// unless the style has a focus look of its own (a `focus` or a
/// `focus_visible` state), and [`Theme::disabled_opacity`] while disabled,
/// unless its disabled state sets an opacity. A disabled one takes no hover
/// or pressed look either: it does not respond, so it does not look as if
/// it would.
fn interactive_defaults(s: &mut Style, theme: &Theme, disabled: bool) {
    if disabled {
        s.hover = None;
        s.pressed = None;
    }
    if s.focus.is_none() && s.focus_visible.is_none() {
        let ring = Style::default().ring_with_offset(
            theme.focus_ring(),
            theme.focus_ring_offset(),
            ColorToken::Focus,
        );
        s.focus_visible = Some(Box::new(ring));
    }
    let disabled = s.disabled.get_or_insert_with(Default::default);
    if disabled.opacity.is_none() {
        disabled.opacity = Some(theme.disabled_opacity());
    }
}

/// Merge the state styles a tool forces into the base, in the engine's
/// layer order, so they no longer apply as deltas.
fn merge_forced(mut s: Style, forced: ForcedStates) -> Style {
    if forced.contains(ForcedStates::FOCUS)
        && let Some(v) = s.focus.take()
    {
        s = s.merge(&v);
    }
    if forced.contains(ForcedStates::FOCUS_VISIBLE)
        && let Some(v) = s.focus_visible.take()
    {
        s = s.merge(&v);
    }
    if forced.contains(ForcedStates::HOVER)
        && let Some(v) = s.hover.take()
    {
        s = s.merge(&v);
    }
    if forced.contains(ForcedStates::PRESSED)
        && let Some(v) = s.pressed.take()
    {
        s = s.merge(&v);
    }
    s
}

/// The host-evaluated state deltas of `s`, if it sets any state.
fn state_styles(s: &Style, theme: &Theme) -> Option<StateStyles> {
    (s.hover.is_some()
        || s.group_hover.is_some()
        || s.group_focus.is_some()
        || s.group
        || s.pressed.is_some()
        || s.focus.is_some()
        || s.focus_visible.is_some()
        || s.enter.is_some()
        || s.exit.is_some())
    .then(|| StateStyles {
        hover: s.hover.as_ref().map(|h| paint_delta(s, h, theme)),
        pressed: s.pressed.as_ref().map(|h| paint_delta(s, h, theme)),
        focus: s.focus.as_ref().map(|h| paint_delta(s, h, theme)),
        focus_visible: s.focus_visible.as_ref().map(|h| {
            // Over the focus state, as the host applies it.
            let focused = match &s.focus {
                Some(focus) => s.clone().merge(focus),
                None => s.clone(),
            };
            paint_delta(&focused, h, theme)
        }),
        group_hover: s.group_hover.as_ref().map(|h| paint_delta(s, h, theme)),
        group_focus: s.group_focus.as_ref().map(|h| paint_delta(s, h, theme)),
        group: s.group,
        enter: s.enter.as_ref().map(|h| paint_delta(s, h, theme)),
        exit: s.exit.as_ref().map(|h| paint_delta(s, h, theme)),
    })
}

/// Compile a style for a node. `disabled`/`selected` variants are merged
/// here; hover/pressed/focus/focus-visible become host-evaluated state
/// deltas.
pub fn compile_style(s: &Style, theme: &Theme, mut state: Interactive) -> CompiledStyle {
    if state.forced.contains(ForcedStates::DISABLED) {
        state.disabled = true;
    }
    if state.forced.contains(ForcedStates::SELECTED) {
        state.selected = true;
    }
    let retained_paint = s.local_paint.clone();
    let mut s = s.clone();
    let unsupported_in_states = unsupported_in_states(&s);
    strip_state_paint(&mut s);
    if state.focusable {
        interactive_defaults(&mut s, theme, state.disabled);
    }
    if state.selected
        && let Some(v) = s.selected.take()
    {
        s = s.merge(&v);
    }
    if state.disabled
        && let Some(v) = s.disabled.take()
    {
        s = s.merge(&v);
    }
    s = merge_forced(s, state.forced);
    let states = state_styles(&s, theme);
    let mut unsupported_text = 0;
    let text = (s.has_text() || state.root).then(|| {
        let (mut t, dropped) = text(&s, theme);
        unsupported_text = dropped;
        if state.root {
            // The theme's text style is what every node inherits.
            t.font_size = t.font_size.or(Some(theme.text().size));
            t.weight = t.weight.or(Some(theme.text().weight));
            t.color = t.color.or(Some(theme.colors().text.resolve(theme)));
            t.family = t.family.or_else(|| theme.text().family.clone());
            t.line_height = t.line_height.or(Some(theme.text().line_height));
        }
        t
    });
    let mut paint = paint(&s, theme);
    paint.transitions = paint_transitions(&s);
    let reports = reports(&s, theme);
    CompiledStyle {
        retained_paint,
        layout: layout(&s),
        paint,
        text,
        states,
        duplicates: s.duplicates,
        misplaced_content: misplaced_content(&s),
        unsupported_text,
        unsupported_in_states,
        reports,
    }
}

/// State styles only carry declarative host properties, never callbacks.
fn strip_state_paint(style: &mut Style) {
    [
        &mut style.hover,
        &mut style.group_hover,
        &mut style.group_focus,
        &mut style.pressed,
        &mut style.focus,
        &mut style.focus_visible,
        &mut style.disabled,
        &mut style.selected,
        &mut style.enter,
        &mut style.exit,
    ]
    .into_iter()
    .flatten()
    .for_each(|state| {
        state.local_paint = None;
        strip_state_paint(state);
    });
}

/// What a hover, pressed, focus, focus-visible, group-hover, group-focus,
/// enter or exit style can change: the fields `paint_delta` reads.
const DELTA: &[&str] = &[
    "background",
    "border_width",
    "border_color",
    "border_style",
    "ring",
    "radius",
    "opacity",
    "shadow",
    "translate",
    "rotate",
    "scale",
    "transform",
    "transform_origin",
    "translate_z",
    "rotate_x",
    "rotate_y",
    "transform_origin_z",
    "perspective",
    "perspective_origin",
    "transform_style",
    "backface_visibility",
    "z_index",
    "scrollbar",
    "cursor",
    "filter",
    "backdrop_filter",
    "blend",
    "mask",
    "clip_path",
    "color",
];

/// Declares `set_properties`, which lists the properties `s` sets by the
/// name of the setter that sets them, and one classifier per group. Every
/// field of `Style` is named in the exhaustive pattern of `set_properties`,
/// so a new field must be classified in one of the groups below.
macro_rules! classify_properties {
    (
        $(
            fn $group:ident {
                $($field:ident => $set:expr, $name:literal;)*
            }
        )*
        @state fn $state_group:ident {
            $($state_field:ident => $state_set:expr, $state_name:literal;)*
        }
        @ignored $($ignored:ident),*
    ) => {
        fn set_properties(s: &Style) -> Vec<&'static str> {
            let Style { $($($field: _,)*)* $($state_field: _,)* $($ignored: _,)* } = s;
            let mut out = Vec::new();
            $(out.extend($group(s));)*
            out.extend($state_group(s));
            out
        }

        /// Every property of the non-state groups that `s` sets, with the
        /// `Debug` of its value.
        fn describe_properties(s: &Style) -> Vec<(&'static str, String)> {
            let Style { $($($field: _,)*)* $($state_field: _,)* $($ignored: _,)* } = s;
            let mut out = Vec::new();
            $({
                let Style { $($field,)* .. } = s;
                $(
                    if $set {
                        out.push(($name, format!("{:?}", $field)));
                    }
                )*
            })*
            out
        }

        $(
            fn $group(s: &Style) -> Vec<&'static str> {
                let Style { $($field,)* .. } = s;
                [$(($set, $name),)*]
                    .into_iter()
                    .filter_map(|(set, name)| set.then_some(name))
                    .collect()
            }
        )*
        fn $state_group(s: &Style) -> Vec<&'static str> {
            let Style { $($state_field,)* .. } = s;
            [$(($state_set, $state_name),)*]
                .into_iter()
                .filter_map(|(set, name)| set.then_some(name))
                .collect()
        }
    };
}

classify_properties! {
    fn layout_properties {
        enter_size => enter_size.is_some(), "enter_size";
        exit_size => exit_size.is_some(), "exit_size";
        local_paint => local_paint.is_some(), "paint";
        padding => *padding != Sides::default(), "padding";
        margin => *margin != Sides::default(), "margin";
        inset => *inset != Sides::default(), "inset";
        row_gap => row_gap.is_some(), "row_gap";
        column_gap => column_gap.is_some(), "column_gap";
        width => width.is_some(), "width";
        height => height.is_some(), "height";
        min_width => min_width.is_some(), "min_width";
        min_height => min_height.is_some(), "min_height";
        max_width => max_width.is_some(), "max_width";
        max_height => max_height.is_some(), "max_height";
        grow => grow.is_some(), "grow";
        shrink => shrink.is_some(), "shrink";
        basis => basis.is_some(), "basis";
        aspect_ratio => aspect_ratio.is_some(), "aspect_ratio";
        direction => direction.is_some(), "direction";
        align => align.is_some(), "align_items";
        align_self => align_self.is_some(), "align_self";
        justify => justify.is_some(), "justify_content";
        align_content => align_content.is_some(), "align_content";
        display => display.is_some(), "display";
        visibility => visibility.is_some(), "visibility";
        grid => grid.is_some(), "grid";
        wrap => wrap.is_some(), "wrap";
    }
    fn position_properties {
        position => position.is_some(), "position";
        overflow => overflow.is_some(), "overflow";
        scroll_snap => scroll_snap.is_some(), "scroll_snap";
        scroll_axes => scroll_axes.is_some(), "with_scroll_axes";
        scrollbar => scrollbar.is_some(), "scrollbar";
        cursor => cursor.is_some(), "cursor";
        z_index => z_index.is_some(), "z_index";
        clip => clip.is_some(), "clip";
    }
    fn paint_properties {
        background => background.is_some(), "background";
        border_width => border_width.iter().any(Option::is_some), "border_width";
        border_color => border_color.iter().any(Option::is_some), "border_color";
        border_style => border_style.iter().any(Option::is_some), "border_style";
        ring => ring.is_some(), "ring";
        radius => radius.iter().any(Option::is_some), "radius";
        shadows => shadows.is_some(), "shadow";
        opacity => opacity.is_some(), "opacity";
        border_image => border_image.is_some(), "border_image";
        isolation => isolation.is_some(), "isolation";
        filter => filter.is_some(), "filter";
        backdrop_filter => backdrop_filter.is_some(), "backdrop_filter";
        blend => blend.is_some(), "blend";
        mask => mask.is_some(), "mask";
        clip_path => clip_path.is_some(), "clip_path";
    }
    fn transform_properties {
        translate => translate.iter().any(Option::is_some), "translate";
        rotate => rotate.is_some(), "rotate";
        scale => scale.is_some(), "scale";
        transform => transform.is_some(), "transform";
        transform_origin => transform_origin.is_some(), "transform_origin";
        translate_z => translate_z.is_some(), "translate_z";
        rotate_x => rotate_x.is_some(), "rotate_x";
        rotate_y => rotate_y.is_some(), "rotate_y";
        transform_origin_z => transform_origin_z.is_some(), "transform_origin_z";
        perspective => perspective.is_some(), "perspective";
        perspective_origin => perspective_origin.is_some(), "perspective_origin";
        transform_style => transform_style.is_some(), "transform_style";
        backface => backface.is_some(), "backface_visibility";
        will_change => will_change.is_some(), "will_change";
    }
    fn text_properties {
        font_size => font_size.is_some(), "font_size";
        weight => weight.is_some(), "weight";
        color => color.is_some(), "color";
        line_height => line_height.is_some(), "line_height";
        text_wrap => text_wrap.is_some(), "text_wrap";
        text_align => text_align.is_some(), "text_align";
        font_family => font_family.is_some(), "font_family";
        italic => italic.is_some(), "italic";
        text => text.is_some(), "text";
    }
    @state fn state_properties {
        hover => hover.is_some(), "hover";
        group => *group, "group";
        group_hover => group_hover.is_some(), "group_hover";
        group_focus => group_focus.is_some(), "group_focus";
        pressed => pressed.is_some(), "pressed";
        focus => focus.is_some(), "focus";
        focus_visible => focus_visible.is_some(), "focus_visible";
        disabled => disabled.is_some(), "disabled";
        selected => selected.is_some(), "selected";
        enter => enter.is_some(), "enter";
        exit => exit.is_some(), "exit";
        transitions => !transitions.is_empty(), "transition";
        layout_transition => layout_transition.is_some(), "layout_transition";
    }
    @ignored duplicates
}

/// Names of the four edges of a `padding`, `margin` or `inset`.
const PADDING: [&str; 4] = [
    "padding_top",
    "padding_right",
    "padding_bottom",
    "padding_left",
];
const MARGIN: [&str; 4] = ["margin_top", "margin_right", "margin_bottom", "margin_left"];
const INSET: [&str; 4] = ["top", "right", "bottom", "left"];
/// Names of the four edges of a `border-width`, `border-color` or
/// `border-style` array.
const BORDER_WIDTH: [&str; 4] = [
    "border_top_width",
    "border_right_width",
    "border_bottom_width",
    "border_left_width",
];
const BORDER_COLOR: [&str; 4] = [
    "border_top_color",
    "border_right_color",
    "border_bottom_color",
    "border_left_color",
];
const BORDER_STYLE: [&str; 4] = [
    "border_top_style",
    "border_right_style",
    "border_bottom_style",
    "border_left_style",
];
/// Names of the four corners of a `radius`, in CSS's order.
const RADIUS: [&str; 4] = [
    "radius_top_left",
    "radius_top_right",
    "radius_bottom_right",
    "radius_bottom_left",
];
/// Names of the two axes of a `translate`.
const TRANSLATE: [&str; 2] = ["translate_x", "translate_y"];

/// Separator of the per-edge values a [`describe_style`] splits. A unit
/// separator never occurs in the `Debug` of a value.
const SEP: &str = "\u{1f}";

/// The `Debug` of each element that is set, separated by [`SEP`] (`""` for
/// an unset one).
fn options_value<T: std::fmt::Debug>(items: &[Option<T>]) -> String {
    items
        .iter()
        .map(|item| item.as_ref().map(|v| format!("{v:?}")).unwrap_or_default())
        .collect::<Vec<_>>()
        .join(SEP)
}

/// The `Debug` of each set edge of `sides`, separated by [`SEP`].
fn sides_value(sides: &Sides) -> String {
    options_value(&[sides.top, sides.right, sides.bottom, sides.left])
}

/// Split one set property's value into the names of its edges or corners.
fn split_sides(name: &'static str, value: String) -> Vec<(&'static str, String)> {
    let names: &[&'static str] = match name {
        "padding" => &PADDING,
        "margin" => &MARGIN,
        "inset" => &INSET,
        "border_width" => &BORDER_WIDTH,
        "border_color" => &BORDER_COLOR,
        "border_style" => &BORDER_STYLE,
        "radius" => &RADIUS,
        "translate" => &TRANSLATE,
        _ => return vec![(name, value)],
    };
    names
        .iter()
        .zip(value.split(SEP))
        .filter(|(_, part)| !part.is_empty())
        .map(|(name, part)| (*name, part.to_owned()))
        .collect()
}

/// The properties `s` sets, with each edge and corner under its own name
/// and the `Debug` of its value.
pub fn describe_style(s: &Style) -> Vec<(&'static str, String)> {
    let mut out = Vec::new();
    for (name, value) in describe_properties(s) {
        let value = match name {
            "padding" => sides_value(&s.padding),
            "margin" => sides_value(&s.margin),
            "inset" => sides_value(&s.inset),
            "border_width" => options_value(&s.border_width),
            "border_color" => options_value(&s.border_color),
            "border_style" => options_value(&s.border_style),
            "radius" => options_value(&s.radius),
            "translate" => options_value(&s.translate),
            _ => value,
        };
        out.extend(split_sides(name, value));
    }
    out
}

/// The properties of each state style `s` sets: `"hover"`, `"pressed"`,
/// `"focus"`, `"focus_visible"`, `"disabled"` and `"selected"`.
pub fn describe_state_styles(s: &Style) -> Vec<(&'static str, Vec<(&'static str, String)>)> {
    [
        ("hover", &s.hover),
        ("pressed", &s.pressed),
        ("focus", &s.focus),
        ("focus_visible", &s.focus_visible),
        ("disabled", &s.disabled),
        ("selected", &s.selected),
    ]
    .into_iter()
    .filter_map(|(name, state)| state.as_deref().map(|state| (name, describe_style(state))))
    .collect()
}

/// Push `prefix` + `state.property` for every property the delta states
/// (hover, pressed, focus, ..) of `s` set that a delta cannot change.
fn delta_misuse(prefix: &str, s: &Style, out: &mut Vec<String>) {
    for (name, state) in [
        ("hover", &s.hover),
        ("group_hover", &s.group_hover),
        ("group_focus", &s.group_focus),
        ("pressed", &s.pressed),
        ("focus", &s.focus),
        ("focus_visible", &s.focus_visible),
        ("enter", &s.enter),
        ("exit", &s.exit),
    ] {
        if let Some(state) = state {
            out.extend(
                set_properties(state)
                    .into_iter()
                    .filter(|property| !DELTA.contains(property))
                    .map(|property| format!("{prefix}{name}.{property}")),
            );
        }
    }
}

/// The properties set in `s`'s state styles that their state cannot change
/// (see `CompiledStyle::unsupported_in_states`).
fn unsupported_in_states(s: &Style) -> Vec<String> {
    let mut out = Vec::new();
    delta_misuse("", s, &mut out);
    if let Some(selected) = &s.selected {
        if selected.local_paint.is_some() {
            out.push("selected.paint".to_string());
        }
        if selected.selected.is_some() {
            out.push("selected.selected".to_string());
        }
        delta_misuse("selected.", selected, &mut out);
    }
    if let Some(disabled) = &s.disabled {
        if disabled.local_paint.is_some() {
            out.push("disabled.paint".to_string());
        }
        if disabled.disabled.is_some() {
            out.push("disabled.disabled".to_string());
        }
        if disabled.selected.is_some() {
            out.push("disabled.selected".to_string());
        }
        delta_misuse("disabled.", disabled, &mut out);
    }
    out
}

/// A background as the engine fills it: one color covering the box stays a
/// plain color (the common case costs no allocation).
fn fill(b: &Background, theme: &Theme) -> Fill {
    match b.solid() {
        Some(c) => Fill::Solid(c.resolve(theme)),
        None => Fill::Layers(b.0.iter().map(|l| fill_layer(l, theme)).collect()),
    }
}

/// The border's widths (0 where unset): the layout's border, as the
/// paint's ([`border`]).
fn border_widths(s: &Style) -> Edges<f64> {
    let w = |i: usize| s.border_width[i].unwrap_or(0.0);
    Edges {
        top: w(0),
        right: w(1),
        bottom: w(2),
        left: w(3),
    }
}

/// The border, when the style sets any of its properties (a width without
/// a color takes the theme's border color).
fn border(s: &Style, theme: &Theme) -> Option<Border> {
    let set = s.border_width.iter().any(Option::is_some)
        || s.border_color.iter().any(Option::is_some)
        || s.border_style.iter().any(Option::is_some);
    set.then(|| {
        let c = |i: usize| {
            s.border_color[i]
                .unwrap_or(theme.colors().border)
                .resolve(theme)
        };
        Border {
            widths: border_widths(s),
            colors: p::Edges {
                top: c(0),
                right: c(1),
                bottom: c(2),
                left: c(3),
            },
            styles: p::Edges {
                top: s.border_style[0].unwrap_or_default(),
                right: s.border_style[1].unwrap_or_default(),
                bottom: s.border_style[2].unwrap_or_default(),
                left: s.border_style[3].unwrap_or_default(),
            },
        }
    })
}

fn corners(s: &Style) -> Corners {
    Corners {
        top_left: corner(s.radius[0]),
        top_right: corner(s.radius[1]),
        bottom_right: corner(s.radius[2]),
        bottom_left: corner(s.radius[3]),
    }
}

fn shadow(sh: &Shadow, theme: &Theme) -> p::Shadow {
    p::Shadow {
        offset: sh.offset,
        blur: sh.blur,
        spread: sh.spread,
        color: sh.color.resolve(theme),
        inset: sh.inset,
    }
}

/// The effects, when the style sets any (`None` otherwise, so a node
/// without effects carries none).
fn effects(s: &Style, theme: &Theme) -> Option<Box<Effects>> {
    let set = s.filter.is_some()
        || s.backdrop_filter.is_some()
        || s.blend.is_some()
        || s.mask.is_some()
        || s.clip_path.is_some();
    set.then(|| {
        Box::new(Effects {
            filter: s
                .filter
                .as_ref()
                .map(|f| filter_ops(f, theme))
                .unwrap_or_default(),
            backdrop: s
                .backdrop_filter
                .as_ref()
                .map(|f| filter_ops(f, theme))
                .unwrap_or_default(),
            blend: s.blend.unwrap_or_default(),
            masks: s.mask.as_ref().map(|m| m.wire(theme)).unwrap_or_default(),
            clip_path: s.clip_path.as_ref().map(|c| clip_shape(&c.0)),
        })
    })
}

/// The transform properties of `s` as one transform: CSS's `translate`
/// (x, y, z), then `rotate` (z, then y, then x), then `scale`, then
/// `transform`, about `transform-origin` (the centre by default).
fn transform_of(s: &Style) -> Transform {
    let mut t = Transform::new();
    if s.translate.iter().any(Option::is_some) || s.translate_z.is_some() {
        // An axis left unset does not move the node.
        let [x, y] = s.translate.map(|axis| axis.unwrap_or(Length::Px(0.0)));
        t = t.translate_3d(x, y, s.translate_z.unwrap_or(0.0));
    }
    if let Some(deg) = s.rotate {
        t = t.rotate_z(deg);
    }
    if let Some(deg) = s.rotate_y {
        t = t.rotate_y(deg);
    }
    if let Some(deg) = s.rotate_x {
        t = t.rotate_x(deg);
    }
    if let Some((x, y)) = s.scale {
        t = t.scale_xy(x, y);
    }
    if let Some(list) = &s.transform {
        t = t.then(list);
    }
    t
}

/// The transform properties of `s` as one element transform, about
/// `transform-origin` (the centre by default) and its z.
fn element_transform(s: &Style) -> ElementTransform {
    let (x, y) = s.transform_origin.unwrap_or((pct(50), pct(50)));
    // `auto` is the default centre.
    transform_of(s).to_wire(
        [x, y].map(|l| match l {
            Length::Auto => p::Len::pct(50.0),
            l => l.to_len(),
        }),
        s.transform_origin_z.unwrap_or(0.0),
    )
}

/// Whether `s` sets a transform property.
fn sets_transform(s: &Style) -> bool {
    s.translate.iter().any(Option::is_some)
        || s.translate_z.is_some()
        || s.rotate.is_some()
        || s.rotate_x.is_some()
        || s.rotate_y.is_some()
        || s.scale.is_some()
        || s.transform.is_some()
        || s.transform_origin.is_some()
        || s.transform_origin_z.is_some()
}

/// The perspective of the node's children: the distance, about the
/// `perspective-origin` (the centre by default).
fn perspective(s: &Style) -> Option<Perspective> {
    s.perspective.map(|distance| Perspective {
        distance,
        origin: s
            .perspective_origin
            .map_or([p::Len::pct(50.0), p::Len::pct(50.0)], |(x, y)| {
                [x.to_len(), y.to_len()]
            }),
    })
}

fn paint(s: &Style, theme: &Theme) -> PaintProps {
    PaintProps {
        transitions: None,
        background: s.background.as_ref().map(|b| fill(b, theme)),
        border: border(s, theme),
        ring: s.ring.map(|(width, offset, c)| Ring {
            width,
            offset,
            color: c.resolve(theme),
        }),
        radius: corners(s),
        shadows: s
            .shadows
            .iter()
            .flatten()
            .map(|sh| shadow(sh, theme))
            .collect(),
        opacity: s.opacity.unwrap_or(1.0),
        transform: element_transform(s),
        perspective: perspective(s),
        transform_style: s.transform_style.unwrap_or_default(),
        backface: s.backface.unwrap_or_default(),
        will_change: s.will_change.unwrap_or_default(),
        isolation: s.isolation.unwrap_or_default(),
        border_image: s.border_image.as_ref().map(|b| Box::new(b.wire(theme))),
        z_index: s.z_index.unwrap_or(0),
        scrollbar: scrollbar(s, theme),
        cursor: s.cursor.unwrap_or_default(),
        clip: s.clip.unwrap_or(false)
            || s.scroll_axes.is_some()
            || s.overflow.is_some_and(|o| o != Overflow::Visible),
        effects: effects(s, theme),
    }
}

/// The paint transitions of `s`: every property but the size, which the
/// layout takes.
fn paint_transitions(
    s: &Style,
) -> Option<std::sync::Arc<[(super::sheet::TransitionProp, Transition)]>> {
    let list: Vec<_> = s
        .transitions
        .iter()
        .filter(|(prop, _)| *prop != super::sheet::TransitionProp::Size)
        .cloned()
        .collect();
    (!list.is_empty()).then(|| list.into())
}

/// What `compile_style` reports about the style itself (see
/// [`CompiledStyle::reports`]).
fn reports(s: &Style, theme: &Theme) -> Vec<UiError> {
    let mut out = Vec::new();
    let mut report = |error: UiError| out.push(error);
    let springs = s
        .transitions
        .iter()
        .map(|(_, t)| t)
        .chain(s.layout_transition.iter());
    for t in springs {
        if let p::Kind::Spring(spring) = &t.kind
            && let Some(problem) = spring.problem()
        {
            report(UiError::msg(format!("spring: {problem}")));
        }
    }
    if let Some(t) = &s.layout_transition
        && (t.repeat != Repeat::Times(1) || t.direction != p::PlayDirection::Normal)
    {
        report(UiError::msg(
            "layout_transition ignores repeat and alternate",
        ));
    }
    if let Some(background) = &s.background {
        for paint in &background.0 {
            for problem in paint.shader_problems() {
                report(problem);
            }
        }
    }
    if let Some(mask) = &s.mask {
        for layer in &mask.0 {
            for problem in layer.paint().shader_problems() {
                report(problem);
            }
        }
    }
    for filter in [&s.filter, &s.backdrop_filter].into_iter().flatten() {
        for problem in filter_problems(filter, theme) {
            report(problem);
        }
    }
    let dropped = transform_of(s).unsupported();
    if dropped > 0 {
        report(UiError::msg(format!(
            "transform: {dropped} function(s) the form cannot keep"
        )));
    }
    out
}

/// The paint a state style `st` changes over the node's style `base`:
/// each group it sets, resolved over the base (a state that sets only a
/// border color keeps the base's widths).
fn paint_delta(base: &Style, st: &Style, theme: &Theme) -> PaintDelta {
    let merged = base.clone().merge(st);
    let full = paint(&merged, theme);
    let sets_border = st.border_width.iter().any(Option::is_some)
        || st.border_color.iter().any(Option::is_some)
        || st.border_style.iter().any(Option::is_some);
    let sets_effects = st.filter.is_some()
        || st.backdrop_filter.is_some()
        || st.blend.is_some()
        || st.mask.is_some()
        || st.clip_path.is_some();
    PaintDelta {
        background: st.background.as_ref().and(full.background),
        border: if sets_border { full.border } else { None },
        ring: st.ring.and(full.ring),
        radius: st.radius.iter().any(Option::is_some).then_some(full.radius),
        opacity: st.opacity,
        shadows: st.shadows.as_ref().map(|_| full.shadows),
        transform: sets_transform(st).then_some(full.transform),
        perspective: (st.perspective.is_some() || st.perspective_origin.is_some())
            .then_some(full.perspective),
        transform_style: st.transform_style.map(|_| full.transform_style),
        backface: st.backface.map(|_| full.backface),
        z_index: st.z_index,
        scrollbar: st
            .scrollbar
            .as_ref()
            .zip(full.scrollbar.as_ref())
            .map(|(style, full)| {
                Box::new(p::ScrollbarDelta {
                    width: style.width.map(|_| full.width),
                    colors: style.colors.map(|_| (full.thumb, full.track)),
                })
            }),
        cursor: st.cursor,
        effects: sets_effects.then_some(full.effects),
        text_color: st.color.map(|c| c.resolve(theme)),
    }
}

fn scrollbar(s: &Style, theme: &Theme) -> Option<Box<p::ScrollbarProps>> {
    let style = s.scrollbar.as_ref()?;
    let mut props = p::ScrollbarProps {
        width: match style.width.unwrap_or_default() {
            ScrollbarWidth::Auto | ScrollbarWidth::Thin => 6.0,
            ScrollbarWidth::None => 0.0,
            ScrollbarWidth::Px(px) => {
                if px.is_finite() {
                    px.clamp(0.0, 64.0)
                } else {
                    6.0
                }
            }
        },
        ..Default::default()
    };
    if let Some((thumb, track)) = style.colors {
        props.thumb = thumb.resolve(theme);
        props.track = track.resolve(theme);
    }
    Some(Box::new(props))
}

impl CompiledStyle {
    /// Whether the style that was compiled sets a base paint callback
    /// (`Style::paint`).
    pub fn has_retained_paint(&self) -> bool {
        self.retained_paint.is_some()
    }

    /// Whether `self` and `other` retain the same paint callback (callbacks
    /// compare by identity, and no callback equals no callback), so a node
    /// that compiled `other` before need not redraw for `self`.
    pub fn same_retained_paint(&self, other: &Self) -> bool {
        self.retained_paint == other.retained_paint
    }

    /// Run the retained base paint on `draw` for a canvas of `size`; nothing
    /// without one. Call it after the application's own drawing of the same
    /// layer, and not while a borrow of the storage that holds this value is
    /// open: the callback is user code (clone this value out first).
    pub fn paint_retained(&self, draw: &mut Draw, size: Size) {
        if let Some(paint) = &self.retained_paint {
            paint.draw(draw, size);
        }
    }
}
