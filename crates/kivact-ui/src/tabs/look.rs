//! Role: The tabs recipe state, tokens, recipe table and default looks.

use crate::recipe::recipe_table;
use kivact_core::style::{Align, FlexDirection, Style, Theme, style};
use kivact_core::view::Orientation;
use kivact_interaction::{TabListState, TabState};
pub(crate) fn tabs_list(_: &Theme, s: &TabListState) -> Style {
    if s.orientation == Orientation::Vertical {
        style()
            .flex_direction(FlexDirection::Column)
            .gap(4)
            .border_right(1, kivact_core::style::color::BORDER)
    } else {
        style()
            .flex_direction(FlexDirection::Row)
            .gap(4)
            .border_bottom(1, kivact_core::style::color::BORDER)
    }
}

pub(crate) fn no_tab_indicator(_: &Theme, _: &TabState) -> Style {
    style()
}

pub(crate) fn tab(t: &Theme, s: &TabState) -> Style {
    let mark = if s.orientation == Orientation::Vertical {
        style().border_right(2, kivact_core::style::color::ACCENT)
    } else {
        style().border_bottom(2, kivact_core::style::color::ACCENT)
    };
    style()
        .padding_x(t.get(&kivact_core::style::space::MD_VAR))
        .height(t.get(&kivact_core::style::control::HEIGHT_VAR))
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(t.get(&kivact_core::style::space::SM_VAR))
        .justify_content(kivact_core::style::Justify::Center)
        .selected(mark)
}

pub(crate) fn panel(t: &Theme, _: &TabState) -> Style {
    style().padding(t.get(&kivact_core::style::space::MD_VAR))
}

recipe_table! {
    /// The recipes of [`tabs`](super::tabs).
    TabsRecipes {
        /// The tab list, which knows its orientation.
        list(list_recipe): TabListState = tabs_list,
        /// A tab, which knows whether it is selected and the list's orientation.
        tab(tab_recipe): TabState = tab,
        /// An optional, persistent decorative layer per tab. It receives
        /// both selected and unselected states so its paint can transition.
        indicator(indicator_recipe): TabState = no_tab_indicator,
        panel(panel_recipe): TabState = panel,
    }
}

impl TabsRecipes {
    pub(super) fn has_custom_indicator(&self) -> bool {
        !self.indicator.same(&Self::default_table().indicator)
    }
}
