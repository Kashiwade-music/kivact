//! Role: The tabs component, its tab list, its selection state and the selected panel.
use crate::layer::{with_base_recipe_style, with_recipe_style};
use crate::tabs::look::TabsTheme;
use kivact_core::custom::Caller;
use kivact_core::state::{Callback, Signal, Value};
use kivact_core::style::{FlexDirection, style, use_theme};
use kivact_core::view::Variants;
use kivact_core::view::{
    EditPhase, IntoView, Node, Orientation, View, dynamic, for_each, node, show, text,
};
use kivact_interaction::{ListActivation, ListItem, TabState, TabsApi as Behavior};
use kivact_macros::component;
use std::rc::Rc;

/// A themed tab list whose selected key is `value`; give it
/// `content` when the tabs also own the selected panel.
///
/// `label` is the tab list's accessible name; it is not drawn. The tabs'
/// root is a column (a row when vertical) that holds the tab list and the
/// selected panel, so the caller's modifiers style the whole tabs.
///
/// A tab list always has a selected tab, so the binding holds a key, not an
/// `Option`: start it at the first tab's key.
///
/// Each tab shows its item's icon and label. `item` replaces what a tab
/// shows, keeping its keys, semantics and recipe; it is built once per tab from
/// the tab's item and its state (both reactive). `on_edit` observes the
/// beginning and commit of each value edit. `activation` and `orientation` are
/// constants: the orientation is the keyboard policy and the shape of the
/// layout ([`Orientation::Vertical`] lays the tabs out in a column, beside
/// their panel). `content` builds the panel for the selected key in a fresh
/// owner each time the selection changes; a selected key that is not among the
/// options builds nothing. `keep_mounted(true)` builds each option's panel once
/// and keeps it mounted, hidden while its tab is not selected, so the state in
/// a panel (a draft, a scroll position) survives switching tabs.
///
/// ```no_run
/// use kivact::prelude::*;
///
/// fn inbox(unread: Signal<u32>) -> impl IntoView {
///     let selected = signal("inbox");
///     tabs("Mailbox", vec![ListItem::new("inbox", "Inbox"), ListItem::new("sent", "Sent")], selected)
///         .item(move |item, _state| {
///             row((
///                 text(move || item.get().label()),
///                 text(move || match item.get().key() {
///                     &"inbox" => unread.get().to_string(),
///                     _ => String::new(),
///                 }),
///             ))
///             .style(style().gap(6))
///         })
/// }
/// ```
#[component]
pub fn tabs<K: kivact_core::state::ItemKey>(
    label: Value<String>,
    items: Value<Vec<ListItem<K>>>,
    value: Signal<K>,
    on_edit: Option<Callback<EditPhase>>,
    #[prop(default = ListActivation::Automatic)] activation: ListActivation,
    #[prop(default = Orientation::Horizontal)] orientation: Orientation,
    item: Option<Rc<dyn Fn(Value<ListItem<K>>, Value<TabState>) -> View>>,
    content: Option<Rc<dyn Fn(K) -> View>>,
    #[prop(default = false)] keep_mounted: bool,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    let name = caller.name().unwrap_or(label);
    let vertical = orientation == Orientation::Vertical;
    let behavior = kivact_interaction::use_tabs(items, value)
        .activation(activation)
        .orientation(orientation);
    if let Some(observer) = on_edit {
        behavior.on_edit(move |phase| observer.call(phase));
    }
    let list = tab_list(behavior, items, name, item, variants.clone(), orientation);
    let panels =
        content.map(|factory| panel_views(behavior, items, factory, keep_mounted, variants));
    // Vertical tabs sit beside their panel, horizontal ones above it.
    let direction = if vertical {
        FlexDirection::Row
    } else {
        FlexDirection::Column
    };
    node()
        .name(label)
        .style(style().flex_direction(direction))
        .child((list, panels))
}

/// What a tab shows, from its item and its state.
type TabContent<K> = Rc<dyn Fn(Value<ListItem<K>>, Value<TabState>) -> View>;

/// The tab list: it carries the tabs' accessible name and the list recipe.
fn tab_list<K: kivact_core::state::ItemKey>(
    behavior: Behavior<K>,
    options: Value<Vec<ListItem<K>>>,
    name: Value<String>,
    slot: Option<TabContent<K>>,
    variants: Variants,
    orientation: Orientation,
) -> Node {
    let part = behavior.list();
    let list_state = part.state();
    let label_variants = variants.clone();
    let labels = for_each(
        options,
        |item| item.key().clone(),
        move |item| {
            let key = item.with_untracked(|item| item.key().clone());
            let part = behavior.item(key);
            let state = part.state();
            let content = match &slot {
                Some(slot) => slot(item, state),
                None => (
                    crate::helpers::item_icon(move || item.get().icon().cloned()),
                    text(move || item.get().label()),
                )
                    .into_view(),
            };
            let indicator_variants = label_variants.clone();
            let indicator = show(
                || use_theme().get().ext::<TabsTheme>().has_custom_indicator(),
                move || {
                    with_recipe_style(
                        node().hit_testable(false),
                        |r: &TabsTheme| r.indicator_recipe(),
                        &indicator_variants,
                        move || state.get(),
                    )
                },
            );
            with_recipe_style(
                node().part(part),
                |r: &TabsTheme| r.tab_recipe(),
                &label_variants,
                move || state.get(),
            )
            .child(content)
            .child(indicator)
        },
    );
    let root = with_base_recipe_style(
        node().name(name),
        |r: &TabsTheme| r.list_recipe(),
        &variants,
        move || list_state.get(),
    );
    let list = root.part(part).child(labels);
    if orientation == Orientation::Vertical {
        list.style(style().flex_direction(FlexDirection::Column))
    } else {
        list
    }
}

/// The selected panel, or every panel when kept mounted. Each panel is the
/// tab panel part, named by its tab, around the factory's view; the part
/// hides it while its tab is not selected.
fn panel_views<K: kivact_core::state::ItemKey>(
    behavior: Behavior<K>,
    options: Value<Vec<ListItem<K>>>,
    factory: Rc<dyn Fn(K) -> View>,
    keep: bool,
    variants: Variants,
) -> View {
    let selected = behavior.selected();
    let panel = Rc::new(move |key: K, factory: &Rc<dyn Fn(K) -> View>| {
        let named = key.clone();
        let name = move || {
            options.with(|options| {
                options
                    .iter()
                    .find(|item| *item.key() == named)
                    .map(|item| item.label())
                    .unwrap_or_default()
            })
        };
        let part = behavior.panel(key.clone());
        let state = part.state();
        with_recipe_style(
            node().part(part).name(name).child(factory(key)),
            |r: &TabsTheme| r.panel_recipe(),
            &variants,
            move || state.get(),
        )
    });
    if keep {
        for_each(
            options,
            |item| item.key().clone(),
            move |item| panel(item.with_untracked(|item| item.key().clone()), &factory),
        )
        .into_view()
    } else {
        let present = kivact_core::state::memo(move || {
            let key = selected.get();
            options
                .with(|rows| rows.iter().any(|row| *row.key() == key))
                .then_some(key)
        });
        dynamic(present, move |key| key.map(|key| panel(key, &factory))).into_view()
    }
}
