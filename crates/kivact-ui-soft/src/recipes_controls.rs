//! Role: Style the standard Soft action controls and register their recipes.
use super::recipe_style::{fast, overlay, ring};
use crate::Tokens;
use kivact::draw::LineCap;
use kivact::draw::Path;
use kivact::draw::Stroke;
use kivact::geometry::Point;
use kivact::style::Align;
use kivact::style::Color;
use kivact::style::Cursor;
use kivact::style::Direction;
use kivact::style::Justify;
use kivact::style::Position;
use kivact::style::Style;
use kivact::style::TextDecorationLine;
use kivact::style::Theme;
use kivact::style::pct;
use kivact::style::style;
use kivact::ui::ButtonState;
use kivact::ui::ButtonVariant;
use kivact::ui::ControlSize;
use kivact::ui::Look;
use kivact::ui::ToggleButtonState;
use kivact::ui::UiRecipes;

use super::{recipes_controls_choice as choice, recipes_controls_value as value};

pub(crate) fn register(recipes: UiRecipes) -> UiRecipes {
    recipes
        .button(|_| button)
        .button_content(|_| button_content)
        .button_icon(|_| button_icon)
        .button_busy(|_| button_busy)
        .toggle(|_| toggle)
        .checkbox(|_| choice::checkbox)
        .checkbox_indicator(|_| choice::checkbox_indicator)
        .checkbox_check(|_| choice::checkbox_check)
        .checkbox_mixed(|_| choice::checkbox_mixed)
        .switch(|_| choice::switch)
        .switch_track(|_| choice::switch_track)
        .switch_thumb(|_| choice::switch_thumb)
        .radio_group(choice::radio_group)
        .slider(value::slider)
        .progress(value::progress)
}

fn action(t: &Tokens, size: ControlSize, square: bool) -> Style {
    let (height, padding, radius, text) = match size {
        ControlSize::ExtraSmall => (
            t.sizes.control_xs,
            t.spacing.s2,
            t.radius.inner,
            t.typography.supporting,
        ),
        ControlSize::Small => (
            t.sizes.control_sm,
            t.spacing.s3,
            t.radius.element,
            t.typography.label,
        ),
        ControlSize::Medium => (
            t.sizes.control_md,
            t.spacing.s3,
            t.radius.element,
            t.typography.label,
        ),
        ControlSize::Large => (
            t.sizes.control_lg,
            t.spacing.s3,
            t.radius.element,
            t.typography.label,
        ),
    };
    let base = style()
        .position(Position::Relative)
        .height(height)
        .max_width(pct(100))
        .radius(radius)
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .gap(t.spacing.s2)
        .font_size(text.size_px)
        .line_height(text.line_height_px / text.size_px)
        .weight(t.typography.weight_medium)
        .disabled(
            style()
                .opacity(t.sizes.disabled_opacity)
                .cursor(Cursor::Default),
        );
    if square {
        base.width(height).shrink(0)
    } else {
        base.padding_x(padding)
    }
}

fn colors(t: &Tokens, variant: ButtonVariant) -> (Color, Color, Color) {
    let c = &t.colors;
    let (bg, fg) = match variant {
        ButtonVariant::Primary => (c.accent, c.on_accent),
        ButtonVariant::Secondary => (c.neutral, c.text_primary),
        ButtonVariant::Danger => (c.error_muted, c.on_error_muted),
        ButtonVariant::Link => (Color::TRANSPARENT, c.text_accent),
        _ => (Color::TRANSPARENT, c.text_primary),
    };
    (
        bg,
        fg,
        if variant == ButtonVariant::Danger {
            c.error
        } else {
            c.focus_ring
        },
    )
}

fn button(theme: &Theme, s: &Look<ButtonState>) -> Style {
    let t = theme.ext::<Tokens>();
    let variant = s.variant::<ButtonVariant>().copied().unwrap_or_default();
    let (bg, fg, focus) = colors(&t, variant);
    let base = action(&t, s.size, s.icon_only)
        .color(fg)
        .focus_visible(ring(&t, focus));
    let base = match variant {
        ButtonVariant::Outline => base.border(t.sizes.border_width, t.colors.border_emphasized),
        ButtonVariant::Link => {
            let base = base.height(t.typography.label.line_height_px).padding_x(0);
            if s.input.hovered && !s.loading {
                base.text_decoration(TextDecorationLine::UNDERLINE)
            } else {
                base
            }
        }
        _ => base,
    };
    let bg = if s.expanded == Some(true) {
        overlay(theme, bg, t.colors.overlay_pressed)
    } else {
        bg
    };
    let base = fast(&t, base.background(bg));
    if s.loading {
        // Busy also sets the behavior's disabled state, so preserve its full opacity.
        base.disabled(style().opacity(1.0).cursor(Cursor::Default))
    } else if variant == ButtonVariant::Link {
        base
    } else {
        press(theme, &t, base, bg)
    }
}

fn press(theme: &Theme, t: &Tokens, base: Style, bg: Color) -> Style {
    base.hover(style().background(overlay(theme, bg, t.colors.overlay_hover)))
        .pressed(
            style()
                .background(overlay(theme, bg, t.colors.overlay_pressed))
                .scale(t.sizes.press_scale),
        )
}

fn button_content(theme: &Theme, s: &Look<ButtonState>) -> Style {
    let t = theme.ext::<Tokens>();
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .gap(t.spacing.s2)
        .opacity(if s.loading { 0.0 } else { 1.0 })
}

fn button_icon(theme: &Theme, s: &Look<ButtonState>) -> Style {
    let t = theme.ext::<Tokens>();
    let side = match s.size {
        ControlSize::ExtraSmall => t.sizes.icon_xsm,
        ControlSize::Small | ControlSize::Medium => t.sizes.icon_sm,
        ControlSize::Large => t.sizes.icon_md,
    };
    style()
        .size(side, side)
        .shrink(0)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
}

fn button_busy(theme: &Theme, s: &Look<ButtonState>) -> Style {
    let t = theme.ext::<Tokens>();
    let (_, ink, _) = colors(
        &t,
        s.variant::<ButtonVariant>().copied().unwrap_or_default(),
    );
    let radius = t.sizes.icon_xsm / 2.0 - t.spacing.s0_5 / 2.0;
    let stroke_width = t.spacing.s0_5;
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

fn toggle(theme: &Theme, s: &Look<ToggleButtonState>) -> Style {
    let t = theme.ext::<Tokens>();
    let bg = if s.pressed {
        t.colors.overlay_pressed
    } else {
        Color::TRANSPARENT
    };
    let weight = if s.pressed {
        t.typography.weight_semibold
    } else {
        t.typography.weight_medium
    };
    let base = action(&t, s.size, s.icon_only)
        .weight(weight)
        .background(bg)
        .color(t.colors.text_primary)
        .focus_visible(ring(&t, t.colors.focus_ring));
    press(theme, &t, fast(&t, base), bg)
}
