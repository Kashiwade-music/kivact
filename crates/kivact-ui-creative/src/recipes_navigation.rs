//! Role: Creative appearance of the standard tabs: a recessed track with tabs that light up when selected.
use crate::recipe_parts::{tab_option_style, tab_track_style};
use kivact::interaction::{TabListState, TabState};
use kivact::style::{Style, Theme, style};
use kivact::ui::theme::{Look, TabsTheme};

/// The theme of the tabs: the track, each tab and an unstyled panel.
pub(crate) fn tabs() -> TabsTheme {
    TabsTheme::default()
        .list(tabs_list)
        .tab(tabs_tab)
        .panel(tabs_panel)
}

fn tabs_list(_t: &Theme, _s: &Look<TabListState>, _base: Style) -> Style {
    tab_track_style()
}

fn tabs_tab(t: &Theme, _s: &Look<TabState>, _base: Style) -> Style {
    tab_option_style(t, false)
}

fn tabs_panel(_t: &Theme, _s: &Look<TabState>, _base: Style) -> Style {
    style()
}

/// Install the navigation recipes into `theme`.
pub(crate) fn install(theme: Theme) -> Theme {
    theme.with(tabs())
}
