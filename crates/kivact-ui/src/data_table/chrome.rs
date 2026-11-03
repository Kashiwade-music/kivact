//! Role: The default regions of a data table: filter field, summary, pager, check boxes.
use super::types::DataTableContext;
use crate::layer::with_recipe_style;
use crate::variants::Variants;
use crate::{
    ButtonVariant, ControlSize, DataTableCheckState, DataTablePageState, ListItem, button,
    checkbox, select, text_input,
};
use kivact_core::control::{NodeBuilder, node, row, text};
use kivact_core::i18n::{message, message_with};
use kivact_core::primitive::PartState;
use kivact_core::style::{pct, style};
use kivact_core::view::CheckedState;
use kivact_core::view::{IntoView, Role, View};
use kivact_core::{Binding, Value, binding, memo};
use std::hash::Hash;

/// The row above the grid: the filter field, when the table has a filter.
pub(super) fn toolbar<R: Clone + PartialEq + 'static, K: Clone + Eq + Hash + 'static>(
    context: DataTableContext<R, K>,
    variants: &Variants,
) -> Option<View> {
    let query = context.filter?;
    let page = context.page.map(|(index, _)| index);
    let edited = binding(query, move |text: String| {
        query.set(text);
        if let Some(page) = page {
            page.set(0);
        }
    });
    let label = message("data_table.filter", "Filter…");
    let field = text_input(label, edited)
        .placeholder(label)
        .style(style().max_width(384).width(pct(100)));
    Some(
        with_recipe_style(
            row(field),
            |r| r.data_table_recipes().toolbar_recipe(),
            variants,
            PartState::default,
        )
        .into_view(),
    )
}

/// The row under the grid: how many of the filtered rows are selected and
/// the pager; nothing when the table has neither.
pub(super) fn footer<R: Clone + PartialEq + 'static, K: Clone + Eq + Hash + 'static>(
    context: DataTableContext<R, K>,
    sizes: Option<Value<Vec<usize>>>,
    variants: &Variants,
) -> Option<View> {
    if context.selection.is_none() && context.page.is_none() {
        return None;
    }
    let summary = context.selection.map(|selection| {
        let filtered = context.model.filtered_keys();
        let text = message_with(
            "data_table.selected",
            "{selected} of {total} row(s) selected.",
            move || {
                let total = filtered.with(Vec::len);
                let chosen = filtered.with(|keys| {
                    selection.with(|chosen| keys.iter().filter(|k| chosen.contains(k)).count())
                });
                [
                    ("selected", chosen.to_string()),
                    ("total", total.to_string()),
                ]
            },
        );
        kivact_core::control::text(text).style(style().grow(1))
    });
    let pager = context
        .page
        .map(|(index, _)| pager(context, index, variants));
    let size_picker = context.page_size.zip(sizes).map(|(size, sizes)| {
        let options = move || {
            sizes.with(|sizes| {
                let mut seen = std::collections::HashSet::new();
                sizes
                    .iter()
                    .copied()
                    .filter(|size| *size > 0 && seen.insert(*size))
                    .map(|size| ListItem::new(size, size.to_string()))
                    .collect::<Vec<_>>()
            })
        };
        select(
            message("data_table.page_size", "Rows per page"),
            options,
            binding(
                move || Some(size.get().max(1)),
                move |next: Option<usize>| {
                    if let Some(next) = next {
                        size.set(next);
                    }
                },
            ),
        )
    });
    let spacer = summary.is_none().then(|| node().style(style().grow(1)));
    Some(
        with_recipe_style(
            row((summary, spacer, size_picker, pager)),
            |r| r.data_table_recipes().footer_recipe(),
            variants,
            PartState::default,
        )
        .into_view(),
    )
}

/// The page shown, Previous and Next.
fn pager<R: Clone + PartialEq + 'static, K: Clone + Eq + Hash + 'static>(
    context: DataTableContext<R, K>,
    index: Binding<usize>,
    variants: &Variants,
) -> View {
    let pages = context.model.page_count();
    // A page past the last one shows the last page and leaves the binding.
    let has_more = context.has_more;
    let shown = memo(move || {
        if has_more.is_some() {
            index.get()
        } else {
            index.get().min(pages.get().saturating_sub(1))
        }
    });
    let can_advance = move || {
        has_more.map_or_else(
            || shown.get().saturating_add(1) < pages.get(),
            |has_more| has_more.get(),
        )
    };
    let state = move || DataTablePageState {
        index: shown.get(),
        pages: if has_more.is_some() { 0 } else { pages.get() },
    };
    let label = if has_more.is_some() {
        message_with("data_table.page_unknown_total", "Page {page}", move || {
            [("page", shown.get().saturating_add(1).to_string())]
        })
    } else {
        message_with("data_table.page", "Page {page} of {pages}", move || {
            [
                ("page", shown.get().saturating_add(1).to_string()),
                ("pages", pages.get().to_string()),
            ]
        })
    };
    with_recipe_style(
        row((
            text(label),
            button(message("data_table.previous", "Previous"))
                .variant(ButtonVariant::Outline)
                .size(ControlSize::Small)
                .disabled(move || shown.get() == 0)
                .on_press(move || index.set(shown.get().saturating_sub(1))),
            button(message("data_table.next", "Next"))
                .variant(ButtonVariant::Outline)
                .size(ControlSize::Small)
                .disabled(move || !can_advance())
                .on_press(move || index.set(shown.get().saturating_add(1))),
        )),
        |r| r.data_table_recipes().page_recipe(),
        variants,
        state,
    )
    .into_view()
}

/// The header check box in its box: it checks or clears the rows of the page
/// and is mixed while some of them are checked.
pub(super) fn select_page<K: Clone + Eq + Hash + 'static>(
    selection: Binding<Vec<K>>,
    keys: Value<Vec<K>>,
    variants: &Variants,
) -> View {
    let chosen = move || {
        keys.with(|keys| {
            selection.with(|chosen| keys.iter().filter(|k| chosen.contains(k)).count())
        })
    };
    let all = move || keys.with(|keys| !keys.is_empty() && chosen() == keys.len());
    let mixed = move || keys.with(|keys| chosen() > 0 && chosen() < keys.len());
    let select = kivact_core::view::callback(move |on: bool| {
        let keys = keys.get();
        let mut next = selection.get();
        next.retain(|k| !keys.contains(k));
        if on {
            next.extend(keys);
        }
        selection.set(next);
    });
    let state = move || DataTableCheckState {
        header: true,
        checked: if all() {
            CheckedState::Checked
        } else if mixed() {
            CheckedState::Indeterminate
        } else {
            CheckedState::Unchecked
        },
    };
    with_recipe_style(
        node().role(Role::ColumnHeader),
        |r| r.data_table_recipes().check_recipe(),
        variants,
        state,
    )
    .child(
        checkbox(
            message("data_table.select_all", "Select all"),
            binding(all, move |on| select.try_call(on)),
        )
        .hide_label(true)
        .indeterminate(mixed)
        .disabled(move || keys.with(Vec::is_empty)),
    )
    .into_view()
}
