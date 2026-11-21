//! Role: The navigation menu recipe state, tokens, recipe table and default looks.

use crate::recipe::recipe_table;
use crate::variants::Look;
use kivact_core::style::{Align, FlexDirection, Style, Theme, style};
pub(crate) fn navigation_root(t: &Theme, _: &Look<kivact_interaction::NavOpen>) -> Style {
    let p = &t.ext::<crate::tokens::UiTokens>().navigation_menu;
    style()
        .flex_direction(FlexDirection::Row)
        .gap(p.gap)
        .padding(p.padding)
        .font_size(t.get(&kivact_core::style::font::SIZE_VAR))
        .color(kivact_core::style::color::TEXT)
}

pub(crate) fn panel(t: &Theme, _: &Look<kivact_interaction::NavOpen>) -> Style {
    let p = &t.ext::<crate::tokens::UiTokens>().navigation_menu;
    style()
        .flex_direction(FlexDirection::Column)
        .padding(p.panel_padding)
        .background(kivact_core::style::color::SURFACE)
        .border(1, kivact_core::style::color::BORDER)
}

pub(crate) fn action(_: &Theme, _: &Look<kivact_interaction::NavOpen>) -> Style {
    style()
        .background(kivact_core::style::Color::TRANSPARENT)
        .color(kivact_core::style::color::TEXT)
        .border(0, kivact_core::style::Color::TRANSPARENT)
}

pub(crate) fn link(t: &Theme, s: &Look<NavigationLinkState>) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(if s.has_description {
            Align::Start
        } else {
            Align::Center
        })
        .gap(t.get(&kivact_core::style::space::SM_VAR))
        .padding(t.get(&kivact_core::style::space::SM_VAR))
        .color(kivact_core::style::color::TEXT)
        .hover(style().background(kivact_core::style::color::HOVER))
        .focus_visible(style().outline(t.focus_ring(), kivact_core::style::color::FOCUS))
}

pub(crate) fn link_title(_: &Theme, _: &Look<NavigationLinkState>) -> Style {
    style().font_weight(600)
}

pub(crate) fn link_description(t: &Theme, _: &Look<NavigationLinkState>) -> Style {
    style()
        .font_size(t.get(&kivact_core::style::font::SIZE_SMALL_VAR))
        .color(kivact_core::style::color::TEXT_MUTED)
}

pub(crate) fn link_icon(_: &Theme, _: &Look<NavigationLinkState>) -> Style {
    style().flex_shrink(0)
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
        trigger(trigger_recipe): NavigationTriggerState = crate::look_common::part,
        action(action_recipe): kivact_interaction::NavOpen = action,
        panel(panel_recipe): kivact_interaction::NavOpen = panel,
        link(link_recipe): NavigationLinkState = link,
        link_title(link_title_recipe): NavigationLinkState = link_title,
        link_description(link_description_recipe): NavigationLinkState = link_description,
        link_icon(link_icon_recipe): NavigationLinkState = link_icon,
    }
}
