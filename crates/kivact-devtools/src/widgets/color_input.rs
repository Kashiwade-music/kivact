//! Role: A color input with a text value and a preview.

use kivact::prelude::*;

/// One color value of a panel.
pub(crate) fn color_input(
    label: &str,
    value: Signal<String>,
    on_commit: impl Fn(String) + 'static,
) -> impl IntoView {
    let _ = (label, value, on_commit);
    node()
}
