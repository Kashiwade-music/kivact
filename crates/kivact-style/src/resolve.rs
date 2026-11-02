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
//! - A paint transition (`TransitionProp::Paint`) works the same for a
//!   signal or style change and for the host's hover, pressed and focus
//!   switches: it interpolates the opacity, a solid background, each
//!   border color, the canonical transform and its origin, the first two
//!   shadows and the filter amounts; gradients, images, shapes, the
//!   z-index and the order of filters switch to the target at once.
//! - A dynamic style that drops a state writes empty `StateStyles`, and one
//!   that drops a text property writes `None`, through the typed setter, so
//!   the node goes back to what it inherits.

use kivact_wire as p;
use kivact_wire::{
    Border, Corners, Edges, Effects, ElementTransform, Fill, LayoutProps, PaintDelta, PaintProps,
    Ring, ScrollAxes, StateStyles, TextProps,
};

use super::color::ColorToken;
use super::draw::{Draw, LocalPaint, Size};
use super::effect::{Shadow, clip_shape, filter_ops};
use super::geometry::corner;
use super::length::{Length, pct};
use super::paint::{Background, fill_layer};
use super::sheet::{Direction, Overflow, Position, ScrollbarWidth, Sides, Style};
use super::theme::value::Theme;
use super::transform::Transform;

/// A [`Style`] compiled against a theme: the engine's node props and the
/// diagnostics of the compilation, made by [`Style::compile`].
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
    pub text: Option<TextProps>,
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
        grid: super::grid::resolve(s.grid.as_deref(), s.grid_box == Some(true)),
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
fn text(s: &Style, theme: &Theme) -> (TextProps, u32) {
    let mut t = TextProps {
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
        ..TextProps::default()
    };
    let dropped = match &s.text {
        Some(x) => super::text::resolve_extras(&mut t, x, theme),
        None => 0,
    };
    (t, dropped)
}

/// How a node takes part in the theme's defaults for interactive elements,
/// the input of [`Style::compile`] next to the style and the theme.
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
        let ring = Style::default().ring(theme.focus_ring(), ColorToken::Focus);
        s.focus_visible = Some(Box::new(ring));
    }
    let disabled = s.disabled.get_or_insert_with(Default::default);
    if disabled.opacity.is_none() {
        disabled.opacity = Some(theme.disabled_opacity());
    }
}

impl Style {
    /// Compile this style against `theme` into the engine's node props, the
    /// diagnostics of the compilation and the retained base paint. `state`
    /// says which state styles and theme defaults apply; its `root` adds the
    /// theme's text defaults that every node inherits.
    pub fn compile(&self, theme: &Theme, state: Interactive) -> CompiledStyle {
        compile(self, theme, state)
    }
}

/// Compile a style for a node. `disabled`/`selected` variants are merged
/// here; hover/pressed/focus/focus-visible become host-evaluated state
/// deltas.
fn compile(s: &Style, theme: &Theme, state: Interactive) -> CompiledStyle {
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
    let states = if s.hover.is_some()
        || s.group_hover.is_some()
        || s.group_focus.is_some()
        || s.group
        || s.pressed.is_some()
        || s.focus.is_some()
        || s.focus_visible.is_some()
        || s.enter.is_some()
        || s.exit.is_some()
    {
        Some(StateStyles {
            hover: s.hover.as_ref().map(|h| paint_delta(&s, h, theme)),
            pressed: s.pressed.as_ref().map(|h| paint_delta(&s, h, theme)),
            focus: s.focus.as_ref().map(|h| paint_delta(&s, h, theme)),
            focus_visible: s.focus_visible.as_ref().map(|h| {
                // Over the focus state, as the host applies it.
                let focused = match &s.focus {
                    Some(focus) => s.clone().merge(focus),
                    None => s.clone(),
                };
                paint_delta(&focused, h, theme)
            }),
            group_hover: s.group_hover.as_ref().map(|h| paint_delta(&s, h, theme)),
            group_focus: s.group_focus.as_ref().map(|h| paint_delta(&s, h, theme)),
            group: s.group,
            enter: s.enter.as_ref().map(|h| paint_delta(&s, h, theme)),
            exit: s.exit.as_ref().map(|h| paint_delta(&s, h, theme)),
        })
    } else {
        None
    };
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
    paint.transition =
        super::paint_transition(&s).map(|t| std::sync::Arc::new(t.as_wire().clone()));
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

/// The properties `s` sets, by the name of the setter that sets them. Every
/// field of `Style` is named in the destructure, so a new field must be
/// classified here.
fn set_properties(s: &Style) -> Vec<&'static str> {
    let Style {
        local_paint,
        padding,
        margin,
        inset,
        row_gap,
        column_gap,
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
        align_content,
        grid_box,
        grid,
        wrap,
        position,
        overflow,
        scroll_axes,
        background,
        border_width,
        border_color,
        border_style,
        ring,
        radius,
        shadows,
        opacity,
        translate,
        rotate,
        scale,
        transform,
        transform_origin,
        scrollbar,
        cursor,
        z_index,
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
        italic,
        text,
        hover,
        group,
        group_hover,
        group_focus,
        pressed,
        focus,
        focus_visible,
        disabled,
        selected,
        enter,
        exit,
        transitions,
        duplicates: _,
    } = s;
    let sides = Sides::default();
    let mut out = Vec::new();
    let mut add = |set: bool, name: &'static str| {
        if set {
            out.push(name);
        }
    };
    add(local_paint.is_some(), "paint");
    add(*padding != sides, "padding");
    add(*margin != sides, "margin");
    add(*inset != sides, "inset");
    add(row_gap.is_some(), "row_gap");
    add(column_gap.is_some(), "column_gap");
    add(width.is_some(), "width");
    add(height.is_some(), "height");
    add(min_width.is_some(), "min_width");
    add(min_height.is_some(), "min_height");
    add(max_width.is_some(), "max_width");
    add(max_height.is_some(), "max_height");
    add(grow.is_some(), "grow");
    add(shrink.is_some(), "shrink");
    add(basis.is_some(), "basis");
    add(aspect_ratio.is_some(), "aspect_ratio");
    add(direction.is_some(), "direction");
    add(align.is_some(), "align_items");
    add(align_self.is_some(), "align_self");
    add(justify.is_some(), "justify_content");
    add(align_content.is_some(), "align_content");
    add(grid_box.is_some(), "display_grid");
    add(grid.is_some(), "grid");
    add(wrap.is_some(), "wrap");
    add(position.is_some(), "position");
    add(overflow.is_some(), "overflow");
    add(scroll_axes.is_some(), "with_scroll_axes");
    add(background.is_some(), "background");
    add(border_width.iter().any(Option::is_some), "border_width");
    add(border_color.iter().any(Option::is_some), "border_color");
    add(border_style.is_some(), "border_style");
    add(ring.is_some(), "ring");
    add(radius.iter().any(Option::is_some), "radius");
    add(shadows.is_some(), "shadow");
    add(opacity.is_some(), "opacity");
    add(translate.iter().any(Option::is_some), "translate");
    add(rotate.is_some(), "rotate");
    add(scale.is_some(), "scale");
    add(transform.is_some(), "transform");
    add(transform_origin.is_some(), "transform_origin");
    add(scrollbar.is_some(), "scrollbar");
    add(cursor.is_some(), "cursor");
    add(z_index.is_some(), "z_index");
    add(clip.is_some(), "clip");
    add(filter.is_some(), "filter");
    add(backdrop_filter.is_some(), "backdrop_filter");
    add(blend.is_some(), "blend");
    add(mask.is_some(), "mask");
    add(clip_path.is_some(), "clip_path");
    add(font_size.is_some(), "font_size");
    add(weight.is_some(), "weight");
    add(color.is_some(), "color");
    add(line_height.is_some(), "line_height");
    add(text_wrap.is_some(), "text_wrap");
    add(text_align.is_some(), "text_align");
    add(font_family.is_some(), "font_family");
    add(italic.is_some(), "italic");
    add(text.is_some(), "text");
    add(hover.is_some(), "hover");
    add(*group, "group");
    add(group_hover.is_some(), "group_hover");
    add(group_focus.is_some(), "group_focus");
    add(pressed.is_some(), "pressed");
    add(focus.is_some(), "focus");
    add(focus_visible.is_some(), "focus_visible");
    add(disabled.is_some(), "disabled");
    add(selected.is_some(), "selected");
    add(enter.is_some(), "enter");
    add(exit.is_some(), "exit");
    add(!transitions.is_empty(), "transition");
    out
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
fn border_widths(s: &Style) -> Edges<f32> {
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
        || s.border_style.is_some();
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
            style: s.border_style.unwrap_or_default(),
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
            mask: s.mask.as_ref().map(|m| fill_layer(m, theme)),
            clip_path: s.clip_path.as_ref().map(|c| clip_shape(&c.0)),
        })
    })
}

/// The transform properties of `s` as one element transform: CSS's
/// `translate`, `rotate` and `scale`, then `transform`, about
/// `transform-origin` (the centre by default).
fn element_transform(s: &Style) -> ElementTransform {
    let mut t = Transform::new();
    if s.translate.iter().any(Option::is_some) {
        // An axis left unset does not move the node.
        let [x, y] = s.translate.map(|axis| axis.unwrap_or(Length::Px(0.0)));
        t = t.translate(x, y);
    }
    if let Some(deg) = s.rotate {
        t = t.rotate(deg);
    }
    if let Some((x, y)) = s.scale {
        t = t.scale_xy(x, y);
    }
    if let Some(list) = &s.transform {
        t = t.then(list);
    }
    let (x, y) = s.transform_origin.unwrap_or((pct(50), pct(50)));
    // `auto` is the default centre.
    t.to_wire([x, y].map(|l| match l {
        Length::Auto => p::Len::pct(50.0),
        l => l.to_len(),
    }))
}

/// Whether `s` sets a transform property.
fn sets_transform(s: &Style) -> bool {
    s.translate.iter().any(Option::is_some)
        || s.rotate.is_some()
        || s.scale.is_some()
        || s.transform.is_some()
        || s.transform_origin.is_some()
}

fn paint(s: &Style, theme: &Theme) -> PaintProps {
    PaintProps {
        transition: None,
        background: s.background.as_ref().map(|b| fill(b, theme)),
        border: border(s, theme),
        ring: s.ring.map(|(w, c)| Ring {
            width: w,
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
        z_index: s.z_index.unwrap_or(0),
        scrollbar: scrollbar(s, theme),
        cursor: s.cursor.unwrap_or_default(),
        clip: s.clip.unwrap_or(false)
            || s.scroll_axes.is_some()
            || s.overflow.is_some_and(|o| o != Overflow::Visible),
        effects: effects(s, theme),
    }
}

/// The paint a state style `st` changes over the node's style `base`:
/// each group it sets, resolved over the base (a state that sets only a
/// border color keeps the base's widths).
fn paint_delta(base: &Style, st: &Style, theme: &Theme) -> PaintDelta {
    let merged = base.clone().merge(st);
    let full = paint(&merged, theme);
    let sets_border = st.border_width.iter().any(Option::is_some)
        || st.border_color.iter().any(Option::is_some)
        || st.border_style.is_some();
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
