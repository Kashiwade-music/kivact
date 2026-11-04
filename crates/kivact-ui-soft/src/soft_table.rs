//! Role: Static table elements and presentation helpers for the standard DataTable.

use crate::{ItemDensity, tokens};
use kivact::state::Binding;
use kivact::state::IntoBinding;
use kivact::state::IntoText;
use kivact::state::IntoValue;
use kivact::state::PropValue;
use kivact::state::Value;
use kivact::style::Align;
use kivact::style::Direction;
use kivact::style::Justify;
use kivact::style::Length;
use kivact::style::pct;
use kivact::style::style;
use kivact::view::IntoView;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::column;
use kivact::view::node;
use kivact::view::row;
use kivact::view::text;

pub type TableDensity = ItemDensity;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TableAlign {
    #[default]
    Start,
    Center,
    End,
}

impl TableAlign {
    fn justify(self) -> Justify {
        match self {
            Self::Start => Justify::Start,
            Self::Center => Justify::Center,
            Self::End => Justify::End,
        }
    }
}

pub fn table(children: impl IntoView) -> impl kivact::view::Component {
    column(children)
        .with_node(move |node| node.role(Role::Table))
        .with_node(move |node| node.base_style(style().width(pct(100)).min_width(0)))
}

pub fn table_header(children: impl IntoView) -> impl kivact::view::Component {
    column(children).with_node(move |node| node.role(Role::RowGroup))
}

pub fn table_body(children: impl IntoView) -> impl kivact::view::Component {
    column(children).with_node(move |node| node.role(Role::RowGroup))
}

pub fn table_footer(children: impl IntoView) -> impl kivact::view::Component {
    column(children)
        .with_node(move |node| node.role(Role::RowGroup))
        .with_node(move |node| {
            node.base_style(move || style().background(tokens().colors.bg_muted))
        })
}

pub struct TableRow {
    root: Node,
    dividers: Value<bool>,
    hover: Value<bool>,
    striped: Value<bool>,
}

pub fn table_row(children: impl IntoView) -> TableRow {
    TableRow {
        root: row(children).with_node(move |node| node.role(Role::Row)),
        dividers: (true).into_value(),
        hover: (false).into_value(),
        striped: (false).into_value(),
    }
}

impl NodeBuilder for TableRow {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl TableRow {
    pub fn selected(mut self, selected: impl IntoValue<bool>) -> Self {
        self.root = self.root.selected(selected);
        self
    }

    pub fn dividers(mut self, dividers: impl IntoValue<bool>) -> Self {
        self.dividers = dividers.into_value();
        self
    }
    pub fn hover(mut self, hover: impl IntoValue<bool>) -> Self {
        self.hover = hover.into_value();
        self
    }
    pub fn striped(mut self, striped: impl IntoValue<bool>) -> Self {
        self.striped = striped.into_value();
        self
    }
}

impl IntoView for TableRow {
    fn into_view(self) -> View {
        let Self {
            root,
            dividers,
            hover,
            striped,
        } = self;
        root.with_node(move |node| {
            node.base_style(move || {
                let p = tokens();
                let mut row = style()
                    .width(pct(100))
                    .min_width(0)
                    .selected(style().background(p.colors.accent_muted));
                if dividers.get() {
                    row = row.border_bottom(1, p.colors.border);
                }
                if hover.get() {
                    row = row.hover(style().background(p.colors.overlay_hover));
                }
                if striped.get() {
                    row = row.background(p.colors.bg_muted);
                }
                row
            })
        })
        .into_view()
    }
}

pub struct TableCell {
    root: Node,
    header: bool,
    content: View,
    width: Option<Value<Length>>,
    span: u16,
    align: Value<TableAlign>,
    density: Value<TableDensity>,
    vertical_align: Value<Align>,
    divider: Value<bool>,
}

pub fn table_cell(content: impl IntoView) -> TableCell {
    TableCell {
        root: node().with_node(move |node| node.role(Role::Cell)),
        content: content.into_view(),
        header: false,
        width: None,
        span: 1,
        align: (TableAlign::Start).into_value(),
        density: (TableDensity::Balanced).into_value(),
        vertical_align: (Align::Center).into_value(),
        divider: (false).into_value(),
    }
}

pub fn table_head(content: impl IntoView) -> TableCell {
    let mut cell = table_cell(content);
    cell.root = cell
        .root
        .with_node(move |node| node.role(Role::ColumnHeader));
    cell.header = true;
    cell
}

impl NodeBuilder for TableCell {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl TableCell {
    pub fn width(mut self, width: impl IntoValue<Length>) -> Self {
        self.width = Some(width.into_value());
        self
    }
    /// Give an equally spaced cell the combined width of this many columns.
    pub fn col_span(mut self, span: u16) -> Self {
        self.span = span.max(1);
        self
    }
    pub fn align(mut self, align: impl IntoValue<TableAlign>) -> Self {
        self.align = align.into_value();
        self
    }
    pub fn density(mut self, density: impl IntoValue<TableDensity>) -> Self {
        self.density = density.into_value();
        self
    }
    pub fn vertical_align(mut self, align: impl IntoValue<Align>) -> Self {
        self.vertical_align = align.into_value();
        self
    }
    pub fn divider(mut self, divider: impl IntoValue<bool>) -> Self {
        self.divider = divider.into_value();
        self
    }
}

impl IntoView for TableCell {
    fn into_view(self) -> View {
        let Self {
            root,
            content,
            header,
            width,
            span,
            align,
            density,
            vertical_align,
            divider,
        } = self;
        root.with_node(move |node| {
            node.base_style(move || {
                let p = tokens();
                let mut cell = style()
                    .direction(Direction::Row)
                    .justify_content(align.get().justify())
                    .align_items(vertical_align.get())
                    .min_width(0)
                    .padding_y(density.get().padding())
                    .padding_x(density.get().padding() + 4)
                    .font_size(14)
                    .line_height(20.0 / 14.0)
                    .color(p.colors.text_primary);
                if header {
                    cell = cell.weight(600);
                }
                if divider.get() {
                    cell = cell.border_right(1, p.colors.border);
                }
                match width {
                    Some(width) => cell.width(width.get()).shrink(0),
                    None => cell.basis(0).grow(f64::from(span)).shrink(1),
                }
            })
        })
        .child(content)
        .into_view()
    }
}

type SelectionLabel = kivact::view::ViewFn<Value<usize>>;

pub struct TableSelectionToolbar<K: PropValue> {
    root: Node,
    selection: Binding<Vec<K>>,
    clear_label: Value<String>,
    selection_label: Option<SelectionLabel>,
    actions: Option<View>,
}

pub fn table_selection_toolbar<K: PropValue>(
    selection: impl IntoBinding<Vec<K>>,
) -> TableSelectionToolbar<K> {
    TableSelectionToolbar {
        root: node()
            .with_node(move |node| node.role(Role::Toolbar))
            .name("Bulk actions"),
        selection: selection.into_binding(),
        clear_label: "Unselect All".into_text_value(),
        selection_label: None,
        actions: None,
    }
}

impl<K: PropValue> NodeBuilder for TableSelectionToolbar<K> {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl<K: PropValue> TableSelectionToolbar<K> {
    pub fn clear_label(mut self, label: impl IntoText) -> Self {
        self.clear_label = label.into_text_value();
        self
    }
    pub fn actions(mut self, actions: impl IntoView) -> Self {
        self.actions = Some(actions.into_view());
        self
    }
    pub fn selection_label<V: IntoView>(
        mut self,
        label: impl Fn(Value<usize>) -> V + 'static,
    ) -> Self {
        self.selection_label = Some(label.into());
        self
    }
}

fn toolbar_button(
    label: impl IntoText,
    disabled: impl IntoValue<bool>,
    action: impl Fn() + 'static,
) -> Node {
    let label = label.into_text_value();
    node()
        .with_node(move |node| node.role(Role::Button))
        .focusable(true)
        .name(label)
        .disabled(disabled)
        .on_press(action)
        .with_node(move |node| {
            node.base_style(move || {
                let p = tokens();
                style()
                    .direction(Direction::Row)
                    .justify_content(Justify::Center)
                    .align_items(Align::Center)
                    .min_height(28)
                    .padding_x(8)
                    .radius(p.radius.element)
                    .color(p.colors.text_primary)
                    .hover(style().background(p.colors.overlay_hover))
                    .focus_visible(style().ring(p.sizes.focus_ring_width, p.colors.focus_ring))
            })
        })
        .child(text(label))
}

impl<K: PropValue> IntoView for TableSelectionToolbar<K> {
    fn into_view(self) -> View {
        let Self {
            root,
            selection,
            clear_label,
            selection_label,
            actions,
        } = self;
        let count = kivact::state::memo(move || selection.with(Vec::len));
        let count_label = match selection_label {
            Some(render) => render.call(count),
            None => text(move || format!("{} selected", count.get())).into_view(),
        };
        let clear = toolbar_button(clear_label, false, move || selection.set(Vec::new()));
        root.hidden(move || count.get() == 0)
            .with_node(move |node| {
                node.base_style(move || {
                    let p = tokens();
                    style()
                        .direction(Direction::Row)
                        .align_items(Align::Center)
                        .padding_x(8)
                        .padding_y(4)
                        .gap(8)
                        .radius(p.radius.element)
                        .background(p.colors.bg_muted)
                        .font_size(14)
                        .color(p.colors.text_primary)
                })
            })
            .child((count_label, clear, node().style(style().grow(1)), actions))
            .into_view()
    }
}

/// Render a pager from the zero-based page binding and the DataTable model's page count.
pub fn table_pagination(
    page: Binding<usize>,
    page_count: impl IntoValue<usize>,
) -> impl kivact::view::Component {
    let page_count = page_count.into_value();
    let last = move || page_count.get().saturating_sub(1);
    let previous = toolbar_button(
        "Previous",
        move || page.get() == 0,
        move || {
            page.set(page.get_untracked().saturating_sub(1));
        },
    );
    let next = toolbar_button(
        "Next",
        move || page.get() >= last(),
        move || {
            let next = page.get_untracked().saturating_add(1).min(last());
            page.set(next);
        },
    );
    let page_label = text(move || {
        let current = page.get().min(last()) + 1;
        format!("Page {current} of {}", page_count.get().max(1))
    });
    row((previous, page_label, next))
        .with_node(move |node| node.role(Role::Navigation))
        .name("Table pagination")
        .with_node(move |node| {
            node.base_style(move || {
                style()
                    .justify_content(Justify::Center)
                    .align_items(Align::Center)
                    .gap(8)
                    .margin_top(8)
                    .font_size(12)
                    .color(tokens().colors.text_secondary)
            })
        })
}

pub fn table_row_index(index: impl IntoValue<usize>) -> impl kivact::view::Component {
    let index = index.into_value();
    text(move || index.get().to_string()).with_node(move |node| {
        node.base_style(move || {
            style()
                .font_size(12)
                .text_align(kivact::style::TextAlign::End)
                .color(tokens().colors.text_secondary)
        })
    })
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RowStatusTone {
    #[default]
    Neutral,
    Success,
    Warning,
    Error,
}

pub fn table_row_status(
    label: impl IntoText,
    tone: impl IntoValue<RowStatusTone>,
) -> impl kivact::view::Component {
    let tone = tone.into_value();
    node()
        .with_node(move |node| node.role(Role::Image))
        .name(label)
        .with_node(move |node| {
            node.base_style(move || {
                let p = tokens();
                let color = match tone.get() {
                    RowStatusTone::Neutral => p.colors.icon_secondary,
                    RowStatusTone::Success => p.colors.text_success,
                    RowStatusTone::Warning => p.colors.text_warning,
                    RowStatusTone::Error => p.colors.text_error,
                };
                style().size(8, 8).radius(4).background(color)
            })
        })
}

kivact::state::constant_value!(RowStatusTone, TableAlign);
