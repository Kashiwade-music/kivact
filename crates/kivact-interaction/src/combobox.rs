//! Role: A combobox filters keyed options by a query and tracks one active option.
//!
//! Unstyled behavior and semantics of a search field over a filtered list
//! box whose active option is the field's `active_descendant`. Pickers and
//! command palettes are built on it.
//!
//! The query is a `Binding<String>` the caller gives to
//! [`use_text_edit`](crate::use_text_edit) on an ordinary node, which also
//! takes the combobox's `input` part; the options take `content` and `item`.
//! `visible` is the matching keys and `active` the current option. The focus
//! stays in the field and a disabled option is never chosen. By default the
//! list is always there and Escape belongs to the surrounding popup or
//! dialog's dismissal; [`popup`](ComboboxApi::popup) makes the list a popup this
//! behavior opens and closes, and a [`Selection::Multiple`] selection lets
//! several options be chosen. Creative's pickers, the command palette
//! (`crate::palette`) and the kits' searchable selects share this behavior.

use crate::Selection;
use kivact_core::state::IntoBinding;
use std::collections::HashMap;
use std::rc::Rc;

use kivact_core::custom::{Part, PartState, part};
use kivact_core::state::IntoValue;
use kivact_core::state::{Binding, Signal, Value, effect, memo, on_cleanup, on_mount, signal};
use kivact_core::view::{
    Handled, Key, KeyEvent, NodeRef, PointerEvent, PointerPhase, Role, node_ref,
};

use super::{ListItem, ListItemState};

type Filter<K> = Rc<dyn Fn(&str, &ListItem<K>) -> bool>;

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
/// With [`popup`](Self::popup) the list is a popup the combobox opens and
/// closes in the caller's `Binding<bool>`: ArrowDown and ArrowUp open a
/// closed list (they move the active option only once it is open), Enter on
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
/// use kivact_core::{state::signal, view::{NodeBuilder, column, node, text, for_each}};
/// use kivact_interaction::{ListItem, use_combobox, use_text_edit};
/// let options = vec![ListItem::new("red", "Red"), ListItem::new("green", "Green")];
/// let chosen = signal(None::<&'static str>);
/// let colors = use_combobox(options.clone(), chosen);
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
    /// The open state of a list that is a popup.
    popup: Signal<Option<Binding<bool>>>,
    query: Signal<String>,
    filter: Signal<Filter<K>>,
    /// The options that match, in order.
    shown: Value<Vec<ListItem<K>>>,
    visible: Value<Vec<K>>,
    active: Signal<Option<K>>,
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

/// A combobox over `options` whose choice is written to `selection`, with an
/// empty query. Setup only.
#[track_caller]
pub fn use_combobox<K: kivact_core::state::ItemKey>(
    options: impl IntoValue<Vec<ListItem<K>>>,
    selection: impl crate::IntoSelection<K>,
) -> ComboboxApi<K> {
    let selection = selection.into_selection();
    let options = options.into_value();
    let query = signal(String::new());
    let filter = signal::<Filter<K>>(Rc::new(contains_query));
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
    let active = signal(None::<K>);
    // Every change of the query makes the first enabled match active.
    effect(move || {
        query.with(|_| ());
        filter.with(|_| ());
        active.set(shown.with_untracked(|shown| first_enabled(shown)));
    });
    // The active option stays an enabled option of the list: when the
    // options change under it, the first enabled match takes over.
    effect(move || {
        let kept = shown.with(|shown| {
            active.with_untracked(|key| {
                key.as_ref().is_some_and(|key| {
                    shown
                        .iter()
                        .any(|option| option.key == *key && !option.disabled)
                })
            })
        });
        if !kept {
            active.set(shown.with_untracked(|shown| first_enabled(shown)));
        }
    });
    ComboboxApi {
        input_state: signal(None),
        content_state: signal(None),
        item_states: signal(HashMap::new()),
        options,
        selection,
        popup: signal(None),
        query,
        filter,
        shown,
        visible,
        active,
        rows: signal(HashMap::new()),
        input_ref: node_ref(),
        list_ref: node_ref(),
    }
}

impl<K: kivact_core::state::ItemKey> ComboboxApi<K> {
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
        memo(move || {
            self.item_states
                .with(|states| states.get(&key).copied())
                .map(|state| state.get())
                .unwrap_or_else(|| ListItemState {
                    input: PartState::default(),
                    active: self.active.with(|active| active.as_ref() == Some(&key)),
                    selected: match self.selection {
                        Selection::Single(value) => {
                            value.with(|value| value.as_ref() == Some(&key))
                        }
                        Selection::Multiple(values) => values.with(|values| values.contains(&key)),
                    },
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

    /// Make the list a popup that is open while `open` is true: the field's
    /// keys open and close it (see the type's documentation), a single choice
    /// closes it and it closes when the field loses the focus. The caller
    /// draws the list while [`shown`](Self::is_open). Setup only.
    #[track_caller]
    pub fn popup(self, open: impl kivact_core::state::IntoBinding<bool>) -> Self {
        self.popup_with_focus_transfer(open, false)
    }

    /// Allow a separate presentation to own focus while the list is open.
    #[track_caller]
    pub fn popup_with_focus_transfer(
        self,
        open: impl kivact_core::state::IntoBinding<bool>,
        transferred: impl IntoValue<bool>,
    ) -> Self {
        let open = open.into_binding();
        let transferred = transferred.into_value();
        self.popup.update(|slot| *slot = Some(open));
        let focused = self.input_ref.focused();
        effect(move || {
            if !focused.get() && !transferred.get() && open.get_untracked() {
                open.set(false);
            }
        });
        self
    }

    /// Whether the list shows: the open state of a [`popup`](Self::popup), and
    /// always for a combobox without one.
    pub fn is_open(self) -> Value<bool> {
        let popup = self.popup;
        (move || popup.with(|popup| popup.is_none_or(|open| open.get()))).into_value()
    }

    /// Open a popup list. Nothing happens without [`popup`](Self::popup).
    pub fn open(self) {
        if let Some(open) = self.popup.get_untracked() {
            open.set(true);
        }
    }

    /// Close a popup list. Nothing happens without [`popup`](Self::popup).
    pub fn close(self) {
        if let Some(open) = self.popup.get_untracked() {
            open.set(false);
        }
    }

    /// Open a closed popup list and close an open one.
    pub fn toggle(self) {
        if let Some(open) = self.popup.get_untracked() {
            open.set(!open.get_untracked());
        }
    }

    /// The query binding for the caller's native editing attachment.
    pub fn query(self) -> Binding<String> {
        self.query.into_binding()
    }

    /// The keys of the options the query shows, in order.
    pub fn visible(self) -> Value<Vec<K>> {
        self.visible
    }

    /// The active option, the one Enter chooses.
    pub fn active(self) -> Value<Option<K>> {
        self.active.into_value()
    }

    /// Empty the query and make the first enabled option active, as when the
    /// combobox was made.
    pub(crate) fn reset(self) {
        self.query.set(String::new());
        self.active
            .set(self.shown.with_untracked(|shown| first_enabled(shown)));
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
                self.query.set(String::new());
            }
            Selection::Single(value) => {
                value.set(Some(key));
                self.close();
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
                (Some(at), true) => (at + 1) % count,
                (Some(at), false) => (at + count - 1) % count,
                (None, true) => 0,
                (None, false) => count - 1,
            };
            Some(enabled[to].clone())
        });
        if let Some(next) = next {
            self.active.set(Some(next));
        }
    }

    /// The search field: it `controls` the surface, publishes the active
    /// option as its `active_descendant` and takes ArrowDown, ArrowUp and
    /// Enter (without modifiers) after the caller's own key handlers, and
    /// Escape and Backspace where [`popup`](Self::popup) and
    /// [`multiple`](Self::multiple) give them a meaning. Bind the field's text
    /// to [`query`](Self::query). Setup only.
    #[track_caller]
    pub fn input(self) -> Part<PartState> {
        let (active, rows) = (self.active, self.rows);
        let input = part(|input| input);
        let part_state = input.state();
        self.input_state.set(Some(part_state));
        let target = kivact_core::view::node_ref();
        Part::new(part_state, move |host| {
            host.part(input)
                .node_ref(target)
                .node_ref(self.input_ref)
                .controls(self.list_ref)
                .active_descendant(move || {
                    active.with(|key| {
                        key.as_ref()
                            .and_then(|key| rows.with(|rows| rows.get(key).copied()))
                    })
                })
                .on_key(move |event: KeyEvent| {
                    let m = event.modifiers();
                    if m.ctrl || m.alt || m.meta || m.shift {
                        return Handled::No;
                    }
                    let closed = self
                        .popup
                        .get_untracked()
                        .is_some_and(|open| !open.get_untracked());
                    match event.key() {
                        Key::ArrowDown | Key::ArrowUp if closed => self.open(),
                        Key::ArrowDown => self.step(1),
                        Key::ArrowUp => self.step(-1),
                        // A closed list has nothing to choose.
                        Key::Enter if closed => {}
                        Key::Enter => {
                            if let Some(key) = self.active.get() {
                                self.choose(key);
                            }
                        }
                        Key::Escape if self.popup.with_untracked(Option::is_some) && !closed => {
                            self.close();
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
        })
    }

    /// The list box that holds the options, which the input `controls`.
    ///
    /// A pointer press on anything inside it that is not the field (a
    /// disabled option, a heading) gives the focus to the input instead.
    /// The name stays the caller's. Setup only.
    #[track_caller]
    pub fn content(self) -> Part<PartState> {
        let input = part(|input| input);
        let part_state = input.state();
        self.content_state.set(Some(part_state));
        let target = kivact_core::view::node_ref();
        Part::new(part_state, move |host| {
            host.part(input)
                .node_ref(target)
                .node_ref(self.list_ref)
                .role(Role::ListBox)
                .focus_on_press(self.input_ref)
        })
    }

    /// The option `key`: a `Role::Option` named by its label, `selected`
    /// while it is active and disabled like its [`ListItem`]. Its state's
    /// `active` is whether it is the active option and `selected` whether it
    /// is the value. The active option is scrolled into view. Setup only, in
    /// the option's row.
    #[track_caller]
    pub fn item(self, key: K) -> Part<ListItemState> {
        let options = self.options;
        let active = self.active;
        let rows = self.rows;
        let selection = self.selection;
        let item = {
            let key = key.clone();
            memo(move || {
                options.with(|all| {
                    all.iter()
                        .find(|option| option.key == key)
                        .map(|option| (option.label.get(), option.disabled))
                })
            })
        };
        let is_active = {
            let key = key.clone();
            memo(move || active.with(|active| active.as_ref() == Some(&key)))
        };
        let is_value = {
            let key = key.clone();
            memo(move || match selection {
                Selection::Multiple(values) => values.with(|values| values.contains(&key)),
                Selection::Single(value) => value.with(|value| value.as_ref() == Some(&key)),
            })
        };
        let disabled = memo(move || item.with(|item| item.as_ref().is_none_or(|(_, d)| *d)));
        let input = part(move |input| ListItemState {
            input,
            selected: is_value.get(),
            active: is_active.get(),
        });
        let part_state = input.state();
        let item_states = self.item_states;
        item_states.update(|states| {
            states.insert(key.clone(), part_state);
        });
        let cleanup_key = key.clone();
        on_cleanup(move || {
            if item_states.try_get_untracked().is_some() {
                item_states.update(|states| {
                    if states.get(&cleanup_key) == Some(&part_state) {
                        states.remove(&cleanup_key);
                    }
                });
            }
        });
        let target = kivact_core::view::node_ref();
        Part::new(part_state, move |host| {
            let attachment = host.part(input).node_ref(target);
            let row = target;
            let hover_key = key.clone();
            let press_key = key.clone();
            let held = key;
            attachment
                .role(Role::Option)
                .name(move || {
                    item.with(|item| item.as_ref().map(|(l, _)| l.clone()))
                        .unwrap_or_default()
                })
                .selected(is_active)
                .disabled(disabled)
                .on_pointer(move |event: PointerEvent| {
                    if matches!(event.phase(), PointerPhase::Enter | PointerPhase::Move)
                        && !disabled.get()
                        && !is_active.get()
                    {
                        active.set(Some(hover_key.clone()));
                    }
                    Handled::No
                })
                .on_press(move || self.choose(press_key.clone()))
                .on_attach(move |_| {
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
                    effect(move || {
                        if is_active.get() {
                            row.scroll_into_view();
                        }
                    });
                    on_mount(move || {
                        if is_active.get() {
                            row.scroll_into_view();
                        }
                    });
                })
        })
    }

    /// Present the same combobox state with a different editor anchor.
    /// Mount one list surface at a time when using the alternate editor.
    pub fn with_input_ref(mut self, reference: NodeRef) -> Self {
        self.input_ref = reference;
        self
    }

    /// The node the input part binds, for a surrounding dialog's initial
    /// focus or a press that should leave the focus in the field.
    pub(crate) fn input_ref(self) -> NodeRef {
        self.input_ref
    }
}
