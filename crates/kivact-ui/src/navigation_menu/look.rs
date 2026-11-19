//! Role: The navigation menu recipe state, tokens, recipe table and default looks.

use crate::Look;
use crate::recipe::recipe_table;
use kivact_core::style::{Align, ColorToken, Direction, Style, Theme, style};
pub(crate) fn navigation_root(t: &Theme, _: &Look<kivact_interaction::NavOpen>) -> Style {
    let p = &t.ext::<crate::UiTokens>().navigation_menu;
    style()
        .direction(Direction::Row)
        .gap(p.gap)
        .padding(p.padding)
        .font_size(t.text().size)
        .color(ColorToken::Text)
}

pub(crate) fn panel(t: &Theme, _: &Look<kivact_interaction::NavOpen>) -> Style {
    let p = &t.ext::<crate::UiTokens>().navigation_menu;
    style()
        .direction(Direction::Column)
        .padding(p.panel_padding)
        .background(ColorToken::Surface)
        .border(1, ColorToken::Border)
}

pub(crate) fn action(_: &Theme, _: &Look<kivact_interaction::NavOpen>) -> Style {
    style()
        .background(kivact_core::style::Color::TRANSPARENT)
        .color(ColorToken::Text)
        .border(0, kivact_core::style::Color::TRANSPARENT)
}

pub(crate) fn link(t: &Theme, s: &Look<NavigationLinkState>) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(if s.has_description {
            Align::Start
        } else {
            Align::Center
        })
        .gap(t.space().sm)
        .padding(t.space().sm)
        .color(ColorToken::Text)
        .hover(style().background(ColorToken::Hover))
        .focus_visible(style().ring(t.focus_ring(), ColorToken::Focus))
}

pub(crate) fn link_title(_: &Theme, _: &Look<NavigationLinkState>) -> Style {
    style().weight(600)
}

pub(crate) fn link_description(t: &Theme, _: &Look<NavigationLinkState>) -> Style {
    style()
        .font_size(t.text().size_small)
        .color(ColorToken::TextMuted)
}

pub(crate) fn link_icon(_: &Theme, _: &Look<NavigationLinkState>) -> Style {
    style().shrink(0)
}

/// Actual link input and independent description-slot availability.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NavigationLinkState {
    pub input: kivact_core::custom::PartState,
    pub has_description: bool,
}

/// Whether this actual navigation trigger's existing panel is visible.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NavigationTriggerState {
    pub open: bool,
}

recipe_table! {
    /// Navigation landmark, panel trigger and panel surface recipes.
    NavigationMenuRecipes {
        root(root_recipe): kivact_interaction::NavOpen = navigation_root,
        trigger(trigger_recipe): NavigationTriggerState = crate::look_navigation::part,
        action(action_recipe): kivact_interaction::NavOpen = action,
        panel(panel_recipe): kivact_interaction::NavOpen = panel,
        link(link_recipe): NavigationLinkState = link,
        link_title(link_title_recipe): NavigationLinkState = link_title,
        link_description(link_description_recipe): NavigationLinkState = link_description,
        link_icon(link_icon_recipe): NavigationLinkState = link_icon,
    }
}
