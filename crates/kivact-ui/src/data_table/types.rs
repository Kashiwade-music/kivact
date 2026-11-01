//! Role: The state of a data table: its builder fields, slots and live context.
use crate::DataColumn;
use crate::variants::Variants;
use kivact_core::control::Node;
use kivact_core::style::Color;
use kivact_core::view::{SortDirection, View};
use kivact_core::{Binding, Callback, Value};
use kivact_interaction::{ListPolicy, TableFocus, TableModel};
use std::rc::Rc;

/// The sorted column (an index into the columns) and its direction.
pub(super) type Sort = Binding<Option<(usize, SortDirection)>>;
/// The page shown and the rows on a page.
pub(super) type Paging = (Binding<usize>, Value<usize>);
pub(super) type Key<R, K> = Rc<dyn Fn(&R) -> K>;
/// What a row shows beside its cells: its actions or its details.
pub(super) type RowSlot<R> = Rc<dyn Fn(Value<R>) -> View>;
/// The controls of one cell, from its row and its column's index.
pub(super) type CellSlot<R> = Rc<dyn Fn(Value<R>, usize) -> View>;
/// A region the caller fills, built once from the table's state.
pub(super) type Slot<R, K> = Box<dyn FnOnce(DataTableContext<R, K>) -> View>;
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
    pub sort: Binding<Option<(usize, SortDirection)>>,
    /// The caller's filter text, when the table has one.
    pub filter: Option<Binding<String>>,
    /// The caller's page number and the rows on a page, when the table
    /// pages.
    pub page: Option<(Binding<usize>, Value<usize>)>,
}

impl<R: 'static, K: 'static> Copy for DataTableContext<R, K> {}

impl<R: 'static, K: 'static> Clone for DataTableContext<R, K> {
    fn clone(&self) -> Self {
        *self
    }
}

/// A table of the caller's rows, created by [`data_table`].
pub struct DataTable<R: 'static, K: 'static> {
    pub(super) root: Node,
    pub(super) rows: Value<Vec<R>>,
    pub(super) key: Key<R, K>,
    pub(super) columns: Rc<Vec<DataColumn<R>>>,
    pub(super) selection: Option<Binding<Vec<K>>>,
    pub(super) sort: Option<Sort>,
    pub(super) page: Option<Paging>,
    pub(super) filter: Option<Binding<String>>,
    pub(super) widths: Option<Binding<Vec<f32>>>,
    pub(super) row_label: Option<RowName<R>>,
    pub(super) actions: Option<(f32, RowSlot<R>)>,
    pub(super) reorder: Option<Callback<(K, usize)>>,
    pub(super) toolbar: Option<Slot<R, K>>,
    pub(super) footer: Option<Slot<R, K>>,
    pub(super) details: Option<RowSlot<R>>,
    pub(super) cells: Option<CellSlot<R>>,
    pub(super) focus: TableFocus,
    pub(super) policy: ListPolicy,
    pub(super) row_height: Option<Value<f32>>,
    pub(super) row_color: Option<RowColor<R>>,
    pub(super) variants: Variants,
}
