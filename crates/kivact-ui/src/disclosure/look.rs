//! Role: The disclosure recipe state, tokens, recipe table and default looks.

use crate::recipe::recipe_table;
use kivact_core::style::{Align, ColorToken, Direction, Justify, Style, Theme, style};
use std::time::Duration;
/// A disclosure: the trigger over the content.
pub(crate) fn disclosure_root(_: &Theme, _: &DisclosureRecipeState) -> Style {
    style().direction(Direction::Column)
}

/// A disclosure's trigger: the title at the start and the indicator at the end.
pub(crate) fn disclosure_trigger(t: &Theme, _: &DisclosureRecipeState) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::SpaceBetween)
        .gap(t.space().md)
        .padding_y(t.space().md)
        .font_size(t.text().size)
        .weight(t.text().weight_strong)
        .color(ColorToken::Text)
        .radius(t.radius().md)
        .hover(style().background(ColorToken::Hover))
        .pressed(style().background(ColorToken::Pressed))
}

/// The box of a disclosure's `chevron-down` icon, turned up while it is open.
pub(crate) fn disclosure_indicator(_: &Theme, state: &DisclosureRecipeState) -> Style {
    style()
        .size(16, 16)
        .shrink(0)
        .color(ColorToken::TextMuted)
        .rotate(if state.open { 180 } else { 0 })
}

/// A disclosure's content under the trigger.
pub(crate) fn disclosure_content(t: &Theme, _: &DisclosureRecipeState) -> Style {
    style()
        .enter_size(kivact_core::view::Orientation::Vertical)
        .exit_size(kivact_core::view::Orientation::Vertical)
        .direction(Direction::Column)
        .padding_bottom(t.space().md)
        .color(ColorToken::TextMuted)
}

/// A panel's fixed position within its accordion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisclosureGroupPosition {
    /// Zero-based panel index.
    pub index: usize,
    /// Number of panels in the group.
    pub count: usize,
}

/// Actual disclosure state with optional group geometry for theme decoration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisclosureRecipeState {
    pub input: kivact_core::custom::PartState,
    pub open: bool,
    pub group: Option<DisclosureGroupPosition>,
}

impl DisclosureRecipeState {
    pub(crate) fn from_part(
        state: kivact_interaction::DisclosureState,
        group: Option<DisclosureGroupPosition>,
    ) -> Self {
        Self {
            input: state.input,
            open: state.open,
            group,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct DisclosureTokens {
    pub motion: DisclosureMotion,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DisclosureMotion {
    pub animate: bool,
    pub duration: Duration,
    /// Optional theme transition; absent uses the duration-based ease-out tween.
    pub transition: Option<kivact_core::motion::Transition>,
}

impl Default for DisclosureMotion {
    fn default() -> Self {
        Self {
            animate: false,
            duration: Duration::from_millis(200),
            transition: None,
        }
    }
}

recipe_table! {
    /// The recipes of a [`disclosure`](super::disclosure); all of them get
    /// the trigger's or the content's [`DisclosureRecipeState`].
    DisclosureRecipes {
        /// The column of the trigger over the content.
        root(root_recipe): DisclosureRecipeState = disclosure_root,
        /// The trigger: the title and the indicator in a row.
        trigger(trigger_recipe): DisclosureRecipeState = disclosure_trigger,
        /// The box of the trigger's `chevron-down` icon, which a look turns
        /// while the content is open.
        indicator(indicator_recipe): DisclosureRecipeState = disclosure_indicator,
        /// The content under the trigger.
        content(content_recipe): DisclosureRecipeState = disclosure_content,
    }
}
