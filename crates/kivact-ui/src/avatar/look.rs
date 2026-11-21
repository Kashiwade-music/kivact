//! Role: The avatar recipe state, recipe table and default looks.
use crate::look_common::{side, tone};
use crate::recipe::recipe_table;
use crate::theme::Look;

use crate::ControlSize;
use kivact_core::style::{Align, Justify, Style, Theme, pct, style};
use kivact_interaction::Tone;
use kivact_macros::Prop;
pub(crate) fn avatar(t: &Theme, s: &Look<AvatarState>) -> Style {
    style()
        .size(side(s.size), side(s.size))
        .border_radius(match s.shape {
            AvatarShape::Circle => 999.0,
            AvatarShape::Rounded => t.get(&kivact_core::style::radius::MD_VAR),
            AvatarShape::Square => 0.0,
        })
        .background(kivact_core::style::color::SURFACE)
        .color(kivact_core::style::color::TEXT)
        .clip(true)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
}

pub(crate) fn avatar_image(t: &Theme, s: &Look<AvatarState>) -> Style {
    style()
        .size(pct(100), pct(100))
        .clip(true)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .border_radius(match s.shape {
            AvatarShape::Circle => 999.0,
            AvatarShape::Rounded => t.get(&kivact_core::style::radius::MD_VAR),
            AvatarShape::Square => 0.0,
        })
}

pub(crate) fn avatar_status(_: &Theme, s: &Look<AvatarState>) -> Style {
    style()
        .position(kivact_core::style::Position::Absolute)
        .right(0)
        .bottom(0)
        .size(8, 8)
        .border_radius(999)
        .background(tone(s.status.unwrap_or_default()))
}

pub(crate) fn avatar_status_content(_: &Theme, _: &Look<AvatarState>) -> Style {
    let base = style()
        .position(kivact_core::style::Position::Absolute)
        .bottom(0);
    if kivact_core::view::use_direction() == kivact_core::view::ReadingDirection::Rtl {
        base.left(0)
    } else {
        base.right(0)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AvatarState {
    pub size: ControlSize,
    pub shape: AvatarShape,
    pub status: Option<Tone>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum AvatarShape {
    #[default]
    Circle,
    Rounded,
    Square,
}

recipe_table! {
    /// The recipes of an avatar: its root, image and status.
    AvatarTheme {
        root(root_recipe): AvatarState = avatar,
        image(image_recipe): AvatarState = avatar_image,
        status(status_recipe): AvatarState = avatar_status,
        status_content(status_content_recipe): AvatarState = avatar_status_content,
    }
    settings {
        /// Whether an avatar shows its name in a tooltip unless the caller says otherwise.
        name_tooltip(name_tooltip_value): bool = false,
    }
}
