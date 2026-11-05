//! Role: Draw the Game badge treatments.
use crate::Tokens;
use crate::display::ToneExt;
use kivact::draw::Shape;
use kivact::geometry::Rect;
use kivact::style::{Align, Color, Style, Theme, pct, style};
use kivact::ui::{BadgeState, Look};

/// Status, version and environment badge treatments.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeVariant {
    #[default]
    Dot,
    Plain,
    Solid,
    Outline,
}
impl kivact::ui::Variant for BadgeVariant {}

pub(crate) fn badge(t: &Theme, s: &Look<BadgeState>) -> Style {
    let variant = s.variant::<BadgeVariant>().copied().unwrap_or_default();
    let tone = s.tone;
    let dot = variant == BadgeVariant::Dot;
    let mut result = style()
        .height(20)
        .min_height(20)
        .align_self(Align::Start)
        .padding_x(9)
        .padding_left(if dot { 20 } else { 9 })
        .gap(5)
        .align_items(Align::Center)
        .radius(0)
        .font_family(t.ext::<Tokens>().mono_family.as_str())
        .font_size(11.5)
        .color(if tone == kivact::interaction::Tone::Accent {
            t.ext::<Tokens>().navy
        } else if variant == BadgeVariant::Solid {
            t.colors().accent_text
        } else {
            tone.color(t)
        })
        .background(match variant {
            BadgeVariant::Solid => tone.color(t),
            BadgeVariant::Outline => Color::TRANSPARENT,
            _ => tone.weak(t),
        });
    if variant == BadgeVariant::Outline {
        result = result.border(1, tone.color(t));
    } else {
        result = result.clip_path(
            Shape::polygon()
                .point(4, 0)
                .point(pct(100), 0)
                .point(pct(100) - 4, pct(100))
                .point(0, pct(100)),
        );
    }
    if dot {
        let color = tone.color(t);
        result = result.paint(move |d, _| d.fill_rect(Rect::new(9., 7., 6., 6.), color));
    }
    result
}

pub(crate) fn badge_label(_: &Theme, _: &Look<BadgeState>) -> Style {
    style()
}
