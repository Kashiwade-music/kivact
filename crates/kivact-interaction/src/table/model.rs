//! Role: A headless row model: filter, sort and page the caller's rows by column.

use super::data::{Header, SortOrder};
use kivact_core::custom::Reactive;
use kivact_core::state::{Binding, Signal, Value, Var, memo, signal, var};
use kivact_core::state::{IntoText, IntoValue};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::rc::Rc;

type Compare<R> = Rc<dyn Fn(&R, &R) -> Ordering>;
type Matches<R> = Rc<dyn Fn(&R, &str) -> bool>;
type MatchesValue<R> = Rc<dyn Fn(&R, &FilterValue) -> bool>;
type KeyOf<R, K> = Rc<dyn Fn(&R) -> K>;
/// The caller's sort column and direction.
type Sort<C> = Value<Vec<(C, SortOrder)>>;
type ColumnQueries<C> = Value<Vec<(C, String)>>;
type RowArrangement<R> = Rc<dyn Fn(&[R], Vec<usize>) -> Vec<usize>>;
type TypedQueries<C> = Value<HashMap<C, FilterValue>>;

/// A committed typed column filter.
#[derive(Clone, Debug, PartialEq)]
pub enum FilterValue {
    /// A text query.
    Text(String),
    /// A finite number.
    Number(f64),
    /// The key of a selected choice.
    Choice(String),
    /// The keys of the selected choices.
    Choices(Vec<String>),
    /// A calendar date without a time zone.
    Date(kivact_core::i18n::CalendarDate),
    /// A local time of day.
    Time { hour: u8, minute: u8 },
    /// A list of text tokens.
    Tokens(Vec<String>),
}

impl FilterValue {
    /// Whether this value has no effective query.
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Text(text) => text.trim().is_empty(),
            Self::Number(value) => !value.is_finite(),
            Self::Choices(values) | Self::Tokens(values) => values.is_empty(),
            Self::Time { hour, minute } => *hour >= 24 || *minute >= 60,
            Self::Choice(_) | Self::Date(_) => false,
        }
    }
}

/// One column of a table: its key, label and starting width, how its rows
/// compare and match a filter (TanStack DataTableApi's column definition), and
/// whether it is shown.
///
/// This is the only column type: a [`DataTableModelApi`] sorts, filters and hides
/// by it, and [`use_data_table`](crate::use_data_table) reads the keys, labels, widths
/// and whether a header sorts from it. A column is sortable exactly when it
/// has a [`compare`](Self::compare); a column without
/// [`filter`](Self::filter) does not take part in filtering.
pub struct ColumnDef<R, C> {
    key: C,
    label: Reactive<String>,
    // xtask-allow(state-fields): builder option - width
    width: Option<f64>,
    // xtask-allow(state-fields): builder option - min width
    pub(super) min_width: Option<f64>,
    // xtask-allow(state-fields): builder option - compare
    compare: Option<Compare<R>>,
    // xtask-allow(state-fields): builder option - matches
    matches: Option<Matches<R>>,
    // xtask-allow(state-fields): builder option - matches value
    matches_value: Option<MatchesValue<R>>,
    visible: Reactive<bool>,
}

impl<R, C: Clone> Clone for ColumnDef<R, C> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            label: self.label.clone(),
            width: self.width,
            min_width: self.min_width,
            compare: self.compare.clone(),
            matches: self.matches.clone(),
            matches_value: self.matches_value.clone(),
            visible: self.visible.clone(),
        }
    }
}

impl<R, C> ColumnDef<R, C> {
    /// A shown column that neither sorts nor filters.
    pub fn new(key: C, label: impl IntoText) -> Self {
        Self {
            key,
            label: Reactive::new(label.into_text()),
            width: None,
            min_width: None,
            compare: None,
            matches: None,
            matches_value: None,
            visible: Reactive::new(true),
        }
    }

    /// Sort by this column with `compare`, ascending; a descending sort
    /// reverses it. The sort is stable, so equal rows keep their order.
    pub fn compare(mut self, compare: impl Fn(&R, &R) -> Ordering + 'static) -> Self {
        self.compare = Some(Rc::new(compare));
        self
    }

    /// Let this column match the filter text: a row is kept when any shown
    /// column's `matches(row, query)` is true. `query` is the filter text as
    /// given, trimmed of surrounding white space.
    pub fn filter(mut self, matches: impl Fn(&R, &str) -> bool + 'static) -> Self {
        self.matches = Some(Rc::new(matches));
        self
    }

    /// Match this column's committed typed filter value.
    pub fn filter_value(mut self, matches: impl Fn(&R, &FilterValue) -> bool + 'static) -> Self {
        self.matches_value = Some(Rc::new(matches));
        self
    }

    /// Show the column while true (a constant or a reactive value). A hidden
    /// column is left out of [`DataTableModelApi::columns`] and of filtering.
    pub fn visible(mut self, visible: impl IntoValue<bool>) -> Self {
        self.visible = Reactive::new(visible);
        self
    }

    /// The starting width in the caller's unit; `None` leaves it to layout.
    pub fn with_width(mut self, width: impl Into<Option<f64>>) -> Self {
        self.width = width.into();
        self
    }

    /// Set the column-specific lower bound for resizing.
    pub fn min_width(mut self, width: Option<f64>) -> Self {
        self.min_width = width.filter(|width| width.is_finite() && *width >= 0.0);
        self
    }

    /// This definition under another key; its label, width, ordering,
    /// match and visibility keep their identity.
    pub fn with_key<D>(self, key: D) -> ColumnDef<R, D> {
        ColumnDef {
            key,
            label: self.label,
            width: self.width,
            min_width: self.min_width,
            compare: self.compare,
            matches: self.matches,
            matches_value: self.matches_value,
            visible: self.visible,
        }
    }

    /// This definition without its ordering, so its header does not sort.
    pub fn without_ordering(mut self) -> Self {
        self.compare = None;
        self
    }

    /// The application's identity of the column.
    pub fn key(&self) -> &C {
        &self.key
    }

    /// The reactive label text of the current owner.
    #[track_caller]
    pub fn label(&self) -> Value<String> {
        self.label.clone().into_value()
    }

    /// Whether a header activation sorts by the column (it has a
    /// [`compare`](Self::compare)).
    pub fn is_sortable(&self) -> bool {
        self.compare.is_some()
    }

    /// Whether the column takes part in filtering ([`filter`](Self::filter)).
    pub fn is_filtered(&self) -> bool {
        self.matches.is_some() || self.matches_value.is_some()
    }

    /// The starting width ([`width`](Self::width)).
    pub fn width(&self) -> Option<f64> {
        self.width
    }
}

impl<R, C: Clone> ColumnDef<R, C> {
    /// What a table reads of the definition.
    pub(super) fn header(&self) -> Header<C> {
        Header {
            key: self.key.clone(),
            label: self.label.clone(),
            sortable: self.compare.is_some(),
            width: self.width,
            min_width: self.min_width,
        }
    }
}

/// Definitions are equal when their keys, labels, widths and visibility are
/// and their comparison and match functions are the same functions (a
/// closure is equal to a clone of itself).
impl<R, C: PartialEq> PartialEq for ColumnDef<R, C> {
    fn eq(&self, other: &Self) -> bool {
        fn same<T: ?Sized>(a: Option<&Rc<T>>, b: Option<&Rc<T>>) -> bool {
            match (a, b) {
                (Some(a), Some(b)) => Rc::ptr_eq(a, b),
                (None, None) => true,
                _ => false,
            }
        }
        self.key == other.key
            && self.label == other.label
            && self.width == other.width
            && self.min_width == other.min_width
            && self.visible == other.visible
            && same(self.compare.as_ref(), other.compare.as_ref())
            && same(self.matches.as_ref(), other.matches.as_ref())
            && same(self.matches_value.as_ref(), other.matches_value.as_ref())
    }
}

/// The page a model shows: its number (from 0) and its size in rows.
#[derive(Clone, Copy)]
struct Paging {
    index: Binding<usize>,
    size: Value<usize>,
}

/// A Copy handle over the caller's rows that filters, sorts and pages them,
/// the headless row model of TanStack DataTableApi.
///
/// The model keeps no rows of its own: it reads the caller's rows, the sort
/// column and direction, the filter text and the page, and derives the rows
/// to show. It draws nothing and has no keys or focus; give its
/// [`rows`](Self::rows) and [`columns`](Self::columns) to
/// [`use_data_table`](crate::use_data_table) for the grid's keyboard and selection, and
/// give both the same sort binding, so that activating a header sorts the
/// model:
///
/// ```no_run
/// # mod kivact {
/// #     //! Role: Supply facade paths for this documentation example.
/// # pub use kivact_core::{view, state, style};
/// # pub mod prelude {
/// # //! Role: Supply the canonical imports used by this documentation example.
/// # pub use kivact_core::state::{Binding, Callback, IntoBinding, IntoText, IntoValue, Signal, Value, binding, effect, memo, on_cleanup, on_mount, signal};
/// # pub use kivact_core::view::{EditPhase, Component, IntoView, Element, Orientation, Role, column, for_each, node, node_ref, row, show, text};
/// # pub use kivact_core::style::{ColorToken, Cursor, Style, style};
/// # }
/// #     pub extern crate kivact_interaction as interaction;
/// # }
/// use kivact::interaction::{ColumnDef, TableFocus, use_data_table, use_data_table_model};
/// use kivact::prelude::*;
/// use kivact::interaction::SortOrder;
/// #[derive(Clone, PartialEq)]
/// struct Person { id: u32, name: String, age: u32 }
/// let people = signal(vec![Person { id: 1, name: "Ada".into(), age: 36 }]);
/// let sort = signal(Some(("name", SortOrder::Ascending)));
/// let query = signal(String::new());
/// let page = signal(0_usize);
/// let model = use_data_table_model(people, |p: &Person| p.id, vec![
///     ColumnDef::new("name", "Name")
///         .compare(|a: &Person, b: &Person| a.name.cmp(&b.name))
///         .filter(|p: &Person, q: &str| p.name.to_lowercase().contains(&q.to_lowercase())),
///     ColumnDef::new("age", "Age").compare(|a: &Person, b: &Person| a.age.cmp(&b.age)),
/// ])
/// .sort(sort)
/// .filter(query)
/// .page(page, 20_usize);
/// let selected = signal(Vec::<u32>::new());
/// let table = use_data_table(model.rows(), |p: &Person| p.id, model.columns(),
///     selected, TableFocus::default())
///     .with_sort(sort);
/// let status = text(move || format!("{} people, page {} of {}",
///     model.row_count().get(), page.get() + 1, model.page_count().get()));
/// # let _ = (table, status);
/// ```
pub struct DataTableModelApi<R: 'static, K: 'static, C: 'static> {
    /// The caller's rows, every one.
    source: Value<Vec<R>>,
    /// The indices into `source` of the rows that pass the filter, sorted.
    ordered: Value<Vec<usize>>,
    key: Var<KeyOf<R, K>>,
    rows: Value<Vec<R>>,
    keys: Value<Vec<K>>,
    columns: Value<Vec<ColumnDef<R, C>>>,
    count: Value<usize>,
    pages: Value<usize>,
    // xtask-allow(state-fields): builder option - the sort order a setter supplied, absent until set
    sort: Signal<Option<Sort<C>>>,
    // xtask-allow(state-fields): builder option - the text query a setter supplied, absent until set
    query: Signal<Option<Value<String>>>,
    // xtask-allow(state-fields): builder option - the column queries a setter supplied, absent until set
    column_queries: Signal<Option<ColumnQueries<C>>>,
    // xtask-allow(state-fields): builder option - the row arrangement a setter supplied, absent until set
    arrangement: Signal<Option<RowArrangement<R>>>,
    // xtask-allow(state-fields): builder option - the typed queries a setter supplied, absent until set
    typed_queries: Signal<Option<TypedQueries<C>>>,
    // xtask-allow(state-fields): builder option - the paging a setter supplied, absent until set
    paging: Signal<Option<Paging>>,
}

impl<R: 'static, K: 'static, C: 'static> Clone for DataTableModelApi<R, K, C> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<R: 'static, K: 'static, C: 'static> Copy for DataTableModelApi<R, K, C> {}

/// Create a row model over `rows`, identified by `key`, with `columns` in
/// display order. Without [`sort`](DataTableModelApi::sort),
/// [`filter`](DataTableModelApi::filter) and [`page`](DataTableModelApi::page) it shows
/// every row in the caller's order. Setup only.
#[track_caller]
pub fn use_data_table_model<R, K, C>(
    rows: impl IntoValue<Vec<R>>,
    key: impl Fn(&R) -> K + 'static,
    columns: Vec<ColumnDef<R, C>>,
) -> DataTableModelApi<R, K, C>
where
    R: kivact_core::state::PropValue,
    K: kivact_core::state::ItemKey,
    C: kivact_core::state::ItemKey,
{
    let rows = rows.into_value();
    let key: KeyOf<R, K> = Rc::new(key);
    let key_of = key.clone();
    let columns: Rc<[ColumnDef<R, C>]> = columns.into();
    let sort = signal(None::<Sort<C>>);
    let query = signal(None::<Value<String>>);
    let column_queries = signal(None::<Value<Vec<(C, String)>>>);
    let arrangement = signal(None::<RowArrangement<R>>);
    let typed_queries = signal(None::<TypedQueries<C>>);
    let paging = signal(None::<Paging>);
    let shown = {
        let columns = columns.clone();
        move || {
            columns
                .iter()
                .filter(|column| column.visible.get())
                .cloned()
                .collect::<Vec<_>>()
        }
    };
    // The indices of the rows that pass the filter, in sorted order.
    let ordered = ordered_rows(
        rows,
        columns.clone(),
        sort,
        query,
        column_queries,
        arrangement,
        typed_queries,
    );
    let count = memo(move || ordered.with(Vec::len));
    let pages = memo(move || {
        let count = count.get();
        match paging.get() {
            Some(paging) if count > 0 => count.div_ceil(paging.size.get().max(1)),
            Some(_) | None => 1,
        }
    });
    let page = memo(move || {
        let range = match paging.get() {
            Some(paging) => {
                let size = paging.size.get().max(1);
                let start = paging.index.get().min(pages.get() - 1).saturating_mul(size);
                Some((start, size))
            }
            None => None,
        };
        ordered.with(|order| match range {
            Some((start, size)) => order
                .iter()
                .skip(start)
                .take(size)
                .copied()
                .collect::<Vec<_>>(),
            None => order.clone(),
        })
    });
    let shown_rows = memo(move || {
        rows.with(|rows| {
            page.with(|page| page.iter().filter_map(|&i| rows.get(i).cloned()).collect())
        })
    });
    let keys =
        memo(move || shown_rows.with(|rows: &Vec<R>| rows.iter().map(|row| key_of(row)).collect()));
    let columns = memo(move || shown().into_iter().collect::<Vec<_>>());
    DataTableModelApi {
        source: rows,
        ordered,
        key: var(key),
        rows: shown_rows,
        keys,
        columns,
        count,
        pages,
        sort,
        query,
        column_queries,
        arrangement,
        typed_queries,
        paging,
    }
}

impl<
    R: kivact_core::state::PropValue,
    K: kivact_core::state::ItemKey,
    C: kivact_core::state::ItemKey,
> DataTableModelApi<R, K, C>
{
    /// Sort by the column and direction this binding holds; `None` or a
    /// column without a compare function keeps the caller's order. Give the same binding to
    /// [`DataTableApi::sort`](crate::DataTableApi::sort). Setup only.
    #[track_caller]
    pub fn sort(self, sort: impl kivact_core::state::IntoBinding<Option<(C, SortOrder)>>) -> Self {
        let sort = sort.into_binding();
        self.multi_sort(move || sort.get().into_iter().collect())
    }

    /// Sort by an ordered list of column keys, with the first key primary.
    /// Rows equal on every key retain their original order. Setup only.
    pub fn multi_sort(self, sort: impl IntoValue<Vec<(C, SortOrder)>>) -> Self {
        self.sort.set(Some(sort.into_value()));
        self
    }

    /// Keep the rows a shown column's [`filter`](ColumnDef::filter) matches
    /// with this text (constant or reactive); an empty or blank text keeps
    /// every row. The model only reads it. Setup only.
    #[track_caller]
    pub fn filter(self, query: impl IntoValue<String>) -> Self {
        let query = query.into_value();
        self.query.update(|current| *current = Some(query));
        self
    }

    /// Require each nonblank column query to match that shown column's
    /// [predicate](ColumnDef::filter), in addition to the global filter.
    /// Unknown, hidden and non-filterable columns are ignored. Repeated
    /// queries for a column must all match. The model only reads the queries.
    #[track_caller]
    pub fn column_filters(self, queries: impl IntoValue<Vec<(C, String)>>) -> Self {
        self.column_queries.set(Some(queries.into_value()));
        self
    }

    /// Arrange or remove matched source indices after sorting, before paging.
    /// The arrangement must return only indices from the supplied order.
    pub fn arrange_rows(self, arrange: impl Fn(&[R], Vec<usize>) -> Vec<usize> + 'static) -> Self {
        self.arrangement
            .update(|current| *current = Some(Rc::new(arrange)));
        self
    }

    /// Require each typed value to match its shown column, in addition to
    /// text queries. Empty values and columns without a predicate are ignored.
    pub fn typed_column_filters(self, values: impl IntoValue<HashMap<C, FilterValue>>) -> Self {
        self.typed_queries.set(Some(values.into_value()));
        self
    }

    /// Show the page `index` (from 0) of `size` rows (at least 1). The model
    /// only reads `index`: when the rows shrink (or have not arrived yet) so
    /// that `index` is past the last page, it shows the last page and leaves
    /// the binding as it is, so the page number comes back when the rows do.
    /// Setup only.
    #[track_caller]
    pub fn page(
        self,
        index: impl kivact_core::state::IntoBinding<usize>,
        size: impl IntoValue<usize>,
    ) -> Self {
        let paging = Paging {
            index: index.into_binding(),
            size: size.into_value(),
        };
        self.paging.update(|current| *current = Some(paging));
        self
    }

    /// The rows of the current page, filtered and sorted.
    pub fn rows(self) -> Value<Vec<R>> {
        self.rows
    }

    /// The keys of [`rows`](Self::rows), in the same order.
    pub fn keys(self) -> Value<Vec<K>> {
        self.keys
    }

    /// The keys of every row that passes the filter, sorted, on every page
    /// (TanStack DataTableApi's filtered row model): what a summary such as
    /// "2 of 40 row(s) selected." counts. Setup only.
    #[track_caller]
    pub fn filtered_keys(self) -> Value<Vec<K>> {
        let (source, ordered, key) = (self.source, self.ordered, self.key.get());
        memo(move || {
            source.with(|rows| {
                ordered.with(|order| {
                    order
                        .iter()
                        .filter_map(|&at| rows.get(at).map(|row| key(row)))
                        .collect()
                })
            })
        })
    }

    /// The shown column definitions, for [`use_data_table`](crate::use_data_table).
    pub fn columns(self) -> Value<Vec<ColumnDef<R, C>>> {
        self.columns
    }

    /// How many rows pass the filter, on every page.
    pub fn row_count(self) -> Value<usize> {
        self.count
    }

    /// How many pages the filtered rows fill; 1 without paging or rows.
    pub fn page_count(self) -> Value<usize> {
        self.pages
    }
}

fn ordered_rows<R: kivact_core::state::PropValue, C: kivact_core::state::ItemKey>(
    rows: Value<Vec<R>>,
    columns: Rc<[ColumnDef<R, C>]>,
    sort: Signal<Option<Sort<C>>>,
    query: Signal<Option<Value<String>>>,
    column_queries: Signal<Option<ColumnQueries<C>>>,
    arrangement: Signal<Option<RowArrangement<R>>>,
    typed_queries: Signal<Option<TypedQueries<C>>>,
) -> Value<Vec<usize>> {
    memo(move || {
        let query = query
            .get()
            .map(|query| query.get().trim().to_string())
            .unwrap_or_default();
        let sort = sort.get().map(|sort| sort.get()).unwrap_or_default();
        let column_queries = column_queries
            .get()
            .map(|queries| queries.get())
            .unwrap_or_default();
        let typed = typed_queries
            .get()
            .map(|queries| queries.get())
            .unwrap_or_default();
        let shown = if query.is_empty() && column_queries.is_empty() && typed.is_empty() {
            Vec::new()
        } else {
            columns
                .iter()
                .filter(|column| column.visible.get())
                .cloned()
                .collect::<Vec<_>>()
        };
        rows.with(|rows| {
            let mut kept = matching_rows(rows, &shown, &query, &column_queries, &typed);
            let comparisons: Vec<_> = sort
                .into_iter()
                .filter_map(|(key, direction)| {
                    let compare = columns.iter().find(|c| c.key == key)?.compare.clone()?;
                    Some((compare, direction))
                })
                .collect();
            if !comparisons.is_empty() {
                kept.sort_by(|&a, &b| {
                    comparisons
                        .iter()
                        .find_map(|(compare, direction)| {
                            let order = compare(&rows[a], &rows[b]);
                            let order = if *direction == SortOrder::Descending {
                                order.reverse()
                            } else {
                                order
                            };
                            (order != Ordering::Equal).then_some(order)
                        })
                        .unwrap_or(Ordering::Equal)
                });
            }
            match arrangement.get() {
                Some(arrange) => arrange(rows, kept),
                None => kept,
            }
        })
    })
}

fn matching_rows<R, C: PartialEq + Eq + std::hash::Hash>(
    rows: &[R],
    shown: &[ColumnDef<R, C>],
    query: &str,
    column_queries: &[(C, String)],
    typed: &HashMap<C, FilterValue>,
) -> Vec<usize> {
    let global_matchers: Vec<_> = shown
        .iter()
        .filter_map(|column| column.matches.as_ref())
        .collect();
    let column_matchers: Vec<_> = column_queries
        .iter()
        .filter_map(|(key, text)| {
            let text = text.trim();
            if text.is_empty() {
                return None;
            }
            let column = shown.iter().find(|column| &column.key == key)?;
            Some((column.matches.as_ref()?, text))
        })
        .collect();
    let typed_matchers: Vec<_> = shown
        .iter()
        .filter_map(|column| {
            let value = typed.get(&column.key).filter(|value| !value.is_empty())?;
            Some((column.matches_value.as_ref()?, value))
        })
        .collect();
    (0..rows.len())
        .filter(|&i| {
            let row = &rows[i];
            (query.is_empty() || global_matchers.iter().any(|matches| matches(row, query)))
                && column_matchers
                    .iter()
                    .all(|(matches, text)| matches(row, text))
                && typed_matchers
                    .iter()
                    .all(|(matches, value)| matches(row, value))
        })
        .collect()
}
