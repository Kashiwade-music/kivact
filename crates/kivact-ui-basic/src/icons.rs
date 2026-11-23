//! Role: Embedded Lucide icons (ISC), which the kit's theme puts in its icon set.

use kivact::style::{IconSet, icons};

/// Every Lucide icon of the kit as `(name, svg)`, in alphabetical order.
const ICONS: [(&str, &[u8]); 125] = icons!(
    "assets/icons",
    [
        "activity",
        "align-center",
        "align-left",
        "align-right",
        "archive",
        "arrow-down",
        "arrow-left",
        "arrow-right",
        "arrow-up",
        "arrow-up-down",
        "arrow-up-right",
        "audio-waveform",
        "badge-check",
        "bell",
        "bold",
        "book-open",
        "bookmark",
        "bot",
        "calendar",
        "calendar-days",
        "chart-column",
        "chart-pie",
        "check",
        "chevron-down",
        "chevron-left",
        "chevron-right",
        "chevron-up",
        "chevrons-left",
        "chevrons-right",
        "chevrons-up-down",
        "circle",
        "circle-alert",
        "circle-check",
        "circle-dashed",
        "circle-help",
        "circle-x",
        "clipboard",
        "clock",
        "cloud",
        "code",
        "command",
        "copy",
        "credit-card",
        "database",
        "download",
        "ellipsis",
        "ellipsis-vertical",
        "external-link",
        "eye",
        "eye-off",
        "file",
        "file-text",
        "filter",
        "flag",
        "folder",
        "frame",
        "gallery-vertical-end",
        "git-branch",
        "globe",
        "grip-vertical",
        "heart",
        "house",
        "image",
        "inbox",
        "info",
        "italic",
        "keyboard",
        "layout-grid",
        "life-buoy",
        "link",
        "list",
        "loader-circle",
        "lock",
        "log-out",
        "mail",
        "map",
        "map-pin",
        "message-square",
        "mic",
        "minus",
        "moon",
        "music",
        "octagon-x",
        "package",
        "panel-left",
        "paperclip",
        "pause",
        "pencil",
        "phone",
        "play",
        "plus",
        "refresh-cw",
        "rotate-ccw",
        "search",
        "send",
        "server",
        "settings",
        "settings-2",
        "share",
        "shield",
        "shopping-cart",
        "sliders-horizontal",
        "smile",
        "sparkles",
        "square-terminal",
        "star",
        "sun",
        "tag",
        "terminal",
        "thumbs-down",
        "thumbs-up",
        "trash-2",
        "trending-down",
        "trending-up",
        "triangle-alert",
        "underline",
        "upload",
        "user",
        "users",
        "volume-2",
        "wallet",
        "wifi",
        "wrench",
        "x",
        "zap",
    ]
);

/// The Lucide names the kit's [`theme`](crate::theme) puts in its icon set,
/// in alphabetical order: `kivact::view::icon("trash-2")` draws one.
pub const ICON_NAMES: &[&str] = &NAMES;

const NAMES: [&str; 125] = names();

const fn names() -> [&'static str; 125] {
    let mut out = [""; 125];
    let mut i = 0;
    while i < 125 {
        out[i] = ICONS[i].0;
        i += 1;
    }
    out
}

/// The 8 px dot of a chosen radio button (`size-2 rounded-full`) in the
/// 16 px icon box.
static RADIO_DOT: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><circle cx="12" cy="12" r="6" fill="#000"/></svg>"##;

thread_local! {
    /// The kit's icon set, built once per UI thread and shared by its light and dark themes.
    static SET: IconSet = build();
}

/// Every Lucide icon of the kit by name, which [`theme`](crate::theme) installs in
/// the theme's [`IconSet`](kivact::style::IconSet), so a standard
/// component's `Icon::named("trash-2")` draws it. The standard marks are
/// Lyra's: the radio button's `dot` is 8 px, and a menu marks its chosen
/// radio item with a `check` (`menu-radio`), as its check items.
///
/// The set is built once per UI thread and shared, so two themes of the kit
/// compare their icons by identity instead of name by name.
pub(crate) fn set() -> IconSet {
    SET.with(IconSet::clone)
}

fn build() -> IconSet {
    let mut set = IconSet::standard();
    set.extend(ICONS);
    set.extend([
        ("dot", RADIO_DOT),
        ("menu-radio", include_bytes!("../assets/icons/check.svg")),
        // The icons of a toast's tones (sonner's).
        ("toast-info", include_bytes!("../assets/icons/info.svg")),
        (
            "toast-success",
            include_bytes!("../assets/icons/circle-check.svg"),
        ),
        (
            "toast-warning",
            include_bytes!("../assets/icons/triangle-alert.svg"),
        ),
        (
            "toast-danger",
            include_bytes!("../assets/icons/octagon-x.svg"),
        ),
        ("toast-accent", include_bytes!("../assets/icons/info.svg")),
    ]);
    set
}
