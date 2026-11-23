//! Role: Lyra recipes of the avatar, its image and status, and the avatar group's
//! members and overflow count.
//!
//! The looks of avatar.tsx. The standard components draw them with
//! `kivact::ui::avatar` and `avatar_group`.

use crate::color::{BACKGROUND, DESTRUCTIVE, MUTED, MUTED_FOREGROUND, PRIMARY, PRIMARY_FOREGROUND};
use kivact::interaction::Tone;
use kivact::style::{
    Align, Color, FlexDirection, Justify, Position, Style, Theme, color, pct, radius, style,
};
use kivact::ui::theme::{AvatarGroupTheme, AvatarState, AvatarTheme, Look, avatar_group};
use kivact::ui::{AvatarShape, ControlSize};

/// The side of an avatar: 20, 24, 32 or 40 px by size.
fn side(size: ControlSize) -> f64 {
    match size {
        ControlSize::ExtraSmall => 20.0,
        ControlSize::Small => 24.0,
        ControlSize::Medium => 32.0,
        ControlSize::Large => 40.0,
    }
}

/// The corner radius of an avatar's shape: a circle, the theme's `md` radius, or square.
fn corner(t: &Theme, shape: AvatarShape) -> f64 {
    match shape {
        AvatarShape::Circle => 999.0,
        AvatarShape::Rounded => t.get(&radius::MD_VAR),
        AvatarShape::Square => 0.0,
    }
}

/// `relative size-8 shrink-0 rounded-full bg-muted text-sm ring-1`: the box
/// of an avatar, its initials centred.
fn avatar_root(t: &Theme, s: &Look<AvatarState>, _base: Style) -> Style {
    style()
        .position(Position::Relative)
        .size(side(s.size), side(s.size))
        .flex_shrink(0.0)
        .border_radius(corner(t, s.shape))
        .background(MUTED)
        .color(MUTED_FOREGROUND)
        .font_size(if s.size == ControlSize::Small {
            12.0
        } else {
            14.0
        })
        .text_wrap(false)
        .clip(false)
        .outline(1.0, crate::color::BORDER)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
}

/// `aspect-square size-full object-cover`: the picture fills the avatar.
fn avatar_image(t: &Theme, s: &Look<AvatarState>, _base: Style) -> Style {
    style()
        .size(pct(100.0), pct(100.0))
        .border_radius(corner(t, s.shape))
        .clip(true)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
}

/// A row of avatars that overlap.
fn avatar_group_root(_: &Theme, _: &Look<()>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
}

/// `rounded-full ring-2 ring-background`: a member has a ring that cuts it off
/// from the one before it.
fn avatar_group_member(_: &Theme, _: &Look<usize>, _base: Style) -> Style {
    style()
        .border_radius(999.0)
        .outline(2.0, BACKGROUND)
        .flex_shrink(0.0)
}

/// `size-8 rounded-full bg-muted ring-2 ring-background`: the count of the
/// avatars that do not show, pulled over the last member.
fn avatar_group_overflow(t: &Theme, _: &Look<usize>, _base: Style) -> Style {
    style()
        .size(32.0, 32.0)
        .flex_shrink(0.0)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .border_radius(999.0)
        .background(MUTED)
        .color(MUTED_FOREGROUND)
        .font_size(12.0)
        .outline(2.0, BACKGROUND)
        .margin_left(-t.get(&avatar_group::OVERLAP_VAR))
}

/// The status dot at the bottom right: the tone's color, ringed in the background.
fn avatar_status(_: &Theme, s: &Look<AvatarState>, _base: Style) -> Style {
    let dot = match s.size {
        ControlSize::ExtraSmall | ControlSize::Small => 8.0,
        ControlSize::Medium => 10.0,
        ControlSize::Large => 12.0,
    };
    style()
        .position(Position::Absolute)
        .right(0.0)
        .bottom(0.0)
        .size(dot, dot)
        .border_radius(999.0)
        .background(status_color(s.status.unwrap_or_default()))
        .color(PRIMARY_FOREGROUND)
        .outline(2.0, BACKGROUND)
        .z_index(10)
}

/// The color of a status tone.
fn status_color(tone: Tone) -> Color {
    match tone {
        Tone::Neutral | Tone::Accent => PRIMARY,
        Tone::Info => color::ACCENT,
        Tone::Success => color::SUCCESS,
        Tone::Warning => color::WARNING,
        Tone::Danger => DESTRUCTIVE,
    }
}

/// Installs the avatar parts and the avatar group's member and overflow.
pub(crate) fn install(theme: Theme) -> Theme {
    theme
        .with(
            AvatarTheme::default()
                .root(avatar_root)
                .image(avatar_image)
                .status(avatar_status),
        )
        .with(
            AvatarGroupTheme::default()
                .root(avatar_group_root)
                .member(avatar_group_member)
                .overflow(avatar_group_overflow),
        )
}
