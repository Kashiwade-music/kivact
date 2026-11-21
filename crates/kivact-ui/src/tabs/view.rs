//! Role: The tabs component, its tab list, its selection state, its overflow and the selected panel.
use crate::ButtonVariant;
use crate::ControlSize;
use crate::layer::{with_base_recipe_style, with_recipe_style};
use crate::menu::menu;
use crate::tabs::look::TabsTheme;
use crate::tabs::overflow::{TabOverflow, visible_count};
use kivact_core::custom::Caller;
use kivact_core::geometry::Rect;
use kivact_core::i18n::message;
use kivact_core::state::{
    Callback, ItemKey, Selector, Signal, Value, effect, memo, on_cleanup, selector, signal,
};
use kivact_core::style::{
    FlexDirection, Position, ScrollbarWidth, Style, Visibility, pct, style, use_theme,
};
use kivact_core::view::Variants;
use kivact_core::view::{
    EditPhase, Element, IntoView, Node, NodeRef, Orientation, ScrollAxes, View, dynamic, for_each,
    icon, node, node_ref, scroll, show, text,
};
use kivact_interaction::{ListActivation, ListItem, MenuEntry, TabState, TabsApi as Behavior};
use kivact_macros::component;
use std::collections::HashMap;
use std::rc::Rc;

/// The gap between two tabs in the list recipe (`tabs_list` in `look.rs`).
const TAB_GAP: f64 = 4.0;

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
/// `overflow` decides what happens to tabs wider than the box. `None` lets
/// them overflow; [`TabOverflow::Menu`] moves the tabs that do not fit into a
/// "More tabs" menu at the end of the strip; [`TabOverflow::Scroll`] scrolls
/// the strip and keeps the selected tab in view.
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
pub fn tabs<K: ItemKey>(
    label: Value<String>,
    items: Value<Vec<ListItem<K>>>,
    value: Signal<K>,
    on_edit: Option<Callback<EditPhase>>,
    #[prop(default = ListActivation::Automatic)] activation: ListActivation,
    #[prop(default = Orientation::Horizontal)] orientation: Orientation,
    item: Option<Rc<dyn Fn(Value<ListItem<K>>, Value<TabState>) -> View>>,
    content: Option<Rc<dyn Fn(K) -> View>>,
    #[prop(default = false)] keep_mounted: bool,
    overflow: Option<TabOverflow>,
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
    let list = tab_list(TabList {
        behavior,
        options: items,
        name,
        slot: item,
        variants: variants.clone(),
        orientation,
        value,
        overflow,
    });
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

/// What a tab list is built from: its behavior, its items, and its look and overflow.
struct TabList<K: ItemKey> {
    behavior: Behavior<K>,
    options: Value<Vec<ListItem<K>>>,
    name: Value<String>,
    slot: Option<TabContent<K>>,
    variants: Variants,
    orientation: Orientation,
    value: Signal<K>,
    overflow: Option<TabOverflow>,
}

/// The room of a tab strip with an overflow: the boxes of its tabs, and how many
/// leading tabs fit. `shown` is `usize::MAX` until the strip is measured, and
/// stays so without a [`TabOverflow::Menu`], where every tab fits.
struct Room<K: ItemKey> {
    options: Value<Vec<ListItem<K>>>,
    strip: NodeRef,
    more: NodeRef,
    sizes: Signal<HashMap<K, Value<Option<Rect>>>>,
    shown: Value<usize>,
}

impl<K: ItemKey> Clone for Room<K> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K: ItemKey> Copy for Room<K> {}

impl<K: ItemKey> Room<K> {
    /// The room of a strip with `overflow`, measured along its `vertical` or horizontal axis.
    fn new(
        options: Value<Vec<ListItem<K>>>,
        overflow: Option<TabOverflow>,
        vertical: bool,
    ) -> Self {
        let strip = node_ref();
        let more = node_ref();
        let sizes: Signal<HashMap<K, Value<Option<Rect>>>> = signal(HashMap::new());
        let shown = if overflow == Some(TabOverflow::Menu) {
            let strip_rect = strip.rect();
            let more_rect = more.rect();
            memo(move || {
                let axis = move |rect: Rect| if vertical { rect.height } else { rect.width };
                let Some(strip_box) = strip_rect.get() else {
                    return usize::MAX;
                };
                let more_length = more_rect.get().map_or(0.0, axis);
                let lengths: Vec<f64> = options.with(|items| {
                    sizes.with(|sizes| {
                        items
                            .iter()
                            .map(|item| {
                                sizes
                                    .get(item.key())
                                    .and_then(|rect| rect.get())
                                    .map_or(0.0, axis)
                            })
                            .collect()
                    })
                });
                visible_count(axis(strip_box), &lengths, TAB_GAP, more_length)
            })
        } else {
            memo(|| usize::MAX)
        };
        Room {
            options,
            strip,
            more,
            sizes,
            shown,
        }
    }

    /// Whether the tab with `key` is past the room of the strip.
    fn hidden(self, key: &K) -> bool {
        self.options
            .with(|items| items.iter().position(|item| item.key() == key))
            .is_some_and(|index| index >= self.shown.get())
    }
}

/// What every tab of a list is built from.
struct TabRows<K: ItemKey> {
    behavior: Behavior<K>,
    slot: Option<TabContent<K>>,
    variants: Variants,
    overflow: Option<TabOverflow>,
    room: Room<K>,
    selected: Selector<K>,
}

/// The style of a tab that does not fit the strip: out of the flow and hidden,
/// but still laid out, so that its natural size keeps being measured.
fn hidden_style() -> Style {
    style()
        .position(Position::Absolute)
        .left(0)
        .top(0)
        .visibility(Visibility::Hidden)
}

/// The tab list: it carries the tabs' accessible name and the list recipe, and
/// lays its tabs out in the room its overflow gives them.
fn tab_list<K: ItemKey>(list: TabList<K>) -> View {
    let TabList {
        behavior,
        options,
        name,
        slot,
        variants,
        orientation,
        value,
        overflow,
    } = list;
    let part = behavior.list();
    let list_state = part.state();
    let vertical = orientation == Orientation::Vertical;
    let room = Room::new(options, overflow, vertical);
    let rows = TabRows {
        behavior,
        slot,
        variants: variants.clone(),
        overflow,
        room,
        selected: selector(move || Some(value.get())),
    };
    let labels = for_each(
        options,
        |item| item.key().clone(),
        move |item| tab_row(&rows, item),
    );
    let root = with_base_recipe_style(
        node().name(name),
        |r: &TabsTheme| r.list_recipe(),
        &variants,
        move || list_state.get(),
    );
    let menu_mode = overflow == Some(TabOverflow::Menu);
    let more = menu_mode.then(|| more_tabs(room, value));
    let list = root
        .part(part)
        .child(labels)
        .child(more)
        .style(list_layout(menu_mode, vertical));
    list_view(list, room.strip, overflow, vertical)
}

/// One tab: what it shows, its indicator, and what its overflow asks of it.
fn tab_row<K: ItemKey>(rows: &TabRows<K>, item: Value<ListItem<K>>) -> Node {
    let key = item.with_untracked(|item| item.key().clone());
    let part = rows.behavior.item(key.clone());
    let state = part.state();
    let content = match &rows.slot {
        Some(slot) => slot(item, state),
        None => (
            crate::helpers::item_icon(move || item.get().icon().cloned()),
            text(move || item.get().label()),
        )
            .into_view(),
    };
    let indicator = tab_indicator(rows.variants.clone(), state);
    let me = node_ref();
    let tab = with_recipe_style(
        node().part(part).node_ref(me),
        |r: &TabsTheme| r.tab_recipe(),
        &rows.variants,
        move || state.get(),
    );
    let tab = match rows.overflow {
        Some(TabOverflow::Menu) => fit_tab(rows.room, tab, me, &key),
        Some(TabOverflow::Scroll) => {
            follow_selection(rows.room.strip, rows.selected, me, key);
            tab
        }
        None => tab,
    };
    tab.child(content).child(indicator)
}

/// The optional decorative layer of a tab, drawn only when the theme has one.
fn tab_indicator(variants: Variants, state: Value<TabState>) -> View {
    show(
        || use_theme().get().ext::<TabsTheme>().has_custom_indicator(),
        move || {
            with_recipe_style(
                node().hit_testable(false),
                |r: &TabsTheme| r.indicator_recipe(),
                &variants,
                move || state.get(),
            )
        },
    )
    .into_view()
}

/// A tab in a [`TabOverflow::Menu`] strip: past the room it is hidden and
/// out of the flow, its box keeps being measured, and the focus moves to the
/// "More tabs" button when it drops out while it has the focus.
fn fit_tab<K: ItemKey>(room: Room<K>, tab: Node, me: NodeRef, key: &K) -> Node {
    let focused = me.focused();
    let drop_key = key.clone();
    effect(move || {
        if room.hidden(&drop_key) && focused.get() {
            room.more.focus();
        }
    });
    let size_key = key.clone();
    room.sizes.update(|sizes| {
        sizes.insert(size_key, me.rect());
    });
    let cleanup_key = key.clone();
    on_cleanup(move || {
        room.sizes.update(|sizes| {
            sizes.remove(&cleanup_key);
        });
    });
    let style_key = key.clone();
    let focus_key = key.clone();
    let hit_key = key.clone();
    tab.style(move || {
        if room.hidden(&style_key) {
            hidden_style()
        } else {
            style()
        }
    })
    .focusable(move || !room.hidden(&focus_key))
    .hit_testable(move || !room.hidden(&hit_key))
}

/// In a [`TabOverflow::Scroll`] strip the selected tab scrolls into view when the
/// selection or the layout of the strip or of the tab changes. The effect only
/// asks the host to scroll; it writes no signal.
fn follow_selection<K: ItemKey>(strip: NodeRef, selected: Selector<K>, me: NodeRef, key: K) {
    let strip_rect = strip.rect();
    let me_rect = me.rect();
    effect(move || {
        if selected.is(&key) {
            let _ = strip_rect.get();
            let _ = me_rect.get();
            me.scroll_into_view();
        }
    });
}

/// The "More tabs" button and its menu, which lists the tabs past the room. The
/// button shows only while some tab is past it.
fn more_tabs<K: ItemKey>(room: Room<K>, value: Signal<K>) -> View {
    let all_shown = memo(move || room.shown.get() >= room.options.with(|items| items.len()));
    let overflow_items = memo(move || {
        let first = room.shown.get();
        let current = value.get();
        room.options.with(|items| {
            items
                .iter()
                .skip(first)
                .map(|item| {
                    MenuEntry::radio(item.key().clone(), item.label(), *item.key() == current)
                })
                .collect::<Vec<_>>()
        })
    });
    let button = crate::button(message("ui.tabs.more", "More tabs"))
        .variant(ButtonVariant::Ghost)
        .hide_label(true)
        .icon(icon("chevron-down"))
        .size(ControlSize::Small)
        .node_ref(room.more)
        .style(move || {
            if all_shown.get() {
                hidden_style()
            } else {
                style()
            }
        });
    menu(message("ui.tabs.more", "More tabs"), overflow_items)
        .on_activate(move |key: K| value.set(key))
        .attach(button)
}

/// The layout of the list: a column when vertical. In a menu strip the list is a
/// relative box that fills the strip along its axis, so the tabs past it can be measured.
fn list_layout(menu_mode: bool, vertical: bool) -> Style {
    let mut layout = style();
    if vertical {
        layout = layout.flex_direction(FlexDirection::Column);
    }
    if menu_mode {
        layout = layout.position(Position::Relative);
        layout = if vertical {
            layout.height(pct(100)).min_height(0)
        } else {
            layout.width(pct(100)).min_width(0)
        };
    }
    layout
}

/// The list itself, or the list inside a scroll when the strip scrolls.
fn list_view(list: Node, strip: NodeRef, overflow: Option<TabOverflow>, vertical: bool) -> View {
    match overflow {
        Some(TabOverflow::Scroll) => {
            let axes = if vertical {
                ScrollAxes::Vertical
            } else {
                ScrollAxes::Horizontal
            };
            scroll(list)
                .axes(axes)
                .node_ref(strip)
                .style(style().scrollbar_width(ScrollbarWidth::None))
                .into_view()
        }
        Some(TabOverflow::Menu) => list.node_ref(strip).into_view(),
        None => list.into_view(),
    }
}

/// The selected panel, or every panel when kept mounted. Each panel is the
/// tab panel part, named by its tab, around the factory's view; the part
/// hides it while its tab is not selected.
fn panel_views<K: ItemKey>(
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
