//! Role: Icon set of the Creative kit: Tabler outlines under their names.
//!
//! The kit draws every icon with `kivact::view::icon(name)`; its theme
//! puts these images in the theme's [`IconSet`] over the standard marks.

use kivact::style::IconSet;

/// The SVG body of each named icon of the kit (Tabler icons, MIT; see
/// LICENSE-tabler-icons.txt), by name.
const BODIES: &[(&str, &str)] = &[
    (
        "click",
        r#"<path d="M3 12l3 0" /> <path d="M12 3l0 3" /> <path d="M7.8 7.8l-2.2 -2.2" /> <path d="M16.2 7.8l2.2 -2.2" /> <path d="M7.8 16.2l-2.2 2.2" /> <path d="M12 12l9 3l-4 2l-2 4l-3 -9" />"#,
    ),
    (
        "adjustments-horizontal",
        r#"<path d="M14 6m-2 0a2 2 0 1 0 4 0a2 2 0 1 0 -4 0" /> <path d="M4 6l8 0" /> <path d="M16 6l4 0" /> <path d="M8 12m-2 0a2 2 0 1 0 4 0a2 2 0 1 0 -4 0" /> <path d="M4 12l2 0" /> <path d="M10 12l10 0" /> <path d="M17 18m-2 0a2 2 0 1 0 4 0a2 2 0 1 0 -4 0" /> <path d="M4 18l11 0" /> <path d="M19 18l1 0" />"#,
    ),
    (
        "checkbox",
        r#"<path d="M9 11l3 3l8 -8" /> <path d="M20 12v6a2 2 0 0 1 -2 2h-12a2 2 0 0 1 -2 -2v-12a2 2 0 0 1 2 -2h9" />"#,
    ),
    (
        "bulb",
        r#"<path d="M3 12h1m8 -9v1m8 8h1m-15.4 -6.4l.7 .7m12.1 -.7l-.7 .7" /> <path d="M9 16a5 5 0 1 1 6 0a3.5 3.5 0 0 0 -1 3a2 2 0 0 1 -4 0a3.5 3.5 0 0 0 -1 -3" /> <path d="M9.7 17l4.6 0" />"#,
    ),
    (
        "adjustments",
        r#"<path d="M4 10a2 2 0 1 0 4 0a2 2 0 0 0 -4 0" /> <path d="M6 4v4" /> <path d="M6 12v8" /> <path d="M10 16a2 2 0 1 0 4 0a2 2 0 0 0 -4 0" /> <path d="M12 4v10" /> <path d="M12 18v2" /> <path d="M16 7a2 2 0 1 0 4 0a2 2 0 0 0 -4 0" /> <path d="M18 4v1" /> <path d="M18 9v11" />"#,
    ),
    (
        "chart-bar",
        r#"<path d="M3 13a1 1 0 0 1 1 -1h4a1 1 0 0 1 1 1v6a1 1 0 0 1 -1 1h-4a1 1 0 0 1 -1 -1z" /> <path d="M15 9a1 1 0 0 1 1 -1h4a1 1 0 0 1 1 1v10a1 1 0 0 1 -1 1h-4a1 1 0 0 1 -1 -1z" /> <path d="M9 5a1 1 0 0 1 1 -1h4a1 1 0 0 1 1 1v14a1 1 0 0 1 -1 1h-4a1 1 0 0 1 -1 -1z" /> <path d="M4 20h14" />"#,
    ),
    (
        "layout-columns",
        r#"<path d="M4 4m0 2a2 2 0 0 1 2 -2h12a2 2 0 0 1 2 2v12a2 2 0 0 1 -2 2h-12a2 2 0 0 1 -2 -2z" /> <path d="M12 4l0 16" />"#,
    ),
    (
        "hierarchy",
        r#"<path d="M12 5m-2 0a2 2 0 1 0 4 0a2 2 0 1 0 -4 0" /> <path d="M5 19m-2 0a2 2 0 1 0 4 0a2 2 0 1 0 -4 0" /> <path d="M19 19m-2 0a2 2 0 1 0 4 0a2 2 0 1 0 -4 0" /> <path d="M6.5 17.5l5.5 -4.5l5.5 4.5" /> <path d="M12 7l0 6" />"#,
    ),
    (
        "table",
        r#"<path d="M3 5a2 2 0 0 1 2 -2h14a2 2 0 0 1 2 2v14a2 2 0 0 1 -2 2h-14a2 2 0 0 1 -2 -2v-14z" /> <path d="M3 10h18" /> <path d="M10 3v18" />"#,
    ),
    (
        "stack",
        r#"<path d="M12 4l-8 4l8 4l8 -4l-8 -4" /> <path d="M4 12l8 4l8 -4" /> <path d="M4 16l8 4l8 -4" />"#,
    ),
    (
        "timeline",
        r#"<path d="M4 16l6 -7l5 5l5 -6" /> <path d="M15 14m-1 0a1 1 0 1 0 2 0a1 1 0 1 0 -2 0" /> <path d="M10 9m-1 0a1 1 0 1 0 2 0a1 1 0 1 0 -2 0" /> <path d="M4 16m-1 0a1 1 0 1 0 2 0a1 1 0 1 0 -2 0" /> <path d="M20 8m-1 0a1 1 0 1 0 2 0a1 1 0 1 0 -2 0" />"#,
    ),
    (
        "keyframes",
        r#"<path d="M9.225 18.412a1.595 1.595 0 0 1 -1.225 .588c-.468 0 -.914 -.214 -1.225 -.588l-4.361 -5.248a1.844 1.844 0 0 1 0 -2.328l4.361 -5.248a1.595 1.595 0 0 1 1.225 -.588c.468 0 .914 .214 1.225 .588l4.361 5.248a1.844 1.844 0 0 1 0 2.328l-4.361 5.248z" /> <path d="M17 5l4.586 5.836a1.844 1.844 0 0 1 0 2.328l-4.586 5.836" /> <path d="M13 5l4.586 5.836a1.844 1.844 0 0 1 0 2.328l-4.586 5.836" />"#,
    ),
    (
        "schema",
        r#"<path d="M5 2h5v4h-5z" /> <path d="M15 10h5v4h-5z" /> <path d="M5 18h5v4h-5z" /> <path d="M5 10h5v4h-5z" /> <path d="M10 12h5" /> <path d="M7.5 6v4" /> <path d="M7.5 14v4" />"#,
    ),
    (
        "palette",
        r#"<path d="M12 21a9 9 0 0 1 0 -18c4.97 0 9 3.582 9 8c0 1.06 -.474 2.078 -1.318 2.828c-.844 .75 -1.989 1.172 -3.182 1.172h-2.5a2 2 0 0 0 -1 3.75a1.3 1.3 0 0 1 -1 2.25" /> <path d="M8.5 10.5m-1 0a1 1 0 1 0 2 0a1 1 0 1 0 -2 0" /> <path d="M12.5 7.5m-1 0a1 1 0 1 0 2 0a1 1 0 1 0 -2 0" /> <path d="M16.5 10.5m-1 0a1 1 0 1 0 2 0a1 1 0 1 0 -2 0" />"#,
    ),
    (
        "color-picker",
        r#"<path d="M11 7l6 6" /> <path d="M4 16l11.7 -11.7a1 1 0 0 1 1.4 0l2.6 2.6a1 1 0 0 1 0 1.4l-11.7 11.7h-4v-4z" />"#,
    ),
    (
        "menu",
        r#"<path d="M4 6l16 0" /> <path d="M4 12l16 0" /> <path d="M4 18l16 0" />"#,
    ),
    (
        "app-window",
        r#"<path d="M3 5m0 2a2 2 0 0 1 2 -2h14a2 2 0 0 1 2 2v10a2 2 0 0 1 -2 2h-14a2 2 0 0 1 -2 -2z" /> <path d="M6 8h.01" /> <path d="M9 8h.01" />"#,
    ),
    ("loader", r#"<path d="M12 3a9 9 0 1 0 9 9" />"#),
    (
        "crosshair",
        r#"<path d="M4 8v-2a2 2 0 0 1 2 -2h2" /> <path d="M4 16v2a2 2 0 0 0 2 2h2" /> <path d="M16 4h2a2 2 0 0 1 2 2v2" /> <path d="M16 20h2a2 2 0 0 0 2 -2v-2" /> <path d="M9 12l6 0" /> <path d="M12 9l0 6" />"#,
    ),
    (
        "ruler",
        r#"<path d="M17 3l4 4l-14 14l-4 -4z" /> <path d="M16 7l-1.5 -1.5" /> <path d="M13 10l-1.5 -1.5" /> <path d="M10 13l-1.5 -1.5" /> <path d="M7 16l-1.5 -1.5" />"#,
    ),
    (
        "range",
        r#"<path d="M7 8l-4 4l4 4" /> <path d="M17 8l4 4l-4 4" /> <path d="M3 12l18 0" />"#,
    ),
    (
        "eraser",
        r#"<path d="M19 20h-10.5l-4.21 -4.3a1 1 0 0 1 0 -1.41l10 -10a1 1 0 0 1 1.41 0l5 5a1 1 0 0 1 0 1.41l-9.2 9.3" /> <path d="M18 13.3l-6.3 -6.3" />"#,
    ),
    (
        "zoom-in",
        r#"<path d="M10 10m-7 0a7 7 0 1 0 14 0a7 7 0 1 0 -14 0" /> <path d="M7 10l6 0" /> <path d="M10 7l0 6" /> <path d="M21 21l-6 -6" />"#,
    ),
    (
        "hand",
        r#"<path d="M8 13v-7.5a1.5 1.5 0 0 1 3 0v6.5" /> <path d="M11 5.5v-2a1.5 1.5 0 1 1 3 0v8.5" /> <path d="M14 5.5a1.5 1.5 0 0 1 3 0v6.5" /> <path d="M17 7.5a1.5 1.5 0 0 1 3 0v8.5a6 6 0 0 1 -6 6h-2h.208a6 6 0 0 1 -5.012 -2.7a69.74 69.74 0 0 1 -.196 -.3c-.312 -.479 -1.407 -2.388 -3.286 -5.728a1.5 1.5 0 0 1 .536 -2.022a1.867 1.867 0 0 1 2.28 .28l1.47 1.47" />"#,
    ),
    (
        "aperture",
        r#"<path d="M12 12m-9 0a9 9 0 1 0 18 0a9 9 0 1 0 -18 0" /> <path d="M3.6 15h10.55" /> <path d="M6.551 4.938l3.26 10.034" /> <path d="M17.032 4.636l-8.535 6.201" /> <path d="M20.559 14.51l-8.535 -6.201" /> <path d="M12.257 20.916l3.261 -10.034" />"#,
    ),
    (
        "box",
        r#"<path d="M12 3l8 4.5l0 9l-8 4.5l-8 -4.5l0 -9l8 -4.5" /> <path d="M12 12l8 -4.5" /> <path d="M12 12l0 9" /> <path d="M12 12l-8 -4.5" />"#,
    ),
    (
        "camera",
        r#"<path d="M5 7h1a2 2 0 0 0 2 -2a1 1 0 0 1 1 -1h6a1 1 0 0 1 1 1a2 2 0 0 0 2 2h1a2 2 0 0 1 2 2v9a2 2 0 0 1 -2 2h-14a2 2 0 0 1 -2 -2v-9a2 2 0 0 1 2 -2" /> <path d="M9 13a3 3 0 1 0 6 0a3 3 0 0 0 -6 0" />"#,
    ),
    ("check", r#"<path d="M5 12l5 5l10 -10" />"#),
    ("chevron-down", r#"<path d="M6 9l6 6l6 -6" />"#),
    ("chevron-right", r#"<path d="M9 6l6 6l-6 6" />"#),
    ("x", r#"<path d="M18 6l-12 12" /> <path d="M6 6l12 12" />"#),
    (
        "copy",
        r#"<path d="M7 7m0 2.667a2.667 2.667 0 0 1 2.667 -2.667h8.666a2.667 2.667 0 0 1 2.667 2.667v8.666a2.667 2.667 0 0 1 -2.667 2.667h-8.666a2.667 2.667 0 0 1 -2.667 -2.667z" /> <path d="M4.012 16.737a2.005 2.005 0 0 1 -1.012 -1.737v-10c0 -1.1 .9 -2 2 -2h10c.75 0 1.158 .385 1.5 1" />"#,
    ),
    (
        "density",
        r#"<path d="M4 4l16 0" /> <path d="M4 20l16 0" /> <path d="M6 9m0 2a2 2 0 0 1 2 -2h8a2 2 0 0 1 2 2v2a2 2 0 0 1 -2 2h-8a2 2 0 0 1 -2 -2z" />"#,
    ),
    (
        "diamond",
        r#"<path d="M6 5h12l3 5l-8.5 9.5a.7 .7 0 0 1 -1 0l-8.5 -9.5l3 -5" /> <path d="M10 12l-2 -2.2l.6 -1" />"#,
    ),
    (
        "eye",
        r#"<path d="M10 12a2 2 0 1 0 4 0a2 2 0 0 0 -4 0" /> <path d="M21 12c-2.4 4 -5.4 6 -9 6c-3.6 0 -6.6 -2 -9 -6c2.4 -4 5.4 -6 9 -6c3.6 0 6.6 2 9 6" />"#,
    ),
    (
        "folder",
        r#"<path d="M5 4h4l3 3h7a2 2 0 0 1 2 2v8a2 2 0 0 1 -2 2h-14a2 2 0 0 1 -2 -2v-11a2 2 0 0 1 2 -2" />"#,
    ),
    ("filter", r#"<path d="M5.5 5h13l-5 6v6l-3 2v-8z" />"#),
    (
        "info",
        r#"<path d="M3 12a9 9 0 1 0 18 0a9 9 0 0 0 -18 0" /> <path d="M12 9h.01" /> <path d="M11 12h1v4h1" />"#,
    ),
    (
        "light",
        r#"<path d="M3 12h1m8 -9v1m8 8h1m-15.4 -6.4l.7 .7m12.1 -.7l-.7 .7" /> <path d="M9 16a5 5 0 1 1 6 0a3.5 3.5 0 0 0 -1 3a2 2 0 0 1 -4 0a3.5 3.5 0 0 0 -1 -3" /> <path d="M9.7 17l4.6 0" />"#,
    ),
    (
        "link",
        r#"<path d="M9 15l6 -6" /> <path d="M11 6l.463 -.536a5 5 0 0 1 7.071 7.072l-.534 .464" /> <path d="M13 18l-.397 .534a5.068 5.068 0 0 1 -7.127 0a4.972 4.972 0 0 1 0 -7.071l.524 -.463" />"#,
    ),
    (
        "lock",
        r#"<path d="M5 13a2 2 0 0 1 2 -2h10a2 2 0 0 1 2 2v6a2 2 0 0 1 -2 2h-10a2 2 0 0 1 -2 -2v-6z" /> <path d="M11 16a1 1 0 1 0 2 0a1 1 0 0 0 -2 0" /> <path d="M8 11v-4a4 4 0 1 1 8 0v4" />"#,
    ),
    (
        "magnet",
        r#"<path d="M4 13v-8a2 2 0 0 1 2 -2h1a2 2 0 0 1 2 2v8a2 2 0 0 0 6 0v-8a2 2 0 0 1 2 -2h1a2 2 0 0 1 2 2v8a8 8 0 0 1 -16 0" /> <path d="M4 8l5 0" /> <path d="M15 8l4 0" />"#,
    ),
    (
        "maximize",
        r#"<path d="M16 4l4 0l0 4" /> <path d="M14 10l6 -6" /> <path d="M8 20l-4 0l0 -4" /> <path d="M4 20l6 -6" /> <path d="M16 20l4 0l0 -4" /> <path d="M14 14l6 6" /> <path d="M8 4l-4 0l0 4" /> <path d="M4 4l6 6" />"#,
    ),
    (
        "more",
        r#"<path d="M5 12m-1 0a1 1 0 1 0 2 0a1 1 0 1 0 -2 0" /> <path d="M12 12m-1 0a1 1 0 1 0 2 0a1 1 0 1 0 -2 0" /> <path d="M19 12m-1 0a1 1 0 1 0 2 0a1 1 0 1 0 -2 0" />"#,
    ),
    (
        "move",
        r#"<path d="M18 9l3 3l-3 3" /> <path d="M15 12h6" /> <path d="M6 9l-3 3l3 3" /> <path d="M3 12h6" /> <path d="M9 18l3 3l3 -3" /> <path d="M12 15v6" /> <path d="M15 6l-3 -3l-3 3" /> <path d="M12 3v6" />"#,
    ),
    (
        "pause",
        r#"<g fill="currentColor" stroke="none"><path d="M9 4h-2a2 2 0 0 0 -2 2v12a2 2 0 0 0 2 2h2a2 2 0 0 0 2 -2v-12a2 2 0 0 0 -2 -2z" /> <path d="M17 4h-2a2 2 0 0 0 -2 2v12a2 2 0 0 0 2 2h2a2 2 0 0 0 2 -2v-12a2 2 0 0 0 -2 -2z" /></g>"#,
    ),
    (
        "pencil",
        r#"<path d="M4 20h4l10.5 -10.5a2.828 2.828 0 1 0 -4 -4l-10.5 10.5v4" /> <path d="M13.5 6.5l4 4" />"#,
    ),
    (
        "play",
        r#"<g fill="currentColor" stroke="none"><path d="M6 4v16a1 1 0 0 0 1.524 .852l13 -8a1 1 0 0 0 0 -1.704l-13 -8a1 1 0 0 0 -1.524 .852z" /></g>"#,
    ),
    ("plus", r#"<path d="M12 5l0 14" /> <path d="M5 12l14 0" />"#),
    (
        "pointer",
        r#"<path d="M7.904 17.563a1.2 1.2 0 0 0 2.228 .308l2.09 -3.093l4.907 4.907a1.067 1.067 0 0 0 1.509 0l1.047 -1.047a1.067 1.067 0 0 0 0 -1.509l-4.907 -4.907l3.113 -2.09a1.2 1.2 0 0 0 -.309 -2.228l-13.582 -3.904l3.904 13.563z" />"#,
    ),
    (
        "record",
        r#"<g fill="currentColor" stroke="none"><path d="M8 5.072a8 8 0 1 1 -3.995 7.213l-.005 -.285l.005 -.285a8 8 0 0 1 3.995 -6.643z" /></g>"#,
    ),
    (
        "redo",
        r#"<path d="M15 14l4 -4l-4 -4" /> <path d="M19 10h-11a4 4 0 1 0 0 8h1" />"#,
    ),
    (
        "repeat",
        r#"<path d="M4 12v-3a3 3 0 0 1 3 -3h13m-3 -3l3 3l-3 3" /> <path d="M20 12v3a3 3 0 0 1 -3 3h-13m3 3l-3 -3l3 -3" />"#,
    ),
    (
        "rotate",
        r#"<path d="M4.05 11a8 8 0 1 1 .5 4m-.5 5v-5h5" />"#,
    ),
    (
        "scale",
        r#"<path d="M16 4l4 0l0 4" /> <path d="M14 10l6 -6" /> <path d="M8 20l-4 0l0 -4" /> <path d="M4 20l6 -6" />"#,
    ),
    (
        "scissors",
        r#"<path d="M6 7m-3 0a3 3 0 1 0 6 0a3 3 0 1 0 -6 0" /> <path d="M6 17m-3 0a3 3 0 1 0 6 0a3 3 0 1 0 -6 0" /> <path d="M8.6 8.6l10.4 10.4" /> <path d="M8.6 15.4l10.4 -10.4" />"#,
    ),
    (
        "search",
        r#"<path d="M10 10m-7 0a7 7 0 1 0 14 0a7 7 0 1 0 -14 0" /> <path d="M21 21l-6 -6" />"#,
    ),
    (
        "settings",
        r#"<path d="M10.325 4.317c.426 -1.756 2.924 -1.756 3.35 0a1.724 1.724 0 0 0 2.573 1.066c1.543 -.94 3.31 .826 2.37 2.37a1.724 1.724 0 0 0 1.065 2.572c1.756 .426 1.756 2.924 0 3.35a1.724 1.724 0 0 0 -1.066 2.573c.94 1.543 -.826 3.31 -2.37 2.37a1.724 1.724 0 0 0 -2.572 1.065c-.426 1.756 -2.924 1.756 -3.35 0a1.724 1.724 0 0 0 -2.573 -1.066c-1.543 .94 -3.31 -.826 -2.37 -2.37a1.724 1.724 0 0 0 -1.065 -2.572c-1.756 -.426 -1.756 -2.924 0 -3.35a1.724 1.724 0 0 0 1.066 -2.573c-.94 -1.543 .826 -3.31 2.37 -2.37c1 .608 2.296 .07 2.572 -1.065z" /> <path d="M9 12a3 3 0 1 0 6 0a3 3 0 0 0 -6 0" />"#,
    ),
    (
        "sidebar",
        r#"<path d="M4 4m0 2a2 2 0 0 1 2 -2h12a2 2 0 0 1 2 2v12a2 2 0 0 1 -2 2h-12a2 2 0 0 1 -2 -2z" /> <path d="M9 4l0 16" />"#,
    ),
    (
        "start",
        r#"<g fill="currentColor" stroke="none"><path d="M19.496 4.136l-12 7a1 1 0 0 0 0 1.728l12 7a1 1 0 0 0 1.504 -.864v-14a1 1 0 0 0 -1.504 -.864z" /> <path d="M4 4a1 1 0 0 1 .993 .883l.007 .117v14a1 1 0 0 1 -1.993 .117l-.007 -.117v-14a1 1 0 0 1 1 -1z" /></g>"#,
    ),
    (
        "stop",
        r#"<g fill="currentColor" stroke="none"><path d="M17 4h-10a3 3 0 0 0 -3 3v10a3 3 0 0 0 3 3h10a3 3 0 0 0 3 -3v-10a3 3 0 0 0 -3 -3z" /></g>"#,
    ),
    (
        "trash",
        r#"<path d="M4 7l16 0" /> <path d="M10 11l0 6" /> <path d="M14 11l0 6" /> <path d="M5 7l1 12a2 2 0 0 0 2 2h8a2 2 0 0 0 2 -2l1 -12" /> <path d="M9 7v-3a1 1 0 0 1 1 -1h4a1 1 0 0 1 1 1v3" />"#,
    ),
    (
        "undo",
        r#"<path d="M9 14l-4 -4l4 -4" /> <path d="M5 10h11a4 4 0 1 1 0 8h-1" />"#,
    ),
    (
        "volume",
        r#"<path d="M15 8a5 5 0 0 1 0 8" /> <path d="M17.7 5a9 9 0 0 1 0 14" /> <path d="M6 15h-2a1 1 0 0 1 -1 -1v-4a1 1 0 0 1 1 -1h2l3.5 -4.5a.8 .8 0 0 1 1.5 .5v14a.8 .8 0 0 1 -1.5 .5l-3.5 -4.5" />"#,
    ),
    (
        "warning",
        r#"<path d="M12 9v4" /> <path d="M10.363 3.591l-8.106 13.534a1.914 1.914 0 0 0 1.636 2.871h16.214a1.914 1.914 0 0 0 1.636 -2.87l-8.106 -13.536a1.914 1.914 0 0 0 -3.274 0z" /> <path d="M12 16h.01" />"#,
    ),
    (
        "wave",
        r#"<path d="M21 12h-2c-.894 0 -1.662 -.857 -1.761 -2c-.296 -3.45 -.749 -6 -2.749 -6s-2.5 3.582 -2.5 8s-.5 8 -2.5 8s-2.452 -2.547 -2.749 -6c-.1 -1.147 -.867 -2 -1.763 -2h-2" />"#,
    ),
    (
        "eye-off",
        r#"<path d="M10.585 10.587a2 2 0 0 0 2.829 2.828" /> <path d="M16.681 16.673a8.717 8.717 0 0 1 -4.681 1.327c-3.6 0 -6.6 -2 -9 -6c1.272 -2.12 2.712 -3.678 4.32 -4.674m2.86 -1.146a9.055 9.055 0 0 1 1.82 -.18c3.6 0 6.6 2 9 6c-.666 1.11 -1.379 2.067 -2.138 2.87" /> <path d="M3 3l18 18" />"#,
    ),
    (
        "lock-open",
        r#"<path d="M5 11m0 2a2 2 0 0 1 2 -2h10a2 2 0 0 1 2 2v6a2 2 0 0 1 -2 2h-10a2 2 0 0 1 -2 -2z" /> <path d="M12 16m-1 0a1 1 0 1 0 2 0a1 1 0 1 0 -2 0" /> <path d="M8 11v-5a4 4 0 0 1 8 0" />"#,
    ),
    (
        "pin",
        r#"<path d="M15 4.5l-4 4l-4 1.5l7 7l1.5 -4l4 -4" /> <path d="M9 15l-4.5 4.5" /> <path d="M14.5 4l5.5 5.5" />"#,
    ),
    (
        "chevrons-left",
        r#"<path d="M11 7l-5 5l5 5" /> <path d="M17 7l-5 5l5 5" />"#,
    ),
    (
        "chevrons-right",
        r#"<path d="M7 7l5 5l-5 5" /> <path d="M13 7l5 5l-5 5" />"#,
    ),
    (
        "step-back",
        r#"<path d="M6 5v14" /> <path d="M18 6v12l-9 -6z" />"#,
    ),
    (
        "step-forward",
        r#"<path d="M18 5v14" /> <path d="M6 6v12l9 -6z" />"#,
    ),
    (
        "play-backward",
        r#"<g fill="currentColor" stroke="none"><path d="M18 4v16a1 1 0 0 1 -1.524 .852l-13 -8a1 1 0 0 1 0 -1.704l13 -8a1 1 0 0 1 1.524 .852z" /></g>"#,
    ),
    (
        "clock",
        r#"<path d="M3 12a9 9 0 1 0 18 0a9 9 0 0 0 -18 0" /> <path d="M12 7v5l3 3" />"#,
    ),
    (
        "link-off",
        r#"<path d="M11 6l.463 -.536a5 5 0 0 1 7.071 7.072l-.534 .464" /> <path d="M13 18l-.397 .534a5.068 5.068 0 0 1 -7.127 0a4.972 4.972 0 0 1 0 -7.071l.524 -.463" /> <path d="M3 3l18 18" />"#,
    ),
];

/// A whole 24x24 SVG around `body`, stroked like the Tabler outlines.
fn svg(body: &str) -> Vec<u8> {
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">{body}</svg>"##
    )
    .into_bytes()
}

/// Every named icon of the kit, which [`theme`](crate::theme) installs as
/// the theme's [`IconSet`] over the standard marks.
///
/// The set is built once and shared, so two themes of the kit compare their
/// icons by identity instead of name by name.
pub(crate) fn set() -> IconSet {
    static SET: std::sync::OnceLock<IconSet> = std::sync::OnceLock::new();
    SET.get_or_init(|| {
        let mut set = IconSet::standard();
        set.extend(BODIES.iter().map(|(name, body)| (*name, svg(body))));
        // The icons of a toast's tones.
        let named = |name: &str| {
            let (_, body) = BODIES
                .iter()
                .find(|(candidate, _)| *candidate == name)
                .expect("invariant: the kit draws this glyph");
            svg(body)
        };
        set.extend([
            ("toast-info", named("info")),
            ("toast-success", named("check")),
            ("toast-warning", named("warning")),
            ("toast-danger", named("warning")),
            ("toast-accent", named("info")),
        ]);
        set
    })
    .clone()
}
