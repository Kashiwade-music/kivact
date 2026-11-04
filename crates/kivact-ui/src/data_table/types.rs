//! Role: The state of a data table: its builder fields, slots and live context.
use crate::DataColumn;
use crate::variants::Variants;
use kivact_core::state::{Binding, Callback, Value};
use kivact_core::style::Color;
use kivact_core::view::Node;
use kivact_core::view::View;
use kivact_interaction::{ScalarRange, SortOrder, TableFocus, TableModel};
use std::rc::Rc;

/// The sorted column (an index into the columns) and its direction.
pub(super) type Sort = Binding<Option<(usize, SortOrder)>>;
pub(super) type ColumnFilters = Binding<Vec<(usize, String)>>;
pub(super) type TypedFilters = Binding<std::collections::HashMap<usize, crate::FilterValue>>;
/// The page shown and the rows on a page.
pub(super) type Paging = (Binding<usize>, Value<usize>);
pub(super) type Key<R, K> = Rc<dyn Fn(&R) -> K>;
/// What a row shows beside its cells: its actions or its details.
pub(super) type RowSlot<R> = Rc<dyn Fn(Value<R>) -> View>;
/// The controls of one cell, from its row and its column's index.
pub(super) type CellSlot<R> = Rc<dyn Fn(Value<R>, usize) -> View>;
/// A region the caller fills, built once from the table's state.
pub(super) type Slot<R, K> = Box<dyn FnOnce(DataTableContext<R, K>) -> View>;
pub(super) type RowContextMenu<R> = Rc<dyn Fn(Value<R>, Node) -> View>;
pub(super) type RowSelectable<R> = Rc<dyn Fn(&R) -> bool>;
pub(super) type RowStatusOf<R> = Rc<dyn Fn(&R) -> Option<super::RowStatus>>;
pub(super) type RowColor<R> = Rc<dyn Fn(&R) -> Option<Color>>;
/// Names a row in text.
pub(super) type RowName<R> = Rc<dyn Fn(&R) -> String>;

/// The live state of a [`data_table`] that its [`toolbar`](DataTable::toolbar)
/// and [`footer`](DataTable::footer) slots build their views from. Every
/// handle is reactive; the context is `Copy` and holds no copy of the rows.
pub struct DataTableContext<R: 'static, K: 'static> {
    /// The filter, sort and page model: its counts and its filtered keys.
    pub model: TableModel<R, K, usize>,
    /// The caller's selection, when the table has one.
    pub selection: Option<Binding<Vec<K>>>,
    /// The sorted column (by position) and direction. It is the caller's
    /// binding, or the table's own when the caller gave none.
    pub sort: Binding<Option<(usize, SortOrder)>>,
    /// The caller's filter text, when the table has one.
    pub filter: Option<Binding<String>>,
    /// The caller's queries by column index, when supplied.
    pub column_filters: Option<Binding<Vec<(usize, String)>>>,
    /// Committed typed values keyed by the column's original position.
    pub typed_column_filters: Option<Binding<std::collections::HashMap<usize, crate::FilterValue>>>,
    /// The caller's page number and the rows on a page, when the table
    /// pages.
    pub page: Option<(Binding<usize>, Value<usize>)>,
    /// The writable page size, when the table offers a size selector.
    pub page_size: Option<Binding<usize>>,
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
    pub(super) grouping: Option<Grouping<R>>,
    pub(super) tree: Option<TreeConfig<R, K>>,
    pub(super) tree_column: usize,
    pub(super) tree_indent: DataTableTreeIndent,
    pub(super) tree_expand_all: bool,
    pub(super) key: Key<R, K>,
    pub(super) columns: Rc<Vec<DataColumn<R>>>,
    pub(super) column_order: Option<Value<Vec<usize>>>,
    pub(super) selection: Option<Binding<Vec<K>>>,
    pub(super) row_selectable: Option<RowSelectable<R>>,
    pub(super) sort: Option<Sort>,
    pub(super) multi_sort: Option<Binding<Vec<(usize, SortOrder)>>>,
    pub(super) page: Option<Paging>,
    pub(super) has_more: Option<Value<bool>>,
    pub(super) page_sizes: Option<(Binding<usize>, Value<Vec<usize>>)>,
    pub(super) row_index: Option<usize>,
    pub(super) filter: Option<Binding<String>>,
    pub(super) column_filters: Option<ColumnFilters>,
    pub(super) typed_column_filters: Option<TypedFilters>,
    pub(super) filter_ui: crate::FilterUi,
    pub(super) sticky: Option<StickyEdges>,
    pub(super) widths: Option<Binding<Vec<f64>>>,
    pub(super) resize_range: Option<ScalarRange>,
    pub(super) row_label: Option<RowName<R>>,
    pub(super) actions: Option<(f64, RowSlot<R>)>,
    pub(super) row_context_menu: Option<RowContextMenu<R>>,
    pub(super) reorder: Option<Callback<(K, usize)>>,
    pub(super) toolbar: Option<Slot<R, K>>,
    pub(super) footer: Option<Slot<R, K>>,
    pub(super) empty: Option<Box<dyn Fn() -> View>>,
    pub(super) details: Option<RowSlot<R>>,
    pub(super) expanded_rows: Option<Binding<Vec<K>>>,
    pub(super) cells: Option<CellSlot<R>>,
    pub(super) focus: TableFocus,
    pub(super) row_height: Option<Value<f64>>,
    pub(super) row_status: Option<RowStatusOf<R>>,
    pub(super) row_color: Option<RowColor<R>>,
    pub(super) variants: Variants,
}
