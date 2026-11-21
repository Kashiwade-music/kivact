//! Role: The document tab slot: a close button and drag reordering on the standard tabs.

use kivact::component;
use kivact::custom::Caller;
use kivact::i18n::message;
use kivact::interaction::{DragCoordinates, DragUpdate, ListItem, TabState, use_drag};
use kivact::state::{Callback, IntoValue, ItemKey, Signal, Value, callback, signal};
use kivact::style::{Align, Cursor, style};
use kivact::ui::tabs;
use kivact::view::{Element, IntoView, View, row, text};

/// How far a tab is dragged past its neighbor's edge before they swap.
const SWAP_DISTANCE: f64 = 60.0;

/// Document tabs: [`kivact::ui::tabs`] named `label` (for assistive
/// technology) of `items` choosing `value`, in the kit's theme, where each
/// tab has a close button and is dragged sideways to swap with its neighbor.
///
/// The tabs, their keys, selection and keyboard navigation are the standard
/// component's; this is its tab slot. Dragging a tab 60 pixels toward a
/// neighbor calls `on_reorder((moved, neighbor))`, and the close button calls
/// `on_close(key)`: the application changes `items`, which the tabs follow,
/// and keeps the selection. `content` builds the selected document's panel.
#[component]
pub fn document_tabs<K: ItemKey>(
    label: Value<String>,
    items: Value<Vec<ListItem<K>>>,
    value: Signal<K>,
    on_reorder: Option<Callback<(K, K)>>,
    on_close: Option<Callback<K>>,
    content: Option<Rc<dyn Fn(K) -> View>>,
    #[caller] caller: Caller,
) -> impl Element {
    let disabled = caller.disabled().unwrap_or_else(|| false.into_value());
    let on_reorder = on_reorder.unwrap_or_else(|| callback(|_| ()));
    let on_close = on_close.unwrap_or_else(|| callback(|_| ()));
    let standard = tabs(label, items, value).item(move |item, _: Value<TabState>| {
        document_tab(items, item, on_reorder, on_close, disabled)
    });
    match content {
        Some(factory) => standard.content(move |key| factory(key)),
        None => standard,
    }
}

/// The travel a tab's reorder drag has used up, between swaps.
#[derive(Clone, Copy, PartialEq)]
enum Reorder {
    Idle,
    Dragging { consumed: f64 },
}

/// One tab's label and close button, which a horizontal drag swaps with a
/// neighbor.
fn document_tab<K: ItemKey>(
    items: Value<Vec<ListItem<K>>>,
    item: Value<ListItem<K>>,
    on_reorder: Callback<(K, K)>,
    on_close: Callback<K>,
    disabled: Value<bool>,
) -> View {
    let key = item.with_untracked(|item| item.key().clone());
    // The travel the last swap used up, so each swap takes another 60 px.
    let reorder_state = signal(Reorder::Idle);
    let reorder = use_drag::<K>(DragCoordinates::Window)
        .on_start(move |_| reorder_state.set(Reorder::Dragging { consumed: 0.0 }))
        .on_update(move |update: DragUpdate<K>| -> kivact::Result<()> {
            if disabled.get() {
                return Ok(());
            }
            let consumed = match reorder_state.get_untracked() {
                Reorder::Dragging { consumed } => consumed,
                Reorder::Idle => 0.0,
            };
            let delta = update.delta.x - consumed;
            if delta.abs() <= SWAP_DISTANCE {
                return Ok(());
            }
            let neighbor = items.with(|items| {
                let index = items
                    .iter()
                    .position(|item| *item.key() == update.payload)?;
                let next = if delta > 0.0 {
                    index + 1
                } else {
                    index.checked_sub(1)?
                };
                items.get(next).map(|item| item.key().clone())
            });
            if let Some(neighbor) = neighbor {
                reorder_state.set(Reorder::Dragging {
                    consumed: update.delta.x,
                });
                on_reorder.try_call((update.payload, neighbor))?;
            }
            Ok(())
        })
        .on_end(move |_| reorder_state.set(Reorder::Idle));
    let close = key.clone();
    row((
        text(move || item.with(|item| item.label())),
        kivact::ui::button(message("creative.tabs.close", "Close tab"))
            .with_node(move |node| node.disabled(disabled))
            .variant(kivact::ui::ButtonVariant::Secondary)
            .hide_label(true)
            .icon(kivact::view::icon("x"))
            .with_node(move |node| node.style(style().size(16, 16).padding(0)))
            .on_press(move || on_close.try_call(close.clone())),
    ))
    .part(reorder.source(key))
    .style(
        style()
            .gap(6)
            .align_items(Align::Center)
            .cursor(Cursor::Grab),
    )
    .into_view()
}
