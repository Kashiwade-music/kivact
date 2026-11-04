//! Role: Roving focus registry shared by list, table and menu levels.
//!
//! One tab stop moves among mounted members. This registry holds what every
//! such controller needs: the mounted members, the member the keyboard or
//! pointer last moved to (`requested`), the focus as last observed and the
//! focus of a removed member. [`Roving::reconcile`] follows focus that
//! reaches a member by other means and hands the focus of a removed or
//! no-longer-focusable member to the tab stop. The owner keeps its domain
//! rules: order, enabled state, selection, typeahead, virtual or pending
//! requests, and the scrolling that goes with a focus.
//!
//! No registry borrow is held while a node is asked to take the focus.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use kivact_core::custom::{NodeRefExt, NodeToken};
use kivact_core::state::{Signal, Value, signal, untrack};
use kivact_core::view::NodeRef;

/// A mounted member and the constraints that decide whether it can take focus.
#[derive(Clone, Copy)]
pub(crate) struct Member {
    pub(crate) node: NodeRef,
    pub(crate) eligible: Value<bool>,
    pub(crate) disabled: Value<bool>,
    pub(crate) hidden: Value<bool>,
}

impl Member {
    /// Whether the member can take the focus now (tracked).
    pub(crate) fn available(self) -> bool {
        self.eligible.get() && !self.disabled.get() && !self.hidden.get() && self.node.is_visible()
    }

    pub(crate) fn available_untracked(self) -> bool {
        untrack(|| self.available())
    }
}

/// What one reconciliation pass is told by its owner.
pub(crate) struct Observation<'a, Id> {
    /// The focused node as the owner just read it.
    pub(crate) current: Option<NodeToken>,
    /// The member holding the tab stop.
    pub(crate) active: Option<&'a Id>,
    /// Whether focus that reaches a member by other means may move
    /// `requested` (false while the owner has a request of its own pending).
    pub(crate) adopt: bool,
    /// Whether focus resting on a member the owner's `enabled` rule or its
    /// availability refuses is handed back to `active`, instead of counting
    /// as the member's focus.
    pub(crate) reject_refused: bool,
}

/// A clone shares the registry and the focus bookkeeping, so a memo can follow
/// the members without borrowing its owner's state.
#[derive(Clone)]
pub(crate) struct Roving<Id: kivact_core::state::ItemKey> {
    /// The mounted members; changed only through `insert` and `remove`.
    members: Rc<RefCell<HashMap<Id, Member>>>,
    /// Notified by every insertion and removal of a member.
    changed: Signal<()>,
    /// The member the keyboard or pointer last moved to.
    pub(crate) requested: Signal<Option<Id>>,
    /// The focus as last seen by `reconcile`.
    observed: Rc<Cell<Option<NodeToken>>>,
    /// A focused member was removed; the tab stop takes the focus once it settles.
    pub(crate) restore_from: Rc<Cell<Option<NodeToken>>>,
}

impl<Id: kivact_core::state::ItemKey> Roving<Id> {
    /// Setup only.
    #[track_caller]
    pub(crate) fn new() -> Self {
        Self {
            members: Rc::new(RefCell::new(HashMap::new())),
            changed: signal(()),
            requested: signal(None),
            observed: Rc::new(Cell::new(None)),
            restore_from: Rc::new(Cell::new(None)),
        }
    }

    /// Follow the membership: a memo or an effect that calls it reruns when a
    /// member is inserted or removed (tracked).
    pub(crate) fn track(&self) {
        self.changed.get();
    }

    pub(crate) fn contains(&self, id: &Id) -> bool {
        self.members.borrow().contains_key(id)
    }

    /// Add a mounted member and notify the followers of the membership.
    pub(crate) fn insert(&self, id: Id, member: Member) {
        self.members.borrow_mut().insert(id, member);
        self.changed.notify();
    }

    /// Remove `id` if its entry is still `node` (the same mounted reference);
    /// whether it was removed. The followers are notified only then.
    pub(crate) fn remove(&self, id: &Id, node: NodeRef) -> bool {
        let removed = {
            let mut members = self.members.borrow_mut();
            if members
                .get(id)
                .is_some_and(|entry| entry.node.same_ref(node))
            {
                members.remove(id);
                true
            } else {
                false
            }
        };
        if removed {
            self.changed.notify();
        }
        removed
    }

    /// Read the members; no registry borrow outlives `f`.
    pub(crate) fn with_members<R>(&self, f: impl FnOnce(&HashMap<Id, Member>) -> R) -> R {
        f(&self.members.borrow())
    }

    /// The mounted node of `id` if the owner's `enabled` rule admits it and it
    /// can take the focus now (untracked).
    pub(crate) fn target(&self, id: &Id, enabled: impl Fn(&Id) -> bool) -> Option<NodeRef> {
        if !enabled(id) {
            return None;
        }
        let member = self.with_members(|members| members.get(id).copied());
        member
            .filter(|member| member.available_untracked())
            .map(|member| member.node)
    }

    /// Ask for the focus on `id`, only if [`target`](Self::target) finds a node.
    /// It neither moves `requested` nor scrolls: the owner decides both.
    pub(crate) fn focus(&self, id: &Id, enabled: impl Fn(&Id) -> bool) -> Option<NodeRef> {
        let node = self.target(id, enabled)?;
        node.focus();
        Some(node)
    }

    /// The member `token` is the focus of, with its entry.
    fn member_of(&self, token: Option<NodeToken>) -> Option<(Id, Member)> {
        let token = token?;
        self.with_members(|members| {
            members
                .iter()
                .find(|(_, member)| member.node.mounted_node() == Some(token))
                .map(|(id, member)| (id.clone(), *member))
        })
    }

    /// Follow focus that reaches a member by other means, and give the focus
    /// of a removed or refused member to the tab stop. It returns the node it
    /// asked to take the focus, if any, so the owner can scroll to it.
    pub(crate) fn reconcile(
        &self,
        observation: Observation<'_, Id>,
        enabled: impl Fn(&Id) -> bool,
    ) -> Option<NodeRef> {
        let Observation {
            current,
            active,
            adopt,
            reject_refused,
        } = observation;
        let previous = self.observed.replace(current);
        let admitted = |id: &Id, member: &Member| enabled(id) && member.available_untracked();
        let focused = self.member_of(current);
        let refused = focused
            .as_ref()
            .is_some_and(|(id, member)| reject_refused && !admitted(id, member));
        if let Some((id, _)) = &focused
            && !refused
        {
            if adopt && current != previous && active != Some(id) {
                self.requested.set(Some(id.clone()));
            }
            self.restore_from.set(None);
            return None;
        }
        let removed = self.restore_from.take();
        let left_refused = reject_refused
            && current.is_none()
            && self
                .member_of(previous)
                .is_some_and(|(id, member)| !admitted(&id, &member));
        if refused
            || left_refused
            || removed.is_some_and(|removed| current.is_none() || current == Some(removed))
        {
            return self.focus(active?, enabled);
        }
        None
    }
}
