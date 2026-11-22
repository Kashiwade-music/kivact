//! Role: The Pop theme with the catalog's fonts.
use crate::fonts::FONTS;
use kivact::style::{Theme, ThemeMode};

/// The Pop theme in `mode`, with the catalog's fonts added.
pub fn theme(mode: ThemeMode) -> Theme {
    kivact_ui_pop::theme(mode).with_fonts(FONTS.iter().copied())
}
