//! Role: Pop button appearance and compact icon actions.
use crate::Tokens;
use kivact::draw::{Filter, Shadow, Transform, linear_gradient};
use kivact::motion::{Easing, Transition};
use kivact::style::{Align, Color, Direction, Justify, Style, Theme, TransitionProp, style};
use kivact::ui::{ButtonVariant, ControlSize};
/// A Pop look of a standard button beyond its [`ButtonVariant`]:
/// `kivact::ui::button(..).variant(ButtonTone::Pink)`. Another theme
/// draws the button's plain variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ButtonTone {
    /// The yellow highlight.
    Accent,
    /// The pink gradient.
    Pink,
    /// A quiet filled button.
    Soft,
    /// A translucent button over a picture.
    Glass,
    /// The navy button.
    Dark,
}

impl kivact::ui::Variant for ButtonTone {}

/// Draw a standard button slanted: `kivact::ui::button(..).variant(Slant)`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Slant;

impl kivact::ui::Variant for Slant {}

pub(crate) fn button_style(
    t: &Theme,
    variant: ButtonVariant,
    hue: Option<ButtonTone>,
    size: ControlSize,
    slant: bool,
) -> Style {
    let p = t.ext::<Tokens>();
    let (height, pad, font) = match size {
        ControlSize::ExtraSmall => (26, 8, 12),
        ControlSize::Small => (32, 12, 13),
        ControlSize::Medium => (36, 16, 14),
        ControlSize::Large => (40, 24, 14),
    };
    let mut s = style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .gap(8)
        .height(height)
        .padding_x(pad)
        .radius(if slant { 6 } else { 8 })
        .font_size(font)
        .weight(500)
        .text_wrap(false)
        .border(1, Color::TRANSPARENT)
        .shadow(Shadow::new(0, 0, 0, Color::TRANSPARENT))
        .cursor(kivact::style::Cursor::Pointer)
        .transition(
            TransitionProp::Paint,
            Transition::tween(150, Easing::EaseOut),
        );
    match (hue, variant) {
        (Some(ButtonTone::Accent | ButtonTone::Pink), _)
        | (None, ButtonVariant::Primary | ButtonVariant::Danger) => {
            let (a, b, fg) = match (hue, variant) {
                (Some(ButtonTone::Accent), _) => (
                    p.yellow,
                    crate::tokens::accent_gradient_end(),
                    crate::tokens::accent_text(),
                ),
                (Some(ButtonTone::Pink), _) => {
                    (p.pink, crate::tokens::pink_gradient_end(), Color::WHITE)
                }
                (_, ButtonVariant::Danger) => {
                    (p.red, crate::tokens::danger_gradient_end(), Color::WHITE)
                }
                _ => (p.blue, p.blue.mix_in(p.cyan, 0.3, t), Color::WHITE),
            };
            s = s
                .background(linear_gradient(90).colors([a, b]))
                .color(fg)
                .shadow(Shadow::new(0, 1, 2, crate::tokens::control_shadow()))
                .hover(style().filter(Filter::new().brightness(0.95)))
                .pressed(style().filter(Filter::new().brightness(0.9)));
        }
        (Some(ButtonTone::Soft), _) => {
            s = s
                .background(p.surface_active)
                .color(p.text)
                .hover(style().background(p.surface_active.mix_in(p.text, 0.15, t)))
                .pressed(style().opacity(0.85))
        }
        (None, ButtonVariant::Secondary | ButtonVariant::Outline) => {
            s = s
                .background(p.surface)
                .color(p.text)
                .border_color(p.line)
                .shadow(Shadow::new(0, 1, 2, crate::tokens::subtle_shadow()))
                .hover(style().background(p.surface_active))
                .pressed(style().background(p.surface_muted))
        }
        (None, ButtonVariant::Ghost | ButtonVariant::Link) => {
            s = s
                .background(Color::TRANSPARENT)
                .color(p.text)
                .hover(style().background(p.surface_active))
                .pressed(style().background(p.surface_muted))
        }
        (Some(ButtonTone::Dark), _) => {
            s = s.background(p.navy).color(Color::WHITE);
        }
        (Some(ButtonTone::Glass), _) => {
            s = s
                .background(crate::tokens::glass_surface())
                .color(Color::WHITE)
                .border_color(crate::tokens::glass_border())
                .backdrop_blur(6)
                .hover(style().background(crate::tokens::glass_hover()))
        }
    }
    if slant {
        s = s.transform(Transform::new().skew_x(-12));
    }
    s
}
