//! Role: The state of a data table: its builder fields, slots and live context.
use crate::column_values::DataColumn;
use crate::variants::Variants;

use kivact_core::state::{Binding, Callback, Value};
use kivact_core::style::Color;
use kivact_core::view::Node;
use kivact_core::view::View;
use kivact_interaction::{DataTableModelApi, ScalarRange, SortOrder, TableFocus};
use std::rc::Rc;

/// The sorted column (an index into the columns) and its direction.
pub(super) type Sort = Binding<Option<(usize, SortOrder)>>;
pub(super) type ColumnFilters = Binding<Vec<(usize, String)>>;
pub(super) type TypedFilters =
    Binding<std::collections::HashMap<usize, kivact_interaction::FilterValue>>;
/// The page shown and the rows on a page.
pub(super) type Paging = (Binding<usize>, Value<usize>);
pub(super) type Key<R, K> = Rc<dyn Fn(&R) -> K>;
/// What a row shows beside its cells: its actions or its details.
pub(super) type RowSlot<R> = kivact_core::view::ViewFn<Value<R>>;
/// The controls of one cell, from its row and its column's index.
pub(super) type CellSlot<R> = kivact_core::view::ViewFn<(Value<R>, usize)>;
/// A region the caller fills, built once from the table's state.
pub(super) type Slot<R, K> = Box<dyn FnOnce(DataTableContext<R, K>) -> View>;
pub(super) type RowContextMenu<R> = kivact_core::view::ViewFn<(Value<R>, Node)>;
pub(super) type RowSelectable<R> = Rc<dyn Fn(&R) -> bool>;
pub(super) type RowStatusOf<R> = Rc<dyn Fn(&R) -> Option<crate::table_values::RowStatus>>;
pub(super) type RowColor<R> = Rc<dyn Fn(&R) -> Option<Color>>;
/// Names a row in text.
pub(super) type RowName<R> = Rc<dyn Fn(&R) -> String>;

/// The live state of a [`data_table`] that its [`toolbar`](DataTable::toolbar)
/// and [`footer`](DataTable::footer) slots build their views from. Every
/// handle is reactive; the context is `Copy` and holds no copy of the rows.
pub struct DataTableContext<R: 'static, K: 'static> {
    /// The filter, sort and page model: its counts and its filtered keys.
    pub model: DataTableModelApi<R, K, usize>,
    // xtask-allow(state-fields): caller binding - The caller's selection, when the table has one
    /// The caller's selection, when the table has one.
    pub selection: Option<Binding<Vec<K>>>,
    /// The sorted column (by position) and direction. It is the caller's
    /// binding, or the table's own when the caller gave none.
    pub sort: Binding<Option<(usize, SortOrder)>>,
    // xtask-allow(state-fields): caller binding - The caller's filter text, when the table has one
    /// The caller's filter text, when the table has one.
    pub filter: Option<Binding<String>>,
    // xtask-allow(state-fields): caller binding - The caller's queries by column index, when supplied
    /// The caller's queries by column index, when supplied.
    pub column_filters: Option<Binding<Vec<(usize, String)>>>,
    // xtask-allow(state-fields): caller binding - Committed typed values keyed by the column's original position
    /// Committed typed values keyed by the column's original position.
    pub typed_column_filters:
        Option<Binding<std::collections::HashMap<usize, kivact_interaction::FilterValue>>>,
    // xtask-allow(state-fields): caller binding - The caller's page number and the rows on a page, when the table pages
    /// The caller's page number and the rows on a page, when the table
    /// pages.
    pub page: Option<(Binding<usize>, Value<usize>)>,
    // xtask-allow(state-fields): caller binding - The writable page size, when the table offers a size selector
    /// The writable page size, when the table offers a size selector.
    pub page_size: Option<Binding<usize>>,
    // xtask-allow(state-fields): caller binding - Whether another remotely supplied page is available
    /// Whether another remotely supplied page is available.
    pub has_more: Option<Value<bool>>,
}

impl<R: 'static, K: 'static> Copy for DataTableContext<R, K> {}

impl<R: 'static, K: 'static> Clone for DataTableContext<R, K> {
    fn clone(&self) -> Self {
        *self
    }
}

/// Cell spacing available to data table theme recipes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DataTableDensity {
    Compact,
    #[default]
    Balanced,
    Spacious,
}

impl crate::variants::Variant for DataTableDensity {}

pub(super) type StickyEdges = (Vec<usize>, Vec<usize>);

pub(super) struct Grouping<R> {
    pub(super) key: Key<R, String>,
    pub(super) collapsed: Binding<Vec<String>>,
}

pub(super) struct TreeConfig<R, K: 'static> {
    pub(super) parent: Key<R, Option<K>>,
    pub(super) expanded: Binding<Vec<K>>,
}

/// Horizontal space for each additional table tree level.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DataTableTreeIndent {
    Small,
    #[default]
    Medium,
    Large,
}

/// A table of the caller's rows, created by [`data_table`].
pub struct DataTable<R: 'static, K: 'static> {
    pub(super) root: Node,
    pub(super) rows: Value<Vec<R>>,
    // xtask-allow(state-fields): builder option - grouping
    pub(super) grouping: Option<Grouping<R>>,
    // xtask-allow(state-fields): builder option - tree
    pub(super) tree: Option<TreeConfig<R, K>>,
    pub(super) tree_column: usize,
    pub(super) tree_indent: DataTableTreeIndent,
    // xtask-allow(state-fields): builder option - tree expand all
    pub(super) tree_expand_all: bool,
    pub(super) key: Key<R, K>,
    pub(super) columns: Rc<Vec<DataColumn<R>>>,
    // xtask-allow(state-fields): builder option - column order
    pub(super) column_order: Option<Value<Vec<usize>>>,
    // xtask-allow(state-fields): builder option - selection
    pub(super) selection: Option<Binding<Vec<K>>>,
    // xtask-allow(state-fields): builder option - row selectable
    pub(super) row_selectable: Option<RowSelectable<R>>,
    // xtask-allow(state-fields): builder option - sort
    pub(super) sort: Option<Sort>,
    // xtask-allow(state-fields): builder option - multi sort
    pub(super) multi_sort: Option<Binding<Vec<(usize, SortOrder)>>>,
    // xtask-allow(state-fields): builder option - page
    pub(super) page: Option<Paging>,
    // xtask-allow(state-fields): builder option - has more
    pub(super) has_more: Option<Value<bool>>,
    // xtask-allow(state-fields): builder option - page sizes
    pub(super) page_sizes: Option<(Binding<usize>, Value<Vec<usize>>)>,
    // xtask-allow(state-fields): builder option - row index
    pub(super) row_index: Option<usize>,
    // xtask-allow(state-fields): builder option - filter
    pub(super) filter: Option<Binding<String>>,
    // xtask-allow(state-fields): builder option - column filters
    pub(super) column_filters: Option<ColumnFilters>,
    // xtask-allow(state-fields): builder option - typed column filters
    pub(super) typed_column_filters: Option<TypedFilters>,
    pub(super) filter_ui: crate::FilterUi,
    // xtask-allow(state-fields): builder option - sticky
    pub(super) sticky: Option<StickyEdges>,
    // xtask-allow(state-fields): builder option - widths
    pub(super) widths: Option<Binding<Vec<f64>>>,
    // xtask-allow(state-fields): builder option - resize range
    pub(super) resize_range: Option<ScalarRange>,
    // xtask-allow(state-fields): builder option - row label
    pub(super) row_label: Option<RowName<R>>,
    // xtask-allow(state-fields): builder option - actions
    pub(super) actions: Option<(f64, RowSlot<R>)>,
    // xtask-allow(state-fields): builder option - row context menu
    pub(super) row_context_menu: Option<RowContextMenu<R>>,
    pub(super) reorder: Option<Callback<kivact_interaction::Reorder<K>>>,
    // xtask-allow(state-fields): builder option - toolbar
    pub(super) toolbar: Option<Slot<R, K>>,
    // xtask-allow(state-fields): builder option - footer
    pub(super) footer: Option<Slot<R, K>>,
    pub(super) empty: Option<Box<dyn Fn() -> View>>,
    // xtask-allow(state-fields): builder option - details
    pub(super) details: Option<RowSlot<R>>,
    // xtask-allow(state-fields): builder option - expanded rows
    pub(super) expanded_rows: Option<Binding<Vec<K>>>,
    // xtask-allow(state-fields): builder option - cells
    pub(super) cells: Option<CellSlot<R>>,
    pub(super) focus: TableFocus,
    // xtask-allow(state-fields): builder option - row height
    pub(super) row_height: Option<Value<f64>>,
    // xtask-allow(state-fields): builder option - row status
    pub(super) row_status: Option<RowStatusOf<R>>,
    // xtask-allow(state-fields): builder option - row color
    pub(super) row_color: Option<RowColor<R>>,
    pub(super) variants: Variants,
}
