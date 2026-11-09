//! Role: A property row with a name, a value and an editor.

use kivact::prelude::*;

/// One editable property of a panel.
pub(crate) fn property_row(
    name: String,
    value: Signal<String>,
    enabled: Signal<bool>,
    on_commit: impl Fn(String) + 'static,
) -> impl IntoView {
    let _ = (name, value, enabled, on_commit);
    node()
}
