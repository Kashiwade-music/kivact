//! Role: The native window frame of the layout demo.

use kivact::component;
use kivact::i18n::message;
use kivact::interaction::ScalarRange;
use kivact::state::{Signal, Value};
use kivact::style::{Align, color, pct, rgb, style};
use kivact::view::{Element, View, column, row, scroll};
use kivact_ui_game as g;

/// A window frame named `title` with the `explorer` pane beside the
/// `document`; `explorer_width` is the explorer's width in pixels, which the
/// splitter between them edits (140 to 420 unless `explorer_range` says
/// otherwise). The menu bar, toolbar and status bar are optional chrome.
#[component]
pub(crate) fn native_window(
    title: Value<String>,
    explorer_width: Signal<f64>,
    #[prop(default = ScalarRange::new(140.0, 420.0).expect("invariant: 140 to 420 is a valid range"))]
    explorer_range: Value<ScalarRange>,
    explorer: View,
    document: View,
    menu_bar: Option<View>,
    toolbar: Option<View>,
    status_bar: Option<View>,
) -> impl Element {
    let titlebar = row((g::mono(title).style(style().color(rgb(0xe3ecf6))), menu_bar)).style(
        style()
            .min_height(34)
            .padding_x(12)
            .align_items(Align::Center)
            .gap(18)
            .background(g::color::NAVY)
            .color(rgb(0xe3ecf6)),
    );
    let panes = kivact::ui::splitter(
        message("game.window.explorer_width", "Explorer width"),
        explorer_width,
        explorer_range,
    )
    .start(scroll(explorer).style(style().height(pct(100))))
    .end(scroll(document).style(style().height(pct(100)).min_width(100)))
    .style(style().height(360).align_items(Align::Stretch));
    column(())
        .style(
            style()
                .width(pct(100))
                .min_width(0)
                .background(color::SURFACE)
                .border(1, color::BORDER),
        )
        .child((titlebar, toolbar, panes, status_bar))
}
