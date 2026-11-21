//! Role: The Creative kit's font family variable (the numeric and code font).

use kivact::style::{FontFamily, Theme, ThemeVar};

/// The family of numeric and code text. The fallback is the system font;
/// the kit's theme sets `monospace`.
pub static MONO_FAMILY_VAR: ThemeVar<FontFamily> =
    ThemeVar::new("creative.mono_family", FontFamily::SYSTEM);

/// The family list the mono variable names, or `monospace` when it names none.
pub(crate) fn mono_family(theme: &Theme) -> String {
    theme
        .get(&MONO_FAMILY_VAR)
        .as_str()
        .unwrap_or("monospace")
        .to_owned()
}
