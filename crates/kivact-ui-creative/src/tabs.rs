//! Role: The document tab slot: a close button and drag reordering on the standard tabs.

use kivact::state::IntoBinding;

use kivact::i18n::message;
use kivact::interaction::{DragCoordinates, DragUpdate, TabState, use_drag};
use kivact::view::{Element, row, text};

use kivact::interaction::ListItem;
use kivact::state::{Callback, Value, signal};
use kivact::state::{IntoText, IntoValue, callback};
use kivact::style::{Align, Cursor, style};
use kivact::ui::{Tabs, tabs};
use kivact::view::{HandlerResult, IntoView, View};

/// How far a tab is dragged past its neighbor's edge before they swap.
const SWAP_DISTANCE: f64 = 60.0;

/// Document tabs: [`kivact::ui::tabs`] named `label` (for assistive
/// technology) of `options` choosing `selected`, in
/// the kit's theme, where each tab has a close button and is dragged sideways
/// to swap with its neighbor.
///
/// The tabs, their keys, selection and keyboard navigation are the standard
/// component's; this is its tab slot ([`Tabs::item`]). Dragging a tab 60
/// pixels toward a neighbor calls `on_reorder((moved, neighbor))`, and the
/// close button calls `on_close(key)`: the application changes `options`,
/// which the tabs follow, and keeps the selection. Give the returned tabs
/// `.content(..)` to show the selected document's panel.
#[track_caller]
pub fn document_tabs<K: kivact::state::ItemKey>(
    label: impl IntoText,
    options: impl IntoValue<Vec<ListItem<K>>>,
    selected: impl IntoBinding<K>,
) -> DocumentTabs<K> {
    let options = options.into_value();
    DocumentTabs {
        tabs: tabs(label, options, selected),
        options,
        on_reorder: callback(|_| ()),
        on_close: callback(|_| ()),
    }
}
/// Document tabs with optional application-owned close and reorder actions.
pub struct DocumentTabs<K: kivact::state::ItemKey> {
    tabs: Tabs<K>,
    options: Value<Vec<ListItem<K>>>,
    on_reorder: Callback<(K, K)>,
    on_close: Callback<K>,
}
impl<K: kivact::state::ItemKey> DocumentTabs<K> {
    /// Report drag reordering; no action by default.
    pub fn on_reorder<R: HandlerResult>(mut self, f: impl Fn((K, K)) -> R + 'static) -> Self {
        self.on_reorder = callback(f);
        self
    }
    /// Report closing a document; no action by default.
    pub fn on_close<R: HandlerResult>(mut self, f: impl Fn(K) -> R + 'static) -> Self {
        self.on_close = callback(f);
        self
    }
    /// Build the selected document panel through the standard tabs contract.
    pub fn content<V: IntoView>(mut self, f: impl Fn(K) -> V + 'static) -> Self {
        self.tabs = self.tabs.content(f);
        self
    }
}
impl<K: kivact::state::ItemKey> Element for DocumentTabs<K> {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut kivact::view::Node {
        self.tabs.node_mut()
    }
}
impl<K: kivact::state::ItemKey> IntoView for DocumentTabs<K> {
    fn into_view(self) -> View {
        let Self {
            mut tabs,
            options,
            on_reorder,
            on_close,
        } = self;
        let disabled = tabs
            .node_mut()
            .accessible_disabled()
            .unwrap_or_else(|| false.into_value());
        tabs.item(move |item, _: Value<TabState>| {
            document_tab(options, item, on_reorder, on_close, disabled)
        })
        .into_view()
    }
}

/// One tab's label and close button, which a horizontal drag swaps with a
/// neighbor.
fn document_tab<K: kivact::state::ItemKey>(
    options: Value<Vec<ListItem<K>>>,
    item: Value<ListItem<K>>,
    on_reorder: Callback<(K, K)>,
    on_close: Callback<K>,
    disabled: Value<bool>,
) -> View {
    let key = item.with_untracked(|item| item.key().clone());
    // The travel the last swap used up, so each swap takes another 60 px.
    let consumed = signal(0.0_f64);
    let reorder = use_drag::<K>(DragCoordinates::Window)
        .on_start(move |_| consumed.set(0.0))
        .on_update(move |update: DragUpdate<K>| -> kivact::Result<()> {
            if disabled.get() {
                return Ok(());
            }
            let delta = update.delta.x - consumed.get();
            if delta.abs() <= SWAP_DISTANCE {
                return Ok(());
            }
            let neighbor = options.with(|items| {
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
                consumed.set(update.delta.x);
                on_reorder.try_call((update.payload, neighbor))?;
            }
            Ok(())
        });
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
