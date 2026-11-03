//! Role: A headless row model: filter, sort and page the caller's rows by column.

use super::data::{Header, SortOrder};
use kivact_core::view::{IntoText, IntoValue, Reactive};
use kivact_core::{Binding, Signal, Value, Var, memo, signal, var};
use std::cmp::Ordering;
use std::hash::Hash;
use std::rc::Rc;

type Compare<R> = Rc<dyn Fn(&R, &R) -> Ordering>;
type Matches<R> = Rc<dyn Fn(&R, &str) -> bool>;
type KeyOf<R, K> = Rc<dyn Fn(&R) -> K>;
/// The caller's sort column and direction.
type Sort<C> = Value<Vec<(C, SortOrder)>>;
type ColumnQueries<C> = Value<Vec<(C, String)>>;
type RowArrangement<R> = Rc<dyn Fn(&[R], Vec<usize>) -> Vec<usize>>;

/// One column of a table: its key, label and starting width, how its rows
/// compare and match a filter (TanStack Table's column definition), and
/// whether it is shown.
///
/// This is the only column type: a [`TableModel`] sorts, filters and hides
/// by it, and [`use_table`](crate::use_table) reads the keys, labels, widths
/// and whether a header sorts from it. A column is sortable exactly when it
/// has a [`compare`](Self::compare); a column without
/// [`filter`](Self::filter) does not take part in filtering.
pub struct ColumnDef<R, C> {
    key: C,
    label: Reactive<String>,
    width: Option<f64>,
    pub(super) min_width: Option<f64>,
    compare: Option<Compare<R>>,
    matches: Option<Matches<R>>,
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

    /// Show the column while true (a constant or a reactive value). A hidden
    /// column is left out of [`TableModel::columns`] and of filtering.
    pub fn visible(mut self, visible: impl IntoValue<bool>) -> Self {
        self.visible = Reactive::new(visible);
        self
    }

    /// The starting width in the caller's unit; `None` leaves it to layout.
    pub fn width(mut self, width: impl Into<Option<f64>>) -> Self {
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

    /// The label's current text; tracked when read inside a computation.
    #[track_caller]
    pub fn label(&self) -> String {
        self.label.get()
    }

    /// The label as a [`Value`] of the current owner. Setup only.
    #[track_caller]
    pub fn label_value(&self) -> Value<String> {
        self.label.clone().into_value()
    }

    /// Whether a header activation sorts by the column (it has a
    /// [`compare`](Self::compare)).
    pub fn is_sortable(&self) -> bool {
        self.compare.is_some()
    }

    /// Whether the column takes part in filtering ([`filter`](Self::filter)).
    pub fn is_filtered(&self) -> bool {
        self.matches.is_some()
    }

    /// The starting width ([`width`](Self::width)).
    pub fn width_value(&self) -> Option<f64> {
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
    }
}

/// The page a model shows: its number (from 0) and its size in rows.
#[derive(Clone, Copy)]
struct Paging {
    index: Binding<usize>,
    size: Value<usize>,
}

/// A Copy handle over the caller's rows that filters, sorts and pages them,
/// the headless row model of TanStack Table.
///
/// The model keeps no rows of its own: it reads the caller's rows, the sort
/// column and direction, the filter text and the page, and derives the rows
/// to show. It draws nothing and has no keys or focus; give its
/// [`rows`](Self::rows) and [`columns`](Self::columns) to
/// [`use_table`](crate::use_table) for the grid's keyboard and selection, and
/// give both the same sort binding, so that activating a header sorts the
/// model:
///
/// ```no_run
/// # mod kivact {
/// #     //! Role: Supply facade paths for this documentation example.
/// #     pub use kivact_core::{control, prelude, view};
/// #     pub extern crate kivact_interaction as interaction;
/// # }
/// use kivact::interaction::{ColumnDef, ListSelection, TableFocus, use_table, use_table_model};
/// use kivact::prelude::*;
/// use kivact::interaction::SortOrder;
/// #[derive(Clone, PartialEq)]
/// struct Person { id: u32, name: String, age: u32 }
/// let people = signal(vec![Person { id: 1, name: "Ada".into(), age: 36 }]);
/// let sort = signal(Some(("name", SortOrder::Ascending)));
/// let query = signal(String::new());
/// let page = signal(0_usize);
/// let model = use_table_model(people, |p: &Person| p.id, vec![
///     ColumnDef::new("name", "Name")
///         .compare(|a: &Person, b: &Person| a.name.cmp(&b.name))
///         .filter(|p: &Person, q: &str| p.name.to_lowercase().contains(&q.to_lowercase())),
///     ColumnDef::new("age", "Age").compare(|a: &Person, b: &Person| a.age.cmp(&b.age)),
/// ])
/// .sort(sort)
/// .filter(query)
/// .page(page, 20_usize);
/// let selected = signal(Vec::<u32>::new());
/// let table = use_table(model.rows(), |p: &Person| p.id, model.columns(),
///     ListSelection::Multiple(selected.into()), TableFocus::default())
///     .sort(sort);
/// let status = text(move || format!("{} people, page {} of {}",
///     model.row_count().get(), page.get() + 1, model.page_count().get()));
/// # let _ = (table, status);
/// ```
pub struct TableModel<R: 'static, K: 'static, C: 'static> {
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
    sort: Signal<Option<Sort<C>>>,
    query: Signal<Option<Value<String>>>,
    column_queries: Signal<Option<ColumnQueries<C>>>,
    arrangement: Signal<Option<RowArrangement<R>>>,
    paging: Signal<Option<Paging>>,
}

impl<R: 'static, K: 'static, C: 'static> Clone for TableModel<R, K, C> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<R: 'static, K: 'static, C: 'static> Copy for TableModel<R, K, C> {}

/// Create a row model over `rows`, identified by `key`, with `columns` in
/// display order. Without [`sort`](TableModel::sort),
/// [`filter`](TableModel::filter) and [`page`](TableModel::page) it shows
/// every row in the caller's order. Setup only.
#[track_caller]
pub fn use_table_model<R, K, C>(
    rows: impl IntoValue<Vec<R>>,
    key: impl Fn(&R) -> K + 'static,
    columns: Vec<ColumnDef<R, C>>,
) -> TableModel<R, K, C>
where
    R: Clone + PartialEq + 'static,
    K: Clone + PartialEq + 'static,
    C: Clone + Eq + Hash + 'static,
{
    let rows = rows.into_value();
    let key: KeyOf<R, K> = Rc::new(key);
    let key_of = key.clone();
    let columns: Rc<[ColumnDef<R, C>]> = columns.into();
    let sort = signal(None::<Sort<C>>);
    let query = signal(None::<Value<String>>);
    let column_queries = signal(None::<Value<Vec<(C, String)>>>);
    let arrangement = signal(None::<RowArrangement<R>>);
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
    let ordered = {
        let (columns, shown) = (columns.clone(), shown.clone());
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
            let shown = if query.is_empty() && column_queries.is_empty() {
                Vec::new()
            } else {
                shown()
            };
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
            rows.with(|rows| {
                let mut kept: Vec<usize> = (0..rows.len())
                    .filter(|&i| {
                        let row = &rows[i];
                        (query.is_empty()
                            || global_matchers.iter().any(|matches| matches(row, &query)))
                            && column_matchers
                                .iter()
                                .all(|(matches, text)| matches(row, text))
                    })
                    .collect();
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
    };
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
    TableModel {
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
        paging,
    }
}

impl<R: Clone + PartialEq + 'static, K: Clone + PartialEq + 'static, C: Clone + Eq + Hash + 'static>
    TableModel<R, K, C>
{
    /// Sort by the column and direction this binding holds; `None` or a
    /// column without a compare function keeps the caller's order. Give the same binding to
    /// [`Table::sort`](crate::Table::sort). Setup only.
    #[track_caller]
    pub fn sort(self, sort: impl Into<Binding<Option<(C, SortOrder)>>>) -> Self {
        let sort = sort.into();
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

    /// Show the page `index` (from 0) of `size` rows (at least 1). The model
    /// only reads `index`: when the rows shrink (or have not arrived yet) so
    /// that `index` is past the last page, it shows the last page and leaves
    /// the binding as it is, so the page number comes back when the rows do.
    /// Setup only.
    #[track_caller]
    pub fn page(self, index: impl Into<Binding<usize>>, size: impl IntoValue<usize>) -> Self {
        let paging = Paging {
            index: index.into(),
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
    /// (TanStack Table's filtered row model): what a summary such as
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

    /// The shown column definitions, for [`use_table`](crate::use_table).
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
