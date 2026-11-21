//! Role: A combobox filters keyed options by a query and tracks one active option.
//!
//! Unstyled behavior and semantics of a search field over a filtered list
//! box whose active option is the field's `active_descendant`. Pickers and
//! command palettes are built on it.
//!
//! The query is the caller's `Signal<String>`; [`query`](ComboboxApi::query) gives it to
//! [`use_text_edit`](crate::use_text_edit) on an ordinary node, which also
//! takes the combobox's `input` part; the options take `content` and `item`.
//! `visible` is the matching keys and `active` the current option. The focus
//! stays in the field and a disabled option is never chosen. The list is a
//! popup this behavior opens and closes in the caller's `open`; with
//! `ComboboxOptions { inline: true }` it is always there and Escape belongs to
//! the surrounding popup or dialog's dismissal; and a [`Selection::Multiple`]
//! selection lets
//! several options be chosen. Creative's pickers, the command palette
//! (`crate::palette`) and the kits' searchable selects share this behavior.

use super::machine;

use crate::Selection;
use crate::collection::{Index, Membership};
use crate::options_cell::OptionsCell;
use crate::surface::Surface;
use kivact_core::state::{MachineHandle, Signal, binding, machine};
use kivact_core::state::{Selector, Var, selector, var};
use std::collections::HashMap;
use std::rc::Rc;

use kivact_core::custom::{Part, PartState, part};
use kivact_core::state::IntoValue;
use kivact_core::state::{Value, effect, memo, on_cleanup, on_mount, signal};
use kivact_core::view::{
    Element, Handled, IntoView, Key, KeyEvent, NodeRef, PointerEvent, PointerPhase, Role, node_ref,
};

use crate::{ListItem, ListItemState};

type Filter<K> = Rc<dyn Fn(&str, &ListItem<K>) -> bool>;

/// Construction-time presentation of a combobox's list.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ComboboxOptions {
    /// The list is always part of the layout (a palette, a picker): no popup keys, roles or focus-out close.
    pub inline: bool,
}

/// What the setup calls of a [`ComboboxApi`] store.
struct SearchOptions<K: 'static> {
    /// The options: the caller's, or the results of [`ComboboxApi::search`].
    source: Value<Vec<ListItem<K>>>,
    debounce: Value<std::time::Duration>,
    search_status: Value<kivact_core::task::AsyncState<()>>,
}
impl<K: 'static> Clone for SearchOptions<K> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<K: 'static> Copy for SearchOptions<K> {}

/// The default search: the query, trimmed and lowercased, is part of the
/// lowercased label. An empty query accepts everything.
fn contains_query<K>(query: &str, option: &ListItem<K>) -> bool {
    let query = query.trim().to_lowercase();
    option.label.get().to_lowercase().contains(&query)
}

/// A search field over a filtered list box of keyed [`ListItem`]s, without a
/// look of its own.
///
/// The combobox holds the query text and the active option. Attach
/// [`input`](Self::input) to an ordinary node carrying
/// [`use_text_edit`](crate::use_text_edit) for [`query`](Self::query); [`content`](Self::content) to the node that holds
/// the rows; and [`item`](Self::item) to each row, rendered with
/// `for_each` over [`visible`](Self::visible). The choice is written to the
/// caller's selection ([`Selection::Single`] writes `Some(key)`,
/// [`Selection::Multiple`] toggles the key in its `Vec<K>`).
///
/// The list shows the options the [`filter`](Self::filter) accepts, in their
/// order. The focus stays in the field: the active option is shown to
/// assistive technology as the field's `active_descendant`, and the options
/// cannot be focused. The first enabled option that matches is active from
/// the start, and again after every change of the query; when the options
/// change under it, the active option stays on its key, or else the first
/// enabled match takes over. With no match nothing is active.
///
/// Keys in the field: ArrowDown and ArrowUp move the active option to the
/// next and the previous enabled one and wrap at the ends (from no active
/// option, ArrowDown goes to the first and ArrowUp to the last); Home and End
/// move the caret; Enter chooses the active option. Escape is left to the
/// surrounding dialog or popup. Moving the pointer onto an enabled option
/// makes it active, and a click chooses it. A disabled option is never active
/// and cannot be chosen.
///
/// Unless it is inline, the list is a popup the combobox opens and closes in
/// the caller's `open`: ArrowDown and ArrowUp open a
/// closed list (they move the active option only once it is open and stop
/// at the ends), Enter on
/// a closed one does nothing, Escape closes an open one, choosing a single
/// option closes it, and the list closes when the field loses the focus.
/// With a [`Selection::Multiple`] selection, choosing toggles the option in
/// the caller's `Vec<K>` and clears the query, the list stays open, and
/// Backspace in an empty field removes the last chosen option.
///
/// Semantics: the field `controls` the surface, a `Role::ListBox`; an option
/// is a `Role::Option` named by its label, `selected` while active.
///
/// ```no_run
/// use kivact::{state::signal, view::{Element, column, node, text, for_each}};
/// use kivact::interaction::{ComboboxOptions, ListItem, use_combobox, use_text_edit};
/// let options = vec![ListItem::new("red", "Red"), ListItem::new("green", "Green")];
/// let chosen = signal(None::<&'static str>);
/// let colors = use_combobox(options.clone(), chosen, signal(String::new()), signal(true), ComboboxOptions { inline: true });
/// let rows = for_each(colors.visible(), |key| *key, move |item| {
///     let key = item.get();
///     let label = options.iter().find(|o| *o.key() == key).map(|o| o.label());
///     node().part(colors.item(key)).child(text(label.unwrap_or_default()))
/// });
/// let view = column((
///     node().name("Color").part(use_text_edit(colors.query()).control()).part(colors.input()),
///     column(rows).part(colors.content()).name("Colors"),
/// ));
/// ```
pub struct ComboboxApi<K: kivact_core::state::ItemKey> {
    input_state: Signal<Option<Value<PartState>>>,
    content_state: Signal<Option<Value<PartState>>>,
    item_states: Signal<HashMap<K, Value<ListItemState>>>,
    options: Value<Vec<ListItem<K>>>,
    /// The caller's selection: one option or several.
    selection: Selection<K>,
    /// The setup options: the sources and the debounce.
    settings: OptionsCell<SearchOptions<K>>,
    /// The caller's open state: the list shows while it is true.
    open: Signal<bool>,
    /// The caller's query text, which the field edits.
    query: Signal<String>,
    /// Whether the list is always part of the layout (see [`ComboboxOptions`]).
    inline: bool,
    session: MachineHandle<machine::Machine<K>>,
    filter: Signal<Filter<K>>,
    /// The options that match, in order.
    shown: Value<Vec<ListItem<K>>>,
    visible: Value<Vec<K>>,
    active: Value<Option<K>>,
    /// The options by key: one index shared by every option part.
    index: Value<Index<K>>,
    /// Whether a key is the active option, per key.
    active_key: Selector<K>,
    /// Which keys the selection holds, per key.
    membership: Membership<K>,
    /// The mounted options by key, for `active_descendant`.
    rows: Signal<HashMap<K, NodeRef>>,
    input_ref: NodeRef,
    list_ref: NodeRef,
}

impl<K: kivact_core::state::ItemKey> Clone for ComboboxApi<K> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<K: kivact_core::state::ItemKey> Copy for ComboboxApi<K> {}

fn first_enabled<K: Clone>(options: &[ListItem<K>]) -> Option<K> {
    options
        .iter()
        .find(|option| !option.disabled)
        .map(|option| option.key.clone())
}

/// A combobox over `items` whose choice is written to `selected`, whose
/// query is `query` and whose list shows while `open` is true. Setup only.
#[track_caller]
pub fn use_combobox<K: kivact_core::state::ItemKey>(
    items: impl IntoValue<Vec<ListItem<K>>>,
    selected: impl crate::IntoSelection<K>,
    query: Signal<String>,
    open: Signal<bool>,
    options: ComboboxOptions,
) -> ComboboxApi<K> {
    let selection = selected.into_selection();
    let inline = options.inline;
    let settings = OptionsCell::new(SearchOptions {
        source: items.into_value(),
        debounce: std::time::Duration::from_millis(150).into_value(),
        search_status: (|| kivact_core::task::AsyncState::<()>::Idle).into_value(),
    });
    let options = memo(move || settings.value().with(|settings| settings.source).get());
    // The machine's state follows the caller's `open`; its effect only clears
    // the query of an inline list when it closes.
    let session = machine(
        if open.get_untracked() {
            machine::ComboState::Open {
                surface: Surface::Popover,
                filtering: false,
                highlight: None::<K>,
            }
        } else {
            machine::ComboState::Closed
        },
        || machine::Machine::<K>::new(),
        move |_, machine::Effect::Closed| {
            // An inline list's next opening starts from an empty query; a
            // popup keeps its query because the field still shows it.
            if inline && !query.with_untracked(String::is_empty) {
                query.set(String::new());
            }
        },
    );
    session.feed(open, |shown| {
        if shown {
            machine::Event::Opened
        } else {
            machine::Event::Closed
        }
    });
    kivact_core::state::effect_on(query, move |_| {
        if open.get_untracked() {
            session.send(machine::Event::QueryChanged);
        }
    });
    let filter = signal::<Filter<K>>(Rc::new(contains_query));
    let (shown, visible, active) = shown_rows(options, query, filter, session);
    let index = memo(move || options.with(|all| Index::of_items(all)));
    let active_key = selector(active);
    let membership = Membership::new(selection);
    ComboboxApi {
        input_state: signal(None),
        content_state: signal(None),
        item_states: signal(HashMap::new()),
        options,
        selection,
        settings,
        open,
        query,
        inline,
        session,
        filter,
        shown,
        visible,
        active,
        index,
        active_key,
        membership,
        rows: signal(HashMap::new()),
        input_ref: node_ref(),
        list_ref: node_ref(),
    }
}

type Shown<K> = (Value<Vec<ListItem<K>>>, Value<Vec<K>>, Value<Option<K>>);

/// The options the query shows, their keys, and the active option: the
/// highlighted one while it is shown and enabled, else the first enabled one.
fn shown_rows<K: kivact_core::state::ItemKey>(
    options: Value<Vec<ListItem<K>>>,
    query: Signal<String>,
    filter: Signal<Filter<K>>,
    session: MachineHandle<machine::Machine<K>>,
) -> Shown<K> {
    let shown = memo(move || {
        let typed = query.get();
        filter.with(|filter| {
            options.with(|all| {
                all.iter()
                    .filter(|option| filter(&typed, option))
                    .cloned()
                    .collect::<Vec<_>>()
            })
        })
    });
    let visible = memo(move || {
        shown.with(|shown| {
            shown
                .iter()
                .map(|option| option.key.clone())
                .collect::<Vec<_>>()
        })
    });
    let active = memo(move || {
        let highlight = session.state().with(|state| state.highlight().cloned());
        shown.with(|rows| {
            highlight
                .filter(|key| rows.iter().any(|row| row.key == *key && !row.disabled))
                .or_else(|| first_enabled(rows))
        })
    });
    (shown, visible, active)
}

impl<K: kivact_core::state::ItemKey> ComboboxApi<K> {
    /// Current source rows, including asynchronous suggestions when configured.
    pub fn options(self) -> Value<Vec<ListItem<K>>> {
        self.options
    }

    /// Readonly status of the single asynchronous suggestion source.
    pub fn search_status(self) -> Value<kivact_core::task::AsyncState<()>> {
        memo(move || {
            self.settings
                .value()
                .with(|settings| settings.search_status)
                .get()
        })
    }

    /// Delay between typing and starting an asynchronous suggestion request.
    #[track_caller]
    pub fn debounce(self, delay: impl IntoValue<std::time::Duration>) -> Self {
        self.settings
            .update(|settings| settings.debounce = delay.into_value());
        self
    }

    /// Fetch suggestions while open. Changing the query or closing cancels
    /// the previous request; opening a browsing list fetches immediately.
    #[track_caller]
    pub fn search<F, Fut>(self, search: F) -> Self
    where
        F: Fn(String) -> Fut + 'static,
        Fut: std::future::Future<Output = kivact_core::Result<Vec<ListItem<K>>>> + 'static,
    {
        let search = std::rc::Rc::new(search);
        let query = self.query();
        let open = self.is_open();
        let filtering = self.is_filtering();
        let request = memo(move || {
            open.get().then(|| {
                if filtering.get() {
                    (
                        query.get(),
                        self.settings
                            .value()
                            .with(|settings| settings.debounce)
                            .get(),
                    )
                } else {
                    (String::new(), std::time::Duration::ZERO)
                }
            })
        });
        let results = kivact_core::task::resource(request, move |request| {
            let search = search.clone();
            async move {
                let Some((query, delay)) = request else {
                    return Ok(None);
                };
                if !delay.is_zero() {
                    kivact_core::task::sleep(delay).await;
                }
                search(query).await.map(Some)
            }
        });
        self.settings.update(|settings| {
            settings.search_status = memo(move || match results.state() {
                kivact_core::task::AsyncState::Idle => kivact_core::task::AsyncState::Idle,
                kivact_core::task::AsyncState::Pending => kivact_core::task::AsyncState::Pending,
                kivact_core::task::AsyncState::Ready(_) => kivact_core::task::AsyncState::Ready(()),
                kivact_core::task::AsyncState::Failed(error) => {
                    kivact_core::task::AsyncState::Failed(error)
                }
            })
        });
        self.settings.update(|settings| {
            settings.source = memo(move || match results.state() {
                kivact_core::task::AsyncState::Ready(Some(rows)) => rows,
                _ => Vec::new(),
            })
        });
        self
    }

    /// Drawing state of the mounted editor anchor.
    pub fn input_state(self) -> Value<PartState> {
        memo(move || {
            self.input_state
                .get()
                .map(|state| state.get())
                .unwrap_or_default()
        })
    }
    /// Drawing state of the mounted list surface.
    pub fn content_state(self) -> Value<PartState> {
        memo(move || {
            self.content_state
                .get()
                .map(|state| state.get())
                .unwrap_or_default()
        })
    }
    /// Drawing state of the actual option, or its logical selection before mounting.
    pub fn item_state(self, key: K) -> Value<ListItemState> {
        let watch = self.membership.watch(key.clone());
        memo(move || {
            self.item_states
                .with(|states| states.get(&key).copied())
                .map(|state| state.get())
                .unwrap_or_else(|| ListItemState {
                    input: PartState::default(),
                    active: self.active_key.is(&key),
                    selected: self.membership.contains(&key, watch),
                })
        })
    }

    /// Choose which options the query shows. It receives the query as typed
    /// and an option, and reads reactive texts with `get()`, so the list
    /// follows a locale switch. The default accepts an option when the query,
    /// trimmed and lowercased, is part of its lowercased label. Setup only.
    #[track_caller]
    pub fn filter(self, filter: impl Fn(&str, &ListItem<K>) -> bool + 'static) -> Self {
        let filter: Filter<K> = Rc::new(filter);
        self.filter.update(move |slot| *slot = filter);
        self
    }

    /// Whether the list shows: the caller's `open` state.
    pub fn is_open(self) -> Value<bool> {
        self.open.into_value()
    }

    /// Whether the open combobox is presented in a touch sheet.
    pub fn is_sheet_open(self) -> Value<bool> {
        memo(move || {
            self.open.get()
                && self
                    .session
                    .state()
                    .with(|state| state.surface() == Some(Surface::Sheet))
        })
    }

    /// Show the list in a touch sheet; an open popover list moves to the
    /// sheet. Nothing changes while the sheet shows. An inline list is always
    /// shown, so this does nothing for it.
    pub fn show_sheet(self) {
        if self.inline || self.is_sheet_open_untracked() {
            return;
        }
        self.session.send(machine::Event::Present(Surface::Sheet));
        self.write_open();
    }

    /// Show the list in the popover. Nothing changes while the list shows (its
    /// query and surface stay). An inline list is always shown, so this does
    /// nothing for it.
    pub fn show(self) {
        if self.inline || self.open.get_untracked() {
            return;
        }
        self.session.send(machine::Event::Present(Surface::Popover));
        self.write_open();
    }

    /// Hide the list. Nothing changes while it is hidden, and an inline list is
    /// always shown, so this does nothing for it.
    pub fn hide(self) {
        if !self.inline && self.open.get_untracked() {
            self.open.set(false);
        }
    }

    /// Show a hidden list and hide a shown one. An inline list is always shown,
    /// so this does nothing for it.
    pub fn toggle(self) {
        if self.inline {
            return;
        }
        if self.open.get_untracked() {
            self.hide();
        } else {
            self.show();
        }
    }

    /// Whether the list shows now, read untracked: the commands run in handlers.
    fn shown(self) -> bool {
        self.open.get_untracked()
    }

    /// Whether the list is shown in the touch sheet, read untracked.
    fn is_sheet_open_untracked(self) -> bool {
        self.open.get_untracked()
            && self
                .session
                .state()
                .with_untracked(|state| state.surface() == Some(Surface::Sheet))
    }

    /// Write the caller's `open` to true, unless it already is.
    fn write_open(self) {
        if !self.open.get_untracked() {
            self.open.set(true);
        }
    }

    /// The query as the field edits it: a write is typing, which opens a closed popup list.
    pub fn query(self) -> Signal<String> {
        binding(
            move || self.query.get(),
            move |text: String| self.type_query(text),
        )
    }

    /// Set the query from typing: a popup list opens with it.
    fn type_query(self, text: String) {
        self.query.set(text);
        if self.inline {
            return;
        }
        self.session.send(machine::Event::Typed);
        self.write_open();
    }

    /// Whether the current query was entered by the user rather than browsing.
    pub fn is_filtering(self) -> Value<bool> {
        memo(move || self.open.get() && self.session.state().with(machine::ComboState::filtering))
    }

    /// The keys of the options the query shows, in order.
    pub fn visible(self) -> Value<Vec<K>> {
        self.visible
    }

    /// The active option, the one Enter chooses.
    pub fn active(self) -> Value<Option<K>> {
        self.active.into_value()
    }

    /// Choose `key` if it is an enabled option the list shows now. A
    /// binding that fails the write reports its error.
    fn choose(self, key: K) {
        let enabled = self.shown.with_untracked(|shown| {
            shown
                .iter()
                .any(|option| option.key == key && !option.disabled)
        });
        if !enabled {
            return;
        }
        match self.selection {
            Selection::Multiple(values) => {
                let mut keys = values.get_untracked();
                match keys.iter().position(|k| *k == key) {
                    Some(at) => {
                        keys.remove(at);
                    }
                    None => keys.push(key),
                }
                values.set(keys);
                if !self.query.with_untracked(String::is_empty) {
                    self.query.set(String::new());
                }
            }
            Selection::Single(value) => {
                value.set(Some(key));
                self.hide();
            }
        }
    }

    /// Move the active option to the next (`1`) or previous (`-1`) enabled
    /// one, wrapping at the ends.
    fn step(self, direction: i32) {
        let next = self.shown.with_untracked(|shown| {
            let enabled: Vec<&K> = shown
                .iter()
                .filter(|option| !option.disabled)
                .map(|option| &option.key)
                .collect();
            let count = enabled.len();
            if count == 0 {
                return None;
            }
            let at = self.active.with_untracked(|key| {
                key.as_ref()
                    .and_then(|key| enabled.iter().position(|k| *k == key))
            });
            let to = match (at, direction > 0) {
                (Some(at), true) if !self.inline => (at + 1).min(count - 1),
                (Some(at), false) if !self.inline => at.saturating_sub(1),
                (Some(at), true) => (at + 1) % count,
                (Some(at), false) => (at + count - 1) % count,
                (None, true) => 0,
                (None, false) => count - 1,
            };
            Some(enabled[to].clone())
        });
        if let Some(next) = next {
            self.session.send(machine::Event::Highlight(next));
        }
    }

    /// The search field: it `controls` the surface, publishes the active
    /// option as its `active_descendant` and takes ArrowDown, ArrowUp and
    /// Enter (without modifiers) after the caller's own key handlers, and
    /// Escape and Backspace where a popup list and a [`Selection::Multiple`]
    /// selection give them a meaning. Bind the field's text to
    /// [`query`](Self::query). Setup only.
    #[track_caller]
    pub fn input(self) -> Part<PartState> {
        self.input_for(self.input_ref)
    }

    /// The field as [`input`](Self::input), bound to the node `target`.
    #[track_caller]
    pub fn input_for(self, target: NodeRef) -> Part<PartState> {
        let (active, rows) = (self.active, self.rows);
        let input = part(|input| input);
        let part_state = input.state();
        crate::part_slot::attach_part_state(self.input_state, part_state);
        let expanded = self.is_open();
        Part::new(part_state, move |host| {
            let host = if !self.inline {
                host.role(Role::ComboBox).expanded(expanded)
            } else {
                host
            };
            host.part(input)
                .node_ref(target)
                .controls(self.list_ref)
                .has_popup(|| Some(kivact_core::view::PopupKind::ListBox))
                .active_descendant(move || {
                    active.with(|key| {
                        key.as_ref()
                            .and_then(|key| rows.with(|rows| rows.get(key).copied()))
                    })
                })
                .on_key(move |event: KeyEvent| {
                    let m = event.modifiers();
                    if m.alt && !m.ctrl && !m.meta && !m.shift {
                        match event.key() {
                            Key::ArrowDown => self.show(),
                            Key::ArrowUp => self.hide(),
                            _ => return Handled::No,
                        }
                        return Handled::Yes;
                    }
                    if m.ctrl || m.alt || m.meta || m.shift {
                        return Handled::No;
                    }
                    let closed = !self.shown();
                    match event.key() {
                        Key::ArrowDown | Key::ArrowUp if closed => self.show(),
                        Key::ArrowDown => self.step(1),
                        Key::ArrowUp => self.step(-1),
                        // A closed list has nothing to choose.
                        Key::Enter if closed => {}
                        Key::Enter => {
                            if let Some(key) = self.active.get() {
                                self.choose(key);
                            }
                        }
                        Key::Escape if !self.inline && !closed => {
                            self.hide();
                        }
                        Key::Backspace if self.query.with_untracked(String::is_empty) => {
                            let Selection::Multiple(values) = self.selection else {
                                return Handled::No;
                            };
                            let mut keys = values.get_untracked();
                            if keys.pop().is_some() {
                                values.set(keys);
                            }
                        }
                        _ => return Handled::No,
                    }
                    Handled::Yes
                })
                .on_attach(move |_| self.watch_focus(target))
        })
    }

    /// Close a popup list when `target` loses the focus, unless the list is in the sheet.
    fn watch_focus(self, target: NodeRef) {
        if self.inline {
            return;
        }
        let focused = target.focused();
        kivact_core::state::effect_on(focused, move |focused: &bool| {
            if !*focused
                && self
                    .session
                    .state()
                    .with_untracked(|s| s.surface() != Some(Surface::Sheet))
            {
                self.hide();
            }
        });
    }

    /// The floating surface of the list: mounted at the field while the list
    /// is shown, with the content part, and closed by the overlay's dismissal.
    #[track_caller]
    pub fn surface<V: kivact_core::view::IntoView>(
        self,
        placement: kivact_core::view::Placement,
        content: impl Fn() -> V + 'static,
    ) -> kivact_core::view::View {
        kivact_core::view::show(self.is_open(), move || {
            kivact_core::view::overlay(self.input_ref, placement, content())
                .with_node(|n| n.part(self.content()))
                .on_dismiss(move |_| self.hide())
        })
        .into_view()
    }

    /// The list box that holds the options, which the input `controls`.
    ///
    /// A pointer press on anything inside it that is not the field (a
    /// disabled option, a heading) gives the focus to the input instead.
    /// The name stays the caller's. Setup only.
    #[track_caller]
    pub fn content(self) -> Part<PartState> {
        self.content_for(self.input_ref)
    }

    /// The list box as [`content`](Self::content), with a press on it giving
    /// the focus to the field `target`.
    #[track_caller]
    pub fn content_for(self, target: NodeRef) -> Part<PartState> {
        let list = part(|input| input);
        let part_state = list.state();
        crate::part_slot::attach_part_state(self.content_state, part_state);
        Part::new(part_state, move |host| {
            host.part(list)
                .node_ref(self.list_ref)
                .role(Role::ListBox)
                .multiselectable(matches!(self.selection, Selection::Multiple(_)))
                .focus_on_press(target)
        })
    }

    /// The option `key`: a `Role::Option` named by its label, `selected`
    /// while it is active and disabled like its [`ListItem`]. Its state's
    /// `active` is whether it is the active option and `selected` whether it
    /// is the value. The active option is scrolled into view. Setup only, in
    /// the option's row.
    #[track_caller]
    pub fn item(self, key: K) -> Part<ListItemState> {
        let ComboboxApi {
            options,
            index,
            active,
            active_key,
            membership,
            rows,
            selection,
            item_states,
            session,
            ..
        } = self;
        let watch = membership.watch(key.clone());
        let key = var(key);
        let name = memo(move || {
            let at = key.with(|key| index.with(|index| index.source(key)));
            at.and_then(|at| options.with(|all| all.get(at).map(|option| option.label.get())))
                .unwrap_or_default()
        });
        let is_active = move || key.with(|key| active_key.is(key));
        let is_value = move || key.with(|key| membership.contains(key, watch));
        let disabled = move || key.with(|key| !index.with(|index| index.enabled(key)));
        let input = part(move |input| ListItemState {
            input,
            selected: is_value(),
            active: is_active(),
        });
        let part_state = input.state();
        item_states.update(|states| {
            states.insert(key.get(), part_state);
        });
        on_cleanup(move || {
            if item_states.try_get_untracked().is_some() {
                item_states.update(|states| {
                    let cleanup_key = key.get();
                    if states.get(&cleanup_key) == Some(&part_state) {
                        states.remove(&cleanup_key);
                    }
                });
            }
        });
        let target = kivact_core::view::node_ref();
        Part::new(part_state, move |host| {
            host.part(input)
                .node_ref(target)
                .role(Role::Option)
                .name(name)
                .selected(move || match selection {
                    Selection::Single(_) => is_active(),
                    Selection::Multiple(_) => is_value(),
                })
                .disabled(disabled)
                .on_pointer(move |event: PointerEvent| {
                    if matches!(event.phase(), PointerPhase::Enter | PointerPhase::Move)
                        && !disabled()
                        && !active
                            .with_untracked(|active| key.with(|key| active.as_ref() == Some(key)))
                    {
                        session.send(machine::Event::Highlight(key.get()));
                    }
                    Handled::No
                })
                .on_press(move || self.choose(key.get()))
                .on_attach(move |_| {
                    register_row(rows, key.get(), target);
                    follow_active_row(target, key, active_key);
                })
        })
    }

    /// The node the input part binds, for a surrounding dialog's initial
    /// focus or a press that should leave the focus in the field.
    pub(crate) fn input_ref(self) -> NodeRef {
        self.input_ref
    }
}

fn register_row<K: kivact_core::state::ItemKey>(
    rows: Signal<HashMap<K, NodeRef>>,
    held: K,
    row: NodeRef,
) {
    rows.update(|rows| {
        rows.insert(held.clone(), row);
    });
    // An option removes its entry only while it still holds it, so
    // a disposed reference is never left in the map.
    let leave = held.clone();
    on_cleanup(move || {
        if rows.try_get_untracked().is_some() {
            rows.update(|rows| {
                if rows.get(&leave) == Some(&row) {
                    rows.remove(&leave);
                }
            });
        }
    });
}

fn follow_active_row<K: kivact_core::state::ItemKey>(
    row: NodeRef,
    key: Var<K>,
    active_key: Selector<K>,
) {
    effect(move || {
        if key.with(|key| active_key.is(key)) {
            row.scroll_into_view();
        }
    });
    on_mount(move || {
        if key.with(|key| active_key.is(key)) {
            row.scroll_into_view();
        }
    });
}
