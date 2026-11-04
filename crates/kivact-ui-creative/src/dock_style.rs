//! Role: Passive dock splitter dimensions and theme appearance.
use kivact::style::{Style, Theme, style};
/// The thickness of a splitter between two panes, in logical pixels. The
/// theme's splitter recipe, which `kivact::ui::split` draws too, uses it.
pub(crate) const SPLITTER_THICKNESS: f64 = 6.0;

/// How the dock draws a splitter: transparent, accent while hovered or
/// focused, above the panes it separates. The splitter recipe shares it.
pub(crate) fn splitter_look(t: &Theme) -> Style {
    style()
        .z_index(1)
        .hover(style().background(t.ext::<crate::tokens::Tokens>().accent))
        .focus(style().background(t.ext::<crate::tokens::Tokens>().accent))
}
