//! Role: Application menus with nesting and typeahead.
//!
//! Unstyled behavior and semantics of an application menu: a menu button, a
//! context menu, nested submenus, typeahead and keyed entries.
//!
//! [`MenuEntry`] is public plain data (items, submenus, separators, radio
//! items, headings and groups, with label, shortcut, icon, description,
//! disabled, destructive, inset and keep-open; the constructor chooses item,
//! check item, radio item, submenu, separator, heading or group), and
//! [`MenuEntry::map_key`] maps the keys of the children too. A level's
//! [`MenuApi::entries`] are `(MenuRow, MenuEntry)` pairs: a group expands in
//! place into its heading (`MenuRow::Label`) and rows of the same level, and
//! separators and headings can be drawn keyed too. A radio item, like a
//! check item, holds no state and reports its key. `destructive` is read only
//! by the drawing, through [`MenuItemState`]. The look, the placement and the
//! check and submenu marks are the caller's; this module has no look and
//! draws nothing.
//!
//! A submenu is a child level of the same session; its `shown`, `origin`
//! and `show` mount the content in a new owner while it is open. A level
//! moves with the list's menu policy (vertical, wrapping, manual activation)
//! plus typeahead, and hovering a submenu item opens it after a delay.
//! A context menu is the same [`MenuApi`] through
//! [`MenuApi::context_trigger`]; there is no separate context menu type. A
//! pointer request reports `MenuOrigin::Pointer` (a window point) and keys
//! and assistive technology report `MenuOrigin::Anchor`, so the caller picks
//! `overlay_at` or an anchored overlay; the focus held before opening comes
//! back on a best-effort basis after Escape, a choice or assistive
//! technology, while Tab and an outside press keep their destination.
//! [`MenuApi::open`] and [`MenuApi::close`] let a component that switches between
//! several menus (a menu bar) drive them.
//!
//! Menus and selects use the list's private mounted selection path, so an
//! option that disappears with the choice gets no extra focus request, and
//! every level of one menu shares one private typeahead slot. The focus
//! after closing is the popup's and the overlay's (see `crate::popup`).

mod handle;
mod level;
mod order;
mod session;
mod state;

pub use handle::{MenuApi, use_menu};
pub use level::SubmenuTiming;
pub use state::{MenuCheck, MenuEntry, MenuItemKind, MenuItemState, MenuOrigin, MenuRow};
