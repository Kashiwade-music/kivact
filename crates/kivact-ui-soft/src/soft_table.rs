//! Role: Static table elements and presentation helpers for the standard DataTable.

use crate::{ItemDensity, tokens};
use kivact::control::{Node, NodeBuilder, column, node, row, text};
use kivact::primitive::{NodeExt, part};
use kivact::style::{Align, Direction, Justify, Length, pct, style};
use kivact::view::{IntoText, IntoValue, IntoView, Role, View};
use kivact::{Binding, Value};

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

pub fn table(children: impl IntoView) -> Node {
    column(children)
        .role(Role::Table)
        .base_style(style().width(pct(100)).min_width(0))
}

pub fn table_header(children: impl IntoView) -> Node {
    column(children).role(Role::RowGroup)
}

pub fn table_body(children: impl IntoView) -> Node {
    column(children).role(Role::RowGroup)
}

pub fn table_footer(children: impl IntoView) -> Node {
    column(children)
        .role(Role::RowGroup)
        .base_style(move || style().background(tokens().colors.bg_muted))
}

pub struct TableRow {
    root: Node,
    dividers: bool,
    hover: bool,
    striped: bool,
}

pub fn table_row(children: impl IntoView) -> TableRow {
    TableRow {
        root: row(children).role(Role::Row),
        dividers: true,
        hover: false,
        striped: false,
    }
}

impl NodeBuilder for TableRow {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl TableRow {
    pub fn selected(mut self, selected: impl IntoValue<bool>) -> Self {
        self.root = self.root.selected(selected);
        self
    }

    pub fn dividers(mut self, dividers: bool) -> Self {
        self.dividers = dividers;
        self
    }
    pub fn hover(mut self, hover: bool) -> Self {
        self.hover = hover;
        self
    }
    pub fn striped(mut self, striped: bool) -> Self {
        self.striped = striped;
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
        root.base_style(move || {
            let p = tokens();
            let mut row = style()
                .width(pct(100))
                .min_width(0)
                .selected(style().background(p.colors.accent_muted));
            if dividers {
                row = row.border_bottom(1, p.colors.border);
            }
            if hover {
                row = row.hover(style().background(p.colors.overlay_hover));
            }
            if striped {
                row = row.background(p.colors.bg_muted);
            }
            row
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
    align: TableAlign,
    density: TableDensity,
    vertical_align: Align,
    divider: bool,
}

pub fn table_cell(content: impl IntoView) -> TableCell {
    TableCell {
        root: node().role(Role::Cell),
        content: content.into_view(),
        header: false,
        width: None,
        span: 1,
        align: TableAlign::Start,
        density: TableDensity::Balanced,
        vertical_align: Align::Center,
        divider: false,
    }
}

pub fn table_head(content: impl IntoView) -> TableCell {
    let mut cell = table_cell(content);
    cell.root = cell.root.role(Role::ColumnHeader);
    cell.header = true;
    cell
}

impl NodeBuilder for TableCell {
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
    pub fn align(mut self, align: TableAlign) -> Self {
        self.align = align;
        self
    }
    pub fn density(mut self, density: TableDensity) -> Self {
        self.density = density;
        self
    }
    pub fn vertical_align(mut self, align: Align) -> Self {
        self.vertical_align = align;
        self
    }
    pub fn divider(mut self, divider: bool) -> Self {
        self.divider = divider;
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
        root.base_style(move || {
            let p = tokens();
            let mut cell = style()
                .direction(Direction::Row)
                .justify_content(align.justify())
                .align_items(vertical_align)
                .min_width(0)
                .padding_y(density.padding())
                .padding_x(density.padding() + 4)
                .font_size(14)
                .line_height(20.0 / 14.0)
                .color(p.colors.text_primary);
            if header {
                cell = cell.weight(600);
            }
            if divider {
                cell = cell.border_right(1, p.colors.border);
            }
            match width {
                Some(width) => cell.width(width.get()).shrink(0),
                None => cell.basis(0).grow(f32::from(span)).shrink(1),
            }
        })
        .child(content)
        .into_view()
    }
}

type SelectionLabel = Box<dyn FnOnce(Value<usize>) -> View>;

pub struct TableSelectionToolbar<K: Clone + PartialEq + 'static> {
    root: Node,
    selection: Binding<Vec<K>>,
    clear_label: Value<String>,
    selection_label: Option<SelectionLabel>,
    actions: Option<View>,
}

pub fn table_selection_toolbar<K: Clone + PartialEq + 'static>(
    selection: impl Into<Binding<Vec<K>>>,
) -> TableSelectionToolbar<K> {
    TableSelectionToolbar {
        root: node().role(Role::Toolbar).name("Bulk actions"),
        selection: selection.into(),
        clear_label: "Unselect All".into_text_value(),
        selection_label: None,
        actions: None,
    }
}

impl<K: Clone + PartialEq + 'static> NodeBuilder for TableSelectionToolbar<K> {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl<K: Clone + PartialEq + 'static> TableSelectionToolbar<K> {
    pub fn clear_label(mut self, label: impl IntoText) -> Self {
        self.clear_label = label.into_text_value();
        self
    }
    pub fn actions<V: IntoView>(mut self, actions: impl FnOnce() -> V) -> Self {
        self.actions = Some(actions().into_view());
        self
    }
    pub fn selection_label<V: IntoView>(
        mut self,
        label: impl FnOnce(Value<usize>) -> V + 'static,
    ) -> Self {
        self.selection_label = Some(Box::new(move |count| label(count).into_view()));
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
        .part(
            part(|state| state)
                .role(Role::Button)
                .focusable(true)
                .name(label)
                .disabled(disabled)
                .on_press(action),
        )
        .base_style(move || {
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
        .child(text(label))
}

impl<K: Clone + PartialEq + 'static> IntoView for TableSelectionToolbar<K> {
    fn into_view(self) -> View {
        let Self {
            root,
            selection,
            clear_label,
            selection_label,
            actions,
        } = self;
        let count = kivact::memo(move || selection.with(Vec::len));
        let count_label = match selection_label {
            Some(render) => render(count),
            None => text(move || format!("{} selected", count.get())).into_view(),
        };
        let clear = toolbar_button(clear_label, false, move || selection.set(Vec::new()));
        root.hidden(move || count.get() == 0)
            .base_style(move || {
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
            .child((count_label, clear, node().style(style().grow(1)), actions))
            .into_view()
    }
}

/// Render a pager from the zero-based page binding and the DataTable model's page count.
pub fn table_pagination(page: Binding<usize>, page_count: impl IntoValue<usize>) -> Node {
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
        .role(Role::Navigation)
        .name("Table pagination")
        .base_style(move || {
            style()
                .justify_content(Justify::Center)
                .align_items(Align::Center)
                .gap(8)
                .margin_top(8)
                .font_size(12)
                .color(tokens().colors.text_secondary)
        })
}

pub fn table_row_index(index: impl IntoValue<usize>) -> Node {
    let index = index.into_value();
    text(move || index.get().to_string()).base_style(move || {
        style()
            .font_size(12)
            .text_align(kivact::style::TextAlign::End)
            .color(tokens().colors.text_secondary)
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

pub fn table_row_status(label: impl IntoText, tone: impl IntoValue<RowStatusTone>) -> Node {
    let tone = tone.into_value();
    node().role(Role::Image).name(label).base_style(move || {
        let p = tokens();
        let color = match tone.get() {
            RowStatusTone::Neutral => p.colors.icon_secondary,
            RowStatusTone::Success => p.colors.text_success,
            RowStatusTone::Warning => p.colors.text_warning,
            RowStatusTone::Error => p.colors.text_error,
        };
        style().size(8, 8).radius(4).background(color)
    })
}
