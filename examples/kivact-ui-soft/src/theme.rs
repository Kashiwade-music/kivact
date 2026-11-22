//! Role: The Soft theme with the catalog's fonts.
use crate::fonts::FONTS;
use kivact::style::{Theme, ThemeMode};

/// The Soft theme in `mode`, with the catalog's Figtree fonts.
pub fn theme(mode: ThemeMode) -> Theme {
    kivact_ui_soft::theme(mode).with_fonts(FONTS.iter().copied())
}
