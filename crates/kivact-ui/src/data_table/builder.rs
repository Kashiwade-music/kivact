//! Role: The data table's constructor and the options a caller turns on.
use super::types::{DataTable, DataTableContext};
use crate::DataColumn;
use crate::layer::with_recipe_style;
use crate::variants::{Variant, Variants};
use kivact_core::control::{Node, NodeBuilder, column};
use kivact_core::primitive::PartState;
use kivact_core::style::Color;
use kivact_core::view::{HandlerResult, IntoValue, IntoView, Role, callback};
use kivact_core::{Binding, Value};
use kivact_interaction::{ScalarRange, SortOrder, TableFocus};
use std::hash::Hash;
use std::rc::Rc;

/// A themed table of the caller's `rows`, identified by `key`, with
/// `columns` in display order.
///
/// The rows are the caller's and the table never writes them. What the table
/// does besides showing them is each the caller's state, given to the method
/// that turns it on:
///
/// - [`selection`](DataTable::selection): a check box for each row and one
///   for the rows of the page, and "n of m row(s) selected." under the table;
/// - [`filter`](DataTable::filter): a filter field over the text the columns'
///   [`filter`](DataColumn::filter) functions match;
/// - [`sort`](DataTable::sort): the sorted column and direction, to start
///   sorted or show the sort elsewhere (the headers of columns with a
///   [`compare`](DataColumn::compare) sort the rows all the same);
/// - [`page`](DataTable::page): pages of rows and the buttons that turn them;
/// - [`resizable`](DataTable::resizable): a resize handle on each header.
///
/// A header sorts when its column is sortable, ascending and then
/// descending. A new sort or filter text shows the first page; a page past
/// the last one shows the last page and leaves the page as it is.
/// [`row_label`](DataTable::row_label) names the rows, their check boxes and
/// the typeahead. Without it a row is named by its first column's text.
///
/// [`toolbar`](DataTable::toolbar) and [`footer`](DataTable::footer) replace
/// the default regions with views built from the table's live state,
/// [`row_details`](DataTable::row_details) and
/// [`cell_controls`](DataTable::cell_controls) put controls in the rows that
/// stay mounted while the rows are edited, sorted and paged, and
/// [`row_actions`](DataTable::row_actions) adds a last column of controls.
/// The look is the theme's [`DataTableRecipes`](crate::DataTableRecipes).
///
/// Messages (a localized text replaces each): `data_table.label` ("Data
/// table", the grid's name), `data_table.filter` ("Filter…", the filter
/// field's placeholder and name), `data_table.empty` ("No results found."),
/// `data_table.actions` ("Actions", the header of the actions column),
/// `data_table.select_all` ("Select all"), `data_table.select_named` ("Select {row}"), `data_table.selected`
/// ("{selected} of {total} row(s) selected."), `data_table.previous`
/// ("Previous"), `data_table.next` ("Next"), `data_table.page` ("Page {page}
/// of {pages}") and `data_table.resize` ("Resize {title}").
///
/// ```no_run
/// use kivact::prelude::*;
///
/// #[derive(Clone, PartialEq)]
/// struct Payment { id: u32, email: String, amount: f64 }
///
/// fn payments(data: Signal<Vec<Payment>>) -> impl IntoView {
///     let selected = signal(Vec::<u32>::new());
///     let query = signal(String::new());
///     let page = signal(0_usize);
///     data_table(
///         data,
///         |p: &Payment| p.id,
///         vec![
///             DataColumn::new("Email", |p: &Payment| p.email.clone())
///                 .sortable()
///                 .filter(|p, query| p.email.contains(query)),
///             DataColumn::new("Amount", |p: &Payment| format!("${:.2}", p.amount))
///                 .compare(|a, b| a.amount.total_cmp(&b.amount))
///                 .numeric(true),
///         ],
///     )
///     .selection(selected)
///     .filter(query)
///     .page(page, 5)
/// }
/// ```
#[track_caller]
pub fn data_table<R, K>(
    rows: impl IntoValue<Vec<R>>,
    key: impl Fn(&R) -> K + 'static,
    columns: Vec<DataColumn<R>>,
) -> DataTable<R, K>
where
    R: Clone + PartialEq + 'static,
    K: Clone + Eq + Hash + 'static,
{
    // The variants are shared: `.variant` adds to them before mount.
    let variants = Variants::default();
    // The recipe is the first style layer, so the caller's styles win.
    let root = with_recipe_style(
        column(()).role(Role::Group),
        |r| r.data_table_recipes().frame_recipe(),
        &variants,
        PartState::default,
    );
    DataTable {
        root,
        rows: rows.into_value(),
        key: Rc::new(key),
        columns: Rc::new(columns),
        selection: None,
        sort: None,
        page: None,
        filter: None,
        widths: None,
        resize_range: None,
        row_label: None,
        actions: None,
        reorder: None,
        toolbar: None,
        footer: None,
        empty: None,
        details: None,
        cells: None,
        focus: TableFocus::default(),
        row_height: None,
        row_color: None,
        variants,
    }
}

impl<R: 'static, K: 'static> DataTable<R, K> {
    /// Give the component a [`Variant`]: a look beyond the meanings its
    /// state carries, which every recipe of the component reads with
    /// [`Look::variant`](crate::Look::variant). A later value of the same
    /// type replaces the earlier one; a theme that does not know the type
    /// draws the plain look.
    ///
    /// The value is a constant: the recipes read the values given before
    /// the component is mounted, and a value never changes afterwards. A
    /// look that changes while the component lives is a state the recipe
    /// reads, or a reactive `.style(..)` of the caller's.
    pub fn variant(self, v: impl Variant) -> Self {
        self.variants.add(v);
        self
    }

    /// Show a check box for each row and one that checks the rows of the
    /// page, keeping the keys of the checked rows in `selection`. Checking
    /// the page adds and removes only its rows, so the selection of rows on
    /// other pages and rows the filter hides stays. With
    /// [`TableFocus::Rows`](TableFocus) there are no check boxes: a press
    /// selects the row, Ctrl toggles and Shift extends.
    pub fn selection(mut self, selection: impl Into<Binding<Vec<K>>>) -> Self {
        self.selection = Some(selection.into());
        self
    }

    /// Keep the sorted column (an index into the columns) and its direction
    /// in `sort`, to start sorted or to show the sort elsewhere; the table
    /// reads the binding back, so the caller may set it too. Without it the
    /// headers sort in the table's own state, unless the table has an
    /// [`on_reorder`](Self::on_reorder), which then keeps the caller's order.
    pub fn sort(mut self, sort: impl Into<Binding<Option<(usize, SortOrder)>>>) -> Self {
        self.sort = Some(sort.into());
        self
    }

    /// Show `size` rows (at least 1) on each page, keeping the zero-based
    /// page in `page`, with Previous and Next buttons under the table.
    pub fn page(mut self, page: impl Into<Binding<usize>>, size: impl IntoValue<usize>) -> Self {
        self.page = Some((page.into(), size.into_value()));
        self
    }

    /// Show a filter field over `query` that keeps the rows a column's
    /// [`filter`](DataColumn::filter) matches; a new text shows the first
    /// page. An empty field keeps every row.
    pub fn filter(mut self, query: impl Into<Binding<String>>) -> Self {
        self.filter = Some(query.into());
        self
    }

    /// Give each header a resize handle, keeping the columns' widths (by
    /// position, in logical pixels, at least 18) in `widths` and reading them
    /// back. A width that is missing or not positive leaves the column's own.
    /// The handles use the shared table's keyboard, accessibility and pointer
    /// resizing. Escape during a drag restores the width from its press;
    /// earlier keyboard adjustments remain committed.
    pub fn resizable(mut self, widths: impl Into<Binding<Vec<f32>>>) -> Self {
        self.widths = Some(widths.into());
        self
    }

    /// Limit column resize operations to the given logical pixel range.
    /// The default range is 18 to 2048 pixels.
    pub fn resize_range(mut self, range: ScalarRange) -> Self {
        self.resize_range = Some(range);
        self
    }

    /// Name each row with `label`: the accessible name of the row, of its
    /// check box and its typeahead text. Without it a row is named by its
    /// first column's text.
    pub fn row_label(mut self, label: impl Fn(&R) -> String + 'static) -> Self {
        self.row_label = Some(Rc::new(label));
        self
    }

    /// A last column, `width` logical pixels wide, of each row's actions
    /// (`actions` of the row), usually a row menu. Its header shows no title
    /// and is named by the message `data_table.actions` ("Actions"). The
    /// actions are not grid cells: Tab reaches them like other controls.
    pub fn row_actions<V: IntoView>(
        mut self,
        width: f32,
        actions: impl Fn(Value<R>) -> V + 'static,
    ) -> Self {
        self.actions = Some((width.max(0.0), Rc::new(move |row| actions(row).into_view())));
        self
    }

    /// Let a row be dragged, and report the row's key and its zero-based
    /// destination in the caller's order. The application applies or
    /// rejects each request; a request clears the [`sort`](Self::sort), so
    /// the rows show the caller's order. A table with a reorder and without
    /// a `sort` does not sort.
    pub fn on_reorder<T: HandlerResult>(mut self, f: impl Fn((K, usize)) -> T + 'static) -> Self {
        self.reorder = Some(callback(f));
        self
    }

    /// Replace the region above the grid, built once with the table's live
    /// state: the default is the filter field (when the table has a
    /// [`filter`](Self::filter)). Put a column visibility menu or the bulk
    /// actions of the selection here.
    pub fn toolbar<V: IntoView>(
        mut self,
        slot: impl FnOnce(DataTableContext<R, K>) -> V + 'static,
    ) -> Self {
        self.toolbar = Some(Box::new(move |context| slot(context).into_view()));
        self
    }

    /// Replace the region under the grid, built once with the table's live
    /// state: the default is the selection summary and the pager.
    pub fn footer<V: IntoView>(
        mut self,
        slot: impl FnOnce(DataTableContext<R, K>) -> V + 'static,
    ) -> Self {
        self.footer = Some(Box::new(move |context| slot(context).into_view()));
        self
    }

    /// Replace the message shown when filtering or paging leaves no rows.
    /// The factory runs each time the empty state appears.
    pub fn empty<V: IntoView>(mut self, empty: impl Fn() -> V + 'static) -> Self {
        self.empty = Some(Box::new(move || empty().into_view()));
        self
    }

    /// Put the view `details` builds under each row, built once per keyed
    /// row from its reactive value, so it stays mounted while the row is
    /// edited, sorted or paged. The caller controls whether it is shown.
    pub fn row_details<V: IntoView>(mut self, details: impl Fn(Value<R>) -> V + 'static) -> Self {
        self.details = Some(Rc::new(move |row| details(row).into_view()));
        self
    }

    /// Draw every cell with `controls(row, column)`: controls that stay
    /// mounted while their row changes (the row is a reactive value) and
    /// keep their identity through sorting, filtering and paging. A column's
    /// own [`cell`](DataColumn::cell) view is made again from each new row,
    /// so a control whose state must survive an edit of its row (a field, a
    /// switch) belongs here. `column` is the column's position, and the
    /// cell's text still names it.
    pub fn cell_controls<V: IntoView>(
        mut self,
        controls: impl Fn(Value<R>, usize) -> V + 'static,
    ) -> Self {
        self.cells = Some(Rc::new(move |row, column| {
            controls(row, column).into_view()
        }));
        self
    }

    /// Choose what takes the keyboard focus: a cell ([`TableFocus::Cells`],
    /// the default, with a check box for each row when the table has a
    /// selection), a row ([`TableFocus::Rows`]), or neither in a semantic
    /// table ([`TableFocus::None`]). The variant carries the
    /// selection range policy (and, for rows, the whole list policy) of the
    /// shared table.
    pub fn focus(mut self, focus: TableFocus) -> Self {
        self.focus = focus;
        self
    }

    /// Give the main rows `height` logical pixels (at least 18, a constant
    /// or a reactive value); the row recipe reads it.
    pub fn row_height(mut self, height: impl IntoValue<f32>) -> Self {
        self.row_height = Some(height.into_value());
        self
    }

    /// Edge each row with the color `color` gives it (`None` for none); the
    /// row recipe reads it and resolves it in the theme in effect.
    pub fn row_color(mut self, color: impl Fn(&R) -> Option<Color> + 'static) -> Self {
        self.row_color = Some(Rc::new(color));
        self
    }
}

impl<R: 'static, K: 'static> NodeBuilder for DataTable<R, K> {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
