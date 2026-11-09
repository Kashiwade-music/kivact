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
use super::sheet::{
    Direction, FlowFields, MotionFields, Overflow, PaintFields, Position, ScrollbarWidth, Sides,
    SizeFields, SpacingFields, StateFields, Style, TextFields, TransformFields, group_or_default,
};
use super::theme::value::Theme;
use super::transform::Transform;

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
    // xtask-allow(state-fields): passive record - retained paint
    retained_paint: Option<LocalPaint>,
    /// Layout props.
    pub layout: LayoutProps,
    /// Paint props.
    pub paint: PaintProps,
    // xtask-allow(state-fields): passive record - Text props: when the style sets any (only what it sets: the rest is inherited property by property), and always for the root, which carries the theme's text defaults that every node inherits
    /// Text props: when the style sets any (only what it sets: the rest
    /// is inherited property by property), and always for the root, which
    /// carries the theme's text defaults that every node inherits.
    pub text: Option<TextStyle>,
    // xtask-allow(state-fields): passive record - Hover/pressed/focus/disabled overrides, if any
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
    let size = group_or_default!(s.size, SizeFields);
    let spacing = group_or_default!(s.spacing, SpacingFields);
    let flow = group_or_default!(s.flow, FlowFields);
    let motion = group_or_default!(s.motion, MotionFields);
    let zero = p::Dimension::Px(0.0);
    let ov = flow.overflow.unwrap_or(p::Overflow::Visible);
    let (ov_x, ov_y) = match flow.scroll_axes {
        Some(ScrollAxes::Vertical) => (p::Overflow::Hidden, p::Overflow::Scroll),
        Some(ScrollAxes::Horizontal) => (p::Overflow::Scroll, p::Overflow::Hidden),
        Some(ScrollAxes::Both) => (p::Overflow::Scroll, p::Overflow::Scroll),
        None => (ov, ov),
    };
    LayoutProps {
        scroll_snap: flow.scroll_snap.unwrap_or_default(),
        enter_size: motion.enter_size,
        exit_size: motion.exit_size,
        layout_transition: motion.layout_transition.clone(),
        size_transition: motion
            .transitions
            .iter()
            .find(|(p, _)| *p == super::sheet::TransitionProp::Size)
            .map(|(_, t)| t.clone()),
        direction: flow.direction.unwrap_or(Direction::Column),
        align_items: flow.align,
        align_self: flow.align_self,
        justify: flow.justify,
        align_content: flow.align_content,
        wrap: flow.wrap.unwrap_or(false),
        row_gap: dim(spacing.row_gap, zero),
        column_gap: dim(spacing.column_gap, zero),
        padding: edges(&spacing.padding, zero),
        border: border_widths(s),
        margin: edges(&spacing.margin, zero),
        width: dim(size.width, p::Dimension::Auto),
        height: dim(size.height, p::Dimension::Auto),
        min_width: dim(size.min_width, p::Dimension::Auto),
        min_height: dim(size.min_height, p::Dimension::Auto),
        max_width: dim(size.max_width, p::Dimension::Auto),
        max_height: dim(size.max_height, p::Dimension::Auto),
        grow: size.grow.unwrap_or(0.0),
        shrink: size.shrink.unwrap_or(1.0),
        basis: dim(size.basis, p::Dimension::Auto),
        aspect_ratio: size.aspect_ratio,
        position: flow.position.unwrap_or(Position::Relative),
        inset: edges(&spacing.inset, p::Dimension::Auto),
        overflow_x: ov_x,
        overflow_y: ov_y,
        hidden: false,
        visibility: flow.visibility.unwrap_or_default(),
        display: flow.display.unwrap_or_default(),
        grid: super::grid::resolve(flow.grid.as_deref(), flow.display == Some(p::Display::Grid)),
    }
}

/// The number of content sizes (`Length::MinContent`, `Length::MaxContent`,
/// `Length::FitContent`) among the lengths of `s` that are not sizes (width,
/// height, the flex basis): minimums, maximums, padding, margins, insets
/// and gaps, where they are `auto`.
fn misplaced_content(s: &Style) -> u32 {
    let size = group_or_default!(s.size, SizeFields);
    let spacing = group_or_default!(s.spacing, SpacingFields);
    let sides = |x: &Sides| [x.top, x.right, x.bottom, x.left];
    [
        spacing.row_gap,
        spacing.column_gap,
        size.min_width,
        size.min_height,
        size.max_width,
        size.max_height,
    ]
    .into_iter()
    .chain(sides(&spacing.padding))
    .chain(sides(&spacing.margin))
    .chain(sides(&spacing.inset))
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
    let text = group_or_default!(s.text, TextFields);
    let mut t = TextStyle {
        font_size: text.font_size,
        weight: text.weight,
        color: text.color.map(|c| c.resolve(theme)),
        line_height: text.line_height,
        wrap: text.text_wrap.map(|wrap| {
            if wrap {
                p::TextWrap::Wrap
            } else {
                p::TextWrap::NoWrap
            }
        }),
        align: text.text_align,
        family: text.font_family.clone(),
        italic: text.italic,
        ..TextStyle::default()
    };
    let dropped = match &text.text {
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
        let states = s.states_mut();
        states.hover = None;
        states.pressed = None;
    }
    let has_focus = s
        .states
        .as_deref()
        .is_some_and(|states| states.focus.is_some() || states.focus_visible.is_some());
    if !has_focus {
        let ring = Style::default().ring_with_offset(
            theme.focus_ring(),
            theme.focus_ring_offset(),
            ColorToken::Focus,
        );
        s.states_mut().focus_visible = Some(Box::new(ring));
    }
    let disabled = s.states_mut().disabled.get_or_insert_with(Default::default);
    if !disabled
        .paint
        .as_deref()
        .is_some_and(|paint| paint.opacity.is_some())
    {
        disabled.paint_mut().opacity = Some(theme.disabled_opacity());
    }
}

/// Merge the state styles a tool forces into the base, in the engine's
/// layer order, so they no longer apply as deltas.
fn merge_forced(mut s: Style, forced: ForcedStates) -> Style {
    if forced.contains(ForcedStates::FOCUS)
        && let Some(v) = s.states_mut().focus.take()
    {
        s = s.merge(&v);
    }
    if forced.contains(ForcedStates::FOCUS_VISIBLE)
        && let Some(v) = s.states_mut().focus_visible.take()
    {
        s = s.merge(&v);
    }
    if forced.contains(ForcedStates::HOVER)
        && let Some(v) = s.states_mut().hover.take()
    {
        s = s.merge(&v);
    }
    if forced.contains(ForcedStates::PRESSED)
        && let Some(v) = s.states_mut().pressed.take()
    {
        s = s.merge(&v);
    }
    s
}

/// The host-evaluated state deltas of `s`, if it sets any state.
fn state_styles(s: &Style, theme: &Theme) -> Option<StateStyles> {
    let states = group_or_default!(s.states, StateFields);
    (states.hover.is_some()
        || states.group_hover.is_some()
        || states.group_focus.is_some()
        || states.group
        || states.pressed.is_some()
        || states.focus.is_some()
        || states.focus_visible.is_some()
        || states.enter.is_some()
        || states.exit.is_some())
    .then(|| StateStyles {
        hover: states.hover.as_ref().map(|h| paint_delta(s, h, theme)),
        pressed: states.pressed.as_ref().map(|h| paint_delta(s, h, theme)),
        focus: states.focus.as_ref().map(|h| paint_delta(s, h, theme)),
        focus_visible: states.focus_visible.as_ref().map(|h| {
            // Over the focus state, as the host applies it.
            let focused = match &states.focus {
                Some(focus) => s.clone().merge(focus),
                None => s.clone(),
            };
            paint_delta(&focused, h, theme)
        }),
        group_hover: states
            .group_hover
            .as_ref()
            .map(|h| paint_delta(s, h, theme)),
        group_focus: states
            .group_focus
            .as_ref()
            .map(|h| paint_delta(s, h, theme)),
        group: states.group,
        enter: states.enter.as_ref().map(|h| paint_delta(s, h, theme)),
        exit: states.exit.as_ref().map(|h| paint_delta(s, h, theme)),
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
    let retained_paint = s
        .paint
        .as_deref()
        .and_then(|paint| paint.local_paint.clone());
    let mut s = s.clone();
    let unsupported_in_states = unsupported_in_states(&s);
    strip_state_paint(&mut s);
    if state.focusable {
        interactive_defaults(&mut s, theme, state.disabled);
    }
    if state.selected
        && let Some(v) = s.states_mut().selected.take()
    {
        s = s.merge(&v);
    }
    if state.disabled
        && let Some(v) = s.states_mut().disabled.take()
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
    if let Some(paint) = style.paint.as_deref_mut() {
        paint.local_paint = None;
    }
    let Some(states) = style.states.as_deref_mut() else {
        return;
    };
    [
        &mut states.hover,
        &mut states.group_hover,
        &mut states.group_focus,
        &mut states.pressed,
        &mut states.focus,
        &mut states.focus_visible,
        &mut states.disabled,
        &mut states.selected,
        &mut states.enter,
        &mut states.exit,
    ]
    .into_iter()
    .flatten()
    .for_each(|state| strip_state_paint(state));
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
/// name of the setter that sets them, and one classifier per group. Each
/// group of `Style` is read through `group_or_default`, so a field whose
/// group is unset reads its default.
macro_rules! classify_properties {
    (
        $(
            fn $group:ident {
                @groups $( $gname:ident = $gty:ident, )*
                @props $($value:expr => $set:expr, $name:literal;)*
            }
        )*
        @state fn $state_group:ident {
            @groups $( $sgname:ident = $sgty:ident, )*
            @props $($state_value:expr => $state_set:expr, $state_name:literal;)*
        }
    ) => {
        fn set_properties(s: &Style) -> Vec<&'static str> {
            let Style {
                size: _,
                spacing: _,
                flow: _,
                paint: _,
                transform: _,
                text: _,
                states: _,
                motion: _,
                duplicates: _,
            } = s;
            let mut out = Vec::new();
            $(out.extend($group(s));)*
            out.extend($state_group(s));
            out
        }

        /// Every property of the non-state groups that `s` sets, with the
        /// `Debug` of its value.
        fn describe_properties(s: &Style) -> Vec<(&'static str, String)> {
            let mut out = Vec::new();
            $({
                $(let $gname = group_or_default!(s.$gname, $gty);)*
                $(
                    if $set {
                        out.push(($name, format!("{:?}", $value)));
                    }
                )*
            })*
            out
        }

        $(
            fn $group(s: &Style) -> Vec<&'static str> {
                $(let $gname = group_or_default!(s.$gname, $gty);)*
                [$(($set, $name),)*]
                    .into_iter()
                    .filter_map(|(set, name)| set.then_some(name))
                    .collect()
            }
        )*
        fn $state_group(s: &Style) -> Vec<&'static str> {
            $(let $sgname = group_or_default!(s.$sgname, $sgty);)*
            [$(($state_set, $state_name),)*]
                .into_iter()
                .filter_map(|(set, name)| set.then_some(name))
                .collect()
        }
    };
}

classify_properties! {
    fn layout_properties {
        @groups size = SizeFields, spacing = SpacingFields, flow = FlowFields,
        paint = PaintFields, motion = MotionFields,
        @props motion.enter_size => motion.enter_size.is_some(), "enter_size";
        motion.exit_size => motion.exit_size.is_some(), "exit_size";
        paint.local_paint => paint.local_paint.is_some(), "paint";
        spacing.padding => spacing.padding != Sides::default(), "padding";
        spacing.margin => spacing.margin != Sides::default(), "margin";
        spacing.inset => spacing.inset != Sides::default(), "inset";
        spacing.row_gap => spacing.row_gap.is_some(), "row_gap";
        spacing.column_gap => spacing.column_gap.is_some(), "column_gap";
        size.width => size.width.is_some(), "width";
        size.height => size.height.is_some(), "height";
        size.min_width => size.min_width.is_some(), "min_width";
        size.min_height => size.min_height.is_some(), "min_height";
        size.max_width => size.max_width.is_some(), "max_width";
        size.max_height => size.max_height.is_some(), "max_height";
        size.grow => size.grow.is_some(), "grow";
        size.shrink => size.shrink.is_some(), "shrink";
        size.basis => size.basis.is_some(), "basis";
        size.aspect_ratio => size.aspect_ratio.is_some(), "aspect_ratio";
        flow.direction => flow.direction.is_some(), "direction";
        flow.align => flow.align.is_some(), "align_items";
        flow.align_self => flow.align_self.is_some(), "align_self";
        flow.justify => flow.justify.is_some(), "justify_content";
        flow.align_content => flow.align_content.is_some(), "align_content";
        flow.display => flow.display.is_some(), "display";
        flow.visibility => flow.visibility.is_some(), "visibility";
        flow.grid => flow.grid.is_some(), "grid";
        flow.wrap => flow.wrap.is_some(), "wrap";
    }
    fn position_properties {
        @groups flow = FlowFields, paint = PaintFields,
        @props flow.position => flow.position.is_some(), "position";
        flow.overflow => flow.overflow.is_some(), "overflow";
        flow.scroll_snap => flow.scroll_snap.is_some(), "scroll_snap";
        flow.scroll_axes => flow.scroll_axes.is_some(), "with_scroll_axes";
        paint.scrollbar => paint.scrollbar.is_some(), "scrollbar";
        paint.cursor => paint.cursor.is_some(), "cursor";
        flow.z_index => flow.z_index.is_some(), "z_index";
        paint.clip => paint.clip.is_some(), "clip";
    }
    fn paint_properties {
        @groups paint = PaintFields,
        @props paint.background => paint.background.is_some(), "background";
        paint.border_width => paint.border_width.iter().any(Option::is_some), "border_width";
        paint.border_color => paint.border_color.iter().any(Option::is_some), "border_color";
        paint.border_style => paint.border_style.iter().any(Option::is_some), "border_style";
        paint.ring => paint.ring.is_some(), "ring";
        paint.radius => paint.radius.iter().any(Option::is_some), "radius";
        paint.shadows => paint.shadows.is_some(), "shadow";
        paint.opacity => paint.opacity.is_some(), "opacity";
        paint.border_image => paint.border_image.is_some(), "border_image";
        paint.isolation => paint.isolation.is_some(), "isolation";
        paint.filter => paint.filter.is_some(), "filter";
        paint.backdrop_filter => paint.backdrop_filter.is_some(), "backdrop_filter";
        paint.blend => paint.blend.is_some(), "blend";
        paint.mask => paint.mask.is_some(), "mask";
        paint.clip_path => paint.clip_path.is_some(), "clip_path";
    }
    fn transform_properties {
        @groups transform = TransformFields,
        @props transform.translate => transform.translate.iter().any(Option::is_some), "translate";
        transform.rotate => transform.rotate.is_some(), "rotate";
        transform.scale => transform.scale.is_some(), "scale";
        transform.transform => transform.transform.is_some(), "transform";
        transform.transform_origin => transform.transform_origin.is_some(), "transform_origin";
        transform.translate_z => transform.translate_z.is_some(), "translate_z";
        transform.rotate_x => transform.rotate_x.is_some(), "rotate_x";
        transform.rotate_y => transform.rotate_y.is_some(), "rotate_y";
        transform.transform_origin_z => transform.transform_origin_z.is_some(), "transform_origin_z";
        transform.perspective => transform.perspective.is_some(), "perspective";
        transform.perspective_origin => transform.perspective_origin.is_some(), "perspective_origin";
        transform.transform_style => transform.transform_style.is_some(), "transform_style";
        transform.backface => transform.backface.is_some(), "backface_visibility";
        transform.will_change => transform.will_change.is_some(), "will_change";
    }
    fn text_properties {
        @groups text = TextFields,
        @props text.font_size => text.font_size.is_some(), "font_size";
        text.weight => text.weight.is_some(), "weight";
        text.color => text.color.is_some(), "color";
        text.line_height => text.line_height.is_some(), "line_height";
        text.text_wrap => text.text_wrap.is_some(), "text_wrap";
        text.text_align => text.text_align.is_some(), "text_align";
        text.font_family => text.font_family.is_some(), "font_family";
        text.italic => text.italic.is_some(), "italic";
        text.text => text.text.is_some(), "text";
    }
    @state fn state_properties {
        @groups states = StateFields, motion = MotionFields,
        @props states.hover => states.hover.is_some(), "hover";
        states.group => states.group, "group";
        states.group_hover => states.group_hover.is_some(), "group_hover";
        states.group_focus => states.group_focus.is_some(), "group_focus";
        states.pressed => states.pressed.is_some(), "pressed";
        states.focus => states.focus.is_some(), "focus";
        states.focus_visible => states.focus_visible.is_some(), "focus_visible";
        states.disabled => states.disabled.is_some(), "disabled";
        states.selected => states.selected.is_some(), "selected";
        states.enter => states.enter.is_some(), "enter";
        states.exit => states.exit.is_some(), "exit";
        motion.transitions => !motion.transitions.is_empty(), "transition";
        motion.layout_transition => motion.layout_transition.is_some(), "layout_transition";
    }
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
    let spacing = group_or_default!(s.spacing, SpacingFields);
    let paint = group_or_default!(s.paint, PaintFields);
    let transform = group_or_default!(s.transform, TransformFields);
    let mut out = Vec::new();
    for (name, value) in describe_properties(s) {
        let value = match name {
            "padding" => sides_value(&spacing.padding),
            "margin" => sides_value(&spacing.margin),
            "inset" => sides_value(&spacing.inset),
            "border_width" => options_value(&paint.border_width),
            "border_color" => options_value(&paint.border_color),
            "border_style" => options_value(&paint.border_style),
            "radius" => options_value(&paint.radius),
            "translate" => options_value(&transform.translate),
            _ => value,
        };
        out.extend(split_sides(name, value));
    }
    out
}

/// The properties of each state style `s` sets: `"hover"`, `"pressed"`,
/// `"focus"`, `"focus_visible"`, `"disabled"` and `"selected"`.
pub fn describe_state_styles(s: &Style) -> Vec<(&'static str, Vec<(&'static str, String)>)> {
    let states = group_or_default!(s.states, StateFields);
    [
        ("hover", &states.hover),
        ("pressed", &states.pressed),
        ("focus", &states.focus),
        ("focus_visible", &states.focus_visible),
        ("disabled", &states.disabled),
        ("selected", &states.selected),
    ]
    .into_iter()
    .filter_map(|(name, state)| state.as_deref().map(|state| (name, describe_style(state))))
    .collect()
}

/// Push `prefix` + `state.property` for every property the delta states
/// (hover, pressed, focus, ..) of `s` set that a delta cannot change.
fn delta_misuse(prefix: &str, s: &Style, out: &mut Vec<String>) {
    let states = group_or_default!(s.states, StateFields);
    for (name, state) in [
        ("hover", &states.hover),
        ("group_hover", &states.group_hover),
        ("group_focus", &states.group_focus),
        ("pressed", &states.pressed),
        ("focus", &states.focus),
        ("focus_visible", &states.focus_visible),
        ("enter", &states.enter),
        ("exit", &states.exit),
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
    let states = group_or_default!(s.states, StateFields);
    let mut out = Vec::new();
    delta_misuse("", s, &mut out);
    if let Some(selected) = &states.selected {
        if selected
            .paint
            .as_deref()
            .is_some_and(|paint| paint.local_paint.is_some())
        {
            out.push("selected.paint".to_string());
        }
        if selected
            .states
            .as_deref()
            .is_some_and(|states| states.selected.is_some())
        {
            out.push("selected.selected".to_string());
        }
        delta_misuse("selected.", selected, &mut out);
    }
    if let Some(disabled) = &states.disabled {
        if disabled
            .paint
            .as_deref()
            .is_some_and(|paint| paint.local_paint.is_some())
        {
            out.push("disabled.paint".to_string());
        }
        if disabled
            .states
            .as_deref()
            .is_some_and(|states| states.disabled.is_some())
        {
            out.push("disabled.disabled".to_string());
        }
        if disabled
            .states
            .as_deref()
            .is_some_and(|states| states.selected.is_some())
        {
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
    let paint = group_or_default!(s.paint, PaintFields);
    let w = |i: usize| paint.border_width[i].unwrap_or(0.0);
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
    let paint = group_or_default!(s.paint, PaintFields);
    let set = paint.border_width.iter().any(Option::is_some)
        || paint.border_color.iter().any(Option::is_some)
        || paint.border_style.iter().any(Option::is_some);
    set.then(|| {
        let c = |i: usize| {
            paint.border_color[i]
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
                top: paint.border_style[0].unwrap_or_default(),
                right: paint.border_style[1].unwrap_or_default(),
                bottom: paint.border_style[2].unwrap_or_default(),
                left: paint.border_style[3].unwrap_or_default(),
            },
        }
    })
}

fn corners(s: &Style) -> Corners {
    let paint = group_or_default!(s.paint, PaintFields);
    Corners {
        top_left: corner(paint.radius[0]),
        top_right: corner(paint.radius[1]),
        bottom_right: corner(paint.radius[2]),
        bottom_left: corner(paint.radius[3]),
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
    let paint = group_or_default!(s.paint, PaintFields);
    let set = paint.filter.is_some()
        || paint.backdrop_filter.is_some()
        || paint.blend.is_some()
        || paint.mask.is_some()
        || paint.clip_path.is_some();
    set.then(|| {
        Box::new(Effects {
            filter: paint
                .filter
                .as_ref()
                .map(|f| filter_ops(f, theme))
                .unwrap_or_default(),
            backdrop: paint
                .backdrop_filter
                .as_ref()
                .map(|f| filter_ops(f, theme))
                .unwrap_or_default(),
            blend: paint.blend.unwrap_or_default(),
            masks: paint
                .mask
                .as_ref()
                .map(|m| m.wire(theme))
                .unwrap_or_default(),
            clip_path: paint.clip_path.as_ref().map(|c| clip_shape(&c.0)),
        })
    })
}

/// The transform properties of `s` as one transform: CSS's `translate`
/// (x, y, z), then `rotate` (z, then y, then x), then `scale`, then
/// `transform`, about `transform-origin` (the centre by default).
fn transform_of(s: &Style) -> Transform {
    let transform = group_or_default!(s.transform, TransformFields);
    let mut t = Transform::new();
    if transform.translate.iter().any(Option::is_some) || transform.translate_z.is_some() {
        // An axis left unset does not move the node.
        let [x, y] = transform
            .translate
            .map(|axis| axis.unwrap_or(Length::Px(0.0)));
        t = t.translate_3d(x, y, transform.translate_z.unwrap_or(0.0));
    }
    if let Some(deg) = transform.rotate {
        t = t.rotate_z(deg);
    }
    if let Some(deg) = transform.rotate_y {
        t = t.rotate_y(deg);
    }
    if let Some(deg) = transform.rotate_x {
        t = t.rotate_x(deg);
    }
    if let Some((x, y)) = transform.scale {
        t = t.scale_xy(x, y);
    }
    if let Some(list) = &transform.transform {
        t = t.then(list);
    }
    t
}

/// The transform properties of `s` as one element transform, about
/// `transform-origin` (the centre by default) and its z.
fn element_transform(s: &Style) -> ElementTransform {
    let transform = group_or_default!(s.transform, TransformFields);
    let (x, y) = transform.transform_origin.unwrap_or((pct(50), pct(50)));
    // `auto` is the default centre.
    transform_of(s).to_wire(
        [x, y].map(|l| match l {
            Length::Auto => p::Len::pct(50.0),
            l => l.to_len(),
        }),
        transform.transform_origin_z.unwrap_or(0.0),
    )
}

/// Whether `s` sets a transform property.
fn sets_transform(s: &Style) -> bool {
    let transform = group_or_default!(s.transform, TransformFields);
    transform.translate.iter().any(Option::is_some)
        || transform.translate_z.is_some()
        || transform.rotate.is_some()
        || transform.rotate_x.is_some()
        || transform.rotate_y.is_some()
        || transform.scale.is_some()
        || transform.transform.is_some()
        || transform.transform_origin.is_some()
        || transform.transform_origin_z.is_some()
}

/// The perspective of the node's children: the distance, about the
/// `perspective-origin` (the centre by default).
fn perspective(s: &Style) -> Option<Perspective> {
    let transform = group_or_default!(s.transform, TransformFields);
    transform.perspective.map(|distance| Perspective {
        distance,
        origin: transform
            .perspective_origin
            .map_or([p::Len::pct(50.0), p::Len::pct(50.0)], |(x, y)| {
                [x.to_len(), y.to_len()]
            }),
    })
}

fn paint(s: &Style, theme: &Theme) -> PaintProps {
    let paint = group_or_default!(s.paint, PaintFields);
    let transform = group_or_default!(s.transform, TransformFields);
    let flow = group_or_default!(s.flow, FlowFields);
    PaintProps {
        transitions: None,
        background: paint.background.as_ref().map(|b| fill(b, theme)),
        border: border(s, theme),
        ring: paint.ring.map(|(width, offset, c)| Ring {
            width,
            offset,
            color: c.resolve(theme),
        }),
        radius: corners(s),
        shadows: paint
            .shadows
            .iter()
            .flatten()
            .map(|sh| shadow(sh, theme))
            .collect(),
        opacity: paint.opacity.unwrap_or(1.0),
        transform: element_transform(s),
        perspective: perspective(s),
        transform_style: transform.transform_style.unwrap_or_default(),
        backface: transform.backface.unwrap_or_default(),
        will_change: transform.will_change.unwrap_or_default(),
        isolation: paint.isolation.unwrap_or_default(),
        border_image: paint.border_image.as_ref().map(|b| Box::new(b.wire(theme))),
        z_index: flow.z_index.unwrap_or(0),
        scrollbar: scrollbar(s, theme),
        cursor: paint.cursor.unwrap_or_default(),
        clip: paint.clip.unwrap_or(false)
            || flow.scroll_axes.is_some()
            || flow.overflow.is_some_and(|o| o != Overflow::Visible),
        effects: effects(s, theme),
    }
}

/// The paint transitions of `s`: every property but the size, which the
/// layout takes.
fn paint_transitions(
    s: &Style,
) -> Option<std::sync::Arc<[(super::sheet::TransitionProp, Transition)]>> {
    let motion = group_or_default!(s.motion, MotionFields);
    let list: Vec<_> = motion
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
    let paint = group_or_default!(s.paint, PaintFields);
    let motion = group_or_default!(s.motion, MotionFields);
    let mut out = Vec::new();
    let mut report = |error: UiError| out.push(error);
    let springs = motion
        .transitions
        .iter()
        .map(|(_, t)| t)
        .chain(motion.layout_transition.iter());
    for t in springs {
        if let p::Kind::Spring(spring) = &t.kind
            && let Some(problem) = spring.problem()
        {
            report(UiError::msg(format!("spring: {problem}")));
        }
    }
    if let Some(t) = &motion.layout_transition
        && (t.repeat != Repeat::Times(1) || t.direction != p::PlayDirection::Normal)
    {
        report(UiError::msg(
            "layout_transition ignores repeat and alternate",
        ));
    }
    if let Some(background) = &paint.background {
        for paint in &background.0 {
            for problem in paint.shader_problems() {
                report(problem);
            }
        }
    }
    if let Some(mask) = &paint.mask {
        for layer in &mask.0 {
            for problem in layer.paint().shader_problems() {
                report(problem);
            }
        }
    }
    for filter in [&paint.filter, &paint.backdrop_filter]
        .into_iter()
        .flatten()
    {
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
    let paint = group_or_default!(st.paint, PaintFields);
    let transform = group_or_default!(st.transform, TransformFields);
    let flow = group_or_default!(st.flow, FlowFields);
    let text = group_or_default!(st.text, TextFields);
    let merged = base.clone().merge(st);
    let full = crate::resolve::paint(&merged, theme);
    let sets_border = paint.border_width.iter().any(Option::is_some)
        || paint.border_color.iter().any(Option::is_some)
        || paint.border_style.iter().any(Option::is_some);
    let sets_effects = paint.filter.is_some()
        || paint.backdrop_filter.is_some()
        || paint.blend.is_some()
        || paint.mask.is_some()
        || paint.clip_path.is_some();
    PaintDelta {
        background: paint.background.as_ref().and(full.background),
        border: if sets_border { full.border } else { None },
        ring: paint.ring.and(full.ring),
        radius: paint
            .radius
            .iter()
            .any(Option::is_some)
            .then_some(full.radius),
        opacity: paint.opacity,
        shadows: paint.shadows.as_ref().map(|_| full.shadows),
        transform: sets_transform(st).then_some(full.transform),
        perspective: (transform.perspective.is_some() || transform.perspective_origin.is_some())
            .then_some(full.perspective),
        transform_style: transform.transform_style.map(|_| full.transform_style),
        backface: transform.backface.map(|_| full.backface),
        z_index: flow.z_index,
        scrollbar: paint
            .scrollbar
            .as_ref()
            .zip(full.scrollbar.as_ref())
            .map(|(style, full)| {
                Box::new(p::ScrollbarDelta {
                    width: style.width.map(|_| full.width),
                    colors: style.colors.map(|_| (full.thumb, full.track)),
                })
            }),
        cursor: paint.cursor,
        effects: sets_effects.then_some(full.effects),
        text_color: text.color.map(|c| c.resolve(theme)),
    }
}

fn scrollbar(s: &Style, theme: &Theme) -> Option<Box<p::ScrollbarProps>> {
    let paint = group_or_default!(s.paint, PaintFields);
    let style = paint.scrollbar.as_ref()?;
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
