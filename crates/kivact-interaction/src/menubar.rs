//! Role: A menu bar moves the open menu among a row of menu triggers.
//!
//! Unstyled behavior and semantics of a menu bar: the row of triggers of
//! several [`MenuApi`]s, where the arrows move the focus along the triggers
//! and, while a menu is open, move the open menu with them.
//!
//! [`use_menubar`] is a Copy handle over the titles of the triggers and the
//! menus they open. Its parts are `bar()`, a `Role::MenuBar` for the row, and
//! `item(at)` for the trigger of the menu `at`, a `Role::MenuItem` that is
//! also the menu's anchor and the bar's tab stop. The caller attaches each
//! menu's own parts (`trigger`, `content`, `item`) as for a single menu.
//!
//! Left and Right (mirrored in a right-to-left
//! [`direction`](kivact_core::view::provide_direction)) move the focus
//! between the triggers and wrap at the ends. While a menu is open, Left and
//! Right also close it and open its neighbour, so a menu stays open while
//! moving. Home and End are the list's: they only move the focus to the first
//! and the last trigger. With a menu open, resting the pointer on another
//! trigger opens that menu.

use kivact_core::custom::{Part, PartState, part};
use kivact_core::state::IntoBinding;
use kivact_core::state::IntoValue;
use kivact_core::state::{Value, Var, memo, signal, var};
use kivact_core::view::{
    Handled, Key, KeyEvent, Orientation, PointerPhase, ReadingDirection, Role, use_direction,
};

use super::{
    ListActivation, ListApi, ListBoundary, ListItem, ListItemState, ListMode, ListOptions, MenuApi,
    Selection, use_list,
};

/// A row of menu triggers that moves the open menu, without a look of its own.
///
/// ```no_run
/// # mod kivact {
/// #     //! Role: Supply facade paths for this documentation example.
/// # pub use kivact_core::{view, state, style};
/// # pub mod prelude {
/// # //! Role: Supply the canonical imports used by this documentation example.
/// # pub use kivact_core::state::{Binding, Callback, IntoBinding, IntoText, IntoValue, Signal, Value, binding, effect, memo, on_cleanup, on_mount, signal};
/// # pub use kivact_core::view::{EditPhase, Component, IntoView, NodeBuilder, Orientation, Role, column, for_each, node, node_ref, row, show, text};
/// # pub use kivact_core::style::{ColorToken, Cursor, Style, style};
/// # }
/// #     pub extern crate kivact_interaction as interaction;
/// # }
/// use kivact::interaction::{MenuEntry, use_menu, use_menubar};
/// use kivact::prelude::*;
/// use kivact_core::view::NodeBuilder;
///
/// fn bar() -> impl IntoView {
///     let file = use_menu(vec![MenuEntry::item("new", "New")]).on_activate(|_| {});
///     let edit = use_menu(vec![MenuEntry::item("undo", "Undo")]).on_activate(|_| {});
///     let titles = vec!["File".to_string(), "Edit".to_string()];
///     let menubar = use_menubar(titles, vec![file, edit]);
///     row((
///         node().part(file.trigger()).part(menubar.item(0)).name("File").child(text("File")),
///         node().part(edit.trigger()).part(menubar.item(1)).name("Edit").child(text("Edit")),
///     ))
///     .part(menubar.bar())
/// }
/// ```
pub struct MenubarApi<K: kivact_core::state::ItemKey> {
    list: ListApi<usize>,
    menus: Var<Vec<MenuApi<K>>>,
    rtl: bool,
}

impl<K: kivact_core::state::ItemKey> Clone for MenubarApi<K> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<K: kivact_core::state::ItemKey> Copy for MenubarApi<K> {}

/// A menu bar over the triggers titled `titles` that open `menus`, in order.
/// The number of menus is read once, when the bar is built; the titles follow
/// the value. Setup only.
#[track_caller]
pub fn use_menubar<K: kivact_core::state::ItemKey>(
    titles: impl IntoValue<Vec<String>>,
    menus: Vec<MenuApi<K>>,
) -> MenubarApi<K> {
    let titles = titles.into_value();
    let items: Value<Vec<ListItem<usize>>> = memo(move || {
        titles.with(|titles| {
            titles
                .iter()
                .enumerate()
                .map(|(at, title)| ListItem::new(at, title.clone()))
                .collect()
        })
    });
    // The triggers are a horizontal list: the arrows move the focus along it.
    let focus = signal(None::<usize>);
    let list = use_list(
        items,
        Selection::Single(focus.into_binding()),
        ListOptions {
            axis: Some(Orientation::Horizontal),
            boundary: ListBoundary::Wrap,
            activation: ListActivation::Manual,
            mode: ListMode::Activate,
            ..ListOptions::default()
        },
    );
    MenubarApi {
        list,
        menus: var(menus),
        rtl: use_direction().get() == ReadingDirection::Rtl,
    }
}

impl<K: kivact_core::state::ItemKey> MenubarApi<K> {
    /// Drawing state of the mounted menu bar.
    pub fn bar_state(self) -> Value<PartState> {
        self.list.list_state()
    }
    /// Drawing state of a mounted top-level trigger.
    pub fn item_state(self, at: usize) -> Value<ListItemState> {
        self.list.item_state(at)
    }

    /// The row of triggers: a `Role::MenuBar`. While a menu is open, the
    /// arrows along the bar move the open menu; between closed triggers the
    /// list moves the focus. Setup only.
    #[track_caller]
    pub fn bar(self) -> Part<PartState> {
        let MenubarApi { list, menus, rtl } = self;
        let input = part(|input| input);
        let part_state = input.state();
        let target = kivact_core::view::node_ref();
        Part::new(part_state, move |host| {
            host.part(input)
                .node_ref(target)
                .role(Role::MenuBar)
                .on_key(move |event: KeyEvent| {
                    let m = event.modifiers();
                    if m.ctrl || m.alt || m.meta || m.shift {
                        return Handled::No;
                    }
                    let forward = match event.key() {
                        Key::ArrowRight => !rtl,
                        Key::ArrowLeft => rtl,
                        _ => return Handled::No,
                    };
                    let moved = menus.with(|menus| {
                        let len = menus.len();
                        let open = menus
                            .iter()
                            .position(|menu| menu.is_open().get_untracked())?;
                        let next = if forward {
                            (open + 1) % len
                        } else {
                            (open + len - 1) % len
                        };
                        if next != open {
                            list.focus(next);
                            menus[open].close();
                            menus[next].open();
                        }
                        Some(())
                    });
                    if moved.is_some() {
                        Handled::Yes
                    } else {
                        Handled::No
                    }
                })
        })
    }

    /// The trigger of the menu `at`: a `Role::MenuItem` and an item of the
    /// bar's list. Resting the pointer on it while another menu is open moves
    /// that menu here. Attach the menu's own `trigger` to the same node
    /// before this part, so that the item's role is the one the node has (a
    /// trigger is a `Role::Button`); the caller names it. Setup only.
    #[track_caller]
    pub fn item(self, at: usize) -> Part<ListItemState> {
        let MenubarApi { list, menus, .. } = self;
        let base = list.item(at);
        let part_state = base.state();
        Part::new(part_state, move |host| {
            host.part(base)
                .role(Role::MenuItem)
                .selected(false)
                .on_pointer(move |event| {
                    if event.phase() == PointerPhase::Enter {
                        menus.with(|menus| {
                            let Some(this) = menus.get(at) else { return };
                            if this.is_open().get_untracked() {
                                return;
                            }
                            let mut moved = false;
                            for other in menus.iter() {
                                if other.is_open().get_untracked() {
                                    other.close();
                                    moved = true;
                                }
                            }
                            if moved {
                                // The trigger of the open menu is the bar's tab
                                // stop, which the focus returns to when the menu
                                // closes.
                                list.focus(at);
                                this.open();
                            }
                        });
                    }
                    Handled::No
                })
        })
    }
}
