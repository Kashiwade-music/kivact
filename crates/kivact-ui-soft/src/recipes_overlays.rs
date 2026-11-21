//! Role: Install Astryx Neutral's recipes of the overlays group: the pickers, the menus, the dialogs and the popups.
use kivact::style::Theme;

/// Install the kit's recipes of the overlays group.
pub(crate) fn install(theme: Theme) -> Theme {
    let theme = crate::recipes_overlays_select::install(theme);
    crate::recipes_overlays_menu::install(theme)
}
