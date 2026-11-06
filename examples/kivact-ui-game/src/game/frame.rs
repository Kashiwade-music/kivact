//! Role: Catalog window frame with application-owned panes and chrome.
use kivact::i18n::message;
use kivact::interaction::ScalarRange;
use kivact::state::{Binding, Value};
use kivact::state::{IntoText, IntoValue};
use kivact::style::{Align, ColorToken, pct, style};
use kivact::view::IntoView;
use kivact::view::{Node, NodeBuilder, column, row, scroll};
use kivact_ui_game::ColorName;
/// A window frame with application-owned panes and optional chrome slots.
pub(super) struct NativeWindow {
    root: Node,
    title: kivact::state::Value<String>,
    explorer: kivact::view::View,
    document: kivact::view::View,
    menu_bar: Option<kivact::view::View>,
    toolbar: Option<kivact::view::View>,
    status_bar: Option<kivact::view::View>,
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
    explorer_width: impl kivact::state::IntoBinding<f64>,
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
        explorer_width: explorer_width.into_binding(),
        explorer_range: ScalarRange::new(140.0, 420.0)
            .expect("invariant: 140 to 420 is a valid range")
            .into_value(),
    }
}
impl NativeWindow {
    /// Optional application menu, omitted by default; mounted once with the window.
    pub(super) fn menu_bar(mut self, content: impl IntoView) -> Self {
        self.menu_bar = Some(content.into_view());
        self
    }
    /// Optional application toolbar, omitted by default; mounted once with the window.
    pub(super) fn toolbar(mut self, content: impl IntoView) -> Self {
        self.toolbar = Some(content.into_view());
        self
    }
    /// Optional application status, omitted by default; mounted once with the window.
    pub(super) fn status_bar(mut self, content: impl IntoView) -> Self {
        self.status_bar = Some(content.into_view());
        self
    }
    /// The bounds (and step) of the explorer's width, 140 to 420 pixels by
    /// default: a constant [`ScalarRange`], or a closure, `Signal` or `Value`
    /// of one.
    #[track_caller]
    pub(super) fn explorer_range(mut self, range: impl IntoValue<ScalarRange>) -> Self {
        self.explorer_range = range.into_value();
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
            self.menu_bar,
        ))
        .style(
            style()
                .min_height(34)
                .padding_x(12)
                .align_items(Align::Center)
                .gap(18)
                .background(ColorName::Navy)
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
            .child((titlebar, self.toolbar, panes, self.status_bar))
            .into_view()
    }
}
