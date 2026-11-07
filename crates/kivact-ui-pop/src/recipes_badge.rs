//! Role: Preserve Pop badge hues and visual variants on the standard badge.
use crate::{Hue, Tokens};
use kivact::draw::{BackgroundOrigin, Paint, Transform, linear_gradient};
use kivact::geometry::Rect;
use kivact::state::{IntoValue, Value};
use kivact::style::{Align, Color, Direction, Justify, Style, Theme, style};
use kivact::ui::{BadgeState, Look, Variant};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeVariant {
    #[default]
    Soft,
    Dot,
    Solid,
    Slant,
    Count,
}
impl Variant for BadgeVariant {}

/// Reactive Pop badge hue supplied as a standard component variant.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BadgeHue(pub Value<Hue>);
impl BadgeHue {
    pub fn new(hue: impl IntoValue<Hue>) -> Self {
        Self(hue.into_value())
    }
}
impl Variant for BadgeHue {}

pub(crate) fn badge(t: &Theme, look: &Look<BadgeState>) -> Style {
    let p = t.ext::<Tokens>();
    if look.variant::<crate::SectionNumber>().is_some() {
        let yellow = p.yellow;
        return style()
            .size(44, 30)
            .direction(Direction::Row)
            .align_items(Align::Center)
            .justify_content(Justify::Center)
            .radius(6)
            .transform(Transform::new().skew_x(-14))
            .color(Color::WHITE)
            .background(linear_gradient(90).colors([p.blue, p.cyan]))
            .paint(move |d, _| d.rounded_rect(Rect::new(48.0, 4.0, 5.0, 22.0), 2.0, yellow));
    }
    let hue = look.variant::<BadgeHue>().map_or_else(
        || {
            look.variant::<Hue>()
                .copied()
                .unwrap_or_else(|| match look.tone {
                    kivact::interaction::Tone::Neutral | kivact::interaction::Tone::Info => {
                        Hue::Blue
                    }
                    kivact::interaction::Tone::Success => Hue::Mint,
                    kivact::interaction::Tone::Warning => Hue::Yellow,
                    kivact::interaction::Tone::Danger => Hue::Red,
                    kivact::interaction::Tone::Accent => Hue::Pink,
                })
        },
        |h| h.0.get(),
    );
    let variant = look.variant::<BadgeVariant>().copied().unwrap_or_default();
    let base = badge_style(t, hue, variant);
    if variant == BadgeVariant::Dot {
        let color = p.hue(hue).1;
        base.paint(move |d, size| {
            d.rounded_rect(
                Rect::new(8.0, (size.height - 6.0) / 2.0, 6.0, 6.0),
                3.0,
                color,
            )
        })
    } else {
        base
    }
}
fn badge_style(t: &Theme, hue: Hue, variant: BadgeVariant) -> Style {
    let p = t.ext::<Tokens>();
    let (bg, fg) = p.hue(hue);
    let s = style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .gap(4)
        .height(22)
        .shrink(0)
        .padding_x(8)
        .radius(6)
        .font_size(12)
        .weight(500)
        .background(bg)
        .color(fg)
        .border(1, Color::TRANSPARENT)
        .text_wrap(false);
    match variant {
        BadgeVariant::Dot => s
            .padding_left(18)
            .background(Color::TRANSPARENT)
            .color(p.text)
            .border_color(p.line),
        BadgeVariant::Solid => s.background(p.text).color(p.surface).weight(600),
        BadgeVariant::Slant => s
            // The gradient sits under the 1px border: keep it on the border box.
            .background(
                Paint::from(linear_gradient(90).colors([
                    p.accent(hue),
                    match hue {
                        Hue::Yellow => crate::tokens::yellow_gradient_end(),
                        Hue::Red => crate::tokens::red_gradient_end(),
                        Hue::Pink => crate::tokens::pink_gradient_end(),
                        _ => p.cyan,
                    },
                ]))
                .origin(BackgroundOrigin::BorderBox),
            )
            .color(if hue == Hue::Yellow {
                crate::tokens::accent_text()
            } else {
                Color::WHITE
            })
            .radius(4)
            .padding_x(12)
            .font_family("Barlow, sans-serif")
            .italic(true)
            .weight(800)
            .transform(Transform::new().skew_x(-16)),
        BadgeVariant::Count => s
            .height(20)
            .min_width(20)
            .padding_x(6)
            .radius(999)
            .background(if hue == Hue::Pink { p.pink } else { bg })
            .color(if hue == Hue::Pink { Color::WHITE } else { fg }),
        _ => s,
    }
}
pub(crate) fn label(_: &Theme, look: &Look<BadgeState>) -> Style {
    if look.variant::<crate::SectionNumber>().is_some() {
        return style()
            .font_family("Barlow, sans-serif")
            .italic(true)
            .weight(800)
            .font_size(16)
            .color(Color::WHITE)
            .transform(Transform::new().skew_x(14));
    }
    if look.variant::<BadgeVariant>() == Some(&BadgeVariant::Slant) {
        style().transform(Transform::new().skew_x(16))
    } else {
        style()
    }
}
