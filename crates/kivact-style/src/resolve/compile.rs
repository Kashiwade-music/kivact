//! Role: Compile a `Style` against a theme into the engine's node props.

use kivact_runtime::UiError;
use kivact_types as p;
use kivact_types::{
    Edges, LayoutProps, PaintProps, ScrollAxes, StateStyles, StyleState, TextStyle,
};

use super::describe::unsupported_in_states;
use super::paint::{border_widths, paint, paint_delta, paint_transitions, reports};
use crate::draw::{Draw, LocalPaint, Size};
use crate::sheet::{
    Direction, FlowFields, MotionFields, Position, Sides, SizeFields, SpacingFields, StateFields,
    Style, TextFields, group_or_default,
};
use crate::theme::value::Theme;
use kivact_types::Length;

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

/// `l` with its theme variables and `em` resolved (`em_px` is the font
/// size `em` is of); an `Invalid` sum counts in `misplaced` and is `Auto`.
pub(crate) fn resolved(l: Length, theme: &Theme, em_px: f64, misplaced: &mut u32) -> Length {
    match l {
        Length::Var(v) => Length::Px(theme.get(v)),
        Length::Em(e) => Length::Px(e * em_px),
        Length::Calc {
            percent,
            px,
            em,
            var,
        } => {
            let px = px + em * em_px + var.map_or(0.0, |(v, k)| k * theme.get(v));
            if percent == 0.0 {
                Length::Px(px)
            } else {
                Length::Calc {
                    percent,
                    px,
                    em: 0.0,
                    var: None,
                }
            }
        }
        Length::Invalid => {
            *misplaced += 1;
            Length::Auto
        }
        other => other,
    }
}

/// What a compile resolves the lengths of a style with: the theme, the
/// font size `em` is of (the style's font size, else the theme's base size)
/// and the count of sums the host cannot take.
pub(super) struct Resolve<'a> {
    theme: &'a Theme,
    em_px: f64,
    misplaced: &'a mut u32,
}

impl<'a> Resolve<'a> {
    pub(super) fn new(s: &Style, theme: &'a Theme, misplaced: &'a mut u32) -> Self {
        let font_size = group_or_default!(s.text, TextFields).font_size;
        Resolve {
            theme,
            em_px: font_size.unwrap_or_else(|| theme.get(&crate::font::SIZE_VAR)),
            misplaced,
        }
    }

    /// `l` with its theme variables and `em` resolved.
    pub(super) fn len(&mut self, l: Length) -> Length {
        resolved(l, self.theme, self.em_px, self.misplaced)
    }
}

fn dim(l: Option<Length>, default: p::Length, cx: &mut Resolve<'_>) -> p::Length {
    l.map_or(default, |l| cx.len(l))
}

fn edges(s: &Sides, default: p::Length, cx: &mut Resolve<'_>) -> Edges<p::Length> {
    Edges {
        top: dim(s.top, default, cx),
        right: dim(s.right, default, cx),
        bottom: dim(s.bottom, default, cx),
        left: dim(s.left, default, cx),
    }
}

fn layout(s: &Style, cx: &mut Resolve<'_>) -> LayoutProps {
    let size = group_or_default!(s.size, SizeFields);
    let spacing = group_or_default!(s.spacing, SpacingFields);
    let flow = group_or_default!(s.flow, FlowFields);
    let motion = group_or_default!(s.motion, MotionFields);
    let zero = p::Length::Px(0.0);
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
            .find(|(p, _)| *p == crate::sheet::TransitionProp::Size)
            .map(|(_, t)| t.clone()),
        direction: flow.direction.unwrap_or(Direction::Column),
        align_items: flow.align,
        align_self: flow.align_self,
        justify: flow.justify,
        align_content: flow.align_content,
        wrap: flow.wrap.unwrap_or(false),
        row_gap: dim(spacing.row_gap, zero, cx),
        column_gap: dim(spacing.column_gap, zero, cx),
        padding: edges(&spacing.padding, zero, cx),
        border: border_widths(s),
        margin: edges(&spacing.margin, zero, cx),
        width: dim(size.width, p::Length::Auto, cx),
        height: dim(size.height, p::Length::Auto, cx),
        min_width: dim(size.min_width, p::Length::Auto, cx),
        min_height: dim(size.min_height, p::Length::Auto, cx),
        max_width: dim(size.max_width, p::Length::Auto, cx),
        max_height: dim(size.max_height, p::Length::Auto, cx),
        grow: size.grow.unwrap_or(0.0),
        shrink: size.shrink.unwrap_or(1.0),
        basis: dim(size.basis, p::Length::Auto, cx),
        aspect_ratio: size.aspect_ratio,
        position: flow.position.unwrap_or(Position::Relative),
        inset: edges(&spacing.inset, p::Length::Auto, cx),
        overflow_x: ov_x,
        overflow_y: ov_y,
        visibility: flow.visibility.unwrap_or_default(),
        display: flow.display.unwrap_or_default(),
        grid: crate::grid::resolve(flow.grid.as_deref(), flow.display == Some(p::Display::Grid)),
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
        Some(x) => crate::text::resolve_extras(&mut t, x, theme),
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
    /// The node is a focus stop (focusable, or a native editor): it gets
    /// the theme's focus ring and disabled opacity unless its style sets
    /// its own.
    pub focusable: bool,
    /// Apply the style's `disabled` state style, and take no hover or pressed
    /// look.
    pub disabled: bool,
    /// Apply the style's `selected` state style.
    pub selected: bool,
    /// The node is the root of the App: its text props always carry the
    /// theme's text defaults, which every node inherits.
    pub root: bool,
    /// States a tool forces on the node; empty for an App.
    pub forced: ForcedStates,
}

/// The theme's defaults under a focusable node's style: a ring of
/// [`Theme::focus_ring`] in `kivact::style::color::FOCUS` while the focus is visible,
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
            crate::color::FOCUS,
        );
        s.states_mut().focus_visible = Some(Box::new(ring));
    }
    let disabled = s.states_mut().disabled.get_or_insert_with(Default::default);
    if !disabled
        .paint
        .as_deref()
        .is_some_and(|paint| paint.opacity.is_some())
    {
        disabled.paint_mut().opacity = Some(theme.get(&crate::control::DISABLED_OPACITY_VAR));
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
    .then(|| {
        let mut out = StateStyles {
            group: states.group,
            ..StateStyles::default()
        };
        let delta = |h: &Style| Some(Box::new(paint_delta(s, h, theme)));
        out.set_delta(StyleState::Hover, states.hover.as_deref().and_then(delta));
        out.set_delta(
            StyleState::Active,
            states.pressed.as_deref().and_then(delta),
        );
        out.set_delta(StyleState::Focus, states.focus.as_deref().and_then(delta));
        out.set_delta(
            StyleState::GroupHover,
            states.group_hover.as_deref().and_then(delta),
        );
        out.set_delta(
            StyleState::GroupFocus,
            states.group_focus.as_deref().and_then(delta),
        );
        out.set_delta(StyleState::Enter, states.enter.as_deref().and_then(delta));
        out.set_delta(StyleState::Exit, states.exit.as_deref().and_then(delta));
        // Over the focus state, as the host applies it.
        out.set_delta(
            StyleState::FocusVisible,
            states.focus_visible.as_deref().map(|h| {
                let focused = match &states.focus {
                    Some(focus) => s.clone().merge(focus),
                    None => s.clone(),
                };
                Box::new(paint_delta(&focused, h, theme))
            }),
        );
        out
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
            t.font_size = t.font_size.or(Some(theme.get(&crate::font::SIZE_VAR)));
            t.weight = t.weight.or(Some(theme.get(&crate::font::WEIGHT_VAR)));
            t.color = t
                .color
                .or(Some(theme.get(&crate::color::TEXT_VAR).resolve(theme)));
            t.family = t.family.or_else(|| {
                theme
                    .get(&crate::font::FAMILY_VAR)
                    .as_str()
                    .map(str::to_owned)
            });
            t.line_height = t
                .line_height
                .or(Some(theme.get(&crate::font::LINE_HEIGHT_VAR)));
        }
        t
    });
    let mut misplaced = 0;
    let mut cx = Resolve::new(&s, theme, &mut misplaced);
    let layout = layout(&s, &mut cx);
    let mut paint = paint(&s, theme, &mut cx);
    paint.transitions = paint_transitions(&s);
    let reports = reports(&s, theme);
    CompiledStyle {
        retained_paint,
        layout,
        paint,
        text,
        states,
        duplicates: s.duplicates,
        misplaced_content: misplaced_content(&s) + misplaced,
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

#[cfg(test)]
mod tests {
    //! Role: Tests of style compilation.
    use super::*;
    use crate::sheet::style;
    use crate::{em, font, space};

    fn width_of(s: &Style, theme: &Theme) -> p::Length {
        compile_style(s, theme, Interactive::default()).layout.width
    }

    #[test]
    fn a_sum_with_a_variable_resolves_in_each_theme() {
        let s = style().width(space::MD + 2);
        assert_eq!(width_of(&s, &Theme::light()), p::Length::Px(10.0));
        let themed = Theme::light().set(&space::MD_VAR, 20);
        assert_eq!(width_of(&s, &themed), p::Length::Px(22.0));
    }

    #[test]
    fn em_is_of_the_font_size_the_style_sets() {
        let theme = Theme::light();
        assert_eq!(
            width_of(&style().font_size(10.0).width(em(2)), &theme),
            p::Length::Px(20.0)
        );
        assert_eq!(width_of(&style().width(em(2)), &theme), p::Length::Px(28.0));
    }

    #[test]
    fn an_invalid_sum_is_counted() {
        let compiled = compile_style(
            &style().width(Length::Auto + 2),
            &Theme::light(),
            Interactive::default(),
        );
        assert_eq!(compiled.misplaced_content, 1);
        assert_eq!(compiled.layout.width, p::Length::Auto);
    }

    #[test]
    fn a_variable_font_size_is_the_base_of_em() {
        let theme = Theme::light().set(&font::SIZE_VAR, 20);
        assert_eq!(width_of(&style().width(em(1)), &theme), p::Length::Px(20.0));
    }
}
