//! Role: Icon set of the Creative kit: Tabler outlines under their names.
//!
//! The kit draws every icon with `kivact::view::icon(name)`; its theme
//! puts these images in the theme's [`IconSet`] over the standard marks.

use kivact::style::{IconSet, icons};

/// Every named icon of the kit (Tabler icons, MIT; see LICENSE-tabler-icons.txt)
/// with its svg under `assets/icons`. The toast tones' icons are copies of the
/// glyphs they show.
const ICONS: [(&str, &[u8]); 81] = icons!(
    "assets/icons",
    [
        "click",
        "adjustments-horizontal",
        "checkbox",
        "bulb",
        "adjustments",
        "chart-bar",
        "layout-columns",
        "hierarchy",
        "table",
        "stack",
        "timeline",
        "keyframes",
        "schema",
        "palette",
        "color-picker",
        "menu",
        "app-window",
        "loader",
        "crosshair",
        "ruler",
        "range",
        "eraser",
        "zoom-in",
        "hand",
        "aperture",
        "box",
        "camera",
        "check",
        "chevron-down",
        "chevron-right",
        "x",
        "copy",
        "density",
        "diamond",
        "eye",
        "folder",
        "filter",
        "info",
        "light",
        "link",
        "lock",
        "magnet",
        "maximize",
        "more",
        "move",
        "pause",
        "pencil",
        "play",
        "plus",
        "pointer",
        "record",
        "redo",
        "repeat",
        "rotate",
        "scale",
        "scissors",
        "search",
        "settings",
        "sidebar",
        "start",
        "stop",
        "trash",
        "undo",
        "volume",
        "warning",
        "wave",
        "eye-off",
        "lock-open",
        "pin",
        "chevrons-left",
        "chevrons-right",
        "step-back",
        "step-forward",
        "play-backward",
        "clock",
        "link-off",
        "toast-info",
        "toast-success",
        "toast-warning",
        "toast-danger",
        "toast-accent",
    ]
);

thread_local! {
    /// The kit's icon set, built once per UI thread and shared by its themes.
    static SET: IconSet = build();
}

/// The kit's icon set. Every call returns the same set, so two themes of the
/// kit compare their icons by identity instead of name by name.
pub(crate) fn set() -> IconSet {
    SET.with(IconSet::clone)
}

fn build() -> IconSet {
    let mut set = IconSet::standard();
    set.extend(ICONS);
    set
}
