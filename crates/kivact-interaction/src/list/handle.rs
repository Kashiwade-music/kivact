//! Role: The `List` handle: setup options, reads, selection commands and parts.

use super::data::{ListGesture, ListItem, ListItemState, ListPolicy, ListSelection};
use super::state::State;
use super::{input, parts, state, typeahead};
use crate::axis::check_list_axis;
use kivact_core::custom::{Owned, Part, PartState, require_effectful, require_setup};
use kivact_core::state::{Binding, Value, Var};
use kivact_core::state::{IntoValue, callback};
use kivact_core::view::{Handled, HandlerResult, Key, KeyEvent};
use std::hash::Hash;
use std::panic::Location;
use std::rc::Rc;

/// A Copy list handle over caller-owned item order and authoritative selection.
/// Attach `list()` and `item(key)` to caller-rendered nodes or canvas regions.
/// Filtering never clears selection; only mounted eligible items take roving focus.
/// The policy is given to [`use_list`]; paging, leave/scroll callbacks and the
/// anchor may be set before or after the parts are made. Boundary wrapping
/// applies to explicit navigation; active removal always clamps from its old
/// position. Reactive inputs and caller binding writes continue to update
/// existing Parts.
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
///
/// The `policy` holds the navigation and the [`ListMode`](super::ListMode) of
/// the list; `ListPolicy::axis` of `Some(Orientation::Unspecified)` is a usage
/// error (use `None` for all four arrow keys). Setup only.
#[track_caller]
pub fn use_list<K: Clone + Eq + Hash + 'static>(
    items: impl IntoValue<Vec<ListItem<K>>>,
    selection: ListSelection<K>,
    policy: ListPolicy,
) -> List<K> {
    check_list_axis(policy.axis, "use_list");
    List {
        state: state::create(items.into_value(), selection, policy, Location::caller()),
    }
}

impl<K: Clone + Eq + Hash + 'static> List<K> {
    #[track_caller]
    fn state(self) -> Rc<State<K>> {
        self.state.get()
    }
    /// Change the policy of a list whose mode stays the same (tabs and
    /// trees adjust the axis and the activation after `use_list`). Setup only.
    #[track_caller]
    pub(crate) fn retune(self, change: impl FnOnce(&mut ListPolicy)) {
        let state = self.state();
        require_setup("List::retune");
        let mut policy = state.policy();
        change(&mut policy);
        check_list_axis(policy.axis, "List::retune");
        assert!(
            policy.mode == state.mode,
            "invariant: a list's mode is fixed by use_list"
        );
        state.policy.set(policy);
    }
    /// Set the Select-policy Page Up/Down distance; values below one count as one.
    /// Setup only.
    #[track_caller]
    pub fn page_size(self, rows: impl IntoValue<usize>) -> Self {
        let state = self.state();
        require_setup("List::page_size");
        state.page.set(rows.into_value());
        self
    }
    /// Prefer another enabled key when the current logical item disappears.
    /// Setup only.
    #[track_caller]
    pub fn on_leave(self, f: impl Fn(&K) -> Option<K> + 'static) -> Self {
        let state = self.state();
        require_setup("List::on_leave");
        *state.leave.borrow_mut() = Some(Rc::new(f));
        self
    }
    /// Scroll through the caller and permit pending focus on unmounted keys.
    /// An `Err` from `f` goes to the nearest error handler. Setup only.
    #[track_caller]
    pub fn scroll_with<R: HandlerResult>(self, f: impl Fn(K) -> R + 'static) -> Self {
        let state = self.state();
        require_setup("List::scroll_with");
        *state.scroll.borrow_mut() = Some(callback(f));
        self
    }
    /// Share an authoritative range anchor with other list instances.
    /// Setup only; later binding writes remain reactive.
    #[track_caller]
    pub fn anchor(self, anchor: impl Into<Binding<Option<K>>>) -> Self {
        let state = self.state();
        require_setup("List::anchor");
        let anchor = anchor.into();
        state.anchor.update(|current| *current = anchor);
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
        state.anchor.get_untracked().set(None);
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
    /// Limit selection gestures and select-all while preserving focus and navigation.
    /// Setup only.
    pub fn selectable(self, test: impl Fn(&K) -> bool + 'static) -> Self {
        require_setup("List::selectable");
        self.state()
            .selectable
            .update(|current| *current = Some(std::rc::Rc::new(test)));
        self
    }

    /// An item's label, selection, exact input state and mounted roving navigation.
    #[track_caller]
    pub fn item(self, key: K) -> Part<ListItemState> {
        self.item_with_disabled_focus(key, false)
    }

    #[track_caller]
    pub(crate) fn item_with_disabled_focus(self, key: K, retain: bool) -> Part<ListItemState> {
        let state = self.state();
        require_setup("List::item");
        parts::item(state, key, Location::caller(), retain)
    }

    pub(crate) fn retain_focus(self, enabled: bool) {
        self.state().retained_focus.set(enabled);
    }

    pub(crate) fn active_ref(self) -> Option<kivact_core::view::NodeRef> {
        let state = self.state();
        let key = state.active.get()?;
        state
            .roving
            .with_members(|members| members.get(&key).map(|member| member.node))
    }

    pub(crate) fn highlight_mounted(self, key: K) {
        let state = self.state();
        if state.target(&key).is_some() {
            state.roving.requested.set(Some(key));
        }
    }

    pub(crate) fn reset_highlight(self) {
        let state = self.state();
        state.roving.requested.set(None);
        state.typed.get().set(typeahead::Typeahead::default());
    }

    pub(crate) fn route_focused_key(
        self,
        key: &K,
        event: KeyEvent,
        command: Option<Key>,
    ) -> Handled {
        input::key(&self.state(), key, event, command)
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
        let state = self.state();
        require_setup("List::shared_typeahead");
        state.typed.set(typed);
        self
    }
}
