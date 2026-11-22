//! Role: Embedded Tabler outline icons from the reference prototype (MIT).

use kivact::style::{IconSet, icons};

const ICONS: [(&str, &[u8]); 99] = icons!(
    "assets/icons",
    [
        "home",
        "layout-dashboard",
        "users",
        "user",
        "user-plus",
        "settings",
        "bell",
        "search",
        "plus",
        "minus",
        "x",
        "check",
        "dot",
        "chevron-down",
        "chevron-right",
        "chevron-left",
        "chevrons-left",
        "chevrons-right",
        "chevron-up",
        "star",
        "heart",
        "message-circle",
        "mail",
        "calendar",
        "clock",
        "file-text",
        "folder",
        "upload",
        "download",
        "trash",
        "pencil",
        "copy",
        "external-link",
        "info-circle",
        "alert-triangle",
        "circle-check",
        "alert-circle",
        "sun",
        "moon",
        "menu-2",
        "dots",
        "dots-vertical",
        "filter",
        "arrows-sort",
        "arrow-up-right",
        "arrow-down-right",
        "trending-up",
        "school",
        "book",
        "sparkles",
        "bolt",
        "shield",
        "target",
        "map-pin",
        "send",
        "paperclip",
        "mood-smile",
        "command",
        "logout",
        "refresh",
        "eye",
        "eye-off",
        "lock",
        "cloud-upload",
        "layout-grid",
        "list",
        "adjustments-horizontal",
        "chart-bar",
        "chart-line",
        "gift",
        "coin",
        "ticket",
        "inbox",
        "archive",
        "tag",
        "flag",
        "bookmark",
        "link",
        "help-circle",
        "selector",
        "arrow-right",
        "arrow-left",
        "loader-2",
        "cloud",
        "rocket",
        "puzzle",
        "palette",
        "keyboard",
        "sword",
        "crosshair",
        "building-community",
        "clipboard-list",
        "layout-sidebar-left-collapse",
        "player-play",
        "bell-ringing",
        "circle-dot",
        "wand",
        "typography",
        "table",
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
/// The set is built once per UI thread, so two themes of the kit compare their
/// icons by identity instead of name by name.
pub(crate) fn set() -> IconSet {
    SET.with(IconSet::clone)
}

fn build() -> IconSet {
    let mut set = IconSet::standard();
    set.extend(ICONS);
    // The icons of a toast's tones.
    set.extend([
        (
            "toast-info",
            include_bytes!("../assets/icons/info-circle.svg").as_slice(),
        ),
        (
            "toast-success",
            include_bytes!("../assets/icons/circle-check.svg").as_slice(),
        ),
        (
            "toast-warning",
            include_bytes!("../assets/icons/alert-triangle.svg").as_slice(),
        ),
        (
            "toast-danger",
            include_bytes!("../assets/icons/alert-circle.svg").as_slice(),
        ),
        (
            "toast-accent",
            include_bytes!("../assets/icons/info-circle.svg").as_slice(),
        ),
    ]);
    set
}
