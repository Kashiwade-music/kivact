//! Role: Install Astryx Neutral's recipes of the input group: the input group, the tag
//! input, the drop zone and the date and time inputs.
use crate::color as c;
use crate::metrics as m;
use crate::radius as r;
use crate::recipe_parts::{flat, frame, text};
use crate::recipes_calendar::{field, wheel};
use crate::recipes_field::height;
use crate::shadows;
use crate::space as s;
use crate::type_scale::{BODY, SUPPORTING, TypeStyle, WEIGHT_MEDIUM};
use crate::variants::DropzoneVariant;
use kivact::custom::PartState;
use kivact::interaction::{DropzoneState, Validity};
use kivact::style::{Align, Color, FlexDirection, Justify, Style, Theme, ThemeMode, pct, style};
use kivact::ui::theme::{
    DateInputTheme, DateRangeInputTheme, DateTimeInputTheme, DropzoneTheme, InputGroupMemberState,
    InputGroupState, InputGroupTheme, Look, TagInputTheme, TimeInputTheme, dropzone,
};
use kivact::view::ReadingDirection;

/// The input group's column: its label above its members.
fn input_group_root(_theme: &Theme, _s: &Look<InputGroupState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .gap(4)
        .min_width(0)
}

/// The input group's label: the body text in the medium weight.
fn input_group_label(_theme: &Theme, _s: &Look<InputGroupState>, _base: Style) -> Style {
    style()
        .font_size(BODY.size_px)
        .font_weight(WEIGHT_MEDIUM)
        .color(c::TEXT_PRIMARY)
}

/// The input group's members in a row, in the reading direction.
fn members(_theme: &Theme, s: &Look<InputGroupState>, _base: Style) -> Style {
    style()
        .flex_direction(if s.direction == ReadingDirection::Rtl {
            FlexDirection::RowReverse
        } else {
            FlexDirection::Row
        })
        .align_items(Align::Stretch)
}

/// A member of the group: the outer corners of the first and last members are rounded, and the
/// members overlap by one px so their borders join.
fn member(theme: &Theme, s: &Look<InputGroupMemberState>, _base: Style) -> Style {
    let rtl = s.group.direction == ReadingDirection::Rtl;
    let (left, right) = if rtl {
        (s.last, s.first)
    } else {
        (s.first, s.last)
    };
    let radius = theme.get(&r::RADIUS_ELEMENT_VAR);
    let left = if left { radius } else { 0.0 };
    let right = if right { radius } else { 0.0 };
    let base = style()
        .height(height(s.group.size))
        .border_top_left_radius(left)
        .border_top_right_radius(right)
        .border_bottom_right_radius(right)
        .border_bottom_left_radius(left);
    let base = if s.first {
        base
    } else if rtl {
        base.margin_right(-1)
    } else {
        base.margin_left(-1)
    };
    if s.focused { base.z_index(1) } else { base }
}

/// An addon: the accent-muted block beside a member, centered on its text.
fn addon(_theme: &Theme, s: &Look<InputGroupState>, _base: Style) -> Style {
    style()
        .height(height(s.size))
        .padding_x(8)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .border(1.0, c::BORDER_EMPHASIZED)
        .background(c::ACCENT_MUTED)
        .color(c::TEXT_PRIMARY)
        .font_size(14)
}

/// The tag input's frame: the field frame with the tags wrapped inside it, its focus ring on the accent.
fn tag_root(theme: &Theme, look: &Look<PartState>, _base: Style) -> Style {
    let inset =
        theme.get(&r::RADIUS_ELEMENT_VAR) - theme.get(&r::RADIUS_INNER_VAR) - m::BORDER_WIDTH;
    let base = frame(theme, look, false);
    let base = if look.is_focused() {
        base.border_color(c::ACCENT)
            .shadow(shadows::input_focus_ring(theme.mode() == ThemeMode::Dark))
    } else {
        base
    };
    base.align_items(Align::Center)
        .column_gap(s::SPACE_S1)
        .row_gap(inset)
        .padding(inset)
        .min_height(m::CONTROL_MD)
}

/// One tag: a neutral pill with the medium supporting text.
fn tag(_theme: &Theme, _look: &Look<PartState>, _base: Style) -> Style {
    let role = TypeStyle {
        weight: WEIGHT_MEDIUM,
        ..SUPPORTING
    };
    text(style(), role, c::TEXT_PRIMARY)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(s::SPACE_S1)
        .height(s::SPACE_S6)
        .padding_left(s::SPACE_S2)
        .padding_right(s::SPACE_S1)
        .border_radius(r::RADIUS_INNER)
        .background(c::NEUTRAL)
        .max_width(pct(100))
}

/// A tag's remove button: a small transparent round button with a focus ring.
fn tag_remove(_theme: &Theme, _look: &Look<PartState>, _base: Style) -> Style {
    let clear = style().background(Color::TRANSPARENT);
    style()
        .size(m::ICON_SM, m::ICON_SM)
        .padding(0)
        .border_radius(r::RADIUS_FULL)
        .color(c::TEXT_PRIMARY)
        .hover(clear.clone())
        .active(clear)
        .focus_visible(
            style()
                .outline(m::FOCUS_RING_WIDTH, c::FOCUS_RING)
                .outline_offset(m::FOCUS_RING_OFFSET),
        )
}

/// The tag input's text entry: a flat field that grows to fill the row.
fn tag_input_field(theme: &Theme, _look: &Look<PartState>, _base: Style) -> Style {
    const EMPTY_INPUT_MIN_WIDTH: f64 = 60.0;
    flat(theme)
        .flex_grow(1.0)
        .min_width(EMPTY_INPUT_MIN_WIDTH)
        .height(s::SPACE_S6)
}

/// Whether a drop zone is the compact field rather than the large area.
fn is_field(look: &Look<DropzoneState>) -> bool {
    look.variant::<DropzoneVariant>() == Some(DropzoneVariant::Field)
}

/// The drop zone's frame: a centered column (or row, as a field) that takes the invalid, drag-over and
/// picking states.
fn dropzone_root(theme: &Theme, look: &Look<DropzoneState>, _base: Style) -> Style {
    let field = is_field(look);
    let invalid = matches!(
        &**look,
        DropzoneState::Failed(_)
            | DropzoneState::DragOver {
                valid: Validity::Invalid(_)
            }
    );
    let base = frame(theme, &PartState::default(), invalid)
        .flex_direction(if field {
            FlexDirection::Row
        } else {
            FlexDirection::Column
        })
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .min_width(0)
        .width(pct(100))
        .min_height(if field {
            32.0
        } else {
            theme.get(&dropzone::MIN_HEIGHT_VAR)
        })
        .gap(s::SPACE_S2)
        .padding_x(s::SPACE_S2)
        .padding_y(if field {
            0.0
        } else {
            theme.get(&dropzone::PADDING_VAR)
        });
    match &**look {
        DropzoneState::DragOver { .. } => base
            .border_color(c::ACCENT)
            .shadow(shadows::input_focus_ring(theme.mode() == ThemeMode::Dark)),
        DropzoneState::Picking => base.opacity(m::DISABLED_OPACITY),
        _ => base,
    }
}

/// The drop zone's label: one line of body text.
fn dropzone_label(_theme: &Theme, _look: &Look<DropzoneState>, _base: Style) -> Style {
    text(style().min_width(0).text_wrap(false), BODY, c::TEXT_PRIMARY)
}

/// The drop zone's hint: the body text in the field, the supporting text in the area.
fn dropzone_hint(_theme: &Theme, look: &Look<DropzoneState>, _base: Style) -> Style {
    let field = is_field(look);
    text(
        style().min_width(0).text_wrap(false),
        if field { BODY } else { SUPPORTING },
        if field {
            c::TEXT_PRIMARY
        } else {
            c::TEXT_SECONDARY
        },
    )
}

/// The drop zone's status line: the supporting text in the error color.
fn dropzone_status(_theme: &Theme, _look: &Look<DropzoneState>, _base: Style) -> Style {
    text(style(), SUPPORTING, c::TEXT_ERROR)
}

/// The drop zone's icon: a small icon that keeps its size in the row.
fn dropzone_icon(_theme: &Theme, _look: &Look<DropzoneState>, _base: Style) -> Style {
    style()
        .size(m::ICON_SM, m::ICON_SM)
        .flex_shrink(0.0)
        .color(c::ICON_SECONDARY)
}

/// Install the kit's recipes of the input group.
pub(crate) fn install(theme: Theme) -> Theme {
    theme
        .with(
            InputGroupTheme::default()
                .root(input_group_root)
                .label(input_group_label)
                .members(members)
                .member(member)
                .addon(addon),
        )
        .with(
            TagInputTheme::default()
                .root(tag_root)
                .tag(tag)
                .remove(tag_remove)
                .input(tag_input_field),
        )
        .with(
            DropzoneTheme::default()
                .root(dropzone_root)
                .label(dropzone_label)
                .hint(dropzone_hint)
                .status(dropzone_status)
                .icon(dropzone_icon),
        )
        .with(DateInputTheme::default().root(field).wheel(wheel))
        .with(TimeInputTheme::default().root(field).wheel(wheel))
        .with(DateRangeInputTheme::default().root(field))
        .with(DateTimeInputTheme::default().root(field))
}
