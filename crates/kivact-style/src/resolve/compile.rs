//! Role: Compile a `Style` against a theme into the engine's node props.

use kivact_runtime::UiError;
use kivact_types as p;
use kivact_types::{
    Edges, LayoutProps, PaintProps, ScrollAxes, StateSet, StateStyles, StyleState, TextStyle,
};

use super::describe::nested_in_states;
use super::paint::{border_widths, paint, paint_delta, paint_transitions, reports};
use crate::draw::LocalPaint;
use crate::sheet::{
    Decl, Direction, Groups, Position, PropClass, Sides, SizeFields, SpacingFields, Style,
    TextFields,
};
use crate::theme::value::Theme;
use kivact_types::Length;

/// A [`Style`] compiled against a theme: the engine's node props and the
/// diagnostics of the compilation, made by [`crate::internal::compile_style`].
///
/// The retained base paint callback (`Style::paint`) is not a node prop. It
/// runs while the node's drawing is recorded: ask [`has_retained_paint`]
/// (whether there is one) and take it with [`retained_paint`].
///
/// [`has_retained_paint`]: CompiledStyle::has_retained_paint
/// [`retained_paint`]: CompiledStyle::retained_paint
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
    /// Lengths a property cannot take (a content size outside width, height
    /// and basis; `auto`, a content size or a variable left unresolved in a
    /// paint position or radius), counted and reported as `Unsupported`.
    pub misplaced_lengths: u32,
    /// Text values the host cannot draw (a feature tag that is not four
    /// ASCII characters, a text shadow's spread or inset), dropped and
    /// reported as `Unsupported`.
    pub unsupported_text: u32,
    /// The states and groups set inside the host state styles, as
    /// `state.name` (`"hover.hover"`, `"hover.group"`): ignored and reported
    /// as `Unsupported`.
    pub nested_in_states: Vec<String>,
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
pub(crate) struct Resolve<'a> {
    pub(crate) theme: &'a Theme,
    em_px: f64,
    misplaced: &'a mut u32,
}

impl<'a> Resolve<'a> {
    pub(super) fn new(g: &Groups, theme: &'a Theme, misplaced: &'a mut u32) -> Self {
        let base = theme.get(&crate::font::SIZE_VAR);
        let em_px = match g.text.font_size {
            Some(size) => match resolved(size, theme, base, &mut 0) {
                Length::Px(px) => px,
                _ => base,
            },
            None => base,
        };
        Resolve {
            theme,
            em_px,
            misplaced,
        }
    }

    /// The resolve of a drawing that has no style: `em` is of the theme's
    /// base size.
    pub(crate) fn of_theme(theme: &'a Theme, misplaced: &'a mut u32) -> Self {
        Resolve {
            theme,
            em_px: theme.get(&crate::font::SIZE_VAR),
            misplaced,
        }
    }

    /// `l` with its theme variables and `em` resolved.
    pub(crate) fn len(&mut self, l: Length) -> Length {
        resolved(l, self.theme, self.em_px, self.misplaced)
    }

    /// `l` in its paint form: resolved, and counted in the misplaced lengths
    /// when a paint position cannot take it (`auto`, a content size).
    pub(crate) fn paint_len(&mut self, l: Length) -> p::Len {
        resolved(l, self.theme, self.em_px, &mut 0).to_len(self.misplaced)
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

fn layout(g: &Groups, cx: &mut Resolve<'_>) -> LayoutProps {
    let (size, spacing, flow, motion) = (&g.size, &g.spacing, &g.flow, &g.motion);
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
        border: border_widths(g),
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
/// `Length::FitContent`) among the lengths of `g` that are not sizes (width,
/// height, the flex basis): minimums, maximums, padding, margins, insets
/// and gaps, where they are `auto`.
fn misplaced(g: &Groups) -> u32 {
    let (size, spacing): (&SizeFields, &SpacingFields) = (&g.size, &g.spacing);
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
fn text(g: &Groups, theme: &Theme, misplaced: &mut u32) -> (TextStyle, u32) {
    let text: &TextFields = &g.text;
    let base = theme.get(&crate::font::SIZE_VAR);
    let font_size = text
        .font_size
        .and_then(|size| match resolved(size, theme, base, &mut 0) {
            Length::Px(px) => Some(px),
            _ => {
                *misplaced += 1;
                None
            }
        });
    let mut t = TextStyle {
        font_size,
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
    pub forced: StateSet,
    /// The host states the engine reported active for this node
    /// (`HostEvent::StatesChanged`): their non-paint declarations are
    /// compiled into the style.
    pub active: StateSet,
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
        s.take_state(StyleState::Hover);
        s.take_state(StyleState::Active);
    }
    let has_focus =
        s.state(StyleState::Focus).is_some() || s.state(StyleState::FocusVisible).is_some();
    if !has_focus {
        let ring = Style::default().ring_with_offset(
            theme.focus_ring(),
            theme.focus_ring_offset(),
            crate::color::FOCUS,
        );
        *s = std::mem::take(s).focus_visible(ring);
    }
    let sets_opacity = s
        .state(StyleState::Disabled)
        .is_some_and(|d| d.decls.iter().any(|x| matches!(x, Decl::Opacity(_))));
    if !sets_opacity {
        let fade = Style::default().opacity(theme.get(&crate::control::DISABLED_OPACITY_VAR));
        *s = std::mem::take(s).disabled(fade);
    }
}

/// Merge the state styles a tool forces into the base, in the engine's
/// layer order, so they no longer apply as deltas.
fn merge_forced(mut s: Style, forced: StateSet) -> Style {
    let order = [
        StyleState::Focus,
        StyleState::FocusVisible,
        StyleState::Hover,
        StyleState::Active,
    ];
    for state in order {
        if forced.contains(state)
            && let Some(v) = s.take_state(state)
        {
            s = s.merge(&v);
        }
    }
    s
}

impl Style {
    /// The declarations that are not paint deltas (`PropClass::Rest`),
    /// without the nested states and groups: what a host state compiles
    /// into the node's style (ST-05).
    pub(crate) fn rest(&self) -> Style {
        Style {
            decls: self
                .decls
                .iter()
                .filter(|d| {
                    d.class() == PropClass::Rest && !matches!(d, Decl::State(..) | Decl::Group)
                })
                .cloned()
                .collect(),
        }
    }
}

/// The host states that set more than paint (a paint callback included):
/// the engine reports them for this node (ST-05). Enter and exit are left
/// out: they are paint transitions.
fn report_of(s: &Style) -> StateSet {
    let mut report = StateSet::EMPTY;
    for host in StyleState::HOST {
        if matches!(host, StyleState::Enter | StyleState::Exit) {
            continue;
        }
        if s.state(host).is_some_and(|st| !st.rest().decls.is_empty()) {
            report.insert(host);
        }
    }
    report
}

/// Merge the non-paint declarations of the host states the engine reports
/// active into the base, in the order their deltas stack (ST-05). Enter and
/// exit are not merged: they are where a paint transition starts and ends.
fn merge_active(mut s: Style, active: StateSet) -> Style {
    for host in StyleState::HOST {
        if active.contains(host)
            && !matches!(host, StyleState::Enter | StyleState::Exit)
            && let Some(st) = s.state(host)
        {
            s = s.merge(&st.rest());
        }
    }
    s
}

/// The host-evaluated state deltas of `s`, if it sets any state, or if the
/// engine has to report one (`report`).
fn state_styles(s: &Style, theme: &Theme, report: StateSet) -> Option<StateStyles> {
    let hover = s.state(StyleState::Hover);
    let group_hover = s.state(StyleState::GroupHover);
    let group_focus = s.state(StyleState::GroupFocus);
    let pressed = s.state(StyleState::Active);
    let focus = s.state(StyleState::Focus);
    let focus_visible = s.state(StyleState::FocusVisible);
    let enter = s.state(StyleState::Enter);
    let exit = s.state(StyleState::Exit);
    let group = s.decls.iter().any(|d| matches!(d, Decl::Group));
    let any_state = [
        &hover,
        &group_hover,
        &group_focus,
        &pressed,
        &focus,
        &focus_visible,
        &enter,
        &exit,
    ]
    .iter()
    .any(|state| state.is_some());
    (any_state || group || !report.is_empty()).then(|| {
        let mut out = StateStyles {
            group,
            ..StateStyles::default()
        };
        out.report = report;
        let delta = |h: &Style| Some(Box::new(paint_delta(s, h, theme)));
        out.set_delta(StyleState::Hover, hover.as_ref().and_then(delta));
        out.set_delta(StyleState::Active, pressed.as_ref().and_then(delta));
        out.set_delta(StyleState::Focus, focus.as_ref().and_then(delta));
        out.set_delta(StyleState::GroupHover, group_hover.as_ref().and_then(delta));
        out.set_delta(StyleState::GroupFocus, group_focus.as_ref().and_then(delta));
        out.set_delta(StyleState::Enter, enter.as_ref().and_then(delta));
        out.set_delta(StyleState::Exit, exit.as_ref().and_then(delta));
        // Over the focus state, as the host applies it.
        out.set_delta(
            StyleState::FocusVisible,
            focus_visible.as_ref().map(|h| {
                let focused = match &focus {
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
pub fn compile_style(mut s: Style, theme: &Theme, mut state: Interactive) -> CompiledStyle {
    if state.forced.contains(StyleState::Disabled) {
        state.disabled = true;
    }
    if state.forced.contains(StyleState::Selected) {
        state.selected = true;
    }
    let nested_in_states = nested_in_states(&s);
    if state.focusable {
        interactive_defaults(&mut s, theme, state.disabled);
    }
    if state.selected
        && let Some(v) = s.take_state(StyleState::Selected)
    {
        s = s.merge(&v);
    }
    if state.disabled
        && let Some(v) = s.take_state(StyleState::Disabled)
    {
        s = s.merge(&v);
    }
    s = merge_forced(s, state.forced);
    // What the host must report: decided from the declarations as written,
    // paint callbacks included, before anything is stripped.
    let report = report_of(&s);
    s = merge_active(s, state.active);
    // The callback is taken after the merge: a selected or disabled
    // style's own callback is the node's (UI-S-02), and the state styles
    // that remain never carry one.
    let retained_paint = s.local_paint();
    s.strip_local_paint();
    // The one fold of this compilation: every resolver below reads it.
    let g = s.fold();
    let states = state_styles(&s, theme, report);
    let mut unsupported_text = 0;
    let mut misplaced = misplaced(&g);
    let text = (s.has_text() || state.root).then(|| {
        let (mut t, dropped) = text(&g, theme, &mut misplaced);
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
    let mut cx = Resolve::new(&g, theme, &mut misplaced);
    let layout = layout(&g, &mut cx);
    let mut paint = paint(&g, theme, &mut cx);
    paint.transitions = paint_transitions(&g);
    let reports = reports(&g, theme);
    CompiledStyle {
        retained_paint,
        layout,
        paint,
        text,
        states,
        misplaced_lengths: misplaced,
        unsupported_text,
        nested_in_states,
        reports,
    }
}

impl CompiledStyle {
    /// Whether the style that was compiled sets a base paint callback
    /// (`Style::paint`).
    pub fn has_retained_paint(&self) -> bool {
        self.retained_paint.is_some()
    }

    /// The base paint callback (`Style::paint`) the style sets, if any. Its
    /// clones share it, and it compares by identity.
    pub fn retained_paint(&self) -> Option<LocalPaint> {
        self.retained_paint.clone()
    }
}

#[cfg(test)]
mod tests {
    //! Role: Tests of style compilation.
    use super::*;
    use crate::sheet::style;
    use crate::{em, font, space};

    fn width_of(s: &Style, theme: &Theme) -> p::Length {
        compile_style(s.to_owned(), theme, Interactive::default())
            .layout
            .width
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
            style().width(Length::Auto + 2),
            &Theme::light(),
            Interactive::default(),
        );
        assert_eq!(compiled.misplaced_lengths, 1);
        assert_eq!(compiled.layout.width, p::Length::Auto);
    }

    #[test]
    fn a_paint_position_that_is_auto_is_counted() {
        // Each corner takes an x and a y radius: four corners, two lengths each.
        let compiled = compile_style(
            style().radius(Length::Auto),
            &Theme::light(),
            Interactive::default(),
        );
        assert_eq!(compiled.misplaced_lengths, 8);
    }

    #[test]
    fn forced_states_merge_their_styles() {
        let forced = Interactive {
            forced: StateSet::EMPTY.with(StyleState::Hover),
            ..Interactive::default()
        };
        let compiled = compile_style(style().hover(style().opacity(0.5)), &Theme::light(), forced);
        assert_eq!(compiled.paint.opacity, 0.5);
        assert!(compiled.states.is_none());
    }

    #[test]
    fn a_variable_font_size_is_the_base_of_em() {
        let theme = Theme::light().set(&font::SIZE_VAR, 20);
        assert_eq!(width_of(&style().width(em(1)), &theme), p::Length::Px(20.0));
    }

    #[test]
    fn a_state_paint_callback_asks_for_reports() {
        let s = style().hover(style().paint(|_, _| {}));
        let theme = Theme::light();
        let outside = compile_style(s.clone(), &theme, Interactive::default());
        assert!(!outside.has_retained_paint());
        let states = outside.states.expect("a hover style");
        assert!(states.report.contains(StyleState::Hover));
        let inside = compile_style(
            s,
            &theme,
            Interactive {
                active: StateSet::EMPTY.with(StyleState::Hover),
                ..Interactive::default()
            },
        );
        assert!(inside.has_retained_paint());
    }

    #[test]
    fn only_states_with_more_than_paint_ask_for_reports() {
        let theme = Theme::light();
        let paint_only = compile_style(
            style().hover(style().background(crate::rgb(0xff0000))),
            &theme,
            Interactive::default(),
        );
        let states = paint_only.states.expect("a hover style");
        assert!(states.report.is_empty());
        let layout = compile_style(
            style().hover(style().width(10)),
            &theme,
            Interactive::default(),
        );
        let states = layout.states.expect("a hover style");
        assert!(states.report.contains(StyleState::Hover));
    }
}
