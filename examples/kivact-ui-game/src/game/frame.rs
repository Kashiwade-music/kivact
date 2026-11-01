//! Role: Catalog window frame with application-owned panes and chrome.
use super::slot::{Slot, slot};
use kivact::control::{Node, NodeBuilder, column, row, scroll};
use kivact::i18n::message;
use kivact::interaction::IntoScalarRange;
use kivact::style::{Align, ColorToken, pct, style};
use kivact::ui::ScalarRange;
use kivact::view::{IntoText, IntoValue, IntoView};
use kivact::{Binding, Value};
use kivact_ui_game::Color;
/// A window frame with application-owned panes and optional chrome slots.
pub(super) struct NativeWindow {
    root: Node,
    title: kivact::Value<String>,
    explorer: kivact::view::View,
    document: kivact::view::View,
    menu_bar: Option<Slot>,
    toolbar: Option<Slot>,
    status_bar: Option<Slot>,
    explorer_width: Binding<f64>,
    explorer_range: Value<ScalarRange>,
}

/// A window frame named `title` with the `explorer` pane beside the
/// `document`; `explorer_width` is the explorer's width in pixels, which the
/// splitter between them edits (140 to 420 unless
/// [`explorer_range`](NativeWindow::explorer_range) says otherwise). All
/// application content comes from the caller.
#[track_caller]
pub(super) fn native_window(
    title: impl IntoText,
    explorer_width: impl Into<Binding<f64>>,
    explorer: impl IntoView,
    document: impl IntoView,
) -> NativeWindow {
    NativeWindow {
        root: column(()).style(
            style()
                .width(pct(100))
                .min_width(0)
                .background(ColorToken::Surface)
                .border(1, ColorToken::Border),
        ),
        title: title.into_text_value(),
        explorer: explorer.into_view(),
        document: document.into_view(),
        menu_bar: None,
        toolbar: None,
        status_bar: None,
        explorer_width: explorer_width.into(),
        explorer_range: ScalarRange::new(140.0, 420.0)
            .expect("invariant: 140 to 420 is a valid range")
            .into_value(),
    }
}
impl NativeWindow {
    /// Optional application menu, omitted by default; built once by
    /// `content` when the window mounts.
    pub(super) fn menu_bar<V: IntoView>(mut self, content: impl FnOnce() -> V + 'static) -> Self {
        self.menu_bar = Some(slot(content));
        self
    }
    /// Optional application toolbar, omitted by default; built once by
    /// `content` when the window mounts.
    pub(super) fn toolbar<V: IntoView>(mut self, content: impl FnOnce() -> V + 'static) -> Self {
        self.toolbar = Some(slot(content));
        self
    }
    /// Optional application status, omitted by default; built once by
    /// `content` when the window mounts.
    pub(super) fn status_bar<V: IntoView>(mut self, content: impl FnOnce() -> V + 'static) -> Self {
        self.status_bar = Some(slot(content));
        self
    }
    /// The bounds (and step) of the explorer's width, 140 to 420 pixels by
    /// default: a constant [`ScalarRange`], or a closure, `Signal` or `Value`
    /// of one.
    #[track_caller]
    pub(super) fn explorer_range(mut self, range: impl IntoScalarRange) -> Self {
        self.explorer_range = range.into_range();
        self
    }
}
impl NodeBuilder for NativeWindow {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for NativeWindow {
    fn into_view(self) -> kivact::view::View {
        let titlebar = row((
            kivact_ui_game::mono(self.title).style(style().color(kivact::style::rgb(0xe3ecf6))),
            self.menu_bar.map(|menu| menu()),
        ))
        .style(
            style()
                .min_height(34)
                .padding_x(12)
                .align_items(Align::Center)
                .gap(18)
                .background(Color::Navy)
                .color(kivact::style::rgb(0xe3ecf6)),
        );
        let panes = kivact::ui::splitter(
            message("game.window.explorer_width", "Explorer width"),
            self.explorer_width,
            self.explorer_range,
        )
        .start(scroll(self.explorer).style(style().height(pct(100))))
        .end(scroll(self.document).style(style().height(pct(100)).min_width(100)))
        .style(style().height(360).align_items(Align::Stretch));
        self.root
            .child((
                titlebar,
                self.toolbar.map(|toolbar| toolbar()),
                panes,
                self.status_bar.map(|status| status()),
            ))
            .into_view()
    }
}
