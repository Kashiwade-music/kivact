//! Role: Style the Astryx Neutral checkboxes and switches (the controls group's choice
//! parts).
use crate::color as c;
use crate::metrics as m;
use crate::radius as r;
use crate::recipe_parts::{fast, overlay, ring};
use crate::shadows;
use crate::space as s;
use crate::type_scale as t;
use kivact::draw::{LineCap, LineJoin, Path, Stroke};
use kivact::geometry::Point;
use kivact::interaction::{Lift, ThumbDrag, TrackInput};
use kivact::motion::{Spring, Transition};
use kivact::style::{
    Align, Color, FlexDirection, Length, Position, Style, Theme, ThemeMode, TransitionProp, pct,
    style,
};
use kivact::ui::ControlSize;
use kivact::ui::theme::{CheckableState, CheckboxTheme, Look, SwitchTheme};
use kivact::view::{CheckedState, ReadingDirection, use_direction};

pub(crate) fn small(size: ControlSize) -> bool {
    matches!(size, ControlSize::ExtraSmall | ControlSize::Small)
}

/// The side of a check box, and of a radio item's indicator box.
pub(crate) fn check_side(theme: &Theme, look: &Look<CheckableState>) -> f64 {
    if small(look.size) {
        theme.get(&s::SPACE_S5_VAR)
    } else {
        theme.get(&s::SPACE_S6_VAR)
    }
}

fn switch_size(theme: &Theme, look: &Look<CheckableState>) -> (f64, f64) {
    let width = if small(look.size) {
        theme.get(&s::SPACE_S8_VAR)
    } else {
        theme.get(&s::SPACE_S10_VAR)
    };
    (width, check_side(theme, look))
}

/// The thumb's base side, its inset from the track and its on side.
fn switch_parts(theme: &Theme, look: &Look<CheckableState>) -> (f64, f64, f64) {
    if small(look.size) {
        (t::SIZE_MD, 3.0, theme.get(&s::SPACE_S4_VAR))
    } else {
        (
            theme.get(&s::SPACE_S4_VAR),
            theme.get(&s::SPACE_S1_VAR),
            theme.get(&s::SPACE_S5_VAR),
        )
    }
}

fn switch_travel(theme: &Theme, look: &Look<CheckableState>) -> f64 {
    let (width, _) = switch_size(theme, look);
    let (base_side, inset, on_side) = switch_parts(theme, look);
    width - theme.get(&s::SPACE_S0_5_VAR) - on_side / 2.0 - inset - base_side / 2.0
}

fn switch_sign() -> f64 {
    if use_direction() == ReadingDirection::Rtl {
        -1.0
    } else {
        1.0
    }
}

/// The side the switch shows while a gesture moves the thumb.
fn switch_on(look: &Look<CheckableState>, drag: Option<ThumbDrag>) -> bool {
    match drag {
        Some(ThumbDrag::Tracking(TrackInput::Touch { on, .. } | TrackInput::Pointer { on })) => on,
        _ => look.is_checked(),
    }
}

fn switch_lifted(drag: Option<ThumbDrag>) -> bool {
    matches!(
        drag,
        Some(
            ThumbDrag::TouchPressed
                | ThumbDrag::Tracking(TrackInput::Touch { .. })
                | ThumbDrag::TouchReleased(Lift::Raised)
        )
    )
}

fn switch_offset(drag: Option<ThumbDrag>) -> Option<f64> {
    match drag {
        Some(ThumbDrag::Tracking(TrackInput::Touch { offset, .. })) => Some(offset),
        _ => None,
    }
}

/// `base` placed at `offset` from the inline start of its box.
fn inline_start(base: Style, offset: f64) -> Style {
    if use_direction() == ReadingDirection::Rtl {
        base.right(offset)
    } else {
        base.left(offset)
    }
}

/// The label row beside a check box or a switch, padded past its box of `width`.
fn label_row(theme: &Theme, width: f64, height: f64) -> Style {
    let text = t::LABEL;
    let base = style()
        .position(Position::Relative)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .min_height(height)
        .font_size(text.size_px)
        .line_height(text.line_height_px / text.size_px)
        .font_weight(text.weight)
        .color(c::TEXT_SECONDARY)
        .focus_visible(style().outline_none())
        .disabled(style().color(c::TEXT_DISABLED));
    let padding = width + theme.get(&s::SPACE_S2_VAR);
    if use_direction() == ReadingDirection::Rtl {
        base.padding_right(padding)
    } else {
        base.padding_left(padding)
    }
}

pub(crate) fn checkbox(theme: &Theme, look: &Look<CheckableState>, _base: Style) -> Style {
    let side = check_side(theme, look);
    label_row(theme, side, side).disabled(style().opacity(m::DISABLED_OPACITY))
}

pub(crate) fn switch(theme: &Theme, look: &Look<CheckableState>, _base: Style) -> Style {
    let (width, height) = switch_size(theme, look);
    label_row(theme, width, height)
}

fn pressed_fill(look: &Look<CheckableState>, theme: &Theme, fill: Color) -> Color {
    if look.input.is_pressed() && !look.input.is_disabled() {
        overlay(theme, fill, c::OVERLAY_PRESSED)
    } else {
        fill
    }
}

/// The box of a check or radio indicator: its fill, border and focus ring.
pub(crate) fn check_surface(theme: &Theme, look: &Look<CheckableState>, radius: Length) -> Style {
    let on = look.checked() != CheckedState::Unchecked;
    let hovered = look.input.is_hovered() && !look.input.is_disabled();
    let (fill, border) = if on {
        let accent = if hovered { c::ACCENT_HOVER } else { c::ACCENT };
        (accent, accent)
    } else if look.input.is_disabled() {
        (c::BG_MUTED, c::BORDER)
    } else if hovered {
        (c::CHOICE_SURFACE_HOVER, c::CHOICE_BORDER_HOVER)
    } else {
        (c::BG_SURFACE, c::BORDER_EMPHASIZED)
    };
    let side = check_side(theme, look);
    let base = style()
        .size(side, side)
        .flex_shrink(0)
        .border_radius(radius)
        .border(m::BORDER_WIDTH, border)
        .background(pressed_fill(look, theme, fill));
    fast(if look.input.is_focus_visible() {
        base.merge(&ring(Color::var(&c::FOCUS_RING_VAR)))
    } else {
        base
    })
}

pub(crate) fn checkbox_indicator(
    theme: &Theme,
    look: &Look<CheckableState>,
    _base: Style,
) -> Style {
    let side = check_side(theme, look);
    inline_start(check_surface(theme, look, r::RADIUS_INNER), 0.0).top(pct(50) - side / 2.0)
}

/// A mark centered in its box, shown while `visible`.
pub(crate) fn centered_mark(width: f64, height: f64, visible: bool) -> Style {
    style()
        .position(Position::Absolute)
        .left(pct(50) - width / 2.0)
        .top(pct(50) - height / 2.0)
        .size(width, height)
        .opacity(if visible { 1.0 } else { 0.0 })
}

pub(crate) fn checkbox_check(theme: &Theme, look: &Look<CheckableState>, _base: Style) -> Style {
    let side = if small(look.size) {
        theme.get(&s::SPACE_S3_VAR)
    } else {
        t::SIZE_MD
    };
    let ink = c::ON_ACCENT;
    centered_mark(side, side, look.is_checked()).paint(move |d, size| {
        // The Astryx indicator path is in a 10 × 10 view box.
        let unit = size.width / 10.0;
        let path = Path::new()
            .move_to(Point::new(8.5 * unit, 2.5 * unit))
            .line_to(Point::new(4.0 * unit, 7.5 * unit))
            .line_to(Point::new(1.5 * unit, 5.0 * unit));
        let stroke = Stroke::new(1.5 * unit)
            .cap(LineCap::Round)
            .join(LineJoin::Round);
        d.stroke_path(&path, &stroke, ink);
    })
}

pub(crate) fn checkbox_mixed(theme: &Theme, look: &Look<CheckableState>, _base: Style) -> Style {
    let width = if small(look.size) {
        t::SIZE_XS
    } else {
        theme.get(&s::SPACE_S3_VAR)
    };
    centered_mark(width, theme.get(&s::SPACE_S0_5_VAR), look.is_mixed())
        .border_radius(r::RADIUS_FULL)
        .background(c::ON_ACCENT)
}

pub(crate) fn switch_track(theme: &Theme, look: &Look<CheckableState>, _base: Style) -> Style {
    let (width, height) = switch_size(theme, look);
    let drag = look.switch_drag();
    let touch = drag.is_some_and(ThumbDrag::is_touch);
    let on = switch_on(look, drag);
    let fill = if on { c::ACCENT } else { c::BORDER_EMPHASIZED };
    let fill = if look.input.is_hovered() && !look.input.is_disabled() && !touch {
        if on {
            c::ACCENT_HOVER
        } else {
            c::SWITCH_OFF_HOVER
        }
    } else {
        fill
    };
    let base = inline_start(
        style().size(width, height).border_radius(r::RADIUS_FULL),
        0.0,
    )
    .top(pct(50) - height / 2.0)
    .background(pressed_fill(look, theme, fill))
    .opacity(if look.input.is_disabled() {
        m::DISABLED_OPACITY
    } else {
        1.0
    });
    let base = if look.input.is_focus_visible() {
        base.merge(&ring(Color::var(&c::FOCUS_RING_VAR)))
    } else {
        base
    };
    if touch {
        let transition = if kivact::platform::is_motion_reduced().get() {
            Transition::tween(200, crate::motion::EASE_STANDARD).essential()
        } else if on {
            spring(85.377, 18.480)
        } else {
            spring(246.740, 31.416)
        };
        base.transition(TransitionProp::All, transition)
    } else {
        fast(base)
    }
}

pub(crate) fn switch_thumb(theme: &Theme, look: &Look<CheckableState>, _base: Style) -> Style {
    let drag = look.switch_drag();
    let on = switch_on(look, drag);
    let (_, height) = switch_size(theme, look);
    let (base_side, inset, on_side) = switch_parts(theme, look);
    let movement = switch_travel(theme, look);
    let sign = switch_sign();
    let reduced = kivact::platform::is_motion_reduced().get();
    let lifted = switch_lifted(drag) && !reduced;
    let side = if on { on_side } else { base_side };
    let lift = if lifted {
        let raise = if on {
            theme.get(&s::SPACE_S0_5_VAR)
        } else {
            inset
        };
        1.0 + raise / side
    } else {
        1.0
    };
    let offset = switch_offset(drag).unwrap_or(if on { sign * movement } else { 0.0 });
    let base = inline_start(style().position(Position::Absolute), inset)
        .top((height - base_side) / 2.0)
        .size(base_side, base_side)
        .border_radius(r::RADIUS_FULL)
        .background(pressed_fill(look, theme, c::BG_SURFACE))
        .translate_x(offset)
        .scale(side / base_side * lift);
    let base = if lifted {
        shadows::low(theme.mode() == ThemeMode::Dark)
            .into_iter()
            .fold(base, Style::shadow)
    } else {
        base
    };
    if let Some(drag) = drag.filter(|drag| drag.is_touch()) {
        if reduced {
            return base;
        }
        let transition = match drag {
            ThumbDrag::Tracking(_) => spring(304.6, 34.91),
            ThumbDrag::TouchReleased(Lift::Settled) => spring(157.9, 17.59),
            ThumbDrag::TouchReleased(Lift::Raised) => spring(438.6, 41.89),
            ThumbDrag::TouchPressed => spring(746.3, 30.05),
        };
        base.transition(TransitionProp::All, transition)
    } else {
        fast(base)
    }
}

fn spring(stiffness: f64, damping: f64) -> Transition {
    Transition::spring(Spring {
        stiffness,
        damping,
        mass: 1.0,
        rest: 1e-3,
    })
}

/// Install the kit's recipes of the check box and the switch.
pub(crate) fn install(theme: Theme) -> Theme {
    theme
        .with(
            CheckboxTheme::default()
                .root(checkbox)
                .indicator(checkbox_indicator)
                .check(checkbox_check)
                .mixed(checkbox_mixed),
        )
        .with(
            SwitchTheme::default()
                .root(switch)
                .track(switch_track)
                .thumb(switch_thumb),
        )
}
