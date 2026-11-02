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
//! [`ListPolicy`] holds the axis, boundary, activation, mode, typeahead
//! and range policy. The default is vertical, clamped, automatic, Select and
//! no typeahead; the `tabs`, `radio`, `menu` and `listbox` presets use
//! Activate, and `menu` and `listbox` add typeahead. With Select a primary
//! press or Space selects with the gesture its modifiers ask for, Enter does
//! nothing, and only Select pages with PageUp/PageDown. Activate chooses once,
//! on the native release, Enter, Space or assistive technology. Wrapping
//! applies to explicit moves only. When the active item disappears, Select
//! clamps from its old position and Activate recovers to a valid selected key
//! or the first one. The policy is the third argument of `use_list`;
//! `page_size`, `on_leave`, `scroll_with` and `anchor` may be set before or
//! after the parts are made, and reactive items and external binding writes
//! reach existing parts.
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
mod handle;
mod input;
mod order;
mod parts;
mod selection;
mod state;
pub(crate) mod typeahead;

pub use data::{
    ListActivation, ListBoundary, ListCommandModifier, ListGesture, ListItem, ListItemState,
    ListMissingAnchor, ListMode, ListPolicy, ListRangeMode, ListRangePolicy, ListSelection,
};
pub use handle::{List, use_list};
