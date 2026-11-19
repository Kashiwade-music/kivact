//! Role: The radio group recipe state, tokens, recipe table and default looks.

use crate::Look;
use crate::control_size::ControlSize;
use crate::recipe::recipe_table;
use crate::toggle::look::CheckableState;
use crate::variants::Variant;
use kivact_core::style::{Align, ColorToken, Direction, Justify, Style, Theme, style};
use kivact_core::view::Orientation;
pub(crate) fn group(t: &Theme, look: &Look<Orientation>) -> Style {
    if look.variant::<RadioGroupVariant>() == Some(&RadioGroupVariant::Segmented) {
        style()
            .direction(Direction::Row)
            .gap(0)
            .padding(2)
            .background(ColorToken::SurfaceAlt)
            .radius(t.radius().sm)
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
        .padding_left(t.space().md)
        .padding_right(t.space().md)
        .padding_top(t.space().sm)
        .padding_bottom(t.space().sm);
    if look.is_checked() {
        base.background(ColorToken::Accent)
            .color(ColorToken::AccentText)
    } else {
        base.background(ColorToken::SurfaceAlt)
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
    let s = style().gap(t.space().md);
    match orientation {
        Orientation::Vertical => s.direction(Direction::Column),
        _ => s.direction(Direction::Row).wrap(true),
    }
}

/// An option: its indicator beside its label.
pub(crate) fn radio_item(t: &Theme, state: &CheckableState) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(t.space().md)
        .radius(t.radius().sm)
        .font_size(match state.size {
            ControlSize::ExtraSmall | ControlSize::Small => t.text().size_small,
            ControlSize::Medium | ControlSize::Large => t.text().size,
        })
        .color(ColorToken::Text)
}

/// The circle of an option, the accent around the `dot` while chosen.
pub(crate) fn radio_indicator(t: &Theme, state: &CheckableState) -> Style {
    let side = (t.text().size + 2.0).round();
    let s = style()
        .size(side, side)
        .shrink(0)
        .radius(side / 2.0)
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .background(ColorToken::Surface);
    if state.is_checked() {
        s.border(1, ColorToken::Accent).color(ColorToken::Accent)
    } else if state.invalid {
        s.border(1, ColorToken::Danger)
    } else {
        s.border(1, ColorToken::Border)
    }
}

pub(crate) fn radio_mark(_: &Theme, state: &CheckableState) -> Style {
    if state.is_checked() {
        style()
    } else {
        style().size(0, 0).opacity(0.0)
    }
}

pub(crate) fn radio_description(t: &Theme, _: &CheckableState) -> Style {
    style()
        .font_size(t.text().size_small)
        .color(ColorToken::TextMuted)
}

recipe_table! {
    /// The recipes of a [`radio_group`](super::radio_group).
    RadioGroupRecipes {
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

impl RadioGroupRecipes {
    pub(super) fn has_custom_mark(&self) -> bool {
        !self.mark.same(&Self::default_table().mark)
    }
}

/// Standard presentation of a radio group.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RadioGroupVariant {
    /// Adjacent choices drawn as a segmented control.
    Segmented,
}

impl Variant for RadioGroupVariant {}
