//! Role: The levels of an open menu: rows, focus, keys and pointer.

use std::hash::Hash;
use std::time::Duration;

use super::order::{Rows, edge_item, rows};
use super::state::{Edge, MenuItemState, Session, Want};
use crate::{
    List, ListItem, ListPolicy, ListSelection, PopupCloseReason, PopupState, use_disclosure,
    use_list,
};
use kivact_core::primitive::Part;
use kivact_core::view::{Handled, IntoValue, Key, KeyEvent, PointerEvent, PointerPhase, Role};
use kivact_core::{Value, binding, effect, memo, on_mount, signal};

/// How long the pointer rests on a submenu item before it opens.
const HOVER_OPEN: Duration = Duration::from_millis(140);

/// One level of a menu: the top level or a submenu. All handles are `Copy`.
pub(super) struct Level<K: Clone + Eq + Hash + 'static> {
    session: Session<K>,
    pub(super) depth: usize,
    /// The keys of the submenu items that lead to this level.
    pub(super) prefix: Value<Vec<K>>,
    pub(super) shown: Value<bool>,
    pub(super) rows: Value<Rows<K>>,
    group: List<K>,
}

impl<K: Clone + Eq + Hash + 'static> Clone for Level<K> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<K: Clone + Eq + Hash + 'static> Copy for Level<K> {}

impl<K: Clone + Eq + Hash + 'static> Level<K> {
    /// Setup only.
    #[track_caller]
    pub(super) fn new(session: Session<K>, prefix: Vec<K>, shown: Value<bool>) -> Self {
        let depth = prefix.len();
        let entries = session.entries;
        let prefix = prefix.into_value();
        let level_rows = memo(move || prefix.with(|prefix| entries.with(|e| rows(e, prefix))));
        // A level the entries no longer keep offers no items, so its group does not
        // move focus to a sibling that is about to go away with it.
        let items = memo(move || {
            if session.kept.get() <= depth {
                return Vec::new();
            }
            level_rows.with(|rows| {
                rows.iter()
                    .filter_map(|(_, entry)| {
                        entry.key.clone().map(|key| ListItem {
                            key,
                            label: entry.label.clone(),
                            disabled: entry.disabled,
                            icon: None,
                            description: None,
                        })
                    })
                    .collect::<Vec<_>>()
            })
        });
        let group = use_list(items, ListSelection::Single(signal(None::<K>).into()))
            .policy(ListPolicy::menu())
            .shared_typeahead(session.typed);
        Self {
            session,
            depth,
            prefix,
            shown,
            rows: level_rows,
            group,
        }
    }

    /// The level's keys and focus requests on the caller's surface part.
    #[track_caller]
    pub(super) fn content(self, base: Part<PopupState>) -> Part<PopupState> {
        let Level {
            session,
            depth,
            rows,
            group,
            ..
        } = self;
        base.role(Role::Menu)
            .on_key(move |event: KeyEvent| on_key(session, depth, rows, event))
            // The host consumes Tab to move focus, so the menu sees only its
            // release, aimed at the item that has the focus now. Tab inside the
            // menu (to another level) leaves it just as Tab out of it does.
            .on_key_up(move |event: KeyEvent| {
                if event.key == Key::Tab {
                    session.close(PopupCloseReason::Tab);
                    Handled::Yes
                } else {
                    Handled::No
                }
            })
            .on_bind(move |_| {
                // The top level takes the focus on its first enabled item
                // unless the opener asked for another one.
                if depth == 0 && session.want.with_untracked(Option::is_none) {
                    session.request(0, Edge::First);
                }
                // Focus requests addressed to this level wait for its nodes
                // to be live.
                let ready = signal(false);
                on_mount(move || ready.set(true));
                effect(move || {
                    if !ready.get() {
                        return;
                    }
                    let Some(want) = session.want.get() else {
                        return;
                    };
                    match want {
                        Want::Edge(at, edge) if at == depth => {
                            let target = rows.with_untracked(|rows| edge_item(rows, edge));
                            if let Some(key) = target {
                                group.focus(key);
                            }
                            session.want.set(None);
                        }
                        Want::Key(at, key) if at == depth => {
                            if !group.focus(key) {
                                let target =
                                    rows.with_untracked(|rows| edge_item(rows, Edge::First));
                                if let Some(key) = target {
                                    group.focus(key);
                                }
                            }
                            session.want.set(None);
                        }
                        _ => {}
                    }
                });
            })
    }

    /// The part of the item `key`. Setup only, in the item's row.
    #[track_caller]
    pub(super) fn item(self, key: K) -> Part<MenuItemState> {
        let Level {
            session,
            depth,
            shown,
            rows,
            group,
            ..
        } = self;
        let entry = {
            let key = key.clone();
            memo(move || {
                rows.with(|rows| {
                    rows.iter()
                        .find(|(_, entry)| entry.key.as_ref() == Some(&key))
                        .map(|(_, entry)| entry.clone())
                })
            })
        };
        // The kind of an item is part of its row's identity.
        let (submenu, check, radio) = entry.with_untracked(|entry| {
            entry.as_ref().map_or((false, false, false), |e| {
                (e.children.is_some(), e.checked.is_some(), e.radio)
            })
        });
        let disabled = memo(move || entry.with(|e| e.as_ref().is_none_or(|e| e.disabled)));
        let shortcut = memo(move || {
            entry.with(|e| e.as_ref().map(|e| e.shortcut.clone()).unwrap_or_default())
        });
        let checked = memo(move || entry.with(|e| e.as_ref().and_then(|e| e.checked)));
        let destructive = memo(move || entry.with(|e| e.as_ref().is_some_and(|e| e.destructive)));
        let keep_open = memo(move || entry.with(|e| e.as_ref().is_some_and(|e| e.keep_open)));
        let expanded = {
            let key = key.clone();
            memo(move || submenu && session.path.with(|path| path.get(depth) == Some(&key)))
        };

        let member = group.item(key.clone());
        let item_ref = member.node_ref();

        // The pointer focuses the item it rests on; a submenu opens after a rest.
        let hovered = signal(false);
        if submenu {
            let key = key.clone();
            let resting = memo(move || hovered.get() && !expanded.get() && !disabled.get());
            let disclosure = use_disclosure(binding(resting, |_| {})).delay(HOVER_OPEN);
            effect(move || {
                if disclosure.shown().get() {
                    session.open_hover(depth, key.clone());
                }
            });
            // A submenu closed by key stays closed under a resting pointer.
            effect(move || {
                if !expanded.get() {
                    disclosure.close();
                }
            });
        } else {
            // Resting on any other item, a disabled one included, closes the
            // submenus of this level. A disabled item receives no pointer
            // events, so this follows the item's hover instead.
            let over = item_ref.hovered();
            effect(move || {
                if over.get() && shown.get_untracked() {
                    session.close_below(depth);
                }
            });
        }

        let focused = item_ref.focused();
        let focus_key = key.clone();
        effect(move || {
            if focused.get() {
                session.focus_key.set(Some((depth, focus_key.clone())));
            }
        });

        let hover_key = key.clone();
        let invoke_key = key;
        let member = member
            .map_state(move |state| MenuItemState {
                input: state.input,
                checked: checked.get(),
                radio,
                expanded: expanded.get(),
                destructive: destructive.get(),
            })
            .role(match (check, radio) {
                (true, true) => Role::MenuItemRadio,
                (true, false) => Role::MenuItemCheckBox,
                _ => Role::MenuItem,
            })
            .description(shortcut)
            .on_pointer(move |event: PointerEvent| {
                match event.phase {
                    PointerPhase::Enter => {
                        hovered.set(true);
                        // Asked for, not requested here: a click delivers its
                        // move, press and release in one turn, and the choice
                        // closes the menu before a request for the hovered row
                        // could be sent. The level's effect carries it out
                        // after the events, and a closed level never does.
                        if !disabled.get() {
                            session.want.set(Some(Want::Key(depth, hover_key.clone())));
                        }
                    }
                    PointerPhase::Leave => hovered.set(false),
                    _ => {}
                }
                Handled::No
            })
            .on_press(move || {
                if disabled.get() {
                    return Ok(());
                }
                if submenu {
                    session.open_submenu(depth, invoke_key.clone());
                    return Ok(());
                }
                session.choose.try_call(invoke_key.clone())?;
                if !keep_open.get() {
                    session.close(PopupCloseReason::Selection);
                }
                Ok::<_, kivact_core::UiError>(())
            });
        let member = if check {
            member.checked(move || checked.get().unwrap_or(false))
        } else {
            member
        };
        if submenu {
            member.expanded(expanded).selected(expanded)
        } else {
            member
        }
    }
}

fn on_key<K: Clone + Eq + Hash + 'static>(
    session: Session<K>,
    depth: usize,
    rows: Value<Rows<K>>,
    event: KeyEvent,
) -> Handled {
    let m = event.modifiers;
    if m.ctrl || m.alt || m.meta {
        return Handled::No;
    }
    match event.key {
        Key::Escape => {
            session.leave(depth);
            Handled::Yes
        }
        Key::ArrowLeft if depth > 0 => {
            session.close_level(depth);
            Handled::Yes
        }
        Key::ArrowRight => {
            let Some(key) = session.focused_at(depth) else {
                return Handled::No;
            };
            let opens = rows.with(|rows| {
                rows.iter().any(|(_, entry)| {
                    entry.key.as_ref() == Some(&key) && entry.children.is_some() && !entry.disabled
                })
            });
            if opens {
                session.open_submenu(depth, key);
                Handled::Yes
            } else {
                Handled::No
            }
        }
        Key::Char(c) if !c.is_control() => Handled::Yes,
        _ => Handled::No,
    }
}
