//! Role: Draw console badges and cut-corner avatars through standard recipes.
use crate::Tokens;
use crate::display::ToneExt;
use kivact::draw::Shape;
use kivact::geometry::Rect;
use kivact::style::{Align, Color, ColorToken, Justify, Position, Style, Theme, pct, style};
use kivact::ui::{AvatarState, BadgeState, Look};

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

/// Console avatar dimensions and application-selected fill.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AvatarVariant {
    pub side: f64,
    pub color: Color,
}
impl kivact::ui::Variant for AvatarVariant {}

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
pub(crate) fn avatar(t: &Theme, s: &Look<AvatarState>) -> Style {
    let variant = s
        .variant::<AvatarVariant>()
        .copied()
        .unwrap_or(AvatarVariant {
            side: match s.size {
                kivact::ui::ControlSize::ExtraSmall => 20.,
                kivact::ui::ControlSize::Small => 24.,
                kivact::ui::ControlSize::Medium => 36.,
                kivact::ui::ControlSize::Large => 48.,
            },
            color: t.colors().accent,
        });
    style()
        .size(variant.side, variant.side)
        .radius(0)
        .clip(false)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .font_family(t.ext::<Tokens>().display_family.as_str())
        .font_size(variant.side * 0.44)
        .weight(700)
        .italic(true)
        .color(t.colors().accent_text)
        .paint(move |d, s| {
            d.fill_shape(
                s.to_rect(),
                &Shape::polygon()
                    .point(0, 0)
                    .point(pct(100), 0)
                    .point(pct(100), pct(100) - variant.side / 4.)
                    .point(pct(100) - variant.side / 4., pct(100))
                    .point(0, pct(100)),
                variant.color,
            );
        })
}
pub(crate) fn avatar_status(t: &Theme, s: &Look<AvatarState>) -> Style {
    style()
        .position(Position::Absolute)
        .right(-2)
        .bottom(-2)
        .size(10, 10)
        .radius(0)
        .border(2, ColorToken::Surface)
        .background(s.status.unwrap_or_default().color(t))
}
pub(crate) fn avatar_overflow(t: &Theme, _: &Look<usize>) -> Style {
    style()
        .size(32, 32)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .font_family(t.ext::<Tokens>().mono_family.as_str())
        .font_size(10)
        .color(t.ext::<Tokens>().text_3)
        .background(t.ext::<Tokens>().surface_3)
        .clip_path(
            Shape::polygon()
                .point(0, 0)
                .point(pct(100), 0)
                .point(pct(100), pct(100) - 8)
                .point(pct(100) - 8, pct(100))
                .point(0, pct(100)),
        )
}
