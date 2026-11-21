//! Role: The Pop looks of the input parts: input groups, tag inputs, dropzones and the date fields.

use crate::color;
use crate::variants::Hue;
use kivact::custom::PartState;
use kivact::interaction::{DropzoneState, Validity};
use kivact::style::Shadow;
use kivact::style::color as core;
use kivact::style::{Align, BorderStyle, Color, FlexDirection, Justify, Style, Theme, style};
use kivact::ui::theme::{
    DateInputTheme, DateRangeInputTheme, DropzoneTheme, InputGroupState, InputGroupTheme, Look,
    TagInputTheme,
};

/// Install the kit's recipes of the input parts.
pub(crate) fn install(theme: Theme) -> Theme {
    let theme = theme
        .with(
            InputGroupTheme::default()
                .label(input_group_label)
                .addon(input_group_addon),
        )
        .with(
            TagInputTheme::default()
                .root(tag_root)
                .tag(tag)
                .remove(tag_remove)
                .input(tag_field),
        )
        .with(
            DropzoneTheme::default()
                .root(dropzone_root)
                .icon(dropzone_icon)
                .label(dropzone_label)
                .hint(dropzone_hint)
                .status(dropzone_status),
        );
    let theme = theme.with(DateInputTheme::default().root(date_input));
    theme.with(DateRangeInputTheme::default().root(date_range_input))
}

fn input_group_label(_theme: &Theme, _: &Look<InputGroupState>, base: Style) -> Style {
    base.font_size(13).font_weight(500).color(color::TEXT)
}

fn input_group_addon(_theme: &Theme, _: &Look<InputGroupState>, base: Style) -> Style {
    base.background(color::SURFACE_MUTED)
        .border_color(color::LINE)
        .border_radius(8)
}

fn tag_root(_theme: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .min_height(36)
        .padding_x(6)
        .padding_y(4)
        .gap(4)
        .flex_wrap(true)
        .align_items(Align::Center)
        .background(color::SURFACE)
        .border(1, color::LINE)
        .border_radius(8)
}

fn tag(_theme: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    let (fill, text) = Hue::Blue.colors();
    style()
        .flex_direction(FlexDirection::Row)
        .height(26)
        .padding_left(10)
        .padding_right(4)
        .gap(2)
        .align_items(Align::Center)
        .border_radius(6)
        .background(fill)
        .color(text)
        .font_size(13)
}

fn tag_remove(_theme: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style().size(18, 18).border_radius(4)
}

fn tag_field(_theme: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .height(26)
        .padding_y(4)
        .padding_x(4)
        .min_width(100)
        .width(130)
        .flex_grow(1)
        .border(0, Color::TRANSPARENT)
        .shadow(Shadow::new(0, 0, 0, Color::TRANSPARENT))
        .focus(
            style()
                .border_color(Color::TRANSPARENT)
                .outline(0, Color::TRANSPARENT),
        )
}

/// The frame of a drop target: a dashed border, red when invalid and blue
/// while a file hovers over it.
fn dropzone_root(_theme: &Theme, s: &Look<DropzoneState>, _base: Style) -> Style {
    let hovered = matches!(**s, DropzoneState::DragOver { .. });
    let invalid = matches!(
        **s,
        DropzoneState::Failed(_)
            | DropzoneState::DragOver {
                valid: Validity::Invalid(_)
            }
    );
    style()
        .flex_direction(FlexDirection::Column)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .padding_x(16)
        .padding_y(24)
        .gap(4)
        .border(
            1,
            if invalid {
                color::RED
            } else if hovered {
                color::BLUE
            } else {
                color::LINE_STRONG
            },
        )
        .border_style(BorderStyle::Dashed)
        .border_radius(10)
        .background(if hovered {
            Hue::Blue.colors().0
        } else {
            color::SURFACE_MUTED
        })
        .hover(style().border_color(color::BLUE))
}

fn dropzone_icon(_theme: &Theme, _: &Look<DropzoneState>, _base: Style) -> Style {
    style()
        .size(36, 36)
        .margin_bottom(8)
        .border_radius(8)
        .background(color::SURFACE)
        .border(1, color::LINE)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .color(color::TEXT_MUTED)
}

fn dropzone_label(_theme: &Theme, _: &Look<DropzoneState>, _base: Style) -> Style {
    style().font_weight(500)
}

fn dropzone_hint(_theme: &Theme, _: &Look<DropzoneState>, _base: Style) -> Style {
    style().font_size(12).color(core::TEXT_MUTED)
}

fn dropzone_status(_theme: &Theme, _: &Look<DropzoneState>, _base: Style) -> Style {
    style().font_size(12).color(core::DANGER)
}

fn date_input(_theme: &Theme, _: &Look<PartState>, base: Style) -> Style {
    base.gap(4).color(color::TEXT)
}

fn date_range_input(_theme: &Theme, _: &Look<PartState>, base: Style) -> Style {
    base.gap(8).color(color::TEXT)
}
