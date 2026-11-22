//! Role: Creative appearance of the standard input group: a muted label and addons.
//! The addons sit on the header surface.
use crate::color;
use kivact::style::{Style, Theme};
use kivact::ui::theme::{InputGroupState, InputGroupTheme, Look};

/// The theme of an input group: a column with a muted label and addons.
pub(crate) fn input_group() -> InputGroupTheme {
    InputGroupTheme::default()
        .root(input_group_root)
        .label(input_group_label)
        .members(input_group_members)
        .addon(input_group_addon)
}

fn input_group_root(_t: &Theme, _s: &Look<InputGroupState>, base: Style) -> Style {
    base.gap(4)
}

fn input_group_label(_t: &Theme, _s: &Look<InputGroupState>, base: Style) -> Style {
    base.font_size(11.5).color(color::MUTED)
}

fn input_group_members(_t: &Theme, _s: &Look<InputGroupState>, base: Style) -> Style {
    base.min_width(0)
}

fn input_group_addon(_t: &Theme, _s: &Look<InputGroupState>, base: Style) -> Style {
    base.background(color::HEADER)
        .color(color::MUTED)
        .border_color(color::HAIR)
        .font_size(11.5)
}

/// Install the input group's recipes into `theme`.
pub(crate) fn install(theme: Theme) -> Theme {
    theme.with(input_group())
}
