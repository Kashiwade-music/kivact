//! Role: Draw the console's input groups, tag and number inputs, and drop targets.
use crate::color::{DROPZONE_STRIPE, LINE_STRONG, PRIMARY_WEAK, SURFACE_3};
use crate::fonts::MONO_FAMILY_VAR;
use crate::size::CONTROL_HEIGHT;
use crate::variants::InputGroupVariant;
use kivact::custom::PartState;
use kivact::draw::{Layer, Path};
use kivact::geometry::Point;
use kivact::interaction::DropzoneState;
use kivact::style::color::{ACCENT, BORDER, SURFACE, TEXT_MUTED};
use kivact::style::{
    Align, BorderStyle, Color, FlexDirection, Position, Style, TextAlign, Theme, pct, style,
};
use kivact::ui::theme::{
    DropzoneTheme, InputGroupMemberState, InputGroupState, InputGroupTheme, Look, NumberInputTheme,
    TagInputTheme,
};

/// Install the input group, tag input, number input and dropzone recipes.
pub(crate) fn install(theme: Theme) -> Theme {
    theme
        .with(
            InputGroupTheme::default()
                .root(input_group_root)
                .label(input_group_label)
                .members(input_group_members)
                .member(input_group_member)
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
            NumberInputTheme::default()
                .root(number_root)
                .input(number_field),
        )
        .with(
            DropzoneTheme::default()
                .root(dropzone_root)
                .icon(dropzone_icon)
                .label(dropzone_label)
                .hint(dropzone_hint)
                .status(dropzone_status),
        )
}

fn input_group_root(_: &Theme, s: &Look<InputGroupState>, base: Style) -> Style {
    let result = base.width(pct(100));
    if s.variant::<InputGroupVariant>().is_some() {
        result.gap(0)
    } else {
        result
    }
}

fn input_group_label(_: &Theme, s: &Look<InputGroupState>, base: Style) -> Style {
    if s.variant::<InputGroupVariant>().is_some() {
        style().size(0, 0).clip(true)
    } else {
        base
    }
}

fn input_group_members(_: &Theme, _: &Look<InputGroupState>, base: Style) -> Style {
    base.width(pct(100)).align_items(Align::Center)
}

fn input_group_member(_: &Theme, s: &Look<InputGroupMemberState>, base: Style) -> Style {
    match s.variant::<InputGroupVariant>() {
        Some(InputGroupVariant::Adorned) => {
            if s.first || s.last {
                let result = style()
                    .position(Position::Absolute)
                    .align_self(Align::Center)
                    .color(TEXT_MUTED);
                if s.first {
                    result.left(10)
                } else {
                    result.right(8)
                }
            } else {
                style().flex_grow(1.).min_width(0)
            }
        }
        Some(InputGroupVariant::Inline) => {
            if s.first || s.last {
                style()
                    .height(CONTROL_HEIGHT)
                    .padding_x(10)
                    .align_items(Align::Center)
                    .background(SURFACE_3)
                    .border(1, LINE_STRONG)
            } else {
                style().flex_grow(1.).min_width(0)
            }
        }
        None => base,
    }
}

fn input_group_addon(t: &Theme, _: &Look<InputGroupState>, base: Style) -> Style {
    base.padding_x(10)
        .background(SURFACE_3)
        .border_color(LINE_STRONG)
        .font_family(t.get(&MONO_FAMILY_VAR).as_str().unwrap_or_default())
}

fn tag_root(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .width(pct(100))
        .flex_wrap(true)
        .align_items(Align::Center)
        .gap(6)
        .padding(6)
        .background(SURFACE)
        .border(1, BORDER)
}

fn tag(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .height(24)
        .padding_left(7)
        .gap(4)
        .align_items(Align::Center)
        .background(kivact::style::color::SELECTION)
        .border(1, BORDER)
        .font_size(11.5)
}

fn tag_remove(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style().size(20, 20)
}

fn tag_field(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .width(130)
        .min_width(90)
        .flex_grow(1.)
        .border_width(0)
        .focus(
            style()
                .border_color(Color::TRANSPARENT)
                .outline(0, Color::TRANSPARENT),
        )
}

fn number_root(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .align_items(Align::Center)
        .border(1, LINE_STRONG)
        .background(SURFACE)
        .clip(true)
}

fn number_field(t: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .width(64)
        .border_width(0)
        .text_align(TextAlign::Center)
        .font_family(t.get(&MONO_FAMILY_VAR).as_str().unwrap_or_default())
        .focus(
            style()
                .border_color(Color::TRANSPARENT)
                .outline(0, Color::TRANSPARENT),
        )
}

fn dropzone_root(_: &Theme, s: &Look<DropzoneState>, _base: Style) -> Style {
    let mut result = style()
        .flex_direction(FlexDirection::Column)
        .align_items(Align::Center)
        .gap(6)
        .padding_y(22)
        .padding_x(16)
        .border(1.5, LINE_STRONG)
        .border_style(BorderStyle::Dashed)
        .paint(move |d, s| {
            d.layer(Layer::new().clip_rect(s.to_rect()), |d| {
                let mut x = -s.height;
                while x < s.width {
                    d.fill_path(
                        &Path::new()
                            .move_to(Point::new(x, s.height))
                            .line_to(Point::new(x + 10., s.height))
                            .line_to(Point::new(x + s.height + 10., 0.))
                            .line_to(Point::new(x + s.height, 0.))
                            .close(),
                        DROPZONE_STRIPE,
                    );
                    x += 20.;
                }
            });
        });
    if matches!(**s, DropzoneState::DragOver { .. }) {
        result = result.background(PRIMARY_WEAK).border_color(ACCENT);
    }
    result
}

fn dropzone_icon(_: &Theme, _: &Look<DropzoneState>, _base: Style) -> Style {
    style().size(28, 28).color(ACCENT)
}

fn dropzone_label(_: &Theme, _: &Look<DropzoneState>, _base: Style) -> Style {
    style().font_weight(700)
}

fn dropzone_hint(_: &Theme, _: &Look<DropzoneState>, _base: Style) -> Style {
    style().font_size(11.5).color(TEXT_MUTED)
}

fn dropzone_status(_: &Theme, _: &Look<DropzoneState>, _base: Style) -> Style {
    style().color(kivact::style::color::DANGER)
}
