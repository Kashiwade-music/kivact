//! Role: A property row with a name, a value and an editor.

use kivact::prelude::*;

use crate::widgets::color_input::color_input;

/// The width of the name column of a property row.
const LABEL_WIDTH: f64 = 110.0;

/// Whether a row takes edits, and why it does not.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum RowAccess {
    /// The checkbox turns the value on and off, and the input edits it.
    Editable,
    /// The property cannot be overridden at all: no checkbox, and the input is disabled.
    ReadOnly,
    /// The row cannot take this value under the current scope: the checkbox and the input are
    /// disabled, and the row shows this hint.
    Unavailable(&'static str),
}

/// One editable property of a panel.
pub(crate) fn property_row(
    name: String,
    value: Signal<String>,
    enabled: Signal<bool>,
    access: RowAccess,
    on_commit: impl Fn(String) + 'static,
) -> View {
    let commit = callback(on_commit);
    let editor = text_input(name.clone(), value)
        .style(style().grow(1).min_width(0).font_size(12.0))
        .disabled(access != RowAccess::Editable)
        .on_submit(move || commit.call(value.get_untracked()));
    frame(name, enabled, access, editor)
}

/// A property row whose editor is a color input with its preview and sliders.
pub(crate) fn color_property_row(
    name: String,
    value: Signal<String>,
    enabled: Signal<bool>,
    access: RowAccess,
    on_commit: impl Fn(String) + 'static,
) -> View {
    let editor = node()
        .style(style().grow(1).min_width(0))
        .child(color_input(
            &name,
            value,
            access != RowAccess::Editable,
            on_commit,
        ));
    frame(name, enabled, access, editor)
}

/// The layout shared by both editors: the checkbox, the name, the editor, and the hint of an
/// unavailable row.
fn frame(name: String, enabled: Signal<bool>, access: RowAccess, editor: impl IntoView) -> View {
    let row_view = row((
        checkbox_slot(name.clone(), enabled, access),
        label(name, enabled, access == RowAccess::Editable),
        editor,
    ))
    .style(row_style());
    match access {
        RowAccess::Unavailable(hint) => kivact::ui::tooltip(hint).attach(row_view),
        RowAccess::Editable | RowAccess::ReadOnly => row_view.into_view(),
    }
}

/// The checkbox of a row; a read-only row keeps an empty slot of its size instead.
fn checkbox_slot(name: String, enabled: Signal<bool>, access: RowAccess) -> View {
    match access {
        RowAccess::ReadOnly => node().style(style().size(16, 16).shrink(0)).into_view(),
        RowAccess::Editable | RowAccess::Unavailable(_) => checkbox(name, enabled)
            .hide_label(true)
            .disabled(access != RowAccess::Editable)
            .into_view(),
    }
}

/// The visible name of a row, dimmed while its override is off or the row cannot be edited.
fn label(name: String, enabled: Signal<bool>, editable: bool) -> impl IntoView {
    text(name).style(move || {
        style()
            .width(LABEL_WIDTH)
            .shrink(0)
            .single_line()
            .font_size(12.0)
            .opacity(if editable && enabled.get() { 1.0 } else { 0.6 })
    })
}

/// The layout shared by every row.
fn row_style() -> Style {
    style().gap(6).align_items(Align::Center).min_height(24)
}
