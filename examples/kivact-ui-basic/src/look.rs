//! Role: The kit tokens of the theme in effect, for the catalog's own styles.

use kivact::style::use_theme;
use kivact_ui_basic as basic;

/// The kit tokens of the theme in effect; call inside style closures.
pub(crate) fn tokens() -> kivact::style::ThemeRef<basic::Tokens> {
    use_theme().get().ext::<basic::Tokens>()
}
