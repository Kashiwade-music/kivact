//! Role: Draw the standard segmented radio group variant.

use super::{CheckableState, Look, RadioGroupVariant};
use kivact_core::style::{ColorToken, Direction, Style, Theme, style};
use kivact_core::view::Orientation;

pub(super) fn group(t: &Theme, look: &Look<Orientation>) -> Style {
    if look.variant::<RadioGroupVariant>() == Some(&RadioGroupVariant::Segmented) {
        style()
            .direction(Direction::Row)
            .gap(0)
            .padding(2)
            .background(ColorToken::SurfaceAlt)
            .radius(t.radius().sm)
    } else {
        super::look_controls::radio_group(t, look)
    }
}

pub(super) fn item(t: &Theme, look: &Look<CheckableState>) -> Style {
    let base = super::look_controls::radio_item(t, look);
    if look.variant::<RadioGroupVariant>() != Some(&RadioGroupVariant::Segmented) {
        return base;
    }
    let base = base
        .padding_left(t.space().md)
        .padding_right(t.space().md)
        .padding_top(t.space().sm)
        .padding_bottom(t.space().sm);
    if look.checked.is_checked() {
        base.background(ColorToken::Accent)
            .color(ColorToken::AccentText)
    } else {
        base.background(ColorToken::SurfaceAlt)
    }
}

pub(super) fn indicator(t: &Theme, look: &Look<CheckableState>) -> Style {
    if look.variant::<RadioGroupVariant>() == Some(&RadioGroupVariant::Segmented) {
        style().size(0, 0).opacity(0.0)
    } else {
        super::look_controls::radio_indicator(t, look)
    }
}
