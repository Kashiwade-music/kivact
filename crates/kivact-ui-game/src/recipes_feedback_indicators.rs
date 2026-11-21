//! Role: Draw the console's badge, avatar, facepile and loading placeholder recipes.
use crate::color::{NAVY, SURFACE_3, TEXT_3};
use crate::fonts::DISPLAY_FAMILY_VAR;
use crate::fonts::MONO_FAMILY_VAR;
use crate::shapes::cut_corner;
use crate::tone::ToneExt;
use crate::variants::{AvatarVariant, BadgeVariant, SkeletonVariant};
use kivact::draw::Layer;
use kivact::geometry::Rect;
use kivact::interaction::Tone;
use kivact::style::Shape;
use kivact::style::color::{ACCENT, ACCENT_TEXT, SURFACE};
use kivact::style::{
    Align, Color, Justify, Position, Style, Theme, linear_gradient, pct, radius, style,
};
use kivact::ui::theme::{
    AvatarGroupTheme, AvatarState, AvatarTheme, BadgeState, BadgeTheme, Look, SkeletonState,
    SkeletonTheme,
};
use kivact::ui::{ControlSize, SkeletonShape};

/// Install the badge, avatar, facepile and skeleton recipes.
pub(crate) fn install(theme: Theme) -> Theme {
    theme
        .with(BadgeTheme::default().root(badge).label(badge_label))
        .with(AvatarTheme::default().root(avatar).status(avatar_status))
        .with(AvatarGroupTheme::default().overflow(avatar_overflow))
        .with(SkeletonTheme::default().root(skeleton))
}

pub(crate) fn badge(t: &Theme, s: &Look<BadgeState>, _base: Style) -> Style {
    let variant = s.variant::<BadgeVariant>().unwrap_or_default();
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
        .border_radius(0)
        .font_family(t.get(&MONO_FAMILY_VAR).as_str().unwrap_or_default())
        .font_size(11.5)
        .color(if tone == Tone::Accent {
            NAVY
        } else if variant == BadgeVariant::Solid {
            ACCENT_TEXT
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

pub(crate) fn badge_label(_: &Theme, _: &Look<BadgeState>, _base: Style) -> Style {
    style()
}

pub(crate) fn avatar(t: &Theme, s: &Look<AvatarState>, _base: Style) -> Style {
    let variant = s.variant::<AvatarVariant>().unwrap_or(AvatarVariant {
        side: match s.size {
            ControlSize::ExtraSmall => 20.,
            ControlSize::Small => 24.,
            ControlSize::Medium => 36.,
            ControlSize::Large => 48.,
        },
        color: ACCENT,
    });
    style()
        .size(variant.side, variant.side)
        .border_radius(0)
        .clip(false)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .font_family(t.get(&DISPLAY_FAMILY_VAR).as_str().unwrap_or_default())
        .font_size(variant.side * 0.44)
        .font_weight(700)
        .font_style(kivact::style::FontStyle::Italic)
        .color(ACCENT_TEXT)
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

pub(crate) fn avatar_status(t: &Theme, s: &Look<AvatarState>, _base: Style) -> Style {
    style()
        .position(Position::Absolute)
        .right(-2)
        .bottom(-2)
        .size(10, 10)
        .border_radius(0)
        .border(2, SURFACE)
        .background(s.status.unwrap_or_default().color(t))
}

pub(crate) fn avatar_overflow(t: &Theme, _: &Look<usize>, _base: Style) -> Style {
    style()
        .size(32, 32)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .font_family(t.get(&MONO_FAMILY_VAR).as_str().unwrap_or_default())
        .font_size(10)
        .color(TEXT_3)
        .background(SURFACE_3)
        .clip_path(
            Shape::polygon()
                .point(0, 0)
                .point(pct(100), 0)
                .point(pct(100), pct(100) - 8)
                .point(pct(100) - 8, pct(100))
                .point(0, pct(100)),
        )
}

/// A console loading row with an avatar and two caption placeholders, or a
/// pulsing block of the shape's radius.
pub(crate) fn skeleton(_: &Theme, s: &Look<SkeletonState>, _base: Style) -> Style {
    if s.variant::<SkeletonVariant>().is_none() {
        let base = style()
            .min_height(16)
            .width(pct(100))
            .background(SURFACE_3)
            .opacity(s.pulse);
        return match s.shape {
            SkeletonShape::Circle => base.size(36, 36).border_radius(999),
            SkeletonShape::Rounded => base.border_radius(radius::MD),
            SkeletonShape::Text => base.border_radius(radius::SM),
            SkeletonShape::Rect => base.border_radius(0),
        };
    }
    let phase = (1.0 - s.pulse) * 2.0;
    style().height(36).width(pct(100)).paint(move |d, _| {
        for rect in [
            Rect::new(0., 0., 36., 36.),
            Rect::new(48., 4., 160., 10.),
            Rect::new(48., 22., 230., 10.),
        ] {
            let layer = if rect.x == 0.0 {
                Layer::new().area(rect).clip(&cut_corner(8.))
            } else {
                Layer::new().clip_rect(rect)
            };
            d.layer(layer, |d| {
                d.fill_rect(rect, SURFACE_3);
                let x = rect.x + rect.width * (phase * 3.0 - 1.0);
                d.fill_rect(
                    Rect::new(x, rect.y, rect.width * 0.3, rect.height),
                    linear_gradient(100).colors([
                        SURFACE_3,
                        kivact::style::color::SURFACE_ALT,
                        SURFACE_3,
                    ]),
                );
            });
        }
    })
}
