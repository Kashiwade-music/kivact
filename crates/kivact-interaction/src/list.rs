//! Role: One list behavior attaches selection and navigation through parts.
//!
//! [`use_list`] takes a reactive `Vec<ListItem<K>>` and a [`ListSelection`]
//! (`Single` over `Binding<Option<K>>`, `Multiple` over `Binding<Vec<K>>`).
//! The order and the selection stay the caller's: a writer that refuses a
//! selection is not replaced by an internal one, and a selected key that a
//! filter or virtualization hides is not cleared. [`List::list`] is the
//! `ListBox` and [`List::item`] the `Option` part with its name, selection,
//! input state and roving focus. A node's `.part(..)` and a canvas region
//! share one registry, and an item's cleanup registers in the owner that
//! actually mounts it. Tabs, radio groups, menus, selects, tables and trees
//! are built on a list.
//!
//! [`ListPolicy`] holds the axis, boundary, activation, action, typeahead
//! and range policy. The default is vertical, clamped, automatic, Select and
//! no typeahead; the `tabs`, `radio`, `menu` and `listbox` presets use
//! Activate, and `menu` and `listbox` add typeahead. With Select a primary
//! press or Space selects with the gesture its modifiers ask for, Enter does
//! nothing, and only Select pages with PageUp/PageDown. Activate chooses once,
//! on the native release, Enter, Space or assistive technology. Wrapping
//! applies to explicit moves only. When the active item disappears, Select
//! clamps from its old position and Activate recovers to a valid selected key
//! or the first one. The policy, `page_size`, `on_leave`, `scroll_with` and
//! `anchor` are fixed during setup before the first part; reactive items and
//! external binding writes still reach existing parts.
//!
//! [`List::select`] offers a gesture for a valid logical key, mounted or
//! not; [`List::choose`] selects and focuses a mounted candidate, and
//! [`List::focus`] focuses without selecting. Ranges, the command modifier
//! and a missing anchor follow the shared range policy, and
//! [`List::anchor`] may share the anchor between lists. [`List::clear`]
//! empties the selection and the anchor; [`List::select_all`] selects the
//! enabled logical keys of a `Multiple` list in order. Menus and selects use
//! a private mounted selection path (so an option that disappears with the
//! choice gets no extra focus request) and share a private typeahead slot
//! over a menu's levels, or a select's closed trigger and open list; no raw
//! input dispatch is public.
//!
//! Invariants:
//! - A repeated logical key keeps its first occurrence and is reported; a
//!   second mounted item with the same key is refused.
//! - A mounted item is eligible by the caller's eligibility, its final
//!   disabled and hidden values and its actual node token, not by its final
//!   focusability (which would loop through the roving tab stop). A node's
//!   disabled value is its own, a hidden ancestor hides its children, and a
//!   canvas owner's disabled value applies to its regions.
//! - Virtualized focus calls `scroll_with` once per request and gives the
//!   focus when the target binds. The request remembers the starting token
//!   and the list's item node: losing the focus while the target is still
//!   bound, a starting focus outside the list that cannot be followed, or the
//!   user focusing elsewhere cancels it; a focus that became `None` because
//!   the starting row unmounted is kept, and a request that started at
//!   `None` may stay `None`.
//! - An item part of another App reports `WrongApp` once and is ignored as a
//!   whole, keeping the caller's role, state, actions and focusability.
//! - Ancestor visibility comes from `NodeRef::is_visible`, whose revision
//!   source the core creates once in the App's root owner on first use and
//!   keeps for the App's lifetime. A change of a hidden value re-evaluates
//!   every list's candidates, but nothing polls per frame, and an App that
//!   never asks (no list) has no such source.
//! - Restoring the active item is separate from the focus authority: a
//!   display change alone never takes the focus from outside the group.

mod data;
mod input;
mod order;
mod parts;
mod selection;
mod state;
pub(crate) mod typeahead;

use kivact_core::diagnostics::ErrorKind;
use kivact_core::primitive::{Owned, Part, PartState, require_effectful, require_setup};
use kivact_core::view::{A11yAction, Handled, HandlerResult, IntoValue, KeyEvent, callback};
use kivact_core::{Binding, Value, Var};
use state::State;
use std::hash::Hash;
use std::panic::Location;
use std::rc::Rc;

pub use data::{
    ListActivation, ListBoundary, ListCommandModifier, ListGesture, ListItem, ListItemState,
    ListMissingAnchor, ListPolicy, ListRangeMode, ListRangePolicy, ListSelection,
};

/// A Copy list handle over caller-owned item order and authoritative selection.
/// Attach `list()` and `item(key)` to caller-rendered nodes or canvas regions.
/// Filtering never clears selection; only mounted eligible items take roving focus.
/// Configure policy, paging, leave/scroll callbacks and anchor during setup,
/// before creating the first container or item Part. Boundary wrapping applies
/// to explicit navigation; active removal always clamps from its old position.
/// Reactive inputs and caller
/// binding writes continue to update existing Parts.
pub struct List<K: Clone + Eq + Hash + 'static> {
    state: Owned<State<K>>,
}
impl<K: Clone + Eq + Hash + 'static> Copy for List<K> {}
impl<K: Clone + Eq + Hash + 'static> Clone for List<K> {
    fn clone(&self) -> Self {
        *self
    }
}

/// Create a setup-owned list without rendering any elements.
#[track_caller]
pub fn use_list<K: Clone + Eq + Hash + 'static>(
    items: impl IntoValue<Vec<ListItem<K>>>,
    selection: ListSelection<K>,
) -> List<K> {
    List {
        state: state::create(items.into_value(), selection, Location::caller()),
    }
}

impl<K: Clone + Eq + Hash + 'static> List<K> {
    #[track_caller]
    fn state(self) -> Rc<State<K>> {
        self.state.get()
    }
    #[track_caller]
    fn configure(self, name: &'static str) -> Option<Rc<State<K>>> {
        let state = self.state();
        require_setup(name);
        if state.started.get() {
            state.owner.report(
                ErrorKind::Message,
                "List must be configured before creating parts",
                Location::caller(),
            );
            None
        } else {
            Some(state)
        }
    }
    /// Configure navigation and selection before constructing any list part.
    #[track_caller]
    pub fn policy(self, policy: ListPolicy) -> Self {
        let state = self.state();
        require_setup("List::policy");
        if !matches!(policy.action, A11yAction::Select | A11yAction::Activate) {
            state.owner.report(
                ErrorKind::Unsupported,
                "List policy action must be Select or Activate",
                Location::caller(),
            );
            return self;
        }
        if let Some(state) = self.configure("List::policy") {
            state.policy.set(policy);
            state.revision.update(|revision| *revision += 1);
        }
        self
    }
    /// Set the Select-policy Page Up/Down distance; values below one count as one.
    /// Setup only, before creating any Part.
    #[track_caller]
    pub fn page_size(self, rows: impl IntoValue<usize>) -> Self {
        if let Some(state) = self.configure("List::page_size") {
            state.page.set(rows.into_value());
        }
        self
    }
    /// Prefer another enabled key when the current logical item disappears.
    /// Setup only, before creating any Part.
    #[track_caller]
    pub fn on_leave(self, f: impl Fn(&K) -> Option<K> + 'static) -> Self {
        if let Some(state) = self.configure("List::on_leave") {
            *state.leave.borrow_mut() = Some(Rc::new(f));
        }
        self
    }
    /// Scroll through the caller and permit pending focus on unmounted keys.
    /// An `Err` from `f` goes to the nearest error handler. Setup only, before
    /// creating any Part.
    #[track_caller]
    pub fn scroll_with<R: HandlerResult>(self, f: impl Fn(K) -> R + 'static) -> Self {
        if let Some(state) = self.configure("List::scroll_with") {
            *state.scroll.borrow_mut() = Some(callback(f));
        }
        self
    }
    /// Share an authoritative range anchor with other list instances.
    /// Setup only, before creating any Part; later binding writes remain reactive.
    #[track_caller]
    pub fn anchor(self, anchor: impl Into<Binding<Option<K>>>) -> Self {
        if let Some(state) = self.configure("List::anchor") {
            state.anchor.set(anchor.into());
            state.revision.update(|revision| *revision += 1);
        }
        self
    }
    /// Read the authoritative selection as zero, one or several keys.
    #[track_caller]
    pub fn selected(self) -> Value<Vec<K>> {
        self.state().selected
    }
    /// Read the enabled mounted key that currently owns the roving tab stop.
    #[track_caller]
    pub fn active(self) -> Value<Option<K>> {
        self.state().active
    }
    /// Read the caller's authoritative range anchor.
    #[track_caller]
    pub fn anchor_value(self) -> Value<Option<K>> {
        self.state().anchor_value
    }
    /// Offer selection and focus of an eligible mounted key.
    #[track_caller]
    pub fn choose(self, key: K) -> bool {
        let state = self.state();
        require_effectful("List::choose");
        state.choose(key)
    }
    /// Request focus without selecting, retaining a virtual pending request if configured.
    #[track_caller]
    pub fn focus(self, key: K) -> bool {
        let state = self.state();
        require_effectful("List::focus");
        state.focus(key)
    }
    /// Offer a gesture for an enabled logical key without moving focus.
    #[track_caller]
    pub fn select(self, key: K, gesture: ListGesture) -> bool {
        let state = self.state();
        require_effectful("List::select");
        state.select(key, gesture)
    }
    /// Offer an empty selection and clear the range anchor.
    #[track_caller]
    pub fn clear(self) {
        let state = self.state();
        require_effectful("List::clear");
        match state.selection {
            ListSelection::Single(binding) => binding.set(None),
            ListSelection::Multiple(binding) => binding.set(Vec::new()),
        }
        state.anchor.get().set(None);
    }
    /// Select enabled logical keys in order for Multiple, leaving its anchor unchanged.
    #[track_caller]
    pub fn select_all(self) {
        let state = self.state();
        require_effectful("List::select_all");
        input::select_all(&state);
    }
    /// The list's default ListBox semantics, without appearance or children.
    #[track_caller]
    pub fn list(self) -> Part<PartState> {
        let state = self.state();
        require_setup("List::container");
        parts::container(state)
    }
    /// An item's label, selection, exact input state and mounted roving navigation.
    #[track_caller]
    pub fn item(self, key: K) -> Part<ListItemState> {
        let state = self.state();
        require_setup("List::item");
        parts::item(state, key, Location::caller())
    }
    #[track_caller]
    pub(crate) fn select_mounted(self, key: K) -> bool {
        let state = self.state();
        require_effectful("List::select_mounted");
        if state.target(&key).is_none() {
            return false;
        }
        state.select(key, ListGesture::Replace)
    }
    #[track_caller]
    pub(crate) fn key_selection(self, event: &KeyEvent) -> Handled {
        let state = self.state();
        require_effectful("List::key_selection");
        input::key_selection(&state, event)
    }
    /// The selection gesture a pointer press or Space with these modifiers asks for.
    pub(crate) fn gesture(self, modifiers: kivact_core::view::Modifiers) -> ListGesture {
        input::gesture(&self.state(), modifiers)
    }
    pub(crate) fn holds_focus(self, key: &K) -> bool {
        self.state().holds_focus(key)
    }
    #[track_caller]
    pub(crate) fn shared_typeahead(self, typed: Var<typeahead::Typeahead>) -> Self {
        if let Some(state) = self.configure("List::shared_typeahead") {
            state.typed.set(typed);
        }
        self
    }
}
