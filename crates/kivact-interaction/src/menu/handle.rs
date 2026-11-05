//! Role: The `MenuApi` handle: open state, triggers, content, items and submenus.

use super::level::Level;
use super::session::Session;
use super::state::Edge;
use super::state::{MenuEntry, MenuItemState, MenuOrigin, MenuRow};
use crate::focus_return::PopupCloseReason;
use crate::popup::PopupState;
use kivact_core::ErrorKind;
use kivact_core::custom::raise;
use kivact_core::custom::{Part, PartState, part};
use kivact_core::platform::{ContextMenuTrigger, capabilities};
use kivact_core::state::{Callback, Value, memo};
use kivact_core::state::{IntoBinding, IntoValue, callback, signal};
use kivact_core::view::HandlerResult;
use kivact_core::view::{
    A11yAction, Handled, IntoView, Key, KeyEvent, MouseButton, PointerEvent, PointerPhase, View,
    dynamic,
};

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
/// Keys inside the menu: Up and Down move through the focusable items of a level
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
/// # pub use kivact_core::{view, state, style};
/// # pub mod prelude {
/// # //! Role: Supply the canonical imports used by this documentation example.
/// # pub use kivact_core::state::{Binding, Callback, IntoBinding, IntoText, IntoValue, Signal, Value, binding, effect, memo, on_cleanup, on_mount, signal};
/// # pub use kivact_core::view::{EditPhase, Component, IntoView, NodeBuilder, Orientation, Role, column, for_each, node, node_ref, row, show, text};
/// # pub use kivact_core::style::{ColorToken, Cursor, Style, style};
/// # }
/// #     pub extern crate kivact_interaction as interaction;
/// # }
/// use kivact::interaction::{MenuEntry, MenuRow, use_menu};
/// use kivact_core::view::NodeBuilder;
/// use kivact::prelude::*;
/// use kivact::view::{Placement, overlay};
///
/// fn actions(last: Signal<Option<&'static str>>) -> impl IntoView {
///     let entries = vec![
///         MenuEntry::item("save", "Save").with_shortcut("Ctrl S"),
///         MenuEntry::item("open", "Open"),
///     ];
///     let file = use_menu(entries).on_activate(move |key| last.set(Some(key)));
///     let trigger = file.trigger();
///     let anchor = node_ref();
///     (
///         node().node_ref(anchor).part(trigger).name("File").child(text("File")),
///         file.show(move || {
///             let rows = for_each(file.entries(), |(row, _)| row.clone(), move |entry| {
///                 let row = entry.with(|(row, _)| row.clone());
///                 match row {
///                     MenuRow::Item { key, .. } => node()
///                         .part(file.item(key))
///                         .child(text(move || entry.with(|(_, e)| e.label())))
///                         .into_view(),
///                     MenuRow::Separator(_) | MenuRow::Label(_) => node().role(Role::Separator).into_view(),
///                 }
///             });
///             overlay(anchor, Placement::Below, column(rows))
///                 .with_node(|node| node.part(file.content()))
///                 .name("File")
///         }),
///     )
/// }
/// ```
pub struct MenuApi<K: kivact_core::state::ItemKey> {
    session: Session<K>,
    level: Level<K>,
}

impl<K: kivact_core::state::ItemKey> Clone for MenuApi<K> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K: kivact_core::state::ItemKey> Copy for MenuApi<K> {}

/// A menu over `entries` that calls `choose` with the key of a chosen item.
/// Setup only.
#[track_caller]
pub fn use_menu<K: kivact_core::state::ItemKey>(
    entries: impl IntoValue<Vec<MenuEntry<K>>>,
) -> MenuApi<K> {
    use_menu_with_open(entries, signal(false))
}
/// A menu whose popup observes the caller's accepted open binding directly.
/// External opening does not request item focus; keyboard triggers retain canonical focus behavior.
#[track_caller]
pub fn use_menu_with_open<K: kivact_core::state::ItemKey>(
    entries: impl IntoValue<Vec<MenuEntry<K>>>,
    open: impl IntoBinding<bool>,
) -> MenuApi<K> {
    let session = Session::new(
        entries.into_value(),
        callback(|_: K| ()),
        open.into_binding(),
    );
    let level = Level::new(session, Vec::new(), session.is_open());
    MenuApi { session, level }
}

impl<K: kivact_core::state::ItemKey> MenuApi<K> {
    pub(crate) fn menubar_navigation(self, navigate: Callback<bool>) {
        self.session
            .options
            .update(|options| options.menubar = Some(navigate));
    }

    /// Drawing state of the mounted trigger.
    pub fn trigger_state(self) -> Value<PopupState> {
        self.session.popup.trigger_state()
    }
    /// Drawing state of the mounted menu level.
    pub fn content_state(self) -> Value<PopupState> {
        memo(move || {
            self.level
                .content_state
                .get()
                .map(|state| state.get())
                .unwrap_or(PopupState {
                    input: PartState::default(),
                    open: self.is_open().get(),
                })
        })
    }
    /// Drawing state of the actual mounted menu item.
    pub fn item_state(self, key: K) -> Value<MenuItemState> {
        memo(move || {
            self.level
                .item_states
                .with(|states| states.get(&key).copied())
                .map(|state| state.get())
                .unwrap_or(MenuItemState {
                    input: PartState::default(),
                    pressed_over: None,
                    check: None,
                    expanded: false,
                    destructive: false,
                })
        })
    }

    /// Replace the actual trigger's press action during setup; this is not an observer.
    /// ArrowUp/Down retain canonical opening and initial focus, and popup dismissal is unchanged.
    pub fn trigger_action(self, action: impl Fn() + 'static) -> Self {
        self.session
            .options
            .update(|options| options.trigger_action = Some(callback(move |()| action())));
        self
    }
    /// Handle activation of a menu item.
    pub fn on_activate<R: HandlerResult>(self, f: impl Fn(K) -> R + 'static) -> Self {
        self.session
            .options
            .update(|options| options.choose = callback(f));
        self
    }

    /// A submenu handle has no `name`: only the menu from `use_menu` does.
    #[track_caller]
    fn top_level(self, name: &'static str) {
        if self.level.depth > 0 {
            raise(
                ErrorKind::Message,
                format!(
                    "MenuApi::{name}: a submenu handle has no {name}; call it on the menu from use_menu"
                ),
            );
        }
    }

    /// The rows of this level, each with the identity to key its `for_each`
    /// by. A submenu level has no rows once its item is gone.
    pub fn entries(self) -> Value<Vec<(MenuRow<K>, MenuEntry<K>)>> {
        self.level.rows
    }

    /// Whether this level is open: the menu for the top level, the submenu's
    /// item expanded for a submenu level.
    pub fn is_open(self) -> Value<bool> {
        self.level.shown
    }

    /// Where the open menu was asked to appear; `None` while it is closed.
    /// A context menu reads it to place its overlay at the pointer.
    pub fn origin(self) -> Value<Option<MenuOrigin>> {
        self.session.origin
    }

    /// Open the menu as the trigger's ArrowDown does: it opens on its first
    /// focusable item, and the focus later returns to the trigger. Does nothing
    /// while the menu is open. Meant for a component that opens one of several
    /// menus itself, such as a menu bar moving between its menus. Top level
    /// only.
    #[track_caller]
    pub fn open(self) {
        self.top_level("open");
        if self.session.is_open().get_untracked() {
            return;
        }
        self.session.open_from_button();
        self.session.request(0, Edge::First);
    }

    /// Close every level of the menu without moving the focus, as
    /// [`PopupCloseReason::Programmatic`] does. Does nothing while the menu
    /// is closed. Top level only.
    #[track_caller]
    pub fn close(self) {
        self.top_level("close");
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
    /// A trigger belonging to a menubar has `Role::MenuItem` in either attachment order.
    ///
    /// Enter, Space, a click and an assistive technology's Activate open the
    /// menu on its first focusable item (and close it again); ArrowDown does the
    /// same and ArrowUp opens on the last one. Expand opens it and Collapse
    /// closes it. Releasing Tab on the trigger closes it. These run after the
    /// caller's own handlers. The part binds the reference the top-level
    /// overlay is anchored to and the focus returns to. Top level only.
    #[track_caller]
    pub fn trigger(self) -> Part<PopupState> {
        self.top_level("trigger");
        let session = self.session;
        let trigger = session.popup.base_trigger();
        let base = trigger;
        let part_state = base.state();
        Part::new(part_state, move |host| {
            let role = if session.options.get().menubar.is_some() {
                kivact_core::view::Role::MenuItem
            } else {
                kivact_core::view::Role::Button
            };
            let trigger = host.part(base).role(role);
            let trigger = if let Some(press) = session.press {
                trigger.on_pointer(move |event| {
                    press.event(session, super::press_machine::Source::Trigger, event)
                })
            } else {
                trigger
            };
            trigger
                .has_popup(Some(kivact_core::view::PopupKind::Menu))
                .on_key(move |event: KeyEvent| {
                    let m = event.modifiers();
                    if m.ctrl || m.alt || m.meta {
                        return Handled::No;
                    }
                    match event.key() {
                        Key::ArrowDown => {
                            session.open_from_button();
                            session.request(0, Edge::First);
                        }
                        Key::ArrowUp => {
                            session.open_from_button();
                            session.request(0, Edge::Last);
                        }
                        _ => return Handled::No,
                    }
                    Handled::Yes
                })
                .on_press(move || {
                    if let Some(action) = session.options.get().trigger_action {
                        action.call(());
                        session.accepted_trigger_press();
                    } else if session.is_open().get() {
                        session.close(PopupCloseReason::Programmatic);
                    } else {
                        session.open_from_button();
                    }
                })
                .a11y_action(A11yAction::Expand, move |_| session.open_from_button())
                .a11y_action(A11yAction::Collapse, move |_| {
                    session.close(PopupCloseReason::Accessibility)
                })
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
        self.context_trigger_with_fallback(false)
    }

    /// Context trigger that handles secondary presses left unclaimed by a
    /// descendant's pointer listener. Keyboard and accessibility use the
    /// same trigger as [`context_trigger`](Self::context_trigger).
    #[track_caller]
    pub fn context_fallback_trigger(self) -> Part<PopupState> {
        self.context_trigger_with_fallback(true)
    }

    #[track_caller]
    fn context_trigger_with_fallback(self, fallback: bool) -> Part<PopupState> {
        self.top_level("context_trigger");
        let session = self.session;
        let open = session.is_open();
        let pointer = move |event: PointerEvent| {
            // The overlay is anchored to the area, and the host does not
            // count a press on its anchor as outside it.
            if matches!(event.phase(), PointerPhase::Down(_)) {
                session.close(PopupCloseReason::OutsidePress);
            }
            // Open on the phase selected by the platform.
            let opens = match event.phase() {
                PointerPhase::Down(press) | PointerPhase::Up(press) => {
                    press.button == MouseButton::Secondary
                        && matches!(event.phase(), PointerPhase::Down(_))
                            == (capabilities().get().context_menu_on == ContextMenuTrigger::Press)
                }
                _ => false,
            };
            if opens {
                session.open_at(MenuOrigin::Pointer(event.window_pos()));
                Handled::Yes
            } else {
                Handled::No
            }
        };
        let input = part(move |input| PopupState {
            input,
            open: open.get(),
        });
        let part_state = input.state();
        let target = kivact_core::view::node_ref();
        Part::new(part_state, move |host| {
            let trigger = host.part(input).node_ref(target).region_events();
            let trigger = if fallback {
                let trigger = trigger.on_secondary_context(pointer);
                if let Some(press) = session.press {
                    trigger.on_pointer(move |event| {
                        press.event(session, super::press_machine::Source::Context, event)
                    })
                } else {
                    trigger
                }
            } else {
                trigger.on_pointer(move |event| {
                    if let Some(press) = session.press {
                        let handled =
                            press.event(session, super::press_machine::Source::Context, event);
                        if handled == Handled::Yes {
                            return handled;
                        }
                    }
                    pointer(event)
                })
            };
            trigger
                .on_key(move |event: KeyEvent| {
                    let m = event.modifiers();
                    if event.repeat() || m.ctrl || m.alt || m.meta {
                        return Handled::No;
                    }
                    let asked = match event.key() {
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
                .a11y_action(A11yAction::ShowMenu, move |_| {
                    session.open_at(MenuOrigin::Anchor)
                })
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
        let part_state = base.state();
        level.content_state.set(Some(part_state));
        Part::new(part_state, move |host| level.content(host.part(base)))
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
    /// `show(menu.is_open(), ..)` as a dialog's or a popup's: the rebuild on a
    /// new origin kind is the menu's own. It is the one exception to
    /// mounting an overlay's content with the `show` function.
    #[track_caller]
    pub fn show<V: IntoView>(self, factory: impl kivact_core::view::IntoViewFn<V>) -> View {
        let factory = factory.into_view_fn();
        let shown = self.level.shown;
        if self.level.depth > 0 {
            return kivact_core::view::show(shown, move || factory.call(())).into_view();
        }
        let origin = self.session.origin;
        let at_pointer = memo(move || matches!(origin.get(), Some(MenuOrigin::Pointer(_))));
        kivact_core::view::show(shown, move || {
            let factory = factory.clone();
            dynamic(at_pointer, move |_| factory.call(()))
        })
        .into_view()
    }
}
