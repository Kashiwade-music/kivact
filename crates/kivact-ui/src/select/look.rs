//! Role: The select recipe state, tokens, recipe table and default looks.
use crate::look_common::{description, mark, trigger};

use crate::control_size::ControlSize;
use crate::recipe::recipe_table;
use kivact_core::custom::PartState;
use kivact_core::style::{Align, Direction, Style, Theme, style};
use kivact_interaction::{ListItemState, PopupState};
/// A select's trigger: a bordered field, its text muted while it shows
/// the placeholder.
pub(crate) fn select_trigger(t: &Theme, state: &SelectTriggerState) -> Style {
    let s = trigger(t).height(crate::look_common::height(t, state.size));
    if state.placeholder {
        s.color(kivact_core::style::color::TEXT_MUTED)
    } else {
        s
    }
}

pub(crate) fn option(t: &Theme, _: &ListItemState) -> Style {
    style()
        .padding_x(t.get(&kivact_core::style::space::MD_VAR))
        .height(t.get(&kivact_core::style::control::HEIGHT_VAR))
        .justify_content(kivact_core::style::Justify::Center)
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(t.get(&kivact_core::style::space::MD_VAR))
        .focus(style().background(kivact_core::style::color::HOVER))
        .selected(style().background(kivact_core::style::color::HOVER))
}

/// The box of the selected option's check, at the row's end.
pub(crate) fn option_check(_: &Theme, _: &ListItemState) -> Style {
    mark().width(16)
}

pub(crate) fn option_description(t: &Theme, _: &ListItemState) -> Style {
    description(t)
}

pub(crate) fn select_indicator(_: &Theme, _: &PopupState) -> Style {
    mark().color(kivact_core::style::color::TEXT_MUTED)
}

/// The state of a select's trigger for its recipe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectTriggerState {
    /// The actual input state of the trigger.
    pub input: PartState,
    /// Whether the list is open.
    pub open: bool,
    /// Whether the trigger shows the placeholder: no option is selected
    /// (the selected key is `None` or not among the options).
    pub placeholder: bool,
    /// The size the caller chose with `Select::size` (default `Medium`).
    pub size: ControlSize,
}

recipe_table! {
    /// The recipes of a [`select`](super::select).
    SelectRecipes {
        /// The trigger, which knows whether it shows the placeholder. Its
        /// [`indicator`](Self::indicator_recipe) holds the theme's `chevron-down`
        /// icon.
        trigger(trigger_recipe): SelectTriggerState = select_trigger,
        /// The floating list.
        surface(surface_recipe): PopupState = crate::look_common::surface,
        /// An option row: its icon, its label and its description.
        option(option_recipe): ListItemState = option,
        /// The trailing box of an option, holding the theme's `check` icon
        /// while the option is selected. Every option has the box, so the
        /// labels line up; a custom [`Select::item`](super::Select::item)
        /// row shows no check.
        check(check_recipe): ListItemState = option_check,
        /// The box of the trigger's `chevron-down` icon.
        indicator(indicator_recipe): PopupState = select_indicator,
        /// The secondary line of an option with a description.
        description(description_recipe): ListItemState = option_description,
    }
}
