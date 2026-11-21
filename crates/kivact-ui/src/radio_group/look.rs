//! Role: The radio group recipe state, tokens, recipe table and default looks.

use crate::control_size::ControlSize;
use crate::recipe::recipe_table;
use crate::theme::Look;
use crate::toggle::look::CheckableState;
use crate::variants::Variant;
use kivact_core::style::{Align, FlexDirection, Justify, Style, Theme, style};
use kivact_core::view::Orientation;
pub(crate) fn group(t: &Theme, look: &Look<Orientation>) -> Style {
    if look.variant::<RadioGroupVariant>() == Some(&RadioGroupVariant::Segmented) {
        style()
            .flex_direction(FlexDirection::Row)
            .gap(0)
            .padding(2)
            .background(kivact_core::style::color::SURFACE_ALT)
            .border_radius(t.get(&kivact_core::style::radius::SM_VAR))
    } else {
        radio_group(t, look)
    }
}

pub(crate) fn item(t: &Theme, look: &Look<CheckableState>) -> Style {
    let base = radio_item(t, look);
    if look.variant::<RadioGroupVariant>() != Some(&RadioGroupVariant::Segmented) {
        return base;
    }
    let base = base
        .padding_left(t.get(&kivact_core::style::space::MD_VAR))
        .padding_right(t.get(&kivact_core::style::space::MD_VAR))
        .padding_top(t.get(&kivact_core::style::space::SM_VAR))
        .padding_bottom(t.get(&kivact_core::style::space::SM_VAR));
    if look.is_checked() {
        base.background(kivact_core::style::color::ACCENT)
            .color(kivact_core::style::color::ACCENT_TEXT)
    } else {
        base.background(kivact_core::style::color::SURFACE_ALT)
    }
}

pub(crate) fn indicator(t: &Theme, look: &Look<CheckableState>) -> Style {
    if look.variant::<RadioGroupVariant>() == Some(&RadioGroupVariant::Segmented) {
        style().size(0, 0).opacity(0.0)
    } else {
        radio_indicator(t, look)
    }
}

/// The options in a column, or a row when horizontal.
pub(crate) fn radio_group(t: &Theme, orientation: &Orientation) -> Style {
    let s = style().gap(t.get(&kivact_core::style::space::MD_VAR));
    match orientation {
        Orientation::Vertical => s.flex_direction(FlexDirection::Column),
        _ => s.flex_direction(FlexDirection::Row).flex_wrap(true),
    }
}

/// An option: its indicator beside its label.
pub(crate) fn radio_item(t: &Theme, state: &CheckableState) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(t.get(&kivact_core::style::space::MD_VAR))
        .border_radius(t.get(&kivact_core::style::radius::SM_VAR))
        .font_size(match state.size {
            ControlSize::ExtraSmall | ControlSize::Small => {
                t.get(&kivact_core::style::font::SIZE_SMALL_VAR)
            }
            ControlSize::Medium | ControlSize::Large => t.get(&kivact_core::style::font::SIZE_VAR),
        })
        .color(kivact_core::style::color::TEXT)
}

/// The circle of an option, the accent around the `dot` while chosen.
pub(crate) fn radio_indicator(t: &Theme, state: &CheckableState) -> Style {
    let side = (t.get(&kivact_core::style::font::SIZE_VAR) + 2.0).round();
    let s = style()
        .size(side, side)
        .flex_shrink(0)
        .border_radius(side / 2.0)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .background(kivact_core::style::color::SURFACE);
    if state.is_checked() {
        s.border(1, kivact_core::style::color::ACCENT)
            .color(kivact_core::style::color::ACCENT)
    } else if state.invalid {
        s.border(1, kivact_core::style::color::DANGER)
    } else {
        s.border(1, kivact_core::style::color::BORDER)
    }
}

pub(crate) fn radio_mark(_: &Theme, state: &Look<CheckableState>) -> Style {
    if state.is_checked() {
        style()
    } else {
        style().size(0, 0).opacity(0.0)
    }
}

pub(crate) fn radio_description(t: &Theme, _: &Look<CheckableState>) -> Style {
    style()
        .font_size(t.get(&kivact_core::style::font::SIZE_SMALL_VAR))
        .color(kivact_core::style::color::TEXT_MUTED)
}

recipe_table! {
    /// The recipes of a [`radio_group`](super::radio_group).
    RadioGroupTheme {
        /// The group, laid out in its orientation.
        group(group_recipe): Orientation = group,
        /// A radio item: its indicator and its label.
        item(item_recipe): CheckableState = item,
        /// The item's indicator, holding the theme's `dot` icon while
        /// it is checked.
        indicator(indicator_recipe): CheckableState = indicator,
        /// The dot icon, mounted through selection changes so its paint can transition.
        mark(mark_recipe): CheckableState = radio_mark,
        /// The secondary line of an option with a description.
        description(description_recipe): CheckableState = radio_description,
    }
}

impl RadioGroupTheme {
    pub(super) fn has_custom_mark(&self) -> bool {
        self.mark.is_overridden()
    }
}

/// Standard presentation of a radio group.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RadioGroupVariant {
    /// Adjacent choices drawn as a segmented control.
    Segmented,
}

impl Variant for RadioGroupVariant {}
