//! Role: Style the Astryx Neutral radio groups, their items and the segmented control the
//! group can show as.
use crate::color as c;
use crate::metrics as m;
use crate::radius as r;
use crate::recipe_parts::{fast, ring};
use crate::recipes_controls_choice::{centered_mark, check_side, check_surface, small};
use crate::space as s;
use crate::type_scale as t;
use crate::variants::{SegmentedIconOnly, SegmentedLayout};
use kivact::style::{
    Align, Color, Cursor, FlexDirection, Justify, Position, Style, Theme, pct, style,
};
use kivact::ui::ControlSize;
use kivact::ui::RadioGroupVariant;
use kivact::ui::theme::{CheckableState, Look, RadioGroupTheme};
use kivact::view::Orientation;

/// Whether the group is a segmented control, which replaces the radio look.
fn segmented<S>(look: &Look<S>) -> bool {
    look.variant::<RadioGroupVariant>() == Some(RadioGroupVariant::Segmented)
}

/// The track of a segmented control: a neutral well the segments sit in.
fn segmented_track(theme: &Theme, look: &Look<Orientation>) -> Style {
    let mut base = style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(s::SPACE_S0_5)
        .padding(theme.get(&s::SPACE_S1_VAR))
        .border_radius(r::RADIUS_ELEMENT)
        .background(c::NEUTRAL);
    if look.variant::<SegmentedLayout>() == Some(SegmentedLayout::Fill) {
        base = base.width(pct(100));
    }
    if look
        .variant::<SegmentedIconOnly>()
        .is_some_and(|icon_only| icon_only.0)
    {
        base.flex_shrink(0)
    } else {
        base.max_width(pct(100)).min_width(0)
    }
}

/// One segment of a segmented control: its height, padding and text by size.
fn segment(look: &Look<CheckableState>) -> Style {
    let (height, padding, font) = match look.size {
        ControlSize::ExtraSmall | ControlSize::Small => (20.0, 8.0, 12.0),
        ControlSize::Medium => (24.0, 12.0, 14.0),
        ControlSize::Large => (28.0, 12.0, 14.0),
    };
    let selected = look.is_checked();
    let (fill, ink) = if selected {
        (c::BG_SURFACE, c::TEXT_PRIMARY)
    } else {
        (Color::TRANSPARENT, c::TEXT_SECONDARY)
    };
    let mut base = style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .height(height)
        .min_width(0)
        .border_radius(8.0)
        .padding_x(padding)
        .background(fill)
        .color(ink)
        .font_size(font)
        .line_height(20.0 / font)
        .font_weight(if selected {
            t::WEIGHT_SEMIBOLD
        } else {
            t::WEIGHT_MEDIUM
        })
        .cursor(Cursor::Pointer)
        .focus_visible(ring(Color::var(&c::FOCUS_RING_VAR)))
        .disabled(style().color(c::TEXT_DISABLED).cursor(Cursor::Default));
    if !selected && !look.input.is_disabled() {
        base = base
            .hover(style().background(c::OVERLAY_HOVER))
            .active(style().background(c::OVERLAY_PRESSED));
    }
    if look.variant::<SegmentedLayout>() == Some(SegmentedLayout::Fill) {
        base = base.flex_grow(1).flex_basis(0);
    }
    fast(base)
}

fn group(theme: &Theme, look: &Look<Orientation>, _base: Style) -> Style {
    if segmented(look) {
        return segmented_track(theme, look);
    }
    if **look == Orientation::Horizontal {
        style()
            .flex_direction(FlexDirection::Row)
            .gap(theme.get(&s::SPACE_S5_VAR))
    } else {
        style()
            .flex_direction(FlexDirection::Column)
            .gap(theme.get(&s::SPACE_S2_VAR))
    }
}

fn item(theme: &Theme, look: &Look<CheckableState>, _base: Style) -> Style {
    if segmented(look) {
        return segment(look);
    }
    let text = t::BODY;
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(theme.get(&s::SPACE_S2_VAR))
        .font_size(text.size_px)
        .line_height(text.line_height_px / text.size_px)
        .font_weight(text.weight)
        .color(c::TEXT_PRIMARY)
        .disabled(style().opacity(m::DISABLED_OPACITY))
        .focus_visible(style().outline_none())
        .min_height(check_side(theme, look))
}

fn indicator(theme: &Theme, look: &Look<CheckableState>, _base: Style) -> Style {
    if segmented(look) {
        return style().size(0, 0).clip(true);
    }
    check_surface(theme, look, r::RADIUS_FULL)
        .position(Position::Relative)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
}

fn mark(theme: &Theme, look: &Look<CheckableState>, _base: Style) -> Style {
    if segmented(look) {
        return style().size(0, 0).clip(true);
    }
    let side = if small(look.size) {
        theme.get(&s::SPACE_S2_VAR)
    } else {
        t::SIZE_XS
    };
    centered_mark(side, side, look.is_checked())
        .border_radius(r::RADIUS_FULL)
        .background(c::ON_ACCENT)
}

fn description(_theme: &Theme, _look: &Look<CheckableState>, _base: Style) -> Style {
    let text = t::SUPPORTING;
    style()
        .font_size(text.size_px)
        .line_height(text.line_height_px / text.size_px)
        .font_weight(text.weight)
        .color(c::TEXT_SECONDARY)
}

/// Install the kit's recipes of the radio group.
pub(crate) fn install(theme: Theme) -> Theme {
    theme.with(
        RadioGroupTheme::default()
            .group(group)
            .item(item)
            .indicator(indicator)
            .mark(mark)
            .description(description),
    )
}
