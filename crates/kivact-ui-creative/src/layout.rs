//! Role: Panels, media tiles and tree rows of the Creative kit.
use crate::color::{ACCENT_HIGH, DIM, HAIR, ISLAND, MUTED, SELECTION, TEXT, TEXT_HIGH};
use crate::size::ROW;
use kivact::component;
use kivact::state::Value;
use kivact::style::{Align, Overflow, color as style_color, style};
use kivact::view::IntoView;
use kivact::view::{Element, Role, View, column, icon, node, row, text};

/// A reusable media thumbnail with consumer supplied preview content and metadata.
#[component]
pub fn media_tile(label: Value<String>, metadata: Value<String>, preview: View) -> impl Element {
    column((
        preview,
        text(label)
            .style(style().font_size(11.5).font_weight(500).single_line())
            .into_view(),
        text(metadata)
            .style(style().font_size(9.5).color(DIM))
            .into_view(),
    ))
    .role(Role::ListItem)
    .name(label)
    .focusable(true)
    .style(
        style()
            .padding(4)
            .gap(5)
            .border_radius(10)
            .min_width(0)
            .hover(style().background(style_color::HOVER))
            .selected(style().background(SELECTION)),
    )
}

/// An island panel with a 36px header and source-matching rounded hairline.
///
/// The header is shown at its height and the content takes the rest of the
/// panel. `header` replaces the title row with a view of the application,
/// such as a toolbar or a tab bar, so the panel has no second title bar; the
/// header keeps its own height and the content takes the rest. `children`
/// appends footers or other content regions to the island.
#[component]
pub fn panel(
    title: Value<String>,
    content: View,
    header: Option<View>,
    children: Option<Vec<View>>,
) -> impl Element {
    let header = match header {
        Some(view) => node()
            .style(style().flex_shrink(0).min_width(0))
            .child(view)
            .into_view(),
        None => title_row(title),
    };
    let body = node()
        .style(style().flex_grow(1).min_width(0).min_height(0))
        .child(content);
    column(())
        .role(Role::Group)
        .name(title)
        .style(
            style()
                .background(ISLAND)
                .color(TEXT)
                .border_radius(12)
                .border(1, HAIR)
                .min_width(0)
                .min_height(0)
                .overflow(Overflow::Hidden),
        )
        .child(header)
        .child(body)
        .child(children.unwrap_or_default())
}

/// The panel's title row: the title and a chevron, at the header height.
fn title_row(title: Value<String>) -> View {
    row((
        text(title).style(style().font_weight(600).flex_grow(1)),
        icon("chevron-down"),
    ))
    .style(
        style()
            .height(36)
            .flex_shrink(0)
            .padding_x(12)
            .gap(6)
            .align_items(Align::Center)
            .border_bottom(1, HAIR),
    )
    .into_view()
}

/// A source-style hierarchical row. Selection, expansion and events use Element.
#[component]
pub fn tree_row(label: Value<String>, depth: Value<usize>) -> impl Element {
    kivact::ui::button(label)
        .variant(kivact::ui::ButtonVariant::Secondary)
        .hide_label(true)
        .children(vec![row((
            icon("chevron-right"),
            icon("box"),
            text(label).style(style().flex_grow(1)),
        ))])
        .with_node(move |node| node.role(Role::TreeItem))
        .with_node(move |node| node.name(label))
        .with_node(move |node| node.focusable(true))
        .with_node(move |node| {
            node.style(move || {
                style()
                    .height(ROW)
                    .gap(6)
                    .padding_left(8.0 + depth.get().min(32) as f64 * 16.0)
                    .padding_right(8)
                    .border_radius(5)
                    .align_items(Align::Center)
                    .color(MUTED)
                    .hover(style().background(style_color::HOVER).color(TEXT_HIGH))
                    .selected(style().background(SELECTION).color(ACCENT_HIGH))
            })
        })
}
