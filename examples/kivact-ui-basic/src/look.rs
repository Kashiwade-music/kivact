//! Role: The kit tokens of the theme in effect, for the catalog's own styles.

use kivact::style::current_theme;
use kivact_ui_basic as basic;

/// The kit tokens of the theme in effect; call inside style closures.
pub(crate) fn tokens() -> std::sync::Arc<basic::Tokens> {
    current_theme().get::<basic::Tokens>()
}
