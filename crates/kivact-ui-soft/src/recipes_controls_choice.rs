//! Role: Style the persistent indicator parts of Soft choice controls.
use crate::Tokens;
use kivact::draw::LineCap;
use kivact::draw::LineJoin;
use kivact::draw::Path;
use kivact::draw::Stroke;
use kivact::geometry::Point;
use kivact::style::Align;
use kivact::style::Color;
use kivact::style::Direction;
use kivact::style::Justify;
use kivact::style::Position;
use kivact::style::Style;
use kivact::style::Theme;
use kivact::style::pct;
use kivact::style::style;
use kivact::ui::CheckableState;
use kivact::ui::ControlSize;
use kivact::ui::Lift;
use kivact::ui::Look;
use kivact::ui::RadioGroupRecipes;
use kivact::ui::SwitchDragState;
use kivact::ui::TrackInput;
use kivact::view::CheckedState;
use kivact::view::Orientation;
use kivact::view::ReadingDirection;
use kivact::view::use_direction;

use super::recipe_style::{fast, overlay, ring};

fn small(size: ControlSize) -> bool {
    matches!(size, ControlSize::ExtraSmall | ControlSize::Small)
}

fn check_side(t: &Tokens, s: &Look<CheckableState>) -> f64 {
    if small(s.size) {
        t.spacing.s5
    } else {
        t.spacing.s6
    }
}

fn switch_size(t: &Tokens, s: &Look<CheckableState>) -> (f64, f64) {
    (
        if small(s.size) {
            t.spacing.s8
        } else {
            t.spacing.s10
        },
        check_side(t, s),
    )
}

fn switch_parts(t: &Tokens, s: &Look<CheckableState>) -> (f64, f64, f64) {
    if small(s.size) {
        (t.typography.size_md, 3.0, t.spacing.s4)
    } else {
        (t.spacing.s4, t.spacing.s1, t.spacing.s5)
    }
}

fn switch_travel(t: &Tokens, s: &Look<CheckableState>) -> f64 {
    let (width, _) = switch_size(t, s);
    let (base_side, inset, on_side) = switch_parts(t, s);
    width - t.spacing.s0_5 - on_side / 2.0 - inset - base_side / 2.0
}

fn switch_sign() -> f64 {
    if use_direction().get() == ReadingDirection::Rtl {
        -1.0
    } else {
        1.0
    }
}

/// The side the switch shows while a gesture moves the thumb.
fn switch_on(t: &Tokens, s: &Look<CheckableState>, drag: Option<SwitchDragState>) -> bool {
    match drag {
        Some(SwitchDragState::Tracking(TrackInput::Touch { offset })) => {
            offset * switch_sign() > switch_travel(t, s) / 2.0
        }
        _ => s.is_checked(),
    }
}

fn switch_lifted(drag: Option<SwitchDragState>) -> bool {
    matches!(
        drag,
        Some(
            SwitchDragState::TouchPressed
                | SwitchDragState::Tracking(TrackInput::Touch { .. })
                | SwitchDragState::TouchReleased(Lift::Raised)
        )
    )
}

fn switch_offset(drag: Option<SwitchDragState>) -> Option<f64> {
    match drag {
        Some(SwitchDragState::Tracking(TrackInput::Touch { offset })) => Some(offset),
        _ => None,
    }
}

fn inline_start(base: Style, offset: f64) -> Style {
    if use_direction().get() == ReadingDirection::Rtl {
        base.right(offset)
    } else {
        base.left(offset)
    }
}

fn label_row(t: &Tokens, width: f64, height: f64) -> Style {
    let text = t.typography.label;
    let base = style()
        .position(Position::Relative)
        .direction(Direction::Row)
        .align_items(Align::Center)
        .min_height(height)
        .font_size(text.size_px)
        .line_height(text.line_height_px / text.size_px)
        .weight(text.weight)
        .color(t.colors.text_secondary)
        .focus_visible(style().ring_none())
        .disabled(style().color(t.colors.text_disabled));
    let padding = width + t.spacing.s2;
    if use_direction().get() == ReadingDirection::Rtl {
        base.padding_right(padding)
    } else {
        base.padding_left(padding)
    }
}

pub(super) fn checkbox(theme: &Theme, s: &Look<CheckableState>) -> Style {
    let t = theme.ext::<Tokens>();
    let side = check_side(&t, s);
    label_row(&t, side, side).disabled(style().opacity(t.sizes.disabled_opacity))
}

pub(super) fn switch(theme: &Theme, s: &Look<CheckableState>) -> Style {
    let t = theme.ext::<Tokens>();
    let (width, height) = switch_size(&t, s);
    label_row(&t, width, height)
}

fn pressed_fill(theme: &Theme, t: &Tokens, s: &Look<CheckableState>, fill: Color) -> Color {
    if s.input.pressed && !s.input.disabled {
        overlay(theme, fill, t.colors.overlay_pressed)
    } else {
        fill
    }
}

fn check_surface(theme: &Theme, t: &Tokens, s: &Look<CheckableState>, radius: f64) -> Style {
    let c = &t.colors;
    let on = s.checked() != CheckedState::Unchecked;
    let hovered = s.input.hovered && !s.input.disabled;
    let (fill, border) = if on {
        let accent = if hovered { c.accent_hover } else { c.accent };
        (accent, accent)
    } else if s.input.disabled {
        (c.bg_muted, c.border)
    } else if hovered {
        (c.choice_surface_hover, c.choice_border_hover)
    } else {
        (c.bg_surface, c.border_emphasized)
    };
    let side = check_side(t, s);
    let base = style()
        .size(side, side)
        .shrink(0)
        .radius(radius)
        .border(t.sizes.border_width, border)
        .background(pressed_fill(theme, t, s, fill));
    fast(
        t,
        if s.input.is_focus_visible() {
            base.merge(&ring(t, c.focus_ring))
        } else {
            base
        },
    )
}

pub(super) fn checkbox_indicator(theme: &Theme, s: &Look<CheckableState>) -> Style {
    let t = theme.ext::<Tokens>();
    let side = check_side(&t, s);
    inline_start(check_surface(theme, &t, s, t.radius.inner), 0.0).top(pct(50) - side / 2.0)
}

fn centered_mark(width: f64, height: f64, visible: bool) -> Style {
    style()
        .position(Position::Absolute)
        .left(pct(50) - width / 2.0)
        .top(pct(50) - height / 2.0)
        .size(width, height)
        .opacity(if visible { 1.0 } else { 0.0 })
}

pub(super) fn checkbox_check(theme: &Theme, s: &Look<CheckableState>) -> Style {
    let t = theme.ext::<Tokens>();
    let side = if small(s.size) {
        t.spacing.s3
    } else {
        t.typography.size_md
    };
    let ink = t.colors.on_accent;
    centered_mark(side, side, s.is_checked()).paint(move |d, size| {
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

pub(super) fn checkbox_mixed(theme: &Theme, s: &Look<CheckableState>) -> Style {
    let t = theme.ext::<Tokens>();
    let width = if small(s.size) {
        t.typography.size_xs
    } else {
        t.spacing.s3
    };
    centered_mark(width, t.spacing.s0_5, s.is_mixed())
        .radius(t.radius.full)
        .background(t.colors.on_accent)
}

pub(super) fn switch_track(theme: &Theme, s: &Look<CheckableState>) -> Style {
    let t = theme.ext::<Tokens>();
    let (width, height) = switch_size(&t, s);
    let c = &t.colors;
    let drag = s.switch_drag();
    let touch = drag.is_some_and(SwitchDragState::is_touch);
    let on = switch_on(&t, s, drag);
    let fill = if on { c.accent } else { c.border_emphasized };
    let fill = if s.input.hovered && !s.input.disabled && !touch {
        if on {
            c.accent_hover
        } else {
            c.switch_off_hover
        }
    } else {
        fill
    };
    let base = inline_start(style().size(width, height).radius(t.radius.full), 0.0)
        .top(pct(50) - height / 2.0)
        .background(pressed_fill(theme, &t, s, fill))
        .opacity(if s.input.disabled {
            t.sizes.disabled_opacity
        } else {
            1.0
        });
    let base = if s.input.is_focus_visible() {
        base.merge(&ring(&t, c.focus_ring))
    } else {
        base
    };
    if touch {
        let transition = if kivact::platform::is_motion_reduced().get() {
            kivact::motion::Transition::tween(200, t.motion.ease_standard.clone()).essential()
        } else if on {
            spring(85.377, 18.480)
        } else {
            spring(246.740, 31.416)
        };
        base.transition(kivact::style::TransitionProp::All, transition)
    } else {
        fast(&t, base)
    }
}

pub(super) fn switch_thumb(theme: &Theme, s: &Look<CheckableState>) -> Style {
    let t = theme.ext::<Tokens>();
    let drag = s.switch_drag();
    let on = switch_on(&t, s, drag);
    let (_, height) = switch_size(&t, s);
    let (base_side, inset, on_side) = switch_parts(&t, s);
    let movement = switch_travel(&t, s);
    let sign = switch_sign();
    let reduced = kivact::platform::is_motion_reduced().get();
    let lifted = switch_lifted(drag) && !reduced;
    let side = if on { on_side } else { base_side };
    let lift = if lifted {
        1.0 + if on { t.spacing.s0_5 } else { inset } / side
    } else {
        1.0
    };
    let offset = switch_offset(drag).unwrap_or(if on { sign * movement } else { 0.0 });
    let base = inline_start(style().position(Position::Absolute), inset)
        .top((height - base_side) / 2.0)
        .size(base_side, base_side)
        .radius(t.radius.full)
        .background(pressed_fill(theme, &t, s, t.colors.bg_surface))
        .translate_x(offset)
        .scale(side / base_side * lift);
    let base = if lifted {
        t.shadows.low.iter().copied().fold(base, Style::shadow)
    } else {
        base
    };
    if let Some(drag) = drag.filter(|drag| drag.is_touch()) {
        if reduced {
            return base;
        }
        let transition = match drag {
            SwitchDragState::Tracking(_) => spring(304.6, 34.91),
            SwitchDragState::TouchReleased(Lift::Settled) => spring(157.9, 17.59),
            SwitchDragState::TouchReleased(Lift::Raised) => spring(438.6, 41.89),
            SwitchDragState::TouchPressed => spring(746.3, 30.05),
        };
        base.transition(kivact::style::TransitionProp::All, transition)
    } else {
        fast(&t, base)
    }
}

fn spring(stiffness: f64, damping: f64) -> kivact::motion::Transition {
    kivact::motion::Transition::spring(kivact::motion::Spring {
        stiffness,
        damping,
        mass: 1.0,
        rest: 1e-3,
    })
}

pub(super) fn radio_group(r: RadioGroupRecipes) -> RadioGroupRecipes {
    r.group(|_| group)
        .item(|_| item)
        .indicator(|_| indicator)
        .mark(|_| mark)
        .description(|_| description)
}

fn group(theme: &Theme, orientation: &Look<Orientation>) -> Style {
    if orientation.variant::<kivact::ui::RadioGroupVariant>()
        == Some(&kivact::ui::RadioGroupVariant::Segmented)
    {
        return crate::segmented_control::track(theme, orientation);
    }
    let t = theme.ext::<Tokens>();
    if **orientation == Orientation::Horizontal {
        style().direction(Direction::Row).gap(t.spacing.s5)
    } else {
        style().direction(Direction::Column).gap(t.spacing.s2)
    }
}

fn item(theme: &Theme, s: &Look<CheckableState>) -> Style {
    if s.variant::<kivact::ui::RadioGroupVariant>()
        == Some(&kivact::ui::RadioGroupVariant::Segmented)
    {
        return crate::segmented_control::segment(theme, s);
    }
    let t = theme.ext::<Tokens>();
    let text = t.typography.body;
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(t.spacing.s2)
        .font_size(text.size_px)
        .line_height(text.line_height_px / text.size_px)
        .weight(text.weight)
        .color(t.colors.text_primary)
        .disabled(style().opacity(t.sizes.disabled_opacity))
        .focus_visible(style().ring_none())
        .min_height(check_side(&t, s))
}

fn indicator(theme: &Theme, s: &Look<CheckableState>) -> Style {
    if s.variant::<kivact::ui::RadioGroupVariant>()
        == Some(&kivact::ui::RadioGroupVariant::Segmented)
    {
        return style().size(0, 0).clip(true);
    }
    let t = theme.ext::<Tokens>();
    check_surface(theme, &t, s, t.radius.full)
        .position(Position::Relative)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
}

fn mark(theme: &Theme, s: &Look<CheckableState>) -> Style {
    if s.variant::<kivact::ui::RadioGroupVariant>()
        == Some(&kivact::ui::RadioGroupVariant::Segmented)
    {
        return style().size(0, 0).clip(true);
    }
    let t = theme.ext::<Tokens>();
    let side = if small(s.size) {
        t.spacing.s2
    } else {
        t.typography.size_xs
    };
    centered_mark(side, side, s.is_checked())
        .radius(t.radius.full)
        .background(t.colors.on_accent)
}

fn description(theme: &Theme, _: &Look<CheckableState>) -> Style {
    let t = theme.ext::<Tokens>();
    let text = t.typography.supporting;
    style()
        .font_size(text.size_px)
        .line_height(text.line_height_px / text.size_px)
        .weight(text.weight)
        .color(t.colors.text_secondary)
}
