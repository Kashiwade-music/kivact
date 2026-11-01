//! Role: Tabler MIT icons retained from the referenced prototype.

/// The names of the kit's icons, which [`set`] holds.
const NAMES: &[&str] = &[
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
];

/// The embedded image of a known name.
fn bytes(name: &str) -> Option<&'static [u8]> {
    Some(match name {
        "menu" => include_bytes!("../assets/icons/menu.svg"),
        "search" => include_bytes!("../assets/icons/search.svg"),
        "plus" => include_bytes!("../assets/icons/plus.svg"),
        "check" => include_bytes!("../assets/icons/check.svg"),
        "x" => include_bytes!("../assets/icons/x.svg"),
        "dot" => include_bytes!("../assets/icons/dot.svg"),
        "chevron-down" => include_bytes!("../assets/icons/chevron-down.svg"),
        "chevron-right" => include_bytes!("../assets/icons/chevron-right.svg"),
        "chevron-left" => include_bytes!("../assets/icons/chevron-left.svg"),
        "chevron-up" => include_bytes!("../assets/icons/chevron-up.svg"),
        "selector" => include_bytes!("../assets/icons/selector.svg"),
        "bell" => include_bytes!("../assets/icons/bell.svg"),
        "adjustments-horizontal" => include_bytes!("../assets/icons/adjustments-horizontal.svg"),
        "info-circle" => include_bytes!("../assets/icons/info-circle.svg"),
        "alert-triangle" => include_bytes!("../assets/icons/alert-triangle.svg"),
        "alert-octagon" => include_bytes!("../assets/icons/alert-octagon.svg"),
        "circle-check" => include_bytes!("../assets/icons/circle-check.svg"),
        "file-text" => include_bytes!("../assets/icons/file-text.svg"),
        "download" => include_bytes!("../assets/icons/download.svg"),
        "upload" => include_bytes!("../assets/icons/upload.svg"),
        "dots" => include_bytes!("../assets/icons/dots.svg"),
        "filter" => include_bytes!("../assets/icons/filter.svg"),
        "moon" => include_bytes!("../assets/icons/moon.svg"),
        "sun" => include_bytes!("../assets/icons/sun.svg"),
        "layout-grid" => include_bytes!("../assets/icons/layout-grid.svg"),
        "user" => include_bytes!("../assets/icons/user.svg"),
        "users" => include_bytes!("../assets/icons/users.svg"),
        "shield-check" => include_bytes!("../assets/icons/shield-check.svg"),
        "bolt" => include_bytes!("../assets/icons/bolt.svg"),
        "trash" => include_bytes!("../assets/icons/trash.svg"),
        "pencil" => include_bytes!("../assets/icons/pencil.svg"),
        "copy" => include_bytes!("../assets/icons/copy.svg"),
        "terminal-2" => include_bytes!("../assets/icons/terminal-2.svg"),
        "clock" => include_bytes!("../assets/icons/clock.svg"),
        "refresh" => include_bytes!("../assets/icons/refresh.svg"),
        "arrow-up-right" => include_bytes!("../assets/icons/arrow-up-right.svg"),
        "arrow-down-right" => include_bytes!("../assets/icons/arrow-down-right.svg"),
        "arrow-up" => include_bytes!("../assets/icons/arrow-up.svg"),
        "arrow-down" => include_bytes!("../assets/icons/arrow-down.svg"),
        "settings" => include_bytes!("../assets/icons/settings.svg"),
        "command" => include_bytes!("../assets/icons/command.svg"),
        "folder" => include_bytes!("../assets/icons/folder.svg"),
        "folder-open" => include_bytes!("../assets/icons/folder-open.svg"),
        "target" => include_bytes!("../assets/icons/target.svg"),
        "radar" => include_bytes!("../assets/icons/radar.svg"),
        "database" => include_bytes!("../assets/icons/database.svg"),
        "cloud-upload" => include_bytes!("../assets/icons/cloud-upload.svg"),
        "logout" => include_bytes!("../assets/icons/logout.svg"),
        "external-link" => include_bytes!("../assets/icons/external-link.svg"),
        "lock" => include_bytes!("../assets/icons/lock.svg"),
        "activity" => include_bytes!("../assets/icons/activity.svg"),
        "box" => include_bytes!("../assets/icons/box.svg"),
        "star" => include_bytes!("../assets/icons/star.svg"),
        "player-play" => include_bytes!("../assets/icons/player-play.svg"),
        "player-pause" => include_bytes!("../assets/icons/player-pause.svg"),
        "minus" => include_bytes!("../assets/icons/minus.svg"),
        "square" => include_bytes!("../assets/icons/square.svg"),
        "maximize" => include_bytes!("../assets/icons/maximize.svg"),
        "device-floppy" => include_bytes!("../assets/icons/device-floppy.svg"),
        "history" => include_bytes!("../assets/icons/history.svg"),
        _ => return None,
    })
}

/// Every icon of the kit by name, which [`theme`](crate::theme) installs in
/// the theme's [`IconSet`](kivact::style::IconSet), so
/// `kivact::control::icon(name)` and a standard component's
/// `Icon::named(..)` draw it. The kit's components draw their icons the same
/// way, so another theme may give a name its own glyph.
///
/// The set is built once and shared, so two themes of the kit compare their
/// icons by identity instead of name by name.
pub(crate) fn set() -> kivact::style::IconSet {
    static SET: std::sync::OnceLock<kivact::style::IconSet> = std::sync::OnceLock::new();
    SET.get_or_init(|| {
        let mut set = kivact::style::IconSet::standard();
        set.extend(
            NAMES
                .iter()
                .filter_map(|name| bytes(name).map(|svg| (*name, svg))),
        );
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
    })
    .clone()
}
