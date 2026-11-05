//! Role: Embed the pinned Lucide SVGs used by Astryx Neutral.
use kivact::style::IconSet;
#[expect(
    clippy::disallowed_types,
    reason = "Immutable process-wide identity cache, not deferred instance initialization"
)]
use std::sync::OnceLock;

const ICONS: [(&str, &[u8]); 29] = [
    (
        "arrow-down",
        include_bytes!("../assets/icons/arrow-down.svg"),
    ),
    ("arrow-up", include_bytes!("../assets/icons/arrow-up.svg")),
    (
        "arrow-up-down",
        include_bytes!("../assets/icons/arrow-up-down.svg"),
    ),
    ("calendar", include_bytes!("../assets/icons/calendar.svg")),
    ("check", include_bytes!("../assets/icons/check.svg")),
    (
        "check-check",
        include_bytes!("../assets/icons/check-check.svg"),
    ),
    (
        "chevron-down",
        include_bytes!("../assets/icons/chevron-down.svg"),
    ),
    (
        "chevron-left",
        include_bytes!("../assets/icons/chevron-left.svg"),
    ),
    (
        "chevron-right",
        include_bytes!("../assets/icons/chevron-right.svg"),
    ),
    (
        "chevrons-left",
        include_bytes!("../assets/icons/chevrons-left.svg"),
    ),
    (
        "chevrons-right",
        include_bytes!("../assets/icons/chevrons-right.svg"),
    ),
    (
        "circle-check",
        include_bytes!("../assets/icons/circle-check.svg"),
    ),
    ("circle-x", include_bytes!("../assets/icons/circle-x.svg")),
    ("clock", include_bytes!("../assets/icons/clock.svg")),
    ("columns-2", include_bytes!("../assets/icons/columns-2.svg")),
    ("copy", include_bytes!("../assets/icons/copy.svg")),
    ("ellipsis", include_bytes!("../assets/icons/ellipsis.svg")),
    (
        "external-link",
        include_bytes!("../assets/icons/external-link.svg"),
    ),
    ("eye-off", include_bytes!("../assets/icons/eye-off.svg")),
    ("funnel", include_bytes!("../assets/icons/funnel.svg")),
    ("info", include_bytes!("../assets/icons/info.svg")),
    ("menu", include_bytes!("../assets/icons/menu.svg")),
    ("mic", include_bytes!("../assets/icons/mic.svg")),
    ("search", include_bytes!("../assets/icons/search.svg")),
    ("square", include_bytes!("../assets/icons/square.svg")),
    (
        "triangle-alert",
        include_bytes!("../assets/icons/triangle-alert.svg"),
    ),
    ("upload", include_bytes!("../assets/icons/upload.svg")),
    ("wrench", include_bytes!("../assets/icons/wrench.svg")),
    ("x", include_bytes!("../assets/icons/x.svg")),
];

/// Lucide icon names available in the Soft theme.
pub const ICON_NAMES: &[&str] = &[
    "arrow-down",
    "arrow-up",
    "arrow-up-down",
    "calendar",
    "check",
    "check-check",
    "chevron-down",
    "chevron-left",
    "chevron-right",
    "chevrons-left",
    "chevrons-right",
    "circle-check",
    "circle-x",
    "clock",
    "columns-2",
    "copy",
    "ellipsis",
    "external-link",
    "eye-off",
    "funnel",
    "info",
    "menu",
    "mic",
    "search",
    "square",
    "triangle-alert",
    "upload",
    "wrench",
    "x",
];

pub(super) fn set() -> IconSet {
    #[expect(
        clippy::disallowed_types,
        reason = "Immutable process-wide identity cache, not deferred instance initialization"
    )]
    static SET: OnceLock<IconSet> = OnceLock::new();
    SET.get_or_init(|| {
        let mut set = IconSet::standard();
        set.extend(ICONS);
        set
    })
    .clone()
}
