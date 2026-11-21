//! Role: The game kit's font family theme variables.
use kivact::style::{FontFamily, ThemeVar, font};

/// `--display-family`: the family of headings, buttons and numbers; the theme's family by default.
pub static DISPLAY_FAMILY_VAR: ThemeVar<FontFamily> =
    ThemeVar::alias("game.display_family", &font::FAMILY_VAR);

/// `--mono-family`: the family of codes and identifiers; `monospace` unless set.
pub static MONO_FAMILY_VAR: ThemeVar<FontFamily> =
    ThemeVar::new("game.mono_family", FontFamily::SYSTEM);
