//! Role: The style helpers that more than one recipe group of the Pop kit shares.

use crate::color;
use kivact::motion::{Easing, Transition};
use kivact::style::color as core;
use kivact::style::{Align, Color, FlexDirection, Justify, Style, TransitionProp, style};
use kivact::style::{BackgroundOrigin, Filter, Paint, Shadow, Transform, linear_gradient};
use kivact::ui::{ButtonVariant, ControlSize};

/// The button look of the kit for a variant, a tone and a size: the fill,
/// the hover and press looks, and the slant of a [`Slant`](crate::Slant) button.
pub(crate) fn button_style(
    variant: ButtonVariant,
    hue: Option<crate::ButtonTone>,
    size: ControlSize,
    slant: bool,
) -> Style {
    use crate::ButtonTone;
    let mut s = button_base(size, slant);
    match (hue, variant) {
        (Some(ButtonTone::Accent | ButtonTone::Pink), _)
        | (None, ButtonVariant::Primary | ButtonVariant::Danger) => {
            let (a, b, fg) = match (hue, variant) {
                (Some(ButtonTone::Accent), _) => (
                    color::YELLOW,
                    color::ACCENT_GRADIENT_END,
                    color::ACCENT_TEXT,
                ),
                (Some(ButtonTone::Pink), _) => {
                    (color::PINK, color::PINK_GRADIENT_END, Color::WHITE)
                }
                (_, ButtonVariant::Danger) => {
                    (color::RED, color::DANGER_GRADIENT_END, Color::WHITE)
                }
                _ => (color::BLUE, color::BLUE_GRADIENT_END, Color::WHITE),
            };
            s = s
                // The gradient sits under the 1px border: keep it on the border box.
                .background(
                    Paint::from(linear_gradient(90).colors([a, b]))
                        .origin(BackgroundOrigin::BorderBox),
                )
                .color(fg)
                .shadow(Shadow::new(0, 1, 2, color::CONTROL_SHADOW))
                .hover(style().filter(Filter::new().brightness(0.95)))
                .active(style().filter(Filter::new().brightness(0.9)));
        }
        (Some(ButtonTone::Soft), _) => {
            s = s
                .background(color::SURFACE_ACTIVE)
                .color(color::TEXT)
                .hover(style().background(color::SURFACE_ACTIVE_BUTTON_HOVER))
                .active(style().opacity(0.85))
        }
        (None, ButtonVariant::Secondary | ButtonVariant::Outline) => {
            s = s
                .background(color::SURFACE)
                .color(color::TEXT)
                .border_color(color::LINE)
                .shadow(Shadow::new(0, 1, 2, color::SUBTLE_SHADOW))
                .hover(style().background(color::SURFACE_ACTIVE))
                .active(style().background(color::SURFACE_MUTED))
        }
        (None, ButtonVariant::Ghost | ButtonVariant::Link) => {
            s = s
                .background(Color::TRANSPARENT)
                .color(color::TEXT)
                .hover(style().background(color::SURFACE_ACTIVE))
                .active(style().background(color::SURFACE_MUTED))
        }
        (Some(ButtonTone::Dark), _) => {
            s = s.background(color::NAVY).color(Color::WHITE);
        }
        (Some(ButtonTone::Glass), _) => {
            s = s
                .background(color::GLASS_SURFACE)
                .color(Color::WHITE)
                .border_color(color::GLASS_BORDER)
                .backdrop_blur(6)
                .hover(style().background(color::GLASS_HOVER))
        }
    }
    if slant {
        s = s.transform(Transform::new().skew_x(-12));
    }
    s
}

fn button_base(size: ControlSize, slant: bool) -> Style {
    let (height, pad, font) = match size {
        ControlSize::ExtraSmall => (26, 8, 12),
        ControlSize::Small => (32, 12, 13),
        ControlSize::Medium => (36, 16, 14),
        ControlSize::Large => (40, 24, 14),
    };
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .gap(8)
        .height(height)
        .padding_x(pad)
        .border_radius(if slant { 6 } else { 8 })
        .font_size(font)
        .font_weight(500)
        .text_wrap(false)
        .border(1, Color::TRANSPARENT)
        .shadow(Shadow::new(0, 0, 0, Color::TRANSPARENT))
        .cursor(kivact::style::Cursor::Pointer)
        .transition(TransitionProp::All, Transition::tween(150, Easing::EaseOut))
}

/// The row of a check: the drawn part beside the label.
pub(crate) fn check_style() -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(12)
        .min_height(22)
        .color(core::TEXT)
}

/// The option of a segmented choice (a pill) or a tab (an underline).
pub(crate) fn choice_option(pill: bool) -> Style {
    let s = style()
        .height(if pill { 30 } else { 36 })
        .padding_x(12)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .gap(6)
        .font_size(14)
        .font_weight(500)
        .color(color::TEXT_MUTED)
        .hover(style().color(color::TEXT));
    if pill {
        s.border_radius(7).selected(
            style()
                .background(color::SELECTED_SEGMENT)
                .color(color::TEXT)
                .shadow(Shadow::new(0, 1, 3, core::SHADOW)),
        )
    } else {
        s.border_bottom(2, Color::TRANSPARENT)
            .selected(style().border_bottom(2, color::BLUE).color(color::TEXT))
    }
}

/// The track of a choice list: a pill track or an underlined row.
pub(crate) fn choice_list(pill: bool) -> Style {
    if pill {
        style()
            .padding(3)
            .border_radius(10)
            .background(color::SURFACE_ACTIVE)
    } else {
        style().gap(4).border_bottom(1, color::LINE)
    }
}

/// A raised surface: the background, the rule and the corner of a popup or a card of the kit.
pub(crate) fn chrome() -> Style {
    style()
        .background(color::SURFACE)
        .border(1, color::LINE)
        .border_radius(10)
        .color(color::TEXT)
        .shadow(Shadow::new(0, 10, 15, core::SHADOW))
}

/// The hue a semantic tone draws in: neutral and info blue, success mint,
/// warning yellow, danger red and accent pink.
pub(crate) fn tone_hue(tone: kivact::interaction::Tone) -> crate::variants::Hue {
    use crate::variants::Hue;
    use kivact::interaction::Tone;
    match tone {
        Tone::Neutral | Tone::Info => Hue::Blue,
        Tone::Success => Hue::Mint,
        Tone::Warning => Hue::Yellow,
        Tone::Danger => Hue::Red,
        Tone::Accent => Hue::Pink,
    }
}
