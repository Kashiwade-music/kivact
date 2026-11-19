//! Role: The state of a data table: its builder fields, slots and live context.

use crate::data_table::column::DataColumn;
use crate::variants::Variants;
use kivact_core::state::{Binding, Callback, Value};
use kivact_core::style::{Color, Icon};
use kivact_core::view::{Node, View};
use kivact_interaction::{DataTableModelApi, ScalarRange, SortOrder, TableFocus, Tone};
use std::rc::Rc;
/// The sorted column (an index into the columns) and its direction.
pub(super) type Sort = Binding<Option<(usize, SortOrder)>>;
pub(super) type ColumnFilters = Binding<Vec<(usize, String)>>;
pub(super) type TypedFilters =
    Binding<std::collections::HashMap<usize, kivact_interaction::FilterValue>>;
/// The page shown and the rows on a page.
pub(super) type PagePlan = (Binding<usize>, Value<usize>);
/// The caller's sort keys, in priority order.
pub(super) type MultiSort = Binding<Vec<(usize, SortOrder)>>;

/// Whose sort the headers write. `DataTable::sort` and
/// `DataTable::multi_sort` each replace it, so the later call wins.
#[derive(Clone, Copy)]
pub(super) enum SortSetting {
    /// The table's own sort by one column (no sort with `on_reorder`).
    Own,
    /// The caller's sorted column, from `DataTable::sort`.
    Single(Sort),
    /// The caller's ordered keys, from `DataTable::multi_sort`.
    Multi(MultiSort),
}

/// How the rows are paged. `DataTable::page` and `DataTable::remote_page`
/// each replace it, so the later call wins; the page sizes the caller
/// offers stay their own setting, kept by either call.
#[derive(Clone, Copy)]
pub(super) enum PageSetting {
    /// Every row in one list.
    Off,
    /// The table slices the rows it holds.
    Local(PagePlan),
    /// The caller supplies the page's rows and whether another page follows.
    Remote {
        page: PagePlan,
        has_more: Value<bool>,
    },
}

pub(super) type Key<R, K> = Rc<dyn Fn(&R) -> K>;
/// What a row shows beside its cells: its actions or its details.
pub(super) type RowSlot<R> = kivact_core::view::ViewFn<Value<R>>;
/// The controls of one cell, from its row and its column's index.
pub(super) type CellSlot<R> = kivact_core::view::ViewFn<(Value<R>, usize)>;
/// A region the caller fills, built once from the table's state.
pub(super) type Slot<R, K> = Box<dyn FnOnce(DataTableContext<R, K>) -> View>;
pub(super) type RowContextMenu<R> = kivact_core::view::ViewFn<(Value<R>, Node)>;
pub(super) type RowSelectable<R> = Rc<dyn Fn(&R) -> bool>;
pub(super) type RowStatusOf<R> = Rc<dyn Fn(&R) -> Option<RowStatus>>;
pub(super) type RowColor<R> = Rc<dyn Fn(&R) -> Option<Color>>;
/// Names a row in text.
pub(super) type RowName<R> = Rc<dyn Fn(&R) -> String>;

/// How a [`data_table`](super::data_table) pages its rows, as its
/// [`toolbar`](DataTable::toolbar) and [`footer`](DataTable::footer) slots see
/// it.
#[derive(Clone, Copy)]
pub enum Paging {
    /// Every row is shown in one list.
    Off,
    /// The table pages the rows it holds.
    Local {
        /// The page shown, from 0.
        page: Binding<usize>,
        /// The rows per page. Writing it chooses a size (and shows the first page) only when the table has [`page_sizes`](DataTable::page_sizes); otherwise a write is ignored.
        page_size: Binding<usize>,
    },
    /// The caller supplies one page and says whether another is available.
    Remote {
        /// The page shown, from 0.
        page: Binding<usize>,
        /// The rows per page. Writing it chooses a size (and shows the first page) only when the table has [`page_sizes`](DataTable::page_sizes); otherwise a write is ignored.
        page_size: Binding<usize>,
        /// Whether another page is available.
        has_more: Value<bool>,
    },
}

/// The live state of a [`data_table`] that its [`toolbar`](DataTable::toolbar)
/// and [`footer`](DataTable::footer) slots build their views from. Every
/// handle is reactive; the context is `Copy` and holds no copy of the rows.
pub struct DataTableContext<R: 'static, K: 'static> {
    /// The filter, sort and page model: its counts and its filtered keys.
    pub model: DataTableModelApi<R, K, usize>,
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
    pub typed_column_filters:
        Option<Binding<std::collections::HashMap<usize, kivact_interaction::FilterValue>>>,
    /// How the table pages its rows.
    pub paging: Paging,
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

// xtask-allow(state): legacy: grouping, tree, tree_expand_all, column_order, selection, row_selectable, page_sizes, row_index, filter, column_filters, typed_column_filters, sticky, widths, resize_range, row_label, actions, row_context_menu, toolbar, footer, details, expanded_rows, cells, row_height, row_status, row_color; DataTable::group_by key and collapsed binding; absent shows each page's rows without group headers
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
    pub(super) sort: SortSetting,
    pub(super) paging: PageSetting,
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
    pub(super) reorder: Option<Callback<kivact_interaction::Reorder<K>>>,
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

/// A status announced by name and shown in the table's leading lane.
#[derive(Clone, Debug, PartialEq)]
pub struct RowStatus {
    pub(crate) label: String,
    pub(crate) tone: Tone,
    pub(crate) icon: Option<Icon>,
    pub(crate) color: Option<Color>,
}

impl RowStatus {
    /// Create a marker in the theme's color for this tone.
    pub fn new(label: impl Into<String>, tone: Tone) -> Self {
        Self {
            label: label.into(),
            tone,
            icon: None,
            color: None,
        }
    }

    /// Use an explicit color instead of the theme's tone.
    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Display a named or custom icon instead of the default mark.
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}
