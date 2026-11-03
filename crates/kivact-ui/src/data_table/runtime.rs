//! Role: Configure one shared model and table and compose the retained regions.
use super::rows::{self, Grid};
use super::types::{DataTable, DataTableContext, RowName};
use kivact_core::control::{NodeBuilder, column, scroll, text};
use kivact_core::i18n::message;
use kivact_core::style::{Length, pct, style};
use kivact_core::view::{IntoValue, IntoView, Role, View, for_each, show};
use kivact_core::{Binding, binding, memo, signal};
use kivact_interaction::{ListSelection, ScalarRange, SortOrder, use_table, use_table_model};
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

/// The narrowest and widest a resized column may be, and the key step.
const MIN_WIDTH: f64 = 18.0;
const MAX_WIDTH: f64 = 2048.0;
const KEY_STEP: f64 = 10.0;

impl<R, K> IntoView for DataTable<R, K>
where
    R: Clone + PartialEq + 'static,
    K: Clone + Eq + Hash + 'static,
{
    fn into_view(self) -> View {
        mount(self)
    }
}

fn mount<R, K>(table: DataTable<R, K>) -> View
where
    R: Clone + PartialEq + 'static,
    K: Clone + Eq + Hash + 'static,
{
    let DataTable {
        root,
        rows: data,
        key,
        columns,
        selection,
        row_selectable,
        sort,
        multi_sort,
        page,
        filter,
        column_filters,
        widths,
        resize_range,
        row_label,
        actions,
        reorder,
        toolbar,
        footer,
        empty,
        details,
        expanded_rows,
        cells,
        focus,
        row_height,
        row_color,
        variants,
    } = table;
    // A reorder keeps the caller's order, which a sort the table cannot
    // clear would break.
    let sortable = sort.is_some() || multi_sort.is_some() || reorder.is_none();
    let definitions = columns
        .iter()
        .enumerate()
        .map(|(index, column)| {
            let definition = column.definition(index);
            if sortable {
                definition
            } else {
                definition.without_ordering()
            }
        })
        .collect();
    let model_key = key.clone();
    let mut model = use_table_model(data, move |row| model_key(row), definitions);
    if let Some(query) = filter {
        model = model.filter(query.into_value());
    }
    if let Some(queries) = column_filters {
        model = model.column_filters(queries.into_value());
    }
    if let Some((index, size)) = page {
        model = model.page(index, size);
    }
    let display = model.rows();
    let table_key = key.clone();
    // Without the caller's selection, nothing is selected.
    let chosen = selection.unwrap_or_else(|| binding(Vec::new, |_| ()));
    let mut behavior = use_table(
        display,
        move |row| table_key(row),
        model.columns(),
        ListSelection::Multiple(chosen),
        focus,
    );
    let eligible = row_selectable.as_ref().map(|selectable| {
        let selectable = selectable.clone();
        let key = key.clone();
        memo(move || display.with(|rows| {
            rows.iter().filter(|row| selectable(row)).map(|row| key(row)).collect::<std::collections::HashSet<_>>()
        }))
    });
    if let Some(eligible) = eligible {
        behavior = behavior.row_selectable(move |key| eligible.with(|keys| keys.contains(key)));
    }
    let page_keys = model.keys();
    let selectable_keys = memo(move || match eligible {
        Some(eligible) => page_keys.with(|keys| eligible.with(|eligible| {
            keys.iter().filter(|key| eligible.contains(*key)).cloned().collect::<Vec<_>>()
        })),
        None => page_keys.get(),
    });
    // A row is named by the caller's label, else by its first column.
    let label: RowName<R> = match row_label {
        Some(label) => label,
        None => {
            let first = columns.first().cloned();
            Rc::new(move |row: &R| first.as_ref().map(|c| c.text_of(row)).unwrap_or_default())
        }
    };
    let (label_key, row_name) = (key.clone(), label.clone());
    behavior = behavior.row_label(move |wanted: &K| {
        data.with(|rows| {
            rows.iter()
                .find(|row| label_key(row) == *wanted)
                .map(|row| row_name(row))
                .unwrap_or_default()
        })
    });
    // The model sorts by the table's sort: the caller's, or the table's own,
    // which the headers write. A new sort shows the first page.
    let sort: Binding<Option<(usize, SortOrder)>> = if let Some(multi_sort) = multi_sort {
        let multi_sort = binding(multi_sort, move |next| {
            multi_sort.set(next);
            if let Some((index, _)) = page {
                index.set(0);
            }
        });
        behavior = behavior.multi_sort(multi_sort);
        model.multi_sort(multi_sort.into_value());
        // Existing toolbar/footer and row reordering use the primary sort.
        binding(
            move || multi_sort.with(|keys| keys.first().copied()),
            move |next: Option<(usize, SortOrder)>| multi_sort.set(next.into_iter().collect()),
        )
    } else {
        let sort = sort.unwrap_or_else(|| signal(None).into());
        behavior = behavior.sort(binding(sort, move |next| {
            sort.set(next);
            if let Some((index, _)) = page {
                index.set(0);
            }
        }));
        model.sort(sort);
        sort
    };
    if let Some(sizes) = widths {
        let range = resize_range.unwrap_or_else(|| {
            ScalarRange::new(MIN_WIDTH, MAX_WIDTH).expect("invariant: finite column bounds")
        });
        behavior = share_widths(behavior, sizes, columns.len(), range);
    }
    let visible = memo(move || {
        model
            .columns()
            .with(|shown| shown.iter().map(|column| *column.key()).collect::<Vec<_>>())
    });
    let context = DataTableContext {
        model,
        selection,
        sort,
        filter,
        column_filters,
        page,
    };
    let toolbar = match toolbar {
        Some(slot) => Some(slot(context)),
        None => super::chrome::toolbar(context, &variants),
    };
    let footer = match footer {
        Some(slot) => Some(slot(context)),
        None => super::chrome::footer(context, &variants),
    };
    let grid = Grid {
        behavior,
        keys: selectable_keys,
        row_selectable,
        columns,
        visible,
        selection,
        widths,
        actions,
        row_label: label,
        cells,
        details,
        expanded_rows,
        reorder,
        source: data,
        key: key.clone(),
        sort,
        focus,
        row_height,
        row_color,
        variants: variants.clone(),
    };
    let head = rows::header(&grid);
    let body = for_each(
        display,
        move |row| key(row),
        move |id, item| rows::body(&grid, id, item),
    );
    let empty_view = show(
        move || display.with(Vec::is_empty),
        move || {
            empty.as_ref().map_or_else(
                || {
                    text(message("data_table.empty", "No results found."))
                        .style(
                            style()
                                .padding(24)
                                .color(kivact_core::style::ColorToken::TextMuted),
                        )
                        .into_view()
                },
                |empty| empty(),
            )
        },
    );
    let grid = column((head, column((body, empty_view)).role(Role::RowGroup)))
        .part(behavior.grid())
        .name(message("data_table.label", "Data table"))
        .style(
            style()
                .width(Length::MaxContent)
                .min_width(pct(100))
                .shrink(0),
        );
    root.child(toolbar)
        .child(
            scroll(grid)
                .axes(kivact_core::control::ScrollAxes::Horizontal)
                // The rows keep their height in a bounded parent: the
                // caller's own scroll view carries them.
                .style(style().shrink(0)),
        )
        .child(footer)
        .into_view()
}

/// Share the widths of the columns, by position, with the caller's `sizes`,
/// and bound a resize handle's writes.
fn share_widths<K: Clone + Eq + Hash + 'static>(
    behavior: kivact_interaction::Table<K, usize>,
    sizes: Binding<Vec<f32>>,
    count: usize,
    range: ScalarRange,
) -> kivact_interaction::Table<K, usize> {
    let current = sizes.into_value();
    behavior
        .widths(binding(
            memo(move || {
                (0..count)
                    .filter_map(|i| {
                        current
                            .with(|widths| widths.get(i).copied())
                            .filter(|width| width.is_finite() && *width > 0.0)
                            .map(|width| (i, f64::from(width)))
                    })
                    .collect::<HashMap<_, _>>()
            }),
            move |values: HashMap<usize, f64>| {
                let mut next = current.get_untracked();
                next.resize(count, 0.0);
                for (i, width) in values {
                    if i < count {
                        next[i] = width as f32;
                    }
                }
                sizes.set(next);
            },
        ))
        .resize_limits(range, KEY_STEP)
        .expect("invariant: positive finite column key step")
}
