//! Role: Style the Astryx Neutral buttons and toggle buttons (the controls group's action parts).
use crate::color as c;
use crate::metrics as m;
use crate::recipe_parts::{fast, overlay, ring};
use crate::radius as r;
use crate::space as s;
use crate::type_scale as t;
use kivact::draw::LineCap;
use kivact::draw::Path;
use kivact::draw::Stroke;
use kivact::geometry::Point;
use kivact::style::{
    Align, Color, Cursor, FlexDirection, Justify, Position, Style, TextDecorationLine, Theme, pct,
    style,
};
use kivact::ui::theme::{
    ButtonState, ButtonTheme, Look, ToggleButtonState, ToggleButtonTheme,
};
use kivact::ui::{ButtonVariant, ControlSize};

/// The look of a button's fill, text and focus ring for `variant`.
fn colors(variant: ButtonVariant) -> (Color, Color, Color) {
    let (bg, fg) = match variant {
        ButtonVariant::Primary => (c::ACCENT, c::ON_ACCENT),
        ButtonVariant::Secondary => (c::NEUTRAL, c::TEXT_PRIMARY),
        ButtonVariant::Danger => (c::ERROR_MUTED, c::ON_ERROR_MUTED),
        ButtonVariant::Link => (Color::TRANSPARENT, c::TEXT_ACCENT),
        _ => (Color::TRANSPARENT, c::TEXT_PRIMARY),
    };
    let focus = if variant == ButtonVariant::Danger {
        c::ERROR
    } else {
        Color::var(&c::FOCUS_RING_VAR)
    };
    (bg, fg, focus)
}

/// The box of a button of `size`: its height, side padding, radius and text.
fn action(size: ControlSize, square: bool) -> Style {
    let (height, padding, radius, text) = match size {
        ControlSize::ExtraSmall => (m::CONTROL_XS, s::SPACE_S2, r::RADIUS_INNER, t::SUPPORTING),
        ControlSize::Small => (m::CONTROL_SM, s::SPACE_S3, r::RADIUS_ELEMENT, t::LABEL),
        ControlSize::Medium => (m::CONTROL_MD, s::SPACE_S3, r::RADIUS_ELEMENT, t::LABEL),
        ControlSize::Large => (m::CONTROL_LG, s::SPACE_S3, r::RADIUS_ELEMENT, t::LABEL),
    };
    let base = style()
        .position(Position::Relative)
        .height(height)
        .max_width(pct(100))
        .border_radius(radius)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .gap(s::SPACE_S2)
        .font_size(text.size_px)
        .line_height(text.line_height_px / text.size_px)
        .font_weight(t::WEIGHT_MEDIUM)
        .disabled(
            style()
                .opacity(m::DISABLED_OPACITY)
                .cursor(Cursor::Default),
        );
    if square {
        base.width(height).flex_shrink(0)
    } else {
        base.padding_x(padding)
    }
}

/// The hover and pressed looks of a filled control over its `fill`.
fn press(theme: &Theme, base: Style, fill: Color) -> Style {
    base.hover(style().background(overlay(theme, fill, c::OVERLAY_HOVER)))
        .active(
            style()
                .background(overlay(theme, fill, c::OVERLAY_PRESSED))
                .scale(m::PRESS_SCALE),
        )
}

fn button(theme: &Theme, s_: &Look<ButtonState>, _base: Style) -> Style {
    let variant = s_.variant::<ButtonVariant>().unwrap_or_default();
    let (bg, fg, focus) = colors(variant);
    let base = action(s_.size, s_.is_icon_only())
        .color(fg)
        .focus_visible(ring(focus));
    let base = match variant {
        ButtonVariant::Outline => base.border(m::BORDER_WIDTH, c::BORDER_EMPHASIZED),
        ButtonVariant::Link => {
            let base = base.height(t::LABEL.line_height_px).padding_x(0);
            if s_.input.is_hovered() && !s_.busy {
                base.text_decoration(TextDecorationLine::UNDERLINE)
            } else {
                base
            }
        }
        _ => base,
    };
    let bg = if s_.expanded == Some(true) {
        overlay(theme, bg, c::OVERLAY_PRESSED)
    } else {
        bg
    };
    let base = fast(base.background(bg));
    if s_.busy {
        // Busy also sets the behavior's disabled state, so preserve its full opacity.
        base.disabled(style().opacity(1.0).cursor(Cursor::Default))
    } else if variant == ButtonVariant::Link {
        base
    } else {
        press(theme, base, bg)
    }
}

fn button_content(_theme: &Theme, s_: &Look<ButtonState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .gap(s::SPACE_S2)
        .opacity(if s_.busy { 0.0 } else { 1.0 })
}

fn button_icon(_theme: &Theme, s_: &Look<ButtonState>, _base: Style) -> Style {
    let side = match s_.size {
        ControlSize::ExtraSmall => m::ICON_XSM,
        ControlSize::Small | ControlSize::Medium => m::ICON_SM,
        ControlSize::Large => m::ICON_MD,
    };
    style()
        .size(side, side)
        .flex_shrink(0)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
}

fn button_busy(theme: &Theme, s_: &Look<ButtonState>, _base: Style) -> Style {
    let (_, ink, _) = colors(s_.variant::<ButtonVariant>().unwrap_or_default());
    let stroke_width = theme.get(&s::SPACE_S0_5_VAR);
    let radius = m::ICON_XSM / 2.0 - stroke_width / 2.0;
    let side = radius * 2.0 + stroke_width * 2.0;
    style()
        .position(Position::Absolute)
        .left(pct(50) - side / 2.0)
        .top(pct(50) - side / 2.0)
        .size(side, side)
        .paint(move |d, size| {
            let center = Point::new(size.width / 2.0, size.height / 2.0);
            let stroke = Stroke::new(stroke_width).cap(LineCap::Round);
            let track = Path::new().arc(center, radius, 0.0, 360.0, false);
            let arc = Path::new().arc(center, radius, -90.0, 45.0, false);
            d.stroke_path(&track, &stroke, ink.with_alpha(0.3));
            d.stroke_path(&arc, &stroke, ink);
        })
}

fn toggle(theme: &Theme, s_: &Look<ToggleButtonState>, _base: Style) -> Style {
    let bg = if s_.pressed {
        c::OVERLAY_PRESSED
    } else {
        Color::TRANSPARENT
    };
    let weight = if s_.pressed {
        t::WEIGHT_SEMIBOLD
    } else {
        t::WEIGHT_MEDIUM
    };
    let base = action(s_.size, s_.is_icon_only())
        .font_weight(weight)
        .background(bg)
        .color(c::TEXT_PRIMARY)
        .focus_visible(ring(Color::var(&c::FOCUS_RING_VAR)));
    press(theme, fast(base), bg)
}

/// Install the kit's recipes of the action controls (button and toggle button).
pub(crate) fn install(theme: Theme) -> Theme {
    theme
        .with(
            ButtonTheme::default()
                .root(button)
                .content(button_content)
                .icon(button_icon)
                .busy(button_busy),
        )
        .with(ToggleButtonTheme::default().root(toggle))
}
