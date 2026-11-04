//! Role: Configure one shared model and table and compose the retained regions.
use super::rows::{self, Grid};
use super::types::{DataTable, DataTableContext, RowName};
use kivact_core::i18n::message;
use kivact_core::state::{IntoBinding, IntoValue};
use kivact_core::view::NodeBuilder;

use kivact_core::state::{Binding, binding, memo, signal};
use kivact_core::style::{Length, pct, style};
use kivact_core::view::{IntoView, Role, View, for_each, show};
use kivact_core::view::{column, scroll, text};
use kivact_interaction::{ScalarRange, Selection, SortOrder, use_data_table, use_data_table_model};
use std::collections::HashMap;
use std::rc::Rc;

/// The narrowest and widest a resized column may be, and the key step.
const MIN_WIDTH: f64 = 18.0;
const MAX_WIDTH: f64 = 2048.0;
const KEY_STEP: f64 = 10.0;

impl<R, K> IntoView for DataTable<R, K>
where
    R: kivact_core::state::PropValue,
    K: kivact_core::state::ItemKey,
{
    fn into_view(self) -> View {
        mount(self)
    }
}

fn mount<R, K>(table: DataTable<R, K>) -> View
where
    R: kivact_core::state::PropValue,
    K: kivact_core::state::ItemKey,
{
    let DataTable {
        root,
        rows: data,
        grouping,
        tree,
        tree_column,
        tree_indent,
        tree_expand_all,
        key,
        columns,
        column_order,
        selection,
        row_selectable,
        sort,
        multi_sort,
        page,
        has_more,
        page_sizes,
        row_index,
        filter,
        column_filters,
        typed_column_filters,
        filter_ui,
        sticky,
        widths,
        resize_range,
        row_label,
        actions,
        row_context_menu,
        reorder,
        toolbar,
        footer,
        empty,
        details,
        expanded_rows,
        cells,
        focus,
        row_height,
        row_status,
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
    let mut model = use_data_table_model(data, move |row| model_key(row), definitions);
    let tree = tree.map(|config| {
        let (key_for_order, parent, expanded) =
            (key.clone(), config.parent.clone(), config.expanded);
        model = model.arrange_rows(move |rows, order| {
            super::tree::arrange(rows, order, &key_for_order, &parent, expanded)
        });
        super::tree::Plan::new(
            data,
            key.clone(),
            config,
            tree_column,
            tree_indent,
            tree_expand_all,
        )
    });
    if let Some(query) = filter {
        model = model.filter(query.into_value());
    }
    if let Some(queries) = column_filters {
        model = model.column_filters(queries.into_value());
    }
    if let Some(filters) = typed_column_filters {
        model = model.typed_column_filters(filters);
    }
    let page_size = page.and_then(|(index, _)| {
        page_sizes.map(|(size, _)| {
            binding(size, move |next: usize| {
                size.set(next.max(1));
                index.set(0);
            })
        })
    });
    let page = page.map(|(index, size)| (index, page_size.map_or(size, |size| size.into_value())));
    if let Some((index, size)) = page
        && has_more.is_none()
    {
        model = model.page(index, size);
    }
    let page_rows = model.rows();
    let grouping_key = grouping.as_ref().map(|grouping| grouping.key.clone());
    let groups = grouping.map(|grouping| super::grouped::plan(page_rows, grouping));
    let display = groups.map_or(page_rows, |plan| plan.rows());
    let shown = model.columns();
    let shown = if let Some(order) = column_order {
        memo(move || {
            let ranks = order.with(|order| {
                let mut ranks = HashMap::new();
                for (rank, index) in order.iter().copied().enumerate() {
                    ranks.entry(index).or_insert(rank);
                }
                ranks
            });
            let mut shown = shown.get();
            shown.sort_by_key(|column| ranks.get(column.key()).copied().unwrap_or(usize::MAX));
            shown
        })
    } else {
        shown
    };
    let accessory_columns = [
        (row_status.is_some(), rows::STATUS, "Row status"),
        (row_index.is_some(), rows::INDEX, "#"),
        (
            selection.is_some() && matches!(focus, kivact_interaction::TableFocus::Cells(_)),
            rows::CHECK,
            "Select rows",
        ),
        (
            details.is_some() && expanded_rows.is_some(),
            rows::DETAILS,
            "Details",
        ),
    ];
    let action_column = actions.is_some();
    let navigation_columns = memo(move || {
        let mut navigation = Vec::new();
        for (present, key, label) in accessory_columns {
            if present {
                navigation.push(kivact_interaction::ColumnDef::new(key, label));
            }
        }
        navigation.extend(shown.get());
        if action_column {
            navigation.push(kivact_interaction::ColumnDef::new(rows::ACTIONS, "Actions"));
        }
        navigation
    });
    let column_metadata = super::metadata::columns(
        columns.len(),
        accessory_columns,
        action_column,
        column_order,
    );
    let row_metadata = super::metadata::rows(
        data,
        model.filtered_keys(),
        key.clone(),
        grouping_key,
        page,
        has_more.is_some(),
    );
    let table_key = key.clone();
    // Without the caller's selection, nothing is selected.
    let chosen = selection.unwrap_or_else(|| binding(Vec::new, |_| ()));
    let mut behavior = use_data_table(
        display,
        move |row| table_key(row),
        navigation_columns,
        Selection::Multiple(chosen),
        focus,
    );
    let eligible = row_selectable.as_ref().map(|selectable| {
        let selectable = selectable.clone();
        let key = key.clone();
        memo(move || {
            display.with(|rows| {
                rows.iter()
                    .filter(|row| selectable(row))
                    .map(|row| key(row))
                    .collect::<std::collections::HashSet<_>>()
            })
        })
    });
    if let Some(eligible) = eligible {
        behavior = behavior.row_selectable(move |key| eligible.with(|keys| keys.contains(key)));
    }
    let page_keys = if groups.is_some() {
        let key = key.clone();
        memo(move || display.with(|rows| rows.iter().map(|row| key(row)).collect::<Vec<_>>()))
    } else {
        model.keys()
    };
    let selectable_keys = memo(move || match eligible {
        Some(eligible) => page_keys.with(|keys| {
            eligible.with(|eligible| {
                keys.iter()
                    .filter(|key| eligible.contains(*key))
                    .cloned()
                    .collect::<Vec<_>>()
            })
        }),
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
        let sort = sort.unwrap_or_else(|| signal(None).into_binding());
        behavior = behavior.with_sort(binding(sort, move |next| {
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
        shown.with(|columns| {
            columns
                .iter()
                .map(|column| *column.key())
                .collect::<Vec<_>>()
        })
    });
    let context = DataTableContext {
        model,
        selection,
        sort,
        filter,
        column_filters,
        typed_column_filters,
        page,
        page_size,
        has_more,
    };
    let toolbar = match toolbar {
        Some(slot) => Some(slot(context)),
        None => super::chrome::toolbar(context, &variants),
    };
    let footer = match footer {
        Some(slot) => Some(slot(context)),
        None => super::chrome::footer(context, page_sizes.map(|(_, options)| options), &variants),
    };
    let sticky = sticky.map(|edges| Rc::new(rows::StickyColumns::new(edges, columns.len())));
    let indexed = row_index.map(|start| {
        let keys = model.keys();
        let pages = model.page_count();
        memo(move || {
            let offset = page.map_or(0, |(index, size)| {
                let index = if has_more.is_some() {
                    index.get()
                } else {
                    index.get().min(pages.get().saturating_sub(1))
                };
                index.saturating_mul(size.get().max(1))
            });
            keys.with(|keys| {
                keys.iter()
                    .cloned()
                    .enumerate()
                    .map(|(index, key)| (key, start.saturating_add(offset).saturating_add(index)))
                    .collect::<HashMap<_, _>>()
            })
        })
    });
    let grid = Grid {
        tree,
        behavior,
        row_metadata,
        column_metadata,
        filters: typed_column_filters.map(|values| super::filters::Filters {
            values,
            ui: filter_ui,
            page: page.map(|(index, _)| index),
        }),
        keys: selectable_keys,
        indexed,
        row_selectable,
        columns,
        visible,
        selection,
        sticky: sticky.clone(),
        widths,
        actions,
        row_context_menu,
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
        row_status,
        row_color,
        variants: variants.clone(),
    };
    let head = rows::header(&grid);
    let body = if let Some(groups) = groups {
        groups.view(grid, key)
    } else {
        for_each(
            display,
            move |row| key(row),
            move |item| {
                let id = item.with_untracked(|row| (grid.key)(row));
                rows::body(&grid, id, item)
            },
        )
    };
    let empty_view = show(
        move || page_rows.with(Vec::is_empty),
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
    let mut grid = column((head, column((body, empty_view)).role(Role::RowGroup)))
        .part(behavior.grid())
        .row_count(move || row_metadata.with(|metadata| metadata.count))
        .column_count(move || column_metadata.with(|metadata| metadata.0))
        .name(message("data_table.label", "Data table"))
        .style(
            style()
                .width(Length::MaxContent)
                .min_width(pct(100))
                .shrink(0),
        );
    if let Some(sticky) = &sticky {
        grid = grid.node_ref(sticky.content);
    }
    let mut scroller = scroll(grid)
        .axes(kivact_core::view::ScrollAxes::Horizontal)
        // The rows keep their height in a bounded parent: the caller's own
        // scroll view carries them.
        .style(style().shrink(0));
    if let Some(sticky) = &sticky {
        scroller = scroller.node_ref(sticky.viewport);
    }
    root.child(toolbar)
        .child(scroller)
        .child(footer)
        .into_view()
}

/// Share the widths of the columns, by position, with the caller's `sizes`,
/// and bound a resize handle's writes.
fn share_widths<K: kivact_core::state::ItemKey>(
    behavior: kivact_interaction::DataTableApi<K, usize>,
    sizes: Binding<Vec<f64>>,
    count: usize,
    range: ScalarRange,
) -> kivact_interaction::DataTableApi<K, usize> {
    let current = sizes.into_value();
    behavior
        .with_widths(binding(
            memo(move || {
                (0..count)
                    .filter_map(|i| {
                        current
                            .with(|widths| widths.get(i).copied())
                            .filter(|width| width.is_finite() && *width > 0.0)
                            .map(|width| (i, width))
                    })
                    .collect::<HashMap<_, _>>()
            }),
            move |values: HashMap<usize, f64>| {
                let mut next = current.get_untracked();
                next.resize(count, 0.0);
                for (i, width) in values {
                    if i < count {
                        next[i] = width;
                    }
                }
                sizes.set(next);
            },
        ))
        .resize_limits(range, KEY_STEP)
        .expect("invariant: positive finite column key step")
}
