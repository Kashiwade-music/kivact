//! Role: Configure one shared model and table and compose the retained regions.
use super::column_keys;
use super::rows::{self, Grid};
use super::types::{
    DataTable, DataTableContext, PagePlan, PageSetting, Paging, RowName, SortSetting,
};
use kivact_core::i18n::message;
use kivact_core::state::{IntoBinding, IntoValue};
use kivact_core::view::Element;

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

/// How the rows are paged once the offered page sizes are applied.
#[derive(Clone, Copy)]
enum PagingPlan {
    Off,
    /// The table slices its rows. `size` is the writable size the footer's
    /// picker shows: the `page_sizes` binding, which shows the first page
    /// when written, or else a read-only view of the size `page` gave.
    Local {
        page: PagePlan,
        size: Binding<usize>,
    },
    /// The caller's rows are the page.
    Remote {
        page: PagePlan,
        has_more: kivact_core::state::Value<bool>,
    },
}

impl PagingPlan {
    /// The page shown and its size, when the table pages its rows.
    fn page(self) -> Option<PagePlan> {
        match self {
            Self::Off => None,
            Self::Local { page, .. } | Self::Remote { page, .. } => Some(page),
        }
    }

    /// The paging the toolbar and footer slots see.
    fn public(self) -> Paging {
        match self {
            Self::Off => Paging::Off,
            Self::Local {
                page: (page, _),
                size,
            } => Paging::Local {
                page,
                page_size: size,
            },
            Self::Remote {
                page: (page, _),
                has_more,
            } => Paging::Remote { page, has_more },
        }
    }
}

/// The page's groups and the key that forms them; both exist exactly when
/// the caller called `DataTable::group_by`.
struct GroupingPlan<R: 'static> {
    groups: super::grouped::Plan<R>,
    key: super::types::Key<R, String>,
}

struct ModelPlan<R: 'static, K: 'static> {
    model: kivact_interaction::DataTableModelApi<R, K, usize>,
    // xtask-allow(state-fields): option: the tree arrangement made from DataTable::tree; absent keeps the model's rows flat in sorted order
    tree: Option<super::tree::Plan<K>>,
    paging: PagingPlan,
    // xtask-allow(state-fields): option: DataTable::group_by, as the page's groups and their key together; absent shows the page's rows ungrouped
    grouping: Option<GroupingPlan<R>>,
    display: kivact_core::state::Value<Vec<R>>,
    page_rows: kivact_core::state::Value<Vec<R>>,
    shown: kivact_core::state::Value<Vec<kivact_interaction::ColumnDef<R, usize>>>,
}
struct NavigationPlan<R: 'static, K: kivact_core::state::ItemKey> {
    behavior: kivact_interaction::DataTableApi<K, usize>,
    row_metadata: kivact_core::state::Value<super::metadata::Rows<K>>,
    column_metadata: kivact_core::state::Value<(Option<u32>, HashMap<usize, u32>)>,
    keys: kivact_core::state::Value<Vec<K>>,
    label: RowName<R>,
    sort: super::types::Sort,
    visible: kivact_core::state::Value<Vec<usize>>,
}
fn mount<R, K>(mut table: DataTable<R, K>) -> View
where
    R: kivact_core::state::PropValue,
    K: kivact_core::state::ItemKey,
{
    let plan = model_plan(&mut table);
    let navigation = navigation_plan(&table, &plan);
    let paging = plan.paging.public();
    let context = DataTableContext {
        model: plan.model,
        selection: table.selection,
        sort: navigation.sort,
        filter: table.filter,
        column_filters: table.column_filters,
        typed_column_filters: table.typed_column_filters,
        paging,
    };
    let toolbar = table.toolbar.take().map_or_else(
        || super::chrome::toolbar(context, &table.variants),
        |slot| Some(slot(context)),
    );
    let footer = table.footer.take().map_or_else(
        || {
            super::chrome::footer(
                context,
                table.page_sizes.map(|(_, options)| options),
                &table.variants,
            )
        },
        |slot| Some(slot(context)),
    );
    let grid = grid_plan(&mut table, &plan, navigation);
    regions(
        crate::layer::with_base_recipe_style(
            table.root,
            |r| r.data_table_recipes().frame_recipe(),
            &table.variants,
            kivact_core::custom::PartState::default,
        ),
        toolbar,
        footer,
        grid,
        plan.grouping.as_ref().map(|grouping| grouping.groups),
        plan.page_rows,
        table.empty,
    )
}
fn model_plan<R, K>(table: &mut DataTable<R, K>) -> ModelPlan<R, K>
where
    R: kivact_core::state::PropValue,
    K: kivact_core::state::ItemKey,
{
    let (data, key, columns) = (table.rows, table.key.clone(), table.columns.clone());
    // A reorder keeps the caller's order, which a sort the table cannot
    // clear would break.
    let sortable = !matches!(table.sort, SortSetting::Own) || table.reorder.is_none();
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
    let tree = table.tree.take().map(|config| {
        let (key_for_order, parent, expanded) =
            (key.clone(), config.parent.clone(), config.expanded);
        model = model.arrange_rows(move |rows, order| {
            super::tree::arrange(rows, order, &key_for_order, &parent, expanded)
        });
        super::tree::Plan::new(
            data,
            key.clone(),
            config,
            table.tree_column,
            table.tree_indent,
            table.tree_expand_all,
        )
    });
    if let Some(query) = table.filter {
        model = model.filter(query.into_value());
    }
    if let Some(queries) = table.column_filters {
        model = model.column_filters(queries.into_value());
    }
    if let Some(filters) = table.typed_column_filters {
        model = model.typed_column_filters(filters);
    }
    let paging = paging(table);
    if let PagingPlan::Local {
        page: (index, size),
        ..
    } = paging
    {
        model = model.page(index, size);
    }
    let page_rows = model.rows();
    let grouping = table.grouping.take().map(|grouping| GroupingPlan {
        key: grouping.key.clone(),
        groups: super::grouped::plan(page_rows, grouping),
    });
    let display = grouping
        .as_ref()
        .map_or(page_rows, |grouping| grouping.groups.rows());
    let shown = ordered_columns(model.columns(), table.column_order);
    ModelPlan {
        model,
        tree,
        paging,
        grouping,
        display,
        page_rows,
        shown,
    }
}
/// The table's paging, with the size chosen from `page_sizes` in place of
/// the size `page` or `remote_page` gave.
fn paging<R, K>(table: &DataTable<R, K>) -> PagingPlan
where
    R: kivact_core::state::PropValue,
    K: kivact_core::state::ItemKey,
{
    let chosen = table.page_sizes.map(|(size, _)| size);
    match table.paging {
        PageSetting::Off => PagingPlan::Off,
        PageSetting::Local((index, size)) => {
            let picked = chosen.map(|chosen| {
                binding(chosen, move |next: usize| {
                    chosen.set(next.max(1));
                    index.set(0);
                })
            });
            PagingPlan::Local {
                page: (index, picked.map_or(size, |picked| picked.into_value())),
                size: picked.unwrap_or_else(|| binding(move || size.get(), |_| ())),
            }
        }
        PageSetting::Remote {
            page: (index, size),
            has_more,
        } => PagingPlan::Remote {
            page: (index, chosen.map_or(size, |chosen| chosen.into_value())),
            has_more,
        },
    }
}
fn ordered_columns<R: kivact_core::state::PropValue>(
    shown: kivact_core::state::Value<Vec<kivact_interaction::ColumnDef<R, usize>>>,
    order: Option<kivact_core::state::Value<Vec<usize>>>,
) -> kivact_core::state::Value<Vec<kivact_interaction::ColumnDef<R, usize>>> {
    if let Some(order) = order {
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
    }
}
fn navigation_columns<R: kivact_core::state::PropValue>(
    shown: kivact_core::state::Value<Vec<kivact_interaction::ColumnDef<R, usize>>>,
    accessory_columns: [(bool, usize, &'static str); 4],
    action_column: bool,
) -> kivact_core::state::Value<Vec<kivact_interaction::ColumnDef<R, usize>>> {
    memo(move || {
        let mut navigation = Vec::new();
        for (present, key, label) in accessory_columns {
            if present {
                navigation.push(kivact_interaction::ColumnDef::new(key, label));
            }
        }
        navigation.extend(shown.get());
        if action_column {
            navigation.push(kivact_interaction::ColumnDef::new(
                column_keys::ACTIONS,
                "Actions",
            ));
        }
        navigation
    })
}
fn navigation_plan<R, K>(table: &DataTable<R, K>, plan: &ModelPlan<R, K>) -> NavigationPlan<R, K>
where
    R: kivact_core::state::PropValue,
    K: kivact_core::state::ItemKey,
{
    let accessory_columns = [
        (
            table.row_status.is_some(),
            column_keys::STATUS,
            "Row status",
        ),
        (table.row_index.is_some(), column_keys::INDEX, "#"),
        (
            table.selection.is_some()
                && matches!(table.focus, kivact_interaction::TableFocus::Cells(_)),
            column_keys::CHECK,
            "Select rows",
        ),
        (
            table.details.is_some() && table.expanded_rows.is_some(),
            column_keys::DETAILS,
            "Details",
        ),
    ];
    let action_column = table.actions.is_some();
    let navigation_columns = navigation_columns(plan.shown, accessory_columns, action_column);
    let column_metadata = super::metadata::columns(
        table.columns.len(),
        accessory_columns,
        action_column,
        table.column_order,
    );
    let row_metadata = super::metadata::rows(
        table.rows,
        plan.model.filtered_keys(),
        table.key.clone(),
        plan.grouping.as_ref().map(|grouping| grouping.key.clone()),
        plan.paging.page(),
        matches!(plan.paging, PagingPlan::Remote { .. }),
    );
    let table_key = table.key.clone();
    // Without the caller's table.selection, nothing is selected.
    let chosen = table.selection.unwrap_or_else(|| binding(Vec::new, |_| ()));
    let behavior = use_data_table(
        plan.display,
        move |row| table_key(row),
        navigation_columns,
        Selection::Multiple(chosen),
        table.focus,
    );
    let (behavior, keys) = selection_policy(table, plan, behavior);
    let (behavior, label) = row_labels(table, behavior);
    let (mut behavior, sort) = sorting(table, plan.model, plan.paging.page(), behavior);
    if let Some(sizes) = table.widths {
        let range = table.resize_range.unwrap_or_else(|| {
            ScalarRange::new(MIN_WIDTH, MAX_WIDTH).expect("invariant: finite column bounds")
        });
        behavior = share_widths(behavior, sizes, table.columns.len(), range);
    }
    let shown = plan.shown;
    let visible = memo(move || {
        shown.with(|columns| {
            columns
                .iter()
                .map(|column| *column.key())
                .collect::<Vec<_>>()
        })
    });
    NavigationPlan {
        behavior,
        row_metadata,
        column_metadata,
        keys,
        label,
        sort,
        visible,
    }
}
fn selection_policy<R, K>(
    table: &DataTable<R, K>,
    plan: &ModelPlan<R, K>,
    mut behavior: kivact_interaction::DataTableApi<K, usize>,
) -> (
    kivact_interaction::DataTableApi<K, usize>,
    kivact_core::state::Value<Vec<K>>,
)
where
    R: kivact_core::state::PropValue,
    K: kivact_core::state::ItemKey,
{
    let display = plan.display;
    let eligible = table.row_selectable.as_ref().map(|selectable| {
        let selectable = selectable.clone();
        let key = table.key.clone();
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
    let page_keys = if plan.grouping.is_some() {
        let key = table.key.clone();
        memo(move || display.with(|rows| rows.iter().map(|row| key(row)).collect::<Vec<_>>()))
    } else {
        plan.model.keys()
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
    (behavior, selectable_keys)
}
fn row_labels<R, K>(
    table: &DataTable<R, K>,
    mut behavior: kivact_interaction::DataTableApi<K, usize>,
) -> (kivact_interaction::DataTableApi<K, usize>, RowName<R>)
where
    R: kivact_core::state::PropValue,
    K: kivact_core::state::ItemKey,
{
    // A row is named by the caller's label, else by its first column.
    let label: RowName<R> = match table.row_label.clone() {
        Some(label) => label,
        None => {
            let first = table.columns.first().cloned();
            Rc::new(move |row: &R| first.as_ref().map(|c| c.text_of(row)).unwrap_or_default())
        }
    };
    let (label_key, row_name) = (table.key.clone(), label.clone());
    let data = table.rows;
    behavior = behavior.row_label(move |wanted: &K| {
        data.with(|rows| {
            rows.iter()
                .find(|row| label_key(row) == *wanted)
                .map(|row| row_name(row))
                .unwrap_or_default()
        })
    });
    (behavior, label)
}
fn sorting<R, K>(
    table: &DataTable<R, K>,
    model: kivact_interaction::DataTableModelApi<R, K, usize>,
    page: Option<super::types::PagePlan>,
    mut behavior: kivact_interaction::DataTableApi<K, usize>,
) -> (
    kivact_interaction::DataTableApi<K, usize>,
    super::types::Sort,
)
where
    R: kivact_core::state::PropValue,
    K: kivact_core::state::ItemKey,
{
    // The model sorts by the table's sort: the caller's, or the table's own,
    // which the headers write. A new sort shows the first page.
    let single = match table.sort {
        SortSetting::Own => signal(None).into_binding(),
        SortSetting::Single(sort) => sort,
        SortSetting::Multi(multi_sort) => {
            return multi_sorting(multi_sort, model, page, behavior);
        }
    };
    behavior = behavior.with_sort(binding(single, move |next| {
        single.set(next);
        if let Some((index, _)) = page {
            index.set(0);
        }
    }));
    model.sort(single);
    (behavior, single)
}
/// Sort by the caller's ordered keys; the toolbar, footer and row
/// reordering see the first key.
fn multi_sorting<R, K>(
    multi_sort: super::types::MultiSort,
    model: kivact_interaction::DataTableModelApi<R, K, usize>,
    page: Option<PagePlan>,
    mut behavior: kivact_interaction::DataTableApi<K, usize>,
) -> (
    kivact_interaction::DataTableApi<K, usize>,
    super::types::Sort,
)
where
    R: kivact_core::state::PropValue,
    K: kivact_core::state::ItemKey,
{
    let multi_sort = binding(multi_sort, move |next| {
        multi_sort.set(next);
        if let Some((index, _)) = page {
            index.set(0);
        }
    });
    behavior = behavior.multi_sort(multi_sort);
    model.multi_sort(multi_sort.into_value());
    let primary = binding(
        move || multi_sort.with(|keys| keys.first().copied()),
        move |next: Option<(usize, SortOrder)>| multi_sort.set(next.into_iter().collect()),
    );
    (behavior, primary)
}
fn row_indexes<R, K>(
    table: &DataTable<R, K>,
    plan: &ModelPlan<R, K>,
) -> Option<kivact_core::state::Value<HashMap<K, usize>>>
where
    R: kivact_core::state::PropValue,
    K: kivact_core::state::ItemKey,
{
    let paging = plan.paging;
    table.row_index.map(|start| {
        let keys = plan.model.keys();
        let pages = plan.model.page_count();
        memo(move || {
            // A local page past the last one shows the last page.
            let offset = match paging {
                PagingPlan::Off => 0,
                PagingPlan::Local {
                    page: (index, size),
                    ..
                } => index
                    .get()
                    .min(pages.get().saturating_sub(1))
                    .saturating_mul(size.get().max(1)),
                PagingPlan::Remote {
                    page: (index, size),
                    ..
                } => index.get().saturating_mul(size.get().max(1)),
            };
            keys.with(|keys| {
                keys.iter()
                    .cloned()
                    .enumerate()
                    .map(|(index, key)| (key, start.saturating_add(offset).saturating_add(index)))
                    .collect::<HashMap<_, _>>()
            })
        })
    })
}
fn grid_plan<R, K>(
    table: &mut DataTable<R, K>,
    plan: &ModelPlan<R, K>,
    navigation: NavigationPlan<R, K>,
) -> Grid<R, K>
where
    R: kivact_core::state::PropValue,
    K: kivact_core::state::ItemKey,
{
    let sticky = table
        .sticky
        .take()
        .map(|edges| Rc::new(rows::StickyColumns::new(edges, table.columns.len())));
    Grid {
        tree: plan.tree,
        behavior: navigation.behavior,
        row_metadata: navigation.row_metadata,
        column_metadata: navigation.column_metadata,
        navigation: super::navigation::NavigationCache::new(),
        filters: table
            .typed_column_filters
            .map(|values| super::filters::Filters {
                values,
                ui: table.filter_ui,
                page: plan.paging.page().map(|(index, _)| index),
            }),
        keys: navigation.keys,
        indexed: row_indexes(table, plan),
        row_selectable: table.row_selectable.take(),
        columns: table.columns.clone(),
        visible: navigation.visible,
        selection: table.selection,
        sticky: sticky.clone(),
        widths: table.widths,
        actions: table.actions.take(),
        row_context_menu: table.row_context_menu.take(),
        row_label: navigation.label,
        cells: table.cells.take(),
        details: table.details.take(),
        expanded_rows: table.expanded_rows,
        reorder: table.reorder,
        source: table.rows,
        key: table.key.clone(),
        sort: navigation.sort,
        focus: table.focus,
        row_height: table.row_height,
        row_status: table.row_status.take(),
        row_color: table.row_color.take(),
        variants: table.variants.clone(),
    }
}
fn regions<R, K>(
    root: kivact_core::view::Node,
    toolbar: Option<View>,
    footer: Option<View>,
    grid: Grid<R, K>,
    groups: Option<super::grouped::Plan<R>>,
    page_rows: kivact_core::state::Value<Vec<R>>,
    empty: Option<Box<dyn Fn() -> View>>,
) -> View
where
    R: kivact_core::state::PropValue,
    K: kivact_core::state::ItemKey,
{
    let display = if let Some(groups) = groups {
        groups.rows()
    } else {
        page_rows
    };
    let head = rows::header(&grid);
    let key = grid.key.clone();
    let behavior = grid.behavior;
    let row_metadata = grid.row_metadata;
    let column_metadata = grid.column_metadata;
    let sticky = grid.sticky.clone();
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
