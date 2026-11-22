//! Role: Tabler MIT icons retained from the referenced prototype.
use kivact::style::{IconSet, icons};

/// The kit's icons by name, embedded from `assets/icons`.
const ICONS: [(&str, &[u8]); 60] = icons!(
    "assets/icons",
    [
        "search",
        "menu",
        "plus",
        "check",
        "x",
        "dot",
        "chevron-down",
        "chevron-right",
        "chevron-left",
        "chevron-up",
        "selector",
        "bell",
        "adjustments-horizontal",
        "info-circle",
        "alert-triangle",
        "alert-octagon",
        "circle-check",
        "file-text",
        "download",
        "upload",
        "dots",
        "filter",
        "moon",
        "sun",
        "layout-grid",
        "user",
        "users",
        "shield-check",
        "bolt",
        "trash",
        "pencil",
        "copy",
        "terminal-2",
        "clock",
        "refresh",
        "arrow-up-right",
        "arrow-down-right",
        "arrow-up",
        "arrow-down",
        "settings",
        "command",
        "folder",
        "folder-open",
        "target",
        "radar",
        "database",
        "cloud-upload",
        "logout",
        "external-link",
        "lock",
        "activity",
        "box",
        "star",
        "player-play",
        "player-pause",
        "minus",
        "square",
        "maximize",
        "device-floppy",
        "history",
    ]
);

thread_local! {
    /// The kit's icon set, built once per UI thread and shared by its light and dark themes.
    static SET: IconSet = build();
}

/// Every icon of the kit by name, which [`theme`](crate::theme) installs in
/// the theme's [`IconSet`](kivact::style::IconSet), so
/// `kivact::view::icon(name)` and a standard component's
/// `Icon::named(..)` draw it. The kit's components draw their icons the same
/// way, so another theme may give a name its own glyph.
///
/// The set is built once per UI thread and shared, so two themes of the kit
/// compare their icons by identity instead of name by name.
pub(crate) fn set() -> IconSet {
    SET.with(IconSet::clone)
}

fn build() -> IconSet {
    let mut set = IconSet::standard();
    set.extend(ICONS);
    // A toast marks every tone with a diamond, which its recipe colors.
    let diamond: &'static [u8] = include_bytes!("../assets/icons/toast-diamond.svg");
    set.extend(
        [
            "toast-info",
            "toast-success",
            "toast-warning",
            "toast-danger",
            "toast-accent",
        ]
        .map(|name| (name, diamond)),
    );
    set
}
