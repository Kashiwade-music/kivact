//! Role: Style the persistent indicator parts of Soft choice controls.
use crate::Tokens;
use kivact::draw::{LineCap, LineJoin, Path, Point, Stroke};
use kivact::style::{Align, Color, Direction, Justify, Position, Style, Theme, pct, style};
use kivact::ui::{CheckState, ControlSize, Look, RadioGroupRecipes};
use kivact::view::{CheckedState, Orientation, ReadingDirection, current_direction};

use super::recipes_controls::{fast, overlay, ring};

fn small(size: ControlSize) -> bool {
    matches!(size, ControlSize::ExtraSmall | ControlSize::Small)
}

fn check_side(t: &Tokens, s: &Look<CheckState>) -> f32 {
    if small(s.size) {
        t.spacing.s5
    } else {
        t.spacing.s6
    }
}

fn switch_size(t: &Tokens, s: &Look<CheckState>) -> (f32, f32) {
    (
        if small(s.size) {
            t.spacing.s8
        } else {
            t.spacing.s10
        },
        check_side(t, s),
    )
}

fn inline_start(base: Style, offset: f32) -> Style {
    if current_direction() == ReadingDirection::Rtl {
        base.right(offset)
    } else {
        base.left(offset)
    }
}

fn label_row(t: &Tokens, width: f32, height: f32) -> Style {
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
    if current_direction() == ReadingDirection::Rtl {
        base.padding_right(padding)
    } else {
        base.padding_left(padding)
    }
}

pub(super) fn checkbox(theme: &Theme, s: &Look<CheckState>) -> Style {
    let t = theme.get::<Tokens>();
    let side = check_side(&t, s);
    label_row(&t, side, side).disabled(style().opacity(t.sizes.disabled_opacity))
}

pub(super) fn switch(theme: &Theme, s: &Look<CheckState>) -> Style {
    let t = theme.get::<Tokens>();
    let (width, height) = switch_size(&t, s);
    label_row(&t, width, height)
}

fn pressed_fill(theme: &Theme, t: &Tokens, s: &Look<CheckState>, fill: Color) -> Color {
    if s.input.pressed && !s.input.disabled {
        overlay(theme, fill, t.colors.overlay_pressed)
    } else {
        fill
    }
}

fn check_surface(theme: &Theme, t: &Tokens, s: &Look<CheckState>, radius: f32) -> Style {
    let c = &t.colors;
    let on = s.checked != CheckedState::Unchecked;
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
        if s.input.focus_visible {
            base.merge(&ring(t, c.focus_ring))
        } else {
            base
        },
    )
}

pub(super) fn checkbox_indicator(theme: &Theme, s: &Look<CheckState>) -> Style {
    let t = theme.get::<Tokens>();
    let side = check_side(&t, s);
    inline_start(check_surface(theme, &t, s, t.radius.inner), 0.0).top(pct(50) - side / 2.0)
}

fn centered_mark(width: f32, height: f32, visible: bool) -> Style {
    style()
        .position(Position::Absolute)
        .left(pct(50) - width / 2.0)
        .top(pct(50) - height / 2.0)
        .size(width, height)
        .opacity(if visible { 1.0 } else { 0.0 })
}

pub(super) fn checkbox_check(theme: &Theme, s: &Look<CheckState>) -> Style {
    let t = theme.get::<Tokens>();
    let side = if small(s.size) {
        t.spacing.s3
    } else {
        t.typography.size_md
    };
    let ink = t.colors.on_accent;
    centered_mark(side, side, s.checked == CheckedState::Checked).paint(move |d, size| {
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

pub(super) fn checkbox_mixed(theme: &Theme, s: &Look<CheckState>) -> Style {
    let t = theme.get::<Tokens>();
    let width = if small(s.size) {
        t.typography.size_xs
    } else {
        t.spacing.s3
    };
    centered_mark(
        width,
        t.spacing.s0_5,
        s.checked == CheckedState::Indeterminate,
    )
    .radius(t.radius.full)
    .background(t.colors.on_accent)
}

pub(super) fn switch_track(theme: &Theme, s: &Look<CheckState>) -> Style {
    let t = theme.get::<Tokens>();
    let (width, height) = switch_size(&t, s);
    let c = &t.colors;
    let on = s.checked.is_checked();
    let fill = if on { c.accent } else { c.border_emphasized };
    let fill = if s.input.hovered && !s.input.disabled {
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
    fast(
        &t,
        if s.input.focus_visible {
            base.merge(&ring(&t, c.focus_ring))
        } else {
            base
        },
    )
}

pub(super) fn switch_thumb(theme: &Theme, s: &Look<CheckState>) -> Style {
    let t = theme.get::<Tokens>();
    let on = s.checked.is_checked();
    let (width, height) = switch_size(&t, s);
    let (base_side, inset, on_side) = if small(s.size) {
        (t.typography.size_md, t.spacing.s0_5, t.spacing.s4)
    } else {
        (t.spacing.s4, t.spacing.s1, t.spacing.s5)
    };
    let movement = width - t.spacing.s0_5 - on_side / 2.0 - inset - base_side / 2.0;
    let sign = if current_direction() == ReadingDirection::Rtl {
        -1.0
    } else {
        1.0
    };
    let base = inline_start(style().position(Position::Absolute), inset)
        .top((height - base_side) / 2.0)
        .size(base_side, base_side)
        .radius(t.radius.full)
        .background(pressed_fill(theme, &t, s, t.colors.bg_surface))
        .translate_x(if on { sign * movement } else { 0.0 })
        .scale(if on { on_side / base_side } else { 1.0 });
    fast(&t, base)
}

pub(super) fn radio_group(r: RadioGroupRecipes) -> RadioGroupRecipes {
    r.group(|_| group)
        .item(|_| item)
        .indicator(|_| indicator)
        .mark(|_| mark)
        .description(|_| description)
}

fn group(theme: &Theme, orientation: &Look<Orientation>) -> Style {
    let t = theme.get::<Tokens>();
    if **orientation == Orientation::Horizontal {
        style().direction(Direction::Row).gap(t.spacing.s5)
    } else {
        style().direction(Direction::Column).gap(t.spacing.s2)
    }
}

fn item(theme: &Theme, s: &Look<CheckState>) -> Style {
    let t = theme.get::<Tokens>();
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

fn indicator(theme: &Theme, s: &Look<CheckState>) -> Style {
    let t = theme.get::<Tokens>();
    check_surface(theme, &t, s, t.radius.full)
        .position(Position::Relative)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
}

fn mark(theme: &Theme, s: &Look<CheckState>) -> Style {
    let t = theme.get::<Tokens>();
    let side = if small(s.size) {
        t.spacing.s2
    } else {
        t.typography.size_xs
    };
    centered_mark(side, side, s.checked.is_checked())
        .radius(t.radius.full)
        .background(t.colors.on_accent)
}

fn description(theme: &Theme, _: &Look<CheckState>) -> Style {
    let t = theme.get::<Tokens>();
    let text = t.typography.supporting;
    style()
        .font_size(text.size_px)
        .line_height(text.line_height_px / text.size_px)
        .weight(text.weight)
        .color(t.colors.text_secondary)
}
