//! Role: The Pop looks of the feedback and indicators: toasts, alerts, empty states,
//! cards,
//! badges, avatars, loading indicators, skeletons and key caps.

use crate::color;
use crate::recipe_parts::{chrome, tone_hue};
use crate::variants::{AvatarHue, AvatarSize, BadgeVariant, Hue, SectionNumber, SpinnerVariant};
use kivact::custom::PartState;
use kivact::geometry::Rect;
use kivact::interaction::{ToastState, Tone};
use kivact::style::color as core;
use kivact::style::{
    Align, Color, Display, FlexDirection, FontStyle, Length, Style, Theme, Track, pct, style,
};
use kivact::style::{BackgroundOrigin, Paint, Shadow, Transform, linear_gradient};
use kivact::ui::theme::{
    AlertState, AlertTheme, AvatarState, AvatarTheme, BadgeState, BadgeTheme, CardState, CardTheme,
    EmptyStateState, EmptyStateTheme, KbdTheme, Look, SkeletonState, SkeletonTheme, SpinnerState,
    SpinnerTheme, ToastTheme,
};
use kivact::ui::{AvatarShape, ControlSize, SkeletonShape};

/// Install the kit's recipes of the feedback and indicators.
pub(crate) fn install(theme: Theme) -> Theme {
    theme
        .with(
            ToastTheme::default()
                .region(toast_region)
                .toast(toast_card)
                .title(toast_title)
                .body(toast_body)
                .icon(toast_icon)
                .close(toast_close),
        )
        .with(
            AlertTheme::default()
                .root(alert_root)
                .title(alert_title)
                .description(alert_description)
                .icon(alert_icon)
                .actions(alert_actions)
                .content(alert_content),
        )
        .with(
            EmptyStateTheme::default()
                .root(empty_root)
                .title(empty_title)
                .description(empty_description),
        )
        .with(CardTheme::default().root(card_root).content(card_content))
        .with(BadgeTheme::default().root(badge).label(badge_label))
        .with(AvatarTheme::default().root(avatar))
        .with(SkeletonTheme::default().root(skeleton))
        .with(
            SpinnerTheme::default()
                .root(spinner)
                .reduced_period(Some(1000.0)),
        )
        .with(KbdTheme::default().root(kbd))
}

/// The stack at the bottom right of the window, above the page.
fn toast_region(_theme: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .position(kivact::style::Position::Fixed)
        .z_index(100)
        .max_width(pct(100) - 16.0)
        .align_items(Align::End)
        .gap(8)
        .bottom(16)
        .right(16)
}

/// A card in the hue's color, which its icon takes.
fn toast_card(_theme: &Theme, s: &Look<ToastState>, _base: Style) -> Style {
    chrome()
        .flex_direction(FlexDirection::Row)
        .width(356)
        .max_width(pct(100))
        .padding_left(16)
        .padding_right(12)
        .padding_y(14)
        .align_items(Align::Start)
        .gap(10)
        .color(tone_hue(s.tone).colors().1)
}

fn toast_title(_theme: &Theme, _: &Look<ToastState>, _base: Style) -> Style {
    style().font_weight(500).color(color::TEXT)
}

fn toast_body(_theme: &Theme, _: &Look<ToastState>, _base: Style) -> Style {
    style().font_size(13).color(core::TEXT_MUTED)
}

/// The tone's icon: 16 px, in the card's color, level with the title.
fn toast_icon(_theme: &Theme, _: &Look<ToastState>, _base: Style) -> Style {
    style().size(16, 16).flex_shrink(0).margin_top(2)
}

/// The 28 px close button around the theme's `x` icon.
fn toast_close(_theme: &Theme, _: &Look<ToastState>, _base: Style) -> Style {
    style()
        .size(28, 28)
        .flex_shrink(0)
        .border_radius(8)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(kivact::style::Justify::Center)
        .color(color::TEXT_MUTED)
        .hover(style().background(color::SURFACE_ACTIVE).color(color::TEXT))
        .active(style().opacity(0.7))
}

fn alert_root(_theme: &Theme, s: &Look<AlertState>, base: Style) -> Style {
    let hue = tone_hue(s.tone);
    base.display(Display::Grid)
        .grid_template_columns((20, Track::fr(1), Length::Auto))
        .padding_x(16)
        .padding_y(12)
        .gap(12)
        .border_radius(10)
        .background(color::SURFACE)
        .border(
            1,
            if hue == Hue::Red {
                color::RED_FOREGROUND_ALERT_BORDER
            } else {
                color::LINE
            },
        )
}

fn alert_title(_theme: &Theme, s: &Look<AlertState>, _base: Style) -> Style {
    style()
        .grid_column(2)
        .grid_row(1)
        .font_size(14)
        .font_weight(500)
        .line_height(1.5)
        .color(if s.tone == Tone::Danger {
            color::RED_FOREGROUND
        } else {
            color::TEXT
        })
}

fn alert_description(_theme: &Theme, _: &Look<AlertState>, _base: Style) -> Style {
    style()
        .grid_column(2)
        .grid_row(2)
        .margin_top(-10)
        .font_size(13)
        .line_height(1.5)
        .color(core::TEXT_MUTED)
}

fn alert_icon(_theme: &Theme, s: &Look<AlertState>, _base: Style) -> Style {
    style()
        .grid_column(1)
        .grid_row((1, 3))
        .align_self(if s.tone == Tone::Danger {
            Align::Center
        } else {
            Align::Start
        })
        .color(tone_hue(s.tone).colors().1)
        .margin_top(2)
}

fn alert_actions(_theme: &Theme, _: &Look<AlertState>, base: Style) -> Style {
    base.grid_column(3).grid_row((1, 3))
}

fn alert_content(_theme: &Theme, _: &Look<AlertState>, base: Style) -> Style {
    base.grid_column((1, 4)).grid_row(3)
}

fn empty_root(_theme: &Theme, _: &Look<EmptyStateState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .padding_x(12)
        .padding_top(24)
        .padding_bottom(12)
        .gap(6)
        .align_items(Align::Center)
}

fn empty_title(_theme: &Theme, _: &Look<EmptyStateState>, _base: Style) -> Style {
    style().font_size(16).font_weight(600)
}

fn empty_description(_theme: &Theme, _: &Look<EmptyStateState>, _base: Style) -> Style {
    style().color(color::TEXT_MUTED).margin_bottom(12)
}

/// A card: the panel padding, the surface and a faint shadow.
fn card_root(_theme: &Theme, _: &Look<CardState>, base: Style) -> Style {
    base.padding(crate::size::PANEL_PADDING)
        .background(color::SURFACE)
        .border_radius(crate::size::PANEL_RADIUS)
        .shadow(Shadow::new(0, 1, 3, core::SHADOW))
}

fn card_content(_theme: &Theme, _: &Look<CardState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .padding(0)
        .gap(0)
}

/// The badge of a tone or a hue: a soft tint, or the shape a variant gives it.
fn badge(_theme: &Theme, look: &Look<BadgeState>, _base: Style) -> Style {
    if look.variant::<SectionNumber>().is_some() {
        return style()
            .size(44, 30)
            .flex_direction(FlexDirection::Row)
            .align_items(Align::Center)
            .justify_content(kivact::style::Justify::Center)
            .border_radius(6)
            .transform(Transform::new().skew_x(-14))
            .color(Color::WHITE)
            .background(linear_gradient(90).colors([color::BLUE, color::CYAN]))
            .paint(move |d, _| {
                d.rounded_rect(Rect::new(48.0, 4.0, 5.0, 22.0), 2.0, color::YELLOW)
            });
    }
    let hue = look.variant::<Hue>().unwrap_or_else(|| tone_hue(look.tone));
    let variant = look.variant::<BadgeVariant>().unwrap_or_default();
    let base = badge_style(hue, variant);
    if variant == BadgeVariant::Dot {
        let color = hue.colors().1;
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

fn badge_style(hue: Hue, variant: BadgeVariant) -> Style {
    let (bg, fg) = hue.colors();
    let s = style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(kivact::style::Justify::Center)
        .gap(4)
        .height(22)
        .flex_shrink(0)
        .padding_x(8)
        .border_radius(6)
        .font_size(12)
        .font_weight(500)
        .background(bg)
        .color(fg)
        .border(1, Color::TRANSPARENT)
        .text_wrap(false);
    match variant {
        BadgeVariant::Dot => s
            .padding_left(18)
            .background(Color::TRANSPARENT)
            .color(color::TEXT)
            .border_color(color::LINE),
        BadgeVariant::Solid => s
            .background(color::TEXT)
            .color(color::SURFACE)
            .font_weight(600),
        BadgeVariant::Slant => s
            // The gradient sits under the 1px border: keep it on the border box.
            .background(
                Paint::from(linear_gradient(90).colors([
                    hue.accent(),
                    match hue {
                        Hue::Yellow => color::YELLOW_GRADIENT_END,
                        Hue::Red => color::RED_GRADIENT_END,
                        Hue::Pink => color::PINK_GRADIENT_END,
                        _ => color::CYAN,
                    },
                ]))
                .origin(BackgroundOrigin::BorderBox),
            )
            .color(if hue == Hue::Yellow {
                color::ACCENT_TEXT
            } else {
                Color::WHITE
            })
            .border_radius(4)
            .padding_x(12)
            .font_style(FontStyle::Italic)
            .font_weight(800)
            .transform(Transform::new().skew_x(-16)),
        BadgeVariant::Count => s
            .height(20)
            .min_width(20)
            .padding_x(6)
            .border_radius(999)
            .background(if hue == Hue::Pink { color::PINK } else { bg })
            .color(if hue == Hue::Pink { Color::WHITE } else { fg }),
        _ => s,
    }
}

fn badge_label(_theme: &Theme, look: &Look<BadgeState>, _base: Style) -> Style {
    if look.variant::<SectionNumber>().is_some() {
        return style()
            .font_style(FontStyle::Italic)
            .font_weight(800)
            .font_size(16)
            .color(Color::WHITE)
            .transform(Transform::new().skew_x(14));
    }
    if look.variant::<BadgeVariant>() == Some(BadgeVariant::Slant) {
        style().transform(Transform::new().skew_x(16))
    } else {
        style()
    }
}

/// The avatar: a gradient disc of the hue, sized by its variant or its size.
fn avatar(_theme: &Theme, look: &Look<AvatarState>, _base: Style) -> Style {
    let size = look.variant::<AvatarSize>().map_or_else(
        || match look.size {
            ControlSize::ExtraSmall => 24.0,
            ControlSize::Small => 32.0,
            ControlSize::Medium => 40.0,
            ControlSize::Large => 56.0,
        },
        |v| v.0.max(16.0),
    );
    let hue = look.variant::<AvatarHue>().map_or(205.0, |v| v.0);
    style()
        .size(size, size)
        .flex_shrink(0)
        .border_radius(match look.shape {
            AvatarShape::Circle => 999.0,
            AvatarShape::Rounded => 8.0,
            AvatarShape::Square => 0.0,
        })
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(kivact::style::Justify::Center)
        .color(Color::WHITE)
        .background(linear_gradient(140).colors([
            crate::color::avatar_gradient_start(hue),
            crate::color::avatar_gradient_end(hue),
        ]))
        .font_style(FontStyle::Italic)
        .font_weight(800)
        .font_size(size * 0.36)
}

fn skeleton(_theme: &Theme, s: &Look<SkeletonState>, _base: Style) -> Style {
    let base = style()
        .flex_grow(1)
        .min_height(16)
        .width(pct(100))
        .background(core::SURFACE_ALT)
        .opacity(s.pulse);
    match s.shape {
        SkeletonShape::Circle => base.size(36, 36).border_radius(999),
        SkeletonShape::Rect => base.border_radius(0),
        _ => base.border_radius(6),
    }
}

/// The loading indicator: three bouncing dots, or a ring turning in the accent.
fn spinner(_theme: &Theme, s: &Look<SpinnerState>, _base: Style) -> Style {
    let phase = s.turn.to_radians();
    if s.variant::<SpinnerVariant>() == Some(SpinnerVariant::Dots) {
        let colors = [color::BLUE, color::CYAN, color::YELLOW];
        return style().size(34, 20).paint(move |d, _| {
            for (i, color) in colors.into_iter().enumerate() {
                let bounce = (phase - i as f64 * 0.95).sin().max(0.0) * 6.0;
                d.rounded_rect(
                    Rect::new(i as f64 * 13.0, 10.0 - bounce, 8.0, 8.0),
                    4.0,
                    color,
                );
            }
        });
    }
    let side = match s.size {
        ControlSize::ExtraSmall => 16,
        ControlSize::Small => 18,
        ControlSize::Medium => 20,
        ControlSize::Large => 32,
    };
    let (track, accent, turn) = (color::SURFACE_ACTIVE, color::BLUE, s.turn);
    style()
        .size(side, side)
        .border_radius(999)
        .rotate(turn)
        .paint(move |d, size| {
            let thickness = if size.width.min(size.height) >= 28.0 {
                3.0
            } else {
                2.0
            };
            let center = size.to_rect().center();
            let radius = (size.width.min(size.height) - thickness) / 2.0;
            let arc = |fraction: f64| {
                let mut path = kivact::draw::Path::new();
                for i in 0..=64 {
                    let angle = -std::f64::consts::FRAC_PI_2
                        + std::f64::consts::TAU * fraction * i as f64 / 64.0;
                    let point = kivact::geometry::Point::new(
                        center.x + radius * angle.cos(),
                        center.y + radius * angle.sin(),
                    );
                    path = if i == 0 {
                        path.move_to(point)
                    } else {
                        path.line_to(point)
                    };
                }
                path
            };
            let stroke = kivact::draw::Stroke::new(thickness)
                .cap(kivact::draw::LineCap::Round)
                .join(kivact::draw::LineJoin::Round);
            d.stroke_path(&arc(1.0), &stroke, track);
            d.stroke_path(&arc(0.25), &stroke, accent);
        })
}

/// A key cap of the keyboard shortcut hints.
fn kbd(_theme: &Theme, _: &Look<()>, _base: Style) -> Style {
    style()
        .min_height(20)
        .min_width(20)
        .padding_x(6)
        .border_radius(4)
        .background(color::SURFACE_MUTED)
        .border(1, color::LINE)
        .border_bottom(2, color::LINE_STRONG)
        .color(color::TEXT_MUTED)
        .font_size(12)
        .text_wrap(false)
}
