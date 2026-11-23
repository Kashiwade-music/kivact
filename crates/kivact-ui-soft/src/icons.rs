//! Role: Embed the pinned Lucide SVGs used by Astryx Neutral.
use kivact::style::{IconSet, icons};

const ICONS: [(&str, &[u8]); 29] = icons!(
    "assets/icons",
    [
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
    ]
);

const NAMES: [&str; 29] = names();

/// Lucide icon names available in the Soft theme, in the order of the embedded icons.
pub const ICON_NAMES: &[&str] = &NAMES;

const fn names() -> [&'static str; 29] {
    let mut out = [""; 29];
    let mut i = 0;
    while i < 29 {
        out[i] = ICONS[i].0;
        i += 1;
    }
    out
}

thread_local! {
    /// The kit's icon set, built once per UI thread and shared by its light and dark themes.
    static SET: IconSet = build();
}

pub(crate) fn set() -> IconSet {
    SET.with(IconSet::clone)
}

fn build() -> IconSet {
    let mut set = IconSet::standard();
    set.extend(ICONS);
    set
}
