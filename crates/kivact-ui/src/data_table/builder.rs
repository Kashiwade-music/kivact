//! Role: The data table's constructor and the options a caller turns on.
use super::types::{DataTable, DataTableContext, DataTableTreeIndent, Grouping, TreeConfig};
use crate::column_values::DataColumn;
use crate::variants::{Variant, Variants};
use kivact_core::state::Value;
use kivact_core::state::{IntoValue, callback};
use kivact_core::style::Color;
use kivact_core::view::{HandlerResult, IntoView, Role};
use kivact_core::view::{Node, NodeBuilder, column};
use kivact_interaction::{ScalarRange, SortOrder, TableFocus};
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
/// of {pages}"), `data_table.resize` ("Resize {title}"), and for a
/// [`tree`](DataTable::tree) `data_table.expand_row` ("Expand row"),
/// `data_table.collapse_row` ("Collapse row"), `data_table.expand_all`
/// ("Expand all rows") and `data_table.collapse_all` ("Collapse all rows").
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
    R: kivact_core::state::PropValue,
    K: kivact_core::state::ItemKey,
{
    let root = column(()).role(Role::Group);
    DataTable {
        root,
        rows: rows.into_value(),
        grouping: None,
        tree: None,
        tree_column: 0,
        tree_indent: DataTableTreeIndent::default(),
        tree_expand_all: false,
        key: Rc::new(key),
        columns: Rc::new(columns),
        column_order: None,
        selection: None,
        row_selectable: None,
        sort: None,
        multi_sort: None,
        page: None,
        has_more: None,
        page_sizes: None,
        row_index: None,
        filter: None,
        column_filters: None,
        typed_column_filters: None,
        filter_ui: crate::FilterUi::default(),
        sticky: None,
        widths: None,
        resize_range: None,
        row_label: None,
        actions: None,
        row_context_menu: None,
        reorder: None,
        toolbar: None,
        footer: None,
        empty: None,
        details: None,
        expanded_rows: None,
        cells: None,
        focus: TableFocus::default(),
        row_height: None,
        row_status: None,
        row_color: None,
        variants: Variants::default(),
    }
}

impl<R: 'static, K: 'static> DataTable<R, K> {
    /// Group the filtered, sorted rows on each page by a string key.
    ///
    /// Groups follow first appearance on that page and their rows keep the
    /// sorted order. Collapsing hides a group's data rows without changing
    /// the page, selection keys, or page counts. The header count is the
    /// number of matching rows from that group on the current page. For
    /// remote paging, groups are formed from the caller's supplied page.
    pub fn group_by<G: Into<String>>(
        mut self,
        group: impl Fn(&R) -> G + 'static,
        collapsed: impl kivact_core::state::IntoBinding<Vec<String>>,
    ) -> Self {
        self.grouping = Some(Grouping {
            key: Rc::new(move |row| group(row).into()),
            collapsed: collapsed.into_binding(),
        });
        self
    }

    /// Display rows as a tree. A row whose parent key is absent becomes a root.
    pub fn tree(
        mut self,
        parent: impl Fn(&R) -> Option<K> + 'static,
        expanded: impl kivact_core::state::IntoBinding<Vec<K>>,
    ) -> Self {
        self.tree = Some(TreeConfig {
            parent: Rc::new(parent),
            expanded: expanded.into_binding(),
        });
        self
    }

    /// Choose the original column index that displays the tree controls.
    pub fn tree_column(mut self, index: usize) -> Self {
        self.tree_column = index;
        self
    }

    /// Choose the horizontal space added at each tree level.
    pub fn tree_indent(mut self, indent: DataTableTreeIndent) -> Self {
        self.tree_indent = indent;
        self
    }

    /// Show the expand/collapse-all control in the tree column header. It
    /// does not change which rows start expanded: the `expanded` binding of
    /// [`tree`](Self::tree) decides that.
    pub fn expand_all_control(mut self) -> Self {
        self.tree_expand_all = true;
        self
    }

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
    pub fn variant(mut self, v: impl Variant) -> Self {
        self.variants.add(v);
        self
    }

    /// Select the density used by themes that style data table cells.
    pub fn density(self, density: super::types::DataTableDensity) -> Self {
        self.variant(density)
    }

    /// Pin visible data columns to the start and end of the horizontal viewport.
    /// Indices are in declaration order; hidden or invalid indices are ignored.
    /// When an index occurs on both sides, start takes precedence.
    pub fn sticky_columns(mut self, start: Vec<usize>, end: Vec<usize>) -> Self {
        self.sticky = (!start.is_empty() || !end.is_empty()).then_some((start, end));
        self
    }

    /// Show a check box for each row and one that checks the rows of the
    /// page, keeping the keys of the checked rows in `selection`. Checking
    /// the page adds and removes only its rows, so the selection of rows on
    /// other pages and rows the filter hides stays. With
    /// [`TableFocus::Rows`](TableFocus) there are no check boxes: a press
    /// selects the row, Ctrl toggles and Shift extends.
    pub fn selection(mut self, selection: impl kivact_core::state::IntoBinding<Vec<K>>) -> Self {
        self.selection = Some(selection.into_binding());
        self
    }

    /// Open a menu from a row's secondary click or context-menu key.
    /// The menu receives the live row value, including updates while mounted.
    pub fn row_context_menu<M: kivact_core::state::ItemKey>(
        mut self,
        menu: impl Fn(Value<R>) -> crate::Menu<M> + 'static,
    ) -> Self {
        self.row_context_menu = Some(kivact_core::view::ViewFn::from(move |(row, target)| {
            menu(row).attach_context_fallback(target)
        }));
        self
    }

    /// Choose which rows can be selected. All rows remain navigable, and
    /// the caller's existing selection is left intact when eligibility changes.
    pub fn row_selectable(mut self, test: impl Fn(&R) -> bool + 'static) -> Self {
        self.row_selectable = Some(Rc::new(test));
        self
    }

    /// Set the visible columns' order by their original zero-based indices.
    /// Hidden columns stay hidden. Duplicate and unknown indices are ignored,
    /// and omitted columns follow in their original order. Sort and resize
    /// bindings continue to use the original indices.
    pub fn column_order(mut self, order: impl IntoValue<Vec<usize>>) -> Self {
        self.column_order = Some(order.into_value());
        self
    }

    /// Keep the sorted column (an index into the columns) and its direction
    /// in `sort`, to start sorted or to show the sort elsewhere; the table
    /// reads the binding back, so the caller may set it too. Without it the
    /// headers sort in the table's own state, unless the table has an
    /// [`on_reorder`](Self::on_reorder), which then keeps the caller's order.
    pub fn sort(
        mut self,
        sort: impl kivact_core::state::IntoBinding<Option<(usize, SortOrder)>>,
    ) -> Self {
        self.sort = Some(sort.into_binding());
        self.multi_sort = None;
        self
    }

    /// Opt in to ordered multi-column sorting. Shift-click or Shift-activate
    /// a header to append or cycle that column without replacing the other
    /// keys. Normal activation replaces the sort. Each key cycles ascending,
    /// descending, and unsorted. Column indices refer to the original order.
    pub fn multi_sort(
        mut self,
        sort: impl kivact_core::state::IntoBinding<Vec<(usize, SortOrder)>>,
    ) -> Self {
        self.multi_sort = Some(sort.into_binding());
        self.sort = None;
        self
    }

    /// Show `size` rows (at least 1) on each page, keeping the zero-based
    /// page in `page`, with Previous and Next buttons under the table.
    pub fn page(
        mut self,
        page: impl kivact_core::state::IntoBinding<usize>,
        size: impl IntoValue<usize>,
    ) -> Self {
        self.page = Some((page.into_binding(), size.into_value()));
        self.has_more = None;
        self
    }

    /// Edit the committed typed values keyed by the original column index.
    /// A column needs `DataColumn::filter_field` to participate.
    pub fn typed_column_filters(
        mut self,
        filters: impl kivact_core::state::IntoBinding<
            std::collections::HashMap<usize, kivact_interaction::FilterValue>,
        >,
    ) -> Self {
        self.typed_column_filters = Some(filters.into_binding());
        self
    }

    /// Choose how the typed filter controls appear in column headers.
    pub fn filter_ui(mut self, ui: crate::FilterUi) -> Self {
        self.filter_ui = ui;
        self
    }

    /// Show caller-supplied page rows without slicing them again. The caller
    /// loads the page indicated by the binding and supplies whether another
    /// page is available.
    pub fn remote_page(
        mut self,
        page: impl kivact_core::state::IntoBinding<usize>,
        size: impl IntoValue<usize>,
        has_more: impl IntoValue<bool>,
    ) -> Self {
        self.page = Some((page.into_binding(), size.into_value()));
        self.has_more = Some(has_more.into_value());
        self
    }

    /// Offer a writable page size and sizes to choose from. Applies with
    /// [`page`](Self::page) or [`remote_page`](Self::remote_page). Choosing a size
    /// resets the page to zero. Values of zero are omitted.
    pub fn page_sizes(
        mut self,
        size: impl kivact_core::state::IntoBinding<usize>,
        options: impl IntoValue<Vec<usize>>,
    ) -> Self {
        self.page_sizes = Some((size.into_binding(), options.into_value()));
        self
    }

    /// Show a fixed-width display-position gutter starting at this number.
    /// For paged tables, numbering includes the displayed page's offset.
    pub fn row_index(mut self, start: usize) -> Self {
        self.row_index = Some(start);
        self
    }

    /// Show a filter field over `query` that keeps the rows a column's
    /// [`filter`](DataColumn::filter) matches; a new text shows the first
    /// page. An empty field keeps every row.
    pub fn filter(mut self, query: impl kivact_core::state::IntoBinding<String>) -> Self {
        self.filter = Some(query.into_binding());
        self
    }

    /// Apply independent queries by column index alongside the global filter.
    /// Only shown columns with filter predicates participate. Use the toolbar
    /// context to build controls over this binding.
    pub fn column_filters(
        mut self,
        queries: impl kivact_core::state::IntoBinding<Vec<(usize, String)>>,
    ) -> Self {
        self.column_filters = Some(queries.into_binding());
        self
    }

    /// Give each header a resize handle, keeping the columns' widths (by
    /// position, in logical pixels, at least 18) in `widths` and reading them
    /// back. A width that is missing or not positive leaves the column's own.
    /// The handles use the shared table's keyboard, accessibility and pointer
    /// resizing. Escape during a drag restores the width from its press;
    /// earlier keyboard adjustments remain committed.
    pub fn resizable(mut self, widths: impl kivact_core::state::IntoBinding<Vec<f64>>) -> Self {
        self.widths = Some(widths.into_binding());
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
    /// In cell focus mode, the actions occupy one grid cell. Enter or F2
    /// enters its controls and Escape returns to cell navigation. In table
    /// or row focus mode the controls keep their normal Tab stops.
    pub fn row_actions<V: IntoView>(
        mut self,
        width: f64,
        actions: impl Fn(Value<R>) -> V + 'static,
    ) -> Self {
        self.actions = Some((width.max(0.0), kivact_core::view::ViewFn::from(actions)));
        self
    }

    /// Let a row be dragged, and report the row's key and its zero-based
    /// destination in the caller's order. The application applies or
    /// rejects each request; a request clears the [`sort`](Self::sort), so
    /// the rows show the caller's order. A table with a reorder and without
    /// a `sort` does not sort.
    pub fn on_reorder<T: HandlerResult>(
        mut self,
        f: impl Fn(kivact_interaction::Reorder<K>) -> T + 'static,
    ) -> Self {
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
        slot: impl Fn(DataTableContext<R, K>) -> V + 'static,
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
        self.details = Some(kivact_core::view::ViewFn::from(details));
        self
    }

    /// Add a disclosure control to each row with [`row_details`](Self::row_details).
    /// The binding contains the expanded row keys, independent of sort and
    /// paging. Details stay mounted while their row is mounted.
    pub fn expanded_rows(mut self, expanded: impl kivact_core::state::IntoBinding<Vec<K>>) -> Self {
        self.expanded_rows = Some(expanded.into_binding());
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
        self.cells = Some(kivact_core::view::ViewFn::from(move |(row, column)| {
            controls(row, column)
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
    pub fn row_height(mut self, height: impl IntoValue<f64>) -> Self {
        self.row_height = Some(height.into_value());
        self
    }

    /// Reserve a leading status lane and resolve each row's optional marker.
    /// The lane is separate from the indexed data columns.
    pub fn row_status(
        mut self,
        status: impl Fn(&R) -> Option<crate::table_values::RowStatus> + 'static,
    ) -> Self {
        self.row_status = Some(Rc::new(status));
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
