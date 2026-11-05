//! Role: Panels, media tiles and tree rows of the Creative kit.
use crate::ColorName;

use kivact::state::IntoText;
use kivact::style::use_theme;
use kivact::style::{Align, ColorToken, Cursor, Direction, Justify, Overflow, Style, Theme, style};
use kivact::view::{IntoView, Role, View};
use kivact::view::{Node, NodeBuilder, column, icon, node, row, text};

/// A reusable media thumbnail with consumer supplied preview content and metadata.
pub fn media_tile(
    label: impl IntoText,
    metadata: impl IntoText,
    preview: impl IntoView,
) -> impl kivact::view::Component {
    let label = label.into_text_value();
    column((
        preview,
        text(label)
            .style(style().font_size(11.5).weight(500).single_line())
            .into_view(),
        text(metadata)
            .style(style().font_size(9.5).color(ColorName::Dim))
            .into_view(),
    ))
    .role(Role::ListItem)
    .name(label)
    .focusable(true)
    .style(
        style()
            .padding(4)
            .gap(5)
            .radius(10)
            .min_width(0)
            .hover(style().background(ColorToken::Hover))
            .selected(style().background(ColorName::Selection)),
    )
}

/// An island panel. Its root accepts arbitrary layout and drawing modifiers.
pub struct Panel {
    root: Node,
    header: View,
    body: Node,
    footer: Vec<View>,
}

/// A floating island with a 36px header and source-matching rounded hairline.
///
/// The header is shown at its height and the content takes the rest of the
/// panel. [`Panel::header`] replaces the header with a view of the application.
#[track_caller]
pub fn panel(title: impl IntoText, content: impl IntoView) -> Panel {
    let title = title.into_text_value();
    Panel {
        root: column(()).role(Role::Group).name(title).style(
            style()
                .background(ColorName::Island)
                .color(ColorName::Text)
                .radius(12)
                .border(1, ColorName::Hair)
                .min_width(0)
                .min_height(0)
                .overflow(Overflow::Hidden),
        ),
        header: {
            row((
                text(title).style(style().weight(600).grow(1)),
                icon("chevron-down"),
            ))
            .style(
                style()
                    .height(36)
                    .shrink(0)
                    .padding_x(12)
                    .gap(6)
                    .align_items(Align::Center)
                    .border_bottom(1, ColorName::Hair),
            )
            .into_view()
        },
        body: node()
            .style(style().grow(1).min_width(0).min_height(0))
            .child(content),
        footer: Vec::new(),
    }
}

impl Panel {
    /// Replace the header with a view of the application, such as a toolbar
    /// or a tab bar, so the panel has no second title bar.
    ///
    /// The header keeps its own height (it does not shrink) and the content
    /// takes the rest of the panel.
    pub fn header(mut self, header: impl IntoView) -> Self {
        self.header = node()
            .style(style().shrink(0).min_width(0))
            .child(header)
            .into_view();
        self
    }

    /// Append a footer or another content region to the island.
    pub fn child(mut self, content: impl IntoView) -> Self {
        self.footer.push(content.into_view());
        self
    }
}
impl NodeBuilder for Panel {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Panel {
    fn into_view(self) -> View {
        self.root
            .child(self.header)
            .child(self.body)
            .child(self.footer)
            .into_view()
    }
}

/// A source-style hierarchical row. Selection, expansion and events use NodeBuilder.
#[track_caller]
pub fn tree_row(
    label: impl IntoText,
    depth: impl kivact::state::IntoValue<usize>,
) -> impl kivact::view::Component {
    let label = label.into_text_value();
    let depth = kivact::state::IntoValue::into_value(depth);
    kivact::ui::button(label)
        .variant(kivact::ui::ButtonVariant::Secondary)
        .hide_label(true)
        .child(row((
            icon("chevron-right"),
            icon("box"),
            text(label).style(style().grow(1)),
        )))
        .with_node(move |node| node.role(Role::TreeItem))
        .with_node(move |node| node.name(label))
        .with_node(move |node| node.focusable(true))
        .with_node(move |node| {
            node.style(move || {
                let t = &use_theme().get();
                let p = t.ext::<crate::tokens::Tokens>();
                style()
                    .height(p.row)
                    .gap(6)
                    .padding_left(8.0 + depth.get().min(32) as f64 * 16.0)
                    .padding_right(8)
                    .radius(5)
                    .align_items(Align::Center)
                    .color(p.muted)
                    .hover(style().background(ColorToken::Hover).color(p.text_high))
                    .selected(style().background(p.selection).color(p.accent_high))
            })
        })
}

pub(crate) fn tab_track_style(t: &Theme) -> Style {
    let p = t.ext::<crate::tokens::Tokens>();
    style()
        .direction(Direction::Row)
        .gap(2)
        .padding(3)
        .radius(10)
        .background(p.canvas)
        .border(1, p.hair)
        .min_width(0)
}

pub(crate) fn tab_option_style(t: &Theme, reorderable: bool) -> Style {
    let p = t.ext::<crate::tokens::Tokens>();
    style()
        .direction(Direction::Row)
        .height(p.row + 2.0)
        .padding_x(12)
        .shrink(0)
        .gap(8)
        .radius(7)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .color(p.muted)
        .cursor(if reorderable {
            Cursor::Grab
        } else {
            Cursor::Default
        })
        .selected(style().background(p.strong).color(p.text_high))
        .hover(style().color(p.text_high))
}
