//! Role: Lyra recipes of the input group, and the adornments and borderless controls it joins.
//!
//! The looks of input-group.tsx. A canonical button and a text input or text
//! area inside a group take their looks from the helpers here, which the
//! button and field recipes call.

use crate::color::{DESTRUCTIVE, INPUT, MUTED_FOREGROUND};
use crate::recipe_parts::{disabled, field_fill, focus_ring};
use crate::variants::InputGroupButtonSize;
use kivact::style::{
    Align, Color, FlexDirection, Justify, LineHeight, Position, Style, Theme, ThemeMode, pct, style,
};
use kivact::ui::theme::InputGroupMemberKind;
use kivact::ui::theme::{
    ButtonState, InputGroupMemberState, InputGroupState, InputGroupTheme, Look,
};
use kivact::view::ReadingDirection;

/// The size of a canonical button inside a group: `h-6` and `h-8` text
/// buttons, and the square icon buttons.
pub(crate) fn button(s: &Look<ButtonState>) -> Option<Style> {
    s.variant::<InputGroupButtonSize>().map(|size| match size {
        InputGroupButtonSize::Xs => style().height(24.0).padding_x(6.0).gap(4.0),
        InputGroupButtonSize::Sm => style().height(32.0).padding_x(10.0).gap(4.0),
        InputGroupButtonSize::IconXs => style().size(24.0, 24.0).padding(0.0),
        InputGroupButtonSize::IconSm => style().size(28.0, 28.0).padding(0.0),
    })
}

/// The icon box of a canonical button inside a group: `size-3.5`, `size-4`.
pub(crate) fn button_icon(s: &Look<ButtonState>) -> Option<f64> {
    s.variant::<InputGroupButtonSize>().map(|size| match size {
        InputGroupButtonSize::Xs | InputGroupButtonSize::IconXs => 14.0,
        _ => 16.0,
    })
}

/// A text input or text area without its own box inside a group: it fills
/// the group, and its height follows the block adornments above and below.
pub(crate) fn control(base: Style, multiline: bool) -> Style {
    let group = kivact::ui::use_input_group_state().map(|state| state.get());
    let (above, below) = group.map_or((false, false), |s| (s.has_block_start, s.has_block_end));
    let base = base
        .flex_grow(1.0)
        .min_width(0.0)
        .border(0.0, Color::TRANSPARENT)
        .outline_none()
        .background(Color::TRANSPARENT)
        .disabled(style().opacity(1.0).background(Color::TRANSPARENT))
        .focus(style().border_color(Color::TRANSPARENT).outline_none())
        .focus_visible(style().border_color(Color::TRANSPARENT).outline_none());
    if multiline {
        base.padding_y(8.0)
    } else {
        let base = base.height(if above || below { 32.0 } else { 30.0 });
        let base = if below { base.padding_top(12.0) } else { base };
        if above {
            base.padding_bottom(12.0)
        } else {
            base
        }
    }
}

/// `rounded-none border bg-input/30` (or transparent in light): the frame
/// around the group, in destructive while invalid and in the ring while
/// focused.
fn root(t: &Theme, s: &Look<InputGroupState>, _base: Style) -> Style {
    let dark = t.mode() == ThemeMode::Dark;
    let base = style()
        .flex_direction(FlexDirection::Column)
        .width(pct(100.0))
        .min_width(0.0)
        .min_height(32.0)
        .flex_shrink(0.0)
        .border_radius(0.0)
        .background(field_fill(dark))
        .border(1.0, INPUT);
    let base = if s.input.is_disabled() {
        base.merge(&disabled(t))
            .background(INPUT.with_alpha(if dark { 0.8 } else { 0.5 }))
    } else {
        base
    };
    if s.invalid {
        base.border_color(DESTRUCTIVE)
            .outline(1.0, DESTRUCTIVE.with_alpha(if dark { 0.4 } else { 0.2 }))
    } else if s.focused {
        base.merge(&focus_ring(t))
    } else {
        base
    }
}

/// The label of a group is read by assistive technology only.
fn label(_: &Theme, _: &Look<InputGroupState>, _base: Style) -> Style {
    style()
        .position(Position::Absolute)
        .size(1.0, 1.0)
        .font_size(1.0)
        .line_height(LineHeight::px(1.0))
        .single_line()
        .clip(true)
        .opacity(0.0)
}

/// The row of the members, reversed in a right-to-left reading direction.
fn members(_: &Theme, s: &Look<InputGroupState>, _base: Style) -> Style {
    style()
        .flex_direction(if s.direction == ReadingDirection::Rtl {
            FlexDirection::RowReverse
        } else {
            FlexDirection::Row
        })
        .align_items(Align::Center)
        .width(pct(100.0))
        .min_width(0.0)
}

/// A member: a control grows and overlaps the adornments' borders; an
/// adornment is a muted run of text with 8 px of room on its outer side.
fn member(_: &Theme, s: &Look<InputGroupMemberState>, _base: Style) -> Style {
    if s.kind == InputGroupMemberKind::Control {
        style()
            .flex_direction(FlexDirection::Row)
            .flex_grow(1.0)
            .min_width(0.0)
            .margin_left(if s.group.has_start { -4.0 } else { 0.0 })
            .margin_right(if s.group.has_end { -4.0 } else { 0.0 })
    } else {
        let base = style()
            .flex_direction(FlexDirection::Row)
            .align_items(Align::Center)
            .justify_content(Justify::Center)
            .gap(8.0)
            .flex_shrink(0.0)
            .font_size(12.0)
            .font_weight(500)
            .color(MUTED_FOREGROUND);
        let start = s.kind == InputGroupMemberKind::Start;
        if start != (s.group.direction == ReadingDirection::Rtl) {
            base.padding_left(8.0)
        } else {
            base.padding_right(8.0)
        }
    }
}

/// An addon: a muted run of text and icons in a row.
fn addon(_: &Theme, _: &Look<InputGroupState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(8.0)
        .font_size(12.0)
        .color(MUTED_FOREGROUND)
}

/// A block adornment across the top or the bottom of the group.
fn block(top: bool) -> Style {
    let base = style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Start)
        .gap(8.0)
        .width(pct(100.0))
        .padding_x(10.0)
        .font_size(12.0)
        .font_weight(500)
        .color(MUTED_FOREGROUND);
    if top {
        base.padding_top(8.0).padding_bottom(6.0)
    } else {
        base.padding_top(6.0).padding_bottom(8.0)
    }
}

fn block_start(_: &Theme, _: &Look<InputGroupState>, _base: Style) -> Style {
    block(true)
}

fn block_end(_: &Theme, _: &Look<InputGroupState>, _base: Style) -> Style {
    block(false)
}

/// Installs the input group's parts.
pub(crate) fn install(theme: Theme) -> Theme {
    theme.with(
        InputGroupTheme::default()
            .root(root)
            .label(label)
            .members(members)
            .member(member)
            .addon(addon)
            .block_start(block_start)
            .block_end(block_end),
    )
}
