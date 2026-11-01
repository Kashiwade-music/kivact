//! Role: Application menus with nesting and typeahead.
//!
//! Unstyled behavior and semantics of an application menu: a menu button, a
//! context menu, nested submenus, typeahead and keyed entries.
//!
//! [`MenuEntry`] is public plain data (items, submenus, separators, radio
//! items, headings and groups, with label, shortcut, icon, description,
//! disabled, checked, destructive, inset and keep-open), and
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
//! plus typeahead, and hovering a submenu item opens it after the
//! disclosure's 140 ms delay. A context menu is the same [`MenuApi`] through
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

mod level;
mod order;
mod session;
mod state;

use kivact_core::view::HandlerResult;
use std::hash::Hash;

use kivact_core::primitive::{Part, part};
use kivact_core::view::{
    A11yAction, Handled, IntoValue, IntoView, Key, KeyEvent, MouseButton, PointerEvent,
    PointerPhase, View, callback, dynamic,
};
use kivact_core::{Value, memo};

use super::focus_return::PopupCloseReason;
use super::popup::PopupState;
use level::Level;
use state::{Edge, Session};
pub use state::{MenuEntry, MenuItemState, MenuOrigin, MenuRow};

/// An application menu over keyed [`MenuEntry`] rows, without a look of its own.
///
/// `MenuApi` adds the behavior and semantics of a menu to parts the caller
/// draws. The caller keeps the entries (a value, so they can change while the
/// menu is open) and a callback that receives the key of the chosen item; the
/// menu holds no state besides which submenus are open.
///
/// Attach [`trigger`](Self::trigger) to a menu button, or
/// [`context_trigger`](Self::context_trigger) to an area that opens it with a
/// right click; give one menu one of the two. [`show`](Self::show) mounts the
/// caller's `overlay(..)` with the [`content`](Self::content) part while the
/// menu is open, and the caller renders the rows of a level with
/// `for_each(menu.entries(), |(row, _)| row.clone(), ..)`, attaching
/// [`item`](Self::item) to each item row and drawing separators itself.
/// [`submenu`](Self::submenu) returns the handle of a submenu level, with its
/// own `entries`, `show`, `surface` and `item`; its overlay is anchored to
/// its item's node, after it.
///
/// Keys inside the menu: Up and Down move through the enabled items of a level
/// and wrap, Home and End jump to its ends, a letter key moves to the next
/// item whose label starts with it (letters typed within 500 ms of each other
/// are matched together), Enter and Space choose, Right opens a submenu and
/// focuses its first item, Left closes a submenu and returns to its item,
/// Escape closes the current level (the whole menu at the top level), and Tab
/// moves focus out of the menu, which closes it. Hovering an item focuses it;
/// hovering a submenu item opens the submenu after 140 ms. Choosing an item
/// (disabled items and submenu items cannot be chosen) calls the callback with
/// its key, closes every level and gives focus back to the trigger, or for a
/// context menu to the element that had it before the menu opened; an entry
/// marked [`keep_open`](MenuEntry::keep_open) leaves the menu open on it. If the
/// callback fails the error is reported and the menu stays open.
///
/// When the entries change while the menu is open, rows are matched by key,
/// never by position. A submenu whose item disappeared or became disabled
/// closes; so does the level whose focused item disappeared (the whole menu
/// at the top level), and focus returns to the parent item. The callback is
/// only ever called with a key of the current entries.
///
/// Semantics: a surface is `Role::Menu` (the caller names it, usually like
/// the trigger or the submenu item), items are `Role::MenuItem`
/// (`Role::MenuItemCheckBox` with a checked state) named by their label, a
/// shortcut is the item's description, and a submenu item publishes whether
/// it is expanded. A separator is the caller's `Role::Separator` node.
///
/// ```no_run
/// # mod kivact {
/// #     //! Role: Supply facade paths for this documentation example.
/// #     pub use kivact_core::{prelude, view};
/// #     pub extern crate kivact_interaction as interaction;
/// # }
/// use kivact::interaction::{MenuEntry, MenuRow, use_menu};
/// use kivact_core::primitive::NodeExt;
/// use kivact::prelude::*;
/// use kivact::view::{Placement, overlay};
///
/// fn actions(last: Signal<Option<&'static str>>) -> impl IntoView {
///     let entries = vec![
///         MenuEntry::item("save", "Save").shortcut("Ctrl S"),
///         MenuEntry::item("open", "Open"),
///     ];
///     let file = use_menu(entries, move |key| last.set(Some(key)));
///     let trigger = file.trigger();
///     let anchor = trigger.node_ref();
///     (
///         node().part(trigger).name("File").child(text("File")),
///         file.show(move || {
///             let rows = for_each(file.entries(), |(row, _)| row.clone(), move |row, entry| {
///                 match row {
///                     MenuRow::Item { key, .. } => node()
///                         .part(file.item(key))
///                         .child(text(move || entry.with(|(_, e)| e.label())))
///                         .into_view(),
///                     MenuRow::Separator(_) | MenuRow::Label(_) => node().role(Role::Separator).into_view(),
///                 }
///             });
///             overlay(anchor, Placement::Below, column(rows))
///                 .part(file.content())
///                 .name("File")
///         }),
///     )
/// }
/// ```
pub struct MenuApi<K: Clone + Eq + Hash + 'static> {
    session: Session<K>,
    level: Level<K>,
}

impl<K: Clone + Eq + Hash + 'static> Clone for MenuApi<K> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<K: Clone + Eq + Hash + 'static> Copy for MenuApi<K> {}

/// A menu over `entries` that calls `choose` with the key of a chosen item.
/// Setup only.
#[track_caller]
pub fn use_menu<K: Clone + Eq + Hash + 'static, R: HandlerResult>(
    entries: impl IntoValue<Vec<MenuEntry<K>>>,
    choose: impl Fn(K) -> R + 'static,
) -> MenuApi<K> {
    let session = Session::new(entries.into_value(), callback(choose));
    let level = Level::new(session, Vec::new(), session.shown());
    MenuApi { session, level }
}

impl<K: Clone + Eq + Hash + 'static> MenuApi<K> {
    /// The rows of this level, each with the identity to key its `for_each`
    /// by. A submenu level has no rows once its item is gone.
    pub fn entries(self) -> Value<Vec<(MenuRow<K>, MenuEntry<K>)>> {
        self.level.rows
    }

    /// Whether this level is open: the menu for the top level, the submenu's
    /// item expanded for a submenu level.
    pub fn shown(self) -> Value<bool> {
        self.level.shown
    }

    /// Where the open menu was asked to appear; `None` while it is closed.
    /// A context menu reads it to place its overlay at the pointer.
    pub fn origin(self) -> Value<Option<MenuOrigin>> {
        self.session.origin.into_value()
    }

    /// Open the menu as the trigger's ArrowDown does: it opens on its first
    /// enabled item, and the focus later returns to the trigger. Does nothing
    /// while the menu is open. Meant for a component that opens one of several
    /// menus itself, such as a menu bar moving between its menus. Top level
    /// only.
    #[track_caller]
    pub fn open(self) {
        if self.session.shown().get_untracked() {
            return;
        }
        self.session.request(0, Edge::First);
        self.session.open_from_button();
    }

    /// Close every level of the menu without moving the focus, as
    /// [`PopupCloseReason::Programmatic`] does. Does nothing while the menu
    /// is closed. Top level only.
    #[track_caller]
    pub fn close(self) {
        self.session.close(PopupCloseReason::Programmatic);
    }

    /// The handle of the submenu that the item `key` of this level opens.
    /// Setup only, usually in that item's row.
    #[track_caller]
    pub fn submenu(self, key: K) -> MenuApi<K> {
        let depth = self.level.depth;
        let path = self.session.path;
        let held = key.clone();
        let shown = memo(move || path.with(|path| path.get(depth) == Some(&held)));
        let mut prefix = self.level.prefix.get_untracked();
        prefix.push(key);
        MenuApi {
            session: self.session,
            level: Level::new(self.session, prefix, shown),
        }
    }

    /// The menu button: a `Role::Button` publishing `expanded` and `controls`.
    ///
    /// Enter, Space, a click and an assistive technology's Activate open the
    /// menu on its first enabled item (and close it again); ArrowDown does the
    /// same and ArrowUp opens on the last one. Expand opens it and Collapse
    /// closes it. Releasing Tab on the trigger closes it. These run after the
    /// caller's own handlers. The part binds the reference the top-level
    /// overlay is anchored to and the focus returns to. Top level only.
    #[track_caller]
    pub fn trigger(self) -> Part<PopupState> {
        let session = self.session;
        session
            .popup
            .trigger()
            .on_key(move |event: KeyEvent| {
                let m = event.modifiers;
                if m.ctrl || m.alt || m.meta {
                    return Handled::No;
                }
                match event.key {
                    Key::ArrowDown => session.request(0, Edge::First),
                    Key::ArrowUp => session.request(0, Edge::Last),
                    _ => return Handled::No,
                }
                session.open_from_button();
                Handled::Yes
            })
            .on_press(move || {
                if session.shown().get() {
                    session.close(PopupCloseReason::Programmatic);
                } else {
                    session.open_from_button();
                }
            })
            .on_a11y(A11yAction::Expand, move || session.open_from_button())
            .on_a11y(A11yAction::Collapse, move || {
                session.close(PopupCloseReason::Accessibility)
            })
    }

    /// The area of a context menu.
    ///
    /// Four requests open the menu: a press of the secondary (right) button
    /// (when the button is released; when it is pressed on macOS), Shift+F10,
    /// the Menu key (`Key::ContextMenu`) and an assistive technology's
    /// `ShowMenu` action. The first sets [`origin`](Self::origin) to
    /// `MenuOrigin::Pointer`, the others to `MenuOrigin::Anchor`, where the
    /// caller places the overlay at the area (this part's `node_ref`). The
    /// area gets no `expanded`, and ArrowDown does not open it, so arrow
    /// navigation inside the area is not disturbed. After Escape, a choice and
    /// assistive technology's Dismiss the focus returns to the element that
    /// had it before the menu opened.
    ///
    /// Assistive technology reaches the area only if it has a role, and keys
    /// reach it only while it or a descendant has the focus, so give it a
    /// role and make it focusable. A descendant with its own pointer handler
    /// receives the pointer events over it and does not open the menu. Unhandled
    /// context requests from regions on the same canvas may delegate to the
    /// area's menu; separate descendant pointer listeners keep their own events. Top
    /// level only.
    #[track_caller]
    pub fn context_trigger(self) -> Part<PopupState> {
        let session = self.session;
        let open = session.shown();
        part(move |input| PopupState {
            input,
            open: open.get(),
        })
        .region_events()
        .on_pointer(move |event: PointerEvent| {
            // The overlay is anchored to the area, and the host does not
            // count a press on its anchor as outside it.
            if event.phase == PointerPhase::Down {
                session.close(PopupCloseReason::OutsidePress);
            }
            let phase = if cfg!(target_os = "macos") {
                PointerPhase::Down
            } else {
                PointerPhase::Up
            };
            if event.button == Some(MouseButton::Secondary) && event.phase == phase {
                session.open_at(MenuOrigin::Pointer(event.window_pos));
            }
            Handled::No
        })
        .on_key(move |event: KeyEvent| {
            let m = event.modifiers;
            if event.repeat || m.ctrl || m.alt || m.meta {
                return Handled::No;
            }
            let asked = match event.key {
                Key::ContextMenu => true,
                Key::F(10) => m.shift,
                _ => false,
            };
            if asked {
                session.open_at(MenuOrigin::Anchor);
                Handled::Yes
            } else {
                Handled::No
            }
        })
        .on_a11y(A11yAction::ShowMenu, move || {
            session.open_at(MenuOrigin::Anchor)
        })
    }

    /// The floating surface of this level, for the caller's `overlay(..)`.
    ///
    /// It is a `Role::Menu` and carries the level's keys (Escape, Left,
    /// Right and Tab) and its focus requests. At the top level it is also the
    /// popup's surface: an outside press and assistive technology's Dismiss
    /// close the menu. The name and placement stay the caller's.
    #[track_caller]
    pub fn content(self) -> Part<PopupState> {
        let session = self.session;
        let level = self.level;
        let open = level.shown;
        let base = if level.depth == 0 {
            session.popup.content()
        } else {
            part(move |input| PopupState {
                input,
                open: open.get(),
            })
        };
        level.content(base)
    }

    /// The item `key` of this level: a `Role::MenuItem`, or
    /// `Role::MenuItemCheckBox` for an entry with a checked state, named by
    /// its label and described by its shortcut. Its kind is read once, so key
    /// the rows by [`MenuRow`]. A submenu item publishes `expanded` (and
    /// `selected` while its submenu is open) and opens the submenu instead of
    /// being chosen.
    #[track_caller]
    pub fn item(self, key: K) -> Part<MenuItemState> {
        self.level.item(key)
    }

    /// Mount the caller's content while this level is open, in a fresh owner
    /// on every opening. At the top level a request of another
    /// [`origin`](Self::origin) kind (pointer or anchor) builds it again, so
    /// the factory can read `origin()` untracked to choose its overlay.
    ///
    /// Mount a menu's content with this method, not with
    /// `show(menu.shown(), ..)` as a dialog's or a popup's: the rebuild on a
    /// new origin kind is the menu's own. It is the one exception to
    /// mounting an overlay's content with the `show` function.
    #[track_caller]
    pub fn show<V: IntoView>(self, factory: impl Fn() -> V + 'static) -> View {
        let shown = self.level.shown;
        if self.level.depth > 0 {
            return kivact_core::view::show(shown, factory).into_view();
        }
        let origin = self.session.origin;
        let at_pointer = memo(move || matches!(origin.get(), Some(MenuOrigin::Pointer(_))));
        let factory = std::rc::Rc::new(factory);
        kivact_core::view::show(shown, move || {
            let factory = factory.clone();
            dynamic(at_pointer, move |_| factory().into_view())
        })
        .into_view()
    }
}
