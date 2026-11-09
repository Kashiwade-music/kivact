//! Role: A property row with a name, a value and an editor.

use kivact::prelude::*;

use crate::widgets::color_input::color_input;

/// The width of the name column of a property row.
const LABEL_WIDTH: f64 = 110.0;

/// One editable property of a panel.
pub(crate) fn property_row(
    name: String,
    value: Signal<String>,
    enabled: Signal<bool>,
    on_commit: impl Fn(String) + 'static,
) -> impl IntoView {
    let commit = callback(on_commit);
    row((
        checkbox(name.clone(), enabled).hide_label(true),
        label(name.clone(), enabled),
        text_input(name, value)
            .style(style().grow(1).min_width(0).font_size(12.0))
            .on_submit(move || commit.call(value.get_untracked())),
    ))
    .style(row_style())
}

/// A property row whose editor is a color input with its preview and sliders.
pub(crate) fn color_property_row(
    name: String,
    value: Signal<String>,
    enabled: Signal<bool>,
    on_commit: impl Fn(String) + 'static,
) -> impl IntoView {
    row((
        checkbox(name.clone(), enabled).hide_label(true),
        label(name.clone(), enabled),
        node()
            .style(style().grow(1).min_width(0))
            .child(color_input(&name, value, on_commit)),
    ))
    .style(row_style())
}

/// The visible name of a row, dimmed while its override is off.
fn label(name: String, enabled: Signal<bool>) -> impl IntoView {
    text(name).style(move || {
        style()
            .width(LABEL_WIDTH)
            .shrink(0)
            .single_line()
            .font_size(12.0)
            .opacity(if enabled.get() { 1.0 } else { 0.6 })
    })
}

/// The layout shared by every row.
fn row_style() -> Style {
    style().gap(6).align_items(Align::Center).min_height(24)
}
