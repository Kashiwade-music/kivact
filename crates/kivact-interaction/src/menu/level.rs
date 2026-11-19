//! Role: The levels of an open menu: rows, focus, keys and pointer.

use crate::Selection;
use crate::collection::Index;
use kivact_core::state::IntoBinding;

use super::order::{Rows, edge_item, rows};
use super::session::Session;
use super::state::{
    Edge, MenuCheck, MenuEntry, MenuItemDisplay, MenuItemKind, MenuItemState, Opened, Target, Want,
};
use crate::{
    ListApi, ListItem, ListOptions, PopupCloseReason, PopupState, use_hover_corridor, use_list,
};
use kivact_core::custom::Part;
use kivact_core::geometry::Rect;
use kivact_core::state::IntoValue;
use kivact_core::state::{
    Selector, Signal, Value, Var, effect, memo, on_cleanup, selector, signal, untrack, var,
};
use kivact_core::style::use_theme;
use kivact_core::view::{Handled, Key, KeyEvent, PointerEvent, Role};

pub use crate::menu_policy::{DisabledMenuFocus, SubmenuTiming};

/// One level of a menu: the top level or a submenu. All handles are `Copy`.
pub(super) struct Level<K: kivact_core::state::ItemKey> {
    pub(super) content_state: Signal<Option<Value<PopupState>>>,
    pub(super) item_states: Signal<std::collections::HashMap<K, Value<MenuItemState>>>,
    session: Session<K>,
    pub(super) depth: usize,
    /// The keys of the submenu items that lead to this level.
    pub(super) prefix: Value<Vec<K>>,
    pub(super) shown: Value<bool>,
    pub(super) rows: Value<Rows<K>>,
    /// This level's rows by key, in O(1).
    index: Value<Index<K>>,
    /// Whether a key is the item whose submenu is open at this level, per key.
    open_key: Selector<K>,
    group: ListApi<K>,
    anchors: Signal<Vec<(K, Rect)>>,
    aiming: Value<bool>,
    timing: SubmenuTiming,
    retain_disabled_focus: bool,
}

impl<K: kivact_core::state::ItemKey> Clone for Level<K> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<K: kivact_core::state::ItemKey> Copy for Level<K> {}

impl<K: kivact_core::state::ItemKey> Level<K> {
    /// Setup only.
    #[track_caller]
    pub(super) fn new(session: Session<K>, prefix: Vec<K>, shown: Value<bool>) -> Self {
        let depth = prefix.len();
        let entries = session.entries;
        let prefix = prefix.into_value();
        let retain_disabled_focus = use_theme()
            .get()
            .ext::<crate::InteractionPolicy>()
            .menu
            .disabled_focus
            == DisabledMenuFocus::Retain;
        let level_rows = memo(move || prefix.with(|prefix| entries.with(|e| rows(e, prefix))));
        let index = memo(move || {
            level_rows.with(|rows| {
                Index::new(
                    rows,
                    |(_, entry)| entry.key(),
                    |(_, entry)| !entry.is_disabled(),
                )
            })
        });
        let open_key = selector(move || session.path.with(|path| path.get(depth).cloned()));
        // A level the entries no longer keep offers no items, so its group does not
        // move focus to a sibling that is about to go away with it.
        let items = memo(move || {
            if session.kept.get() <= depth {
                return Vec::new();
            }
            level_rows.with(|rows| {
                rows.iter()
                    .filter_map(|(_, entry)| {
                        entry.key().cloned().map(|key| ListItem {
                            key,
                            label: entry.label.clone(),
                            // Menu activation is independently guarded by the row part.
                            disabled: entry.is_disabled() && !retain_disabled_focus,
                            icon: None,
                            pressed_icon: None,
                            description: None,
                        })
                    })
                    .collect::<Vec<_>>()
            })
        });
        let group = use_list(
            items,
            Selection::Single(signal(None::<K>).into_binding()),
            ListOptions::menu(),
        )
        .shared_highlight(kivact_core::state::binding(
            move || session.focused_at(depth),
            move |key| {
                session
                    .machine
                    .send(super::machine::Event::Focus(depth, key))
            },
        ));
        let anchors = signal(Vec::<(K, Rect)>::new());
        let timing = use_theme()
            .get()
            .ext::<crate::InteractionPolicy>()
            .menu
            .submenu;
        let aiming = corridor(session, anchors, depth, shown, timing);
        Self {
            content_state: signal(None),
            item_states: signal(std::collections::HashMap::new()),
            session,
            depth,
            prefix,
            shown,
            rows: level_rows,
            index,
            open_key,
            group,
            anchors,
            aiming,
            timing,
            retain_disabled_focus,
        }
    }

    /// The level's keys and focus requests on the caller's surface part.
    #[track_caller]
    pub(super) fn content(self, base: kivact_core::view::Node) -> kivact_core::view::Node {
        let Level {
            session,
            depth,
            rows,
            group,
            prefix,
            timing,
            retain_disabled_focus,
            ..
        } = self;
        let surface = kivact_core::view::node_ref();
        let base = base.node_ref(surface);
        if depth > 0 && !timing.close.is_zero() {
            observe_rect(
                session.surface_rects,
                prefix.get_untracked(),
                surface.rect(),
            );
        }
        let base = if let Some(press) = session.press {
            base.on_pointer(move |event| {
                press.event(session, super::press_machine::Source::Menu, event)
            })
        } else {
            base
        };
        base.role(Role::Menu)
            .on_key(move |event: KeyEvent| {
                // Core skips handlers on a disabled row itself. Its menu still
                // routes navigation through the same roving list from that row.
                if retain_disabled_focus
                    && let Some(key) = session.focused_at(depth)
                    && rows.with(|rows| {
                        rows.iter()
                            .any(|(_, entry)| entry.key() == Some(&key) && entry.is_disabled())
                    })
                    && group.route_focused_key(&key, event.clone(), None) == Handled::Yes
                {
                    return Handled::Yes;
                }
                on_key(session, depth, rows, event)
            })
            // The host consumes Tab to move focus, so the menu sees only its
            // release, aimed at the item that has the focus now. Tab inside the
            // menu (to another level) leaves it just as Tab out of it does.
            .on_key_up(move |event: KeyEvent| {
                if event.key() == Key::Tab {
                    session.close(PopupCloseReason::Tab);
                    Handled::Yes
                } else {
                    Handled::No
                }
            })
            .on_attach(move |_| {
                self.mount_content(surface);
            })
    }

    fn mount_content(self, surface: kivact_core::view::NodeRef) {
        let Self {
            session,
            depth,
            rows,
            group,
            retain_disabled_focus,
            ..
        } = self;
        if let Some(press) = session.press {
            press.surface(surface);
        }
        if depth == 0 {
            // Where the focus returns for this opening.
            session.popup.set_opening_policy(session.focus_return());
            // The top level takes the focus on its first focusable item
            // unless the opener asked for another one.
            if !matches!(session.opened.get_untracked(), Some(Opened::External))
                && session.want.with_untracked(Option::is_none)
            {
                session.request(0, Edge::First);
            }
            // The top level outlives its surface: closing the menu ends its search.
            on_cleanup(move || group.clear_typeahead());
        }
        // Focus requests addressed to this level wait for its nodes
        // to be live.
        let ready = kivact_core::view::is_mounted();
        effect(move || {
            if !ready.get() {
                return;
            }
            let Some(want) = session.want.get() else {
                return;
            };
            if want.depth() != depth {
                return;
            }
            let target = match want {
                Want::Command { target, .. } => target,
                Want::Hover { key, .. } => Target::Key(key),
            };
            match target {
                Target::Edge(edge) => {
                    let target =
                        rows.with_untracked(|rows| edge_item(rows, edge, retain_disabled_focus));
                    if let Some(key) = target {
                        group.focus(key);
                    }
                }
                Target::Key(key) => {
                    if !group.focus(key) {
                        let target = rows.with_untracked(|rows| {
                            edge_item(rows, Edge::First, retain_disabled_focus)
                        });
                        if let Some(key) = target {
                            group.focus(key);
                        }
                    }
                }
            }
            session.machine.send(super::machine::Event::Request(None));
        });
    }

    fn watch_item(
        self,
        key: K,
        submenu: bool,
        item_ref: kivact_core::view::NodeRef,
        disabled: Value<bool>,
        pressed_over: Value<super::state::PressOver>,
    ) {
        let Self {
            session,
            depth,
            shown,
            ..
        } = self;
        // The engine owns hover: the item's node is hovered while the
        // pointer is over it, whatever presses or captures there are.
        let hovered = item_ref.hovered();
        // The pointer focuses the item it rests on; a submenu opens after a rest.
        let timing = self.timing;
        let aiming = self.aiming;
        if submenu && !timing.close.is_zero() {
            observe_rect(self.anchors, key.clone(), item_ref.rect());
        }
        // One event carries the pointer's whole observation of the row: the
        // hover that moves the focus and starts or stops a submenu timer, and
        // the press the row is under. The machine decides what they mean.
        let key = key.clone();
        effect(move || {
            session.machine.send(super::machine::Event::Pointer {
                depth,
                key: key.clone(),
                hovered: hovered.get() && !aiming.get() && !disabled.get() && shown.get(),
                press: if aiming.get() || !shown.get() {
                    super::state::PressOver::None
                } else {
                    pressed_over.get()
                },
                submenu,
            });
        });
    }

    fn observe_item(self, observed_key: K, part_state: Value<MenuItemState>) {
        let observed = self.item_states;
        observed.update(|states| {
            states.insert(observed_key.clone(), part_state);
        });
        on_cleanup(move || {
            if observed.try_get_untracked().is_some() {
                observed.update(|states| {
                    if states.get(&observed_key) == Some(&part_state) {
                        states.remove(&observed_key);
                    }
                });
            }
        });
    }

    /// Pass a pointer event on to the menu press, when the theme opted in.
    fn item_pointer(self, event: PointerEvent) -> Handled {
        match self.session.press {
            Some(press) => press.event(self.session, super::press_machine::Source::Menu, event),
            None => Handled::No,
        }
    }

    fn activate_item(
        self,
        invoke_key: &K,
        disabled: bool,
        submenu: bool,
        keep_open: bool,
    ) -> Result<(), kivact_core::UiError> {
        let Self { session, depth, .. } = self;
        if disabled {
            return Ok(());
        }
        if submenu {
            session.open_submenu(depth, invoke_key.clone());
            return Ok(());
        }
        session.options.get().choose.try_call(invoke_key.clone())?;
        if !keep_open {
            session.close(PopupCloseReason::Selection);
        }
        Ok::<_, kivact_core::UiError>(())
    }

    /// The part of the item `key`. Setup only, in the item's row.
    #[track_caller]
    pub(super) fn item(self, key: K) -> Part<MenuItemState> {
        let Level {
            session,
            rows,
            index,
            open_key,
            group,
            retain_disabled_focus,
            ..
        } = self;
        let member = group.base_item(key.clone(), retain_disabled_focus);
        let key = var(key);
        // The kind of an item is part of its row's identity.
        let kind = untrack(|| with_entry(rows, index, key, |e| e.and_then(MenuEntry::item_kind)));
        let submenu = kind == Some(MenuItemKind::Submenu);
        // A Value: the menu press keeps it for its hit test.
        let disabled =
            memo(move || with_entry(rows, index, key, |e| e.is_none_or(MenuEntry::is_disabled)));
        let check = move || with_entry(rows, index, key, |e| e.and_then(MenuEntry::check));
        let destructive = move || {
            with_entry(rows, index, key, |e| {
                e.is_some_and(MenuEntry::is_destructive)
            })
        };
        let expanded = move || submenu && key.with(|key| open_key.is(key));
        let item_ref = kivact_core::view::node_ref();
        if let Some(press) = session.press {
            press.row(item_ref, disabled);
        }
        let pressed_over =
            memo(
                move || match session.press.and_then(|press| press.highlighted(item_ref)) {
                    None => super::state::PressOver::None,
                    Some(true) => super::state::PressOver::Over,
                    Some(false) => super::state::PressOver::Away,
                },
            );
        self.watch_item(key.get(), submenu, item_ref, disabled, pressed_over);
        let member = member.map_state(move |state| MenuItemState {
            pressed_over: pressed_over.get(),
            input: state.input,
            display: item_display(check(), submenu, expanded()),
            destructive: destructive(),
        });
        let part_state = member.state();
        self.observe_item(key.get(), part_state);
        Part::new(part_state, move |host| {
            let member = host
                .part(member)
                .node_ref(item_ref)
                .disabled(disabled)
                .description(move || {
                    let disabled = disabled.get();
                    with_entry(rows, index, key, |e| {
                        item_description(e, disabled, retain_disabled_focus)
                    })
                })
                .on_pointer(move |event: PointerEvent| self.item_pointer(event))
                .on_press(move || {
                    let keep_open =
                        with_entry(rows, index, key, |e| e.is_some_and(MenuEntry::keeps_open));
                    self.activate_item(&key.get(), disabled.get(), submenu, keep_open)
                });
            item_role(member, kind, check, expanded)
        })
    }
}

/// What kind of row an item is: its checked state for a check or radio item,
/// whether its submenu is open for a submenu item.
fn item_display(check: Option<MenuCheck>, submenu: bool, expanded: bool) -> MenuItemDisplay {
    match check {
        Some(check) => MenuItemDisplay::Check(check),
        None if submenu => MenuItemDisplay::Submenu { expanded },
        None => MenuItemDisplay::Action,
    }
}

/// Read the entry of the item `key` in a level's rows, tracked, through
/// the level's index in O(1); `None` when the level has no such item.
fn with_entry<K: kivact_core::state::ItemKey, R>(
    rows: Value<Rows<K>>,
    index: Value<Index<K>>,
    key: Var<K>,
    f: impl FnOnce(Option<&MenuEntry<K>>) -> R,
) -> R {
    let at = key.with(|key| index.with(|index| index.source(key)));
    rows.with(|rows| f(at.and_then(|at| rows.get(at)).map(|(_, entry)| entry)))
}

/// The role and the check or submenu state of an item's node.
fn item_role(
    member: kivact_core::view::Node,
    kind: Option<MenuItemKind>,
    check: impl Fn() -> Option<MenuCheck> + 'static,
    expanded: impl Fn() -> bool + Copy + 'static,
) -> kivact_core::view::Node {
    match kind {
        Some(MenuItemKind::Radio) => member
            .role(Role::MenuItemRadio)
            .checked(move || check().is_some_and(MenuCheck::is_checked)),
        Some(MenuItemKind::Checkbox) => member
            .role(Role::MenuItemCheckBox)
            .checked(move || check().is_some_and(MenuCheck::is_checked)),
        Some(MenuItemKind::Submenu) => member
            .role(Role::MenuItem)
            .has_popup(Some(kivact_core::view::PopupKind::Menu))
            .expanded(expanded)
            .selected(expanded),
        _ => member.role(Role::MenuItem),
    }
}

fn on_key<K: kivact_core::state::ItemKey>(
    session: Session<K>,
    depth: usize,
    rows: Value<Rows<K>>,
    event: KeyEvent,
) -> Handled {
    let m = event.modifiers();
    if m.ctrl || m.alt || m.meta {
        return Handled::No;
    }
    match event.key() {
        Key::Escape => {
            session.leave(depth);
            Handled::Yes
        }
        Key::ArrowLeft if depth > 0 => {
            session.close_level(depth);
            Handled::Yes
        }
        Key::ArrowLeft => {
            if let Some(navigate) = session.options.get().menubar {
                navigate.call(false);
                Handled::Yes
            } else {
                Handled::No
            }
        }
        Key::ArrowRight => {
            let Some(key) = session.focused_at(depth) else {
                return Handled::No;
            };
            let opens = rows.with(|rows| {
                rows.iter().any(|(_, entry)| {
                    entry.key() == Some(&key)
                        && entry.item_kind() == Some(MenuItemKind::Submenu)
                        && !entry.is_disabled()
                })
            });
            if opens {
                session.open_submenu(depth, key);
                Handled::Yes
            } else if let Some(navigate) = session.options.get().menubar {
                navigate.call(true);
                Handled::Yes
            } else {
                Handled::No
            }
        }
        Key::Char(c) if !c.is_control() => Handled::Yes,
        _ => Handled::No,
    }
}

fn observe_rect<K: kivact_core::state::PropValue>(
    rects: Signal<Vec<(K, Rect)>>,
    key: K,
    rect: Value<Option<Rect>>,
) {
    let cleanup = key.clone();
    on_cleanup(move || rects.update(|rects| rects.retain(|(key, _)| key != &cleanup)));
    effect(move || {
        let rect = rect.get();
        // xtask-allow(effect-writes): S-9 (a pure cache stays apart from authoritative state): the geometry cache mirrors mounted surface rectangles; this function owns replacement and cleanup.
        rects.update(|rects| {
            rects.retain(|(candidate, _)| candidate != &key);
            if let Some(rect) = rect {
                rects.push((key.clone(), rect));
            }
        });
    });
}

fn corridor<K: kivact_core::state::ItemKey>(
    session: Session<K>,
    anchors: Signal<Vec<(K, Rect)>>,
    depth: usize,
    shown: Value<bool>,
    timing: SubmenuTiming,
) -> Value<bool> {
    if timing.close.is_zero() {
        false.into_value()
    } else {
        let anchor = memo(move || {
            session.path.with(|path| {
                let key = path.get(depth)?;
                anchors.with(|anchors| {
                    anchors
                        .iter()
                        .find(|(candidate, _)| candidate == key)
                        .map(|(_, rect)| *rect)
                })
            })
        });
        let surface = memo(move || {
            session.path.with(|path| {
                let prefix = path.get(..=depth)?;
                session.surface_rects.with(|rects| {
                    rects
                        .iter()
                        .find(|(candidate, _)| candidate == prefix)
                        .map(|(_, rect)| *rect)
                })
            })
        });
        use_hover_corridor(anchor, surface, shown, timing.close).is_holding()
    }
}

/// The row's description: its disabled reason while a disabled row keeps
/// the focus, else its shortcut.
fn item_description<K>(
    entry: Option<&MenuEntry<K>>,
    disabled: bool,
    retain_disabled_focus: bool,
) -> String {
    let Some(entry) = entry else {
        return String::new();
    };
    if retain_disabled_focus
        && disabled
        && let Some(reason) = entry.disabled_reason()
    {
        return reason;
    }
    entry.shortcut().to_string()
}
