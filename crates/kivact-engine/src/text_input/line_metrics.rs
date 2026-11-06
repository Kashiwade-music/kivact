//! Role: Line height of a text style and a line's vertical offset in its row.

/// Line height of `style` as Arvane's shaper lays lines out: the height of
/// the caret of an empty text (the font size if `style` cannot be shaped).
pub(crate) fn line_height(
    engine: &mut arvane::Engine,
    style: &kivact_types::ComputedTextStyle,
) -> f64 {
    engine
        .shape("", style, None)
        .ok()
        .and_then(|s| s.carets.first().map(|c| c.rect.height))
        .unwrap_or(style.font_size)
}

/// How far below the top of a content box `height` high a single-line
/// input's line of `line` height starts: the line is centered, and a box
/// lower than the line (negative `height - line`) overflows it evenly on
/// both sides. Callers can share this value for content-box placement and
/// measured baseline positioning.
pub(crate) fn line_offset(height: f64, line: f64) -> f64 {
    (height - line) / 2.0
}
