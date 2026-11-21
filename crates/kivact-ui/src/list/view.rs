//! Role: The list component, its keyed item rendering and its edit handling.

use crate::layer::{with_base_recipe_style, with_recipe_style};
use crate::list::look::ListTheme;
use kivact_core::custom::Caller;
use kivact_core::state::{Callback, IntoValue, Signal, Value, binding};
use kivact_core::view::{EditPhase, View, for_each, node};
use kivact_interaction::{EditSession, ListItem, ListItemState, ListOptions, Selection, use_list};
use kivact_macros::component;

/// `value` whose writes are each reported to `on_edit` as one edit.
fn edited<T: kivact_core::state::PropValue>(
    value: Signal<T>,
    on_edit: Option<Callback<EditPhase>>,
) -> Signal<T> {
    let Some(on_edit) = on_edit else {
        return value;
    };
    let edit = EditSession::new(move |initial: T| value.set(initial));
    edit.on_edit(move |phase| on_edit.call(phase));
    binding(value.into_value(), move |next: T| {
        edit.once(value.get(), || value.set(next));
    })
}

/// A themed selectable list named `label` over `items` that writes the picked
/// key to `selected`.
///
/// Each item is a `Role::Option` named by its label, showing its icon, its
/// label and its description. The arrow keys move the focus and select
/// (Shift extends when `selected` is a multiple selection), Tab enters at the
/// selected item, a press or Space selects, typing a name jumps to the item
/// that starts with it, and a disabled item is skipped. The label is the
/// list's accessible name; it is not drawn. `.item(..)` replaces what an item
/// shows; `.on_edit(..)` reports each change as one edit ([`EditPhase`]).
///
/// ```no_run
/// use kivact::prelude::*;
///
/// fn colors() -> impl IntoView {
///     let selected = signal(Some("red"));
///     list(
///         "Colors",
///         vec![ListItem::new("red", "Red"), ListItem::new("blue", "Blue")],
///         selected,
///     )
/// }
/// ```
///
/// `.item(..)` builds what each item shows, once per item from its data and
/// its state (both reactive: a renamed item or a selected one updates the
/// view). The item itself, its keys, its semantics and its recipe stay the
/// component's; without a slot an item shows its icon, its label and its
/// description.
///
/// ```no_run
/// use kivact::prelude::*;
///
/// fn files() -> impl IntoView {
///     let selected = signal(Some(1_u32));
///     list(
///         "Files",
///         vec![ListItem::new(1, "notes.txt"), ListItem::new(2, "plan.md")],
///         selected,
///     )
///     .item(|item, state| {
///         text(move || {
///             let mark = if state.get().selected { "> " } else { "" };
///             format!("{mark}{}", item.get().label())
///         })
///     })
/// }
/// ```
#[component]
pub fn list<K: kivact_core::state::ItemKey, S: kivact_interaction::IntoSelection<K>>(
    label: Value<String>,
    items: Value<Vec<ListItem<K>>>,
    selected: S,
    on_edit: Option<Callback<EditPhase>>,
    item: Option<Rc<dyn Fn(Value<ListItem<K>>, Value<ListItemState>) -> View>>,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    let selection = match selected.into_selection() {
        Selection::Multiple(keys) => Selection::Multiple(edited(keys, on_edit)),
        Selection::Single(selected) => Selection::Single(edited(selected, on_edit)),
    };
    let behavior = use_list(
        items,
        selection,
        ListOptions {
            typeahead: true,
            ..ListOptions::default()
        },
    );
    let container = behavior.list();
    let list_state = container.state();
    let looks = variants.clone();
    let slot = item;
    let rows = for_each(
        items,
        |item| item.key().clone(),
        move |item| {
            let key = item.with_untracked(|item| item.key().clone());
            let part = behavior.item(key);
            let state = part.state();
            let content: View = match &slot {
                Some(slot) => slot(item, state),
                None => (
                    crate::helpers::item_icon(move || item.get().icon().cloned()),
                    crate::helpers::label_and_description(
                        move || item.get().label(),
                        move || item.get().description(),
                        {
                            let looks = looks.clone();
                            move |line| {
                                with_recipe_style(
                                    line,
                                    |r: &ListTheme| r.description_recipe(),
                                    &looks,
                                    move || state.get(),
                                )
                            }
                        },
                    ),
                )
                    .into_view(),
            };
            with_recipe_style(
                node().part(part),
                |r: &ListTheme| r.item_recipe(),
                &looks,
                move || state.get(),
            )
            .child(content)
        },
    );
    // The recipe is the base style, so the caller's styles win.
    with_base_recipe_style(
        node().name(label),
        |r: &ListTheme| r.root_recipe(),
        &variants,
        move || list_state.get(),
    )
    .part(container)
    .child(rows)
}
