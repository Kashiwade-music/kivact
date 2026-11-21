//! Role: Render themed icons with Neutral sizing and optional color.

use crate::color as c;
use crate::metrics as m;
use kivact::Prop;
use kivact::component;
use kivact::state::Value;
use kivact::style::Icon as IconSource;
use kivact::style::Style;
use kivact::style::style;
use kivact::view::icon as control_icon;

/// The icon size option.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Prop)]
pub enum IconSize {
    /// Use the extra-small icon dimensions from the theme.
    Xs,
    /// Use the small icon dimensions from the theme.
    Sm,
    #[default]
    /// Use the standard icon dimensions from the theme.
    Md,
    /// Use the large icon dimensions from the theme.
    Lg,
}

/// The icon color option.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Prop)]
pub enum IconColor {
    /// Use primary icon ink.
    Primary,
    /// Use secondary icon ink.
    Secondary,
    /// Use the secondary ink for a tertiary treatment.
    Tertiary,
    /// Use disabled icon ink.
    Disabled,
    /// Use accent icon ink.
    Accent,
    /// Use informational text ink.
    Info,
    /// Use successful text ink.
    Success,
    /// Use error text ink.
    Error,
    /// Use warning text ink.
    Warning,
    #[default]
    /// Inherit foreground ink from the surrounding content.
    Inherit,
    /// Use the blue categorical icon ink.
    Blue,
    /// Use the red categorical icon ink.
    Red,
    /// Use the green categorical icon ink.
    Green,
    /// Use the gray categorical icon ink.
    Gray,
    /// Use the cyan categorical icon ink.
    Cyan,
    /// Use the teal categorical icon ink.
    Teal,
    /// Use the yellow categorical icon ink.
    Yellow,
    /// Use the orange categorical icon ink.
    Orange,
    /// Use the pink categorical icon ink.
    Pink,
    /// Use the purple categorical icon ink.
    Purple,
}

/// Show an icon from the current theme, with a Neutral size and color.
#[component]
pub fn icon_view(
    source: Value<IconSource>,
    #[prop(default = IconSize::Md)] size: Value<IconSize>,
    #[prop(default = IconColor::Inherit)] color: Value<IconColor>,
    label: Option<Value<String>>,
) -> impl Element {
    let root = control_icon(source).style(move || icon_style(size.get(), color.get()));
    match label {
        Some(label) => root.name(label),
        None => root,
    }
}

/// Resolve the Neutral icon size and color against the current theme.
pub(crate) fn icon_style(size: IconSize, color: IconColor) -> Style {
    let side = match size {
        IconSize::Xs => m::ICON_XSM,
        IconSize::Sm => m::ICON_SM,
        IconSize::Md => m::ICON_MD,
        IconSize::Lg => m::ICON_LG,
    };
    let look = style().size(side, side).flex_shrink(0);
    match color {
        IconColor::Primary => look.color(c::ICON_PRIMARY),
        IconColor::Secondary | IconColor::Tertiary => look.color(c::ICON_SECONDARY),
        IconColor::Disabled => look.color(c::ICON_DISABLED),
        IconColor::Accent => look.color(c::ICON_ACCENT),
        IconColor::Info => look.color(c::TEXT_INFO),
        IconColor::Success => look.color(c::TEXT_SUCCESS),
        IconColor::Error => look.color(c::TEXT_ERROR),
        IconColor::Warning => look.color(c::TEXT_WARNING),
        IconColor::Inherit => look,
        IconColor::Blue => look.color(c::HUE_BLUE_ICON),
        IconColor::Red => look.color(c::HUE_RED_ICON),
        IconColor::Green => look.color(c::HUE_GREEN_ICON),
        IconColor::Gray => look.color(c::HUE_GRAY_ICON),
        IconColor::Cyan => look.color(c::HUE_CYAN_ICON),
        IconColor::Teal => look.color(c::HUE_TEAL_ICON),
        IconColor::Yellow => look.color(c::HUE_YELLOW_ICON),
        IconColor::Orange => look.color(c::HUE_ORANGE_ICON),
        IconColor::Pink => look.color(c::HUE_PINK_ICON),
        IconColor::Purple => look.color(c::HUE_PURPLE_ICON),
    }
}
