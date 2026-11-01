//! Role: Container chrome of a dock: the tab strip, inline rename and the container menu.
use crate::Color;
use kivact::ui::{ButtonVariant, ListItem, MenuEntry};

use kivact::primitive::NodeExt;
use kivact::style::current_theme;
use std::hash::Hash;
use std::rc::Rc;

use kivact::control::{NodeBuilder, column, icon, node, row, text};
use kivact::i18n::message_with;
use kivact::interaction::{
    DragCoordinates, DragPhase, DragUpdate, EditPhase, List, ListPolicy, ListSelection, use_drag,
    use_list as make_list,
};
use kivact::style::{Align, Cursor, Direction, Overflow, style};
use kivact::view::{
    Handled, IntoView, Key, MouseButton, PointerPhase, Role, View, callback, dynamic, for_each,
    node_ref, show,
};
use kivact::{Callback, Value, binding, memo, var};

use super::data::{DockCx, Item, MenuInput, TabDrag, Texts};
use super::geometry::{STRIP, drop_target};
use super::model::{DockLayout, DockPlacement, DockSide, DockTab, DockTabs};

/// Pixels a pressed tab must travel before it becomes a drag.
const DRAG_THRESHOLD: f32 = 5.0;

/// Application entries for the menu of a container.
type AppEntries<K, C> = Rc<dyn Fn(&DockTabs<K>) -> Vec<MenuEntry<C>>>;

/// The application's entries for a container menu and its callback.
pub(super) struct AppMenu<K: 'static, C: 'static> {
    pub(super) entries: AppEntries<K, C>,
    pub(super) choose: Callback<(K, C)>,
}

impl<K: 'static, C: 'static> Clone for AppMenu<K, C> {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
            choose: self.choose,
        }
    }
}

pub(super) fn container<K: Clone + Eq + Hash + 'static>(
    cx: DockCx<K>,
    item: Value<Item<K>>,
) -> View {
    let tabs = memo(move || match item.with(Clone::clone) {
        Item::Chrome { tabs, .. } => tabs,
        _ => DockTabs {
            tabs: Vec::new(),
            active: None,
        },
    });
    let maximized = memo(
        move || matches!(item.with(Clone::clone), Item::Chrome { maximized, .. } if maximized),
    );
    let title = memo(move || {
        tabs.with(|tabs| {
            tabs.active_tab()
                .map(|tab| tab.title.clone())
                .unwrap_or_default()
        })
    });
    let list = memo(move || tabs.with(|tabs| tabs.tabs.clone()));
    let active = memo(move || tabs.with(|tabs| tabs.active.clone()));
    let selected = binding(active, move |key: Option<K>| {
        if let Some(key) = key {
            cx.edit(|layout| layout.activate(&key));
        }
    });
    let group = make_list(
        move || {
            list.with(|tabs| {
                tabs.iter()
                    .map(|tab| ListItem::new(tab.id.clone(), tab.title.clone()))
                    .collect::<Vec<_>>()
            })
        },
        ListSelection::Single(selected),
    )
    .policy(ListPolicy::tabs());
    let strip = node()
        .part(group.list().role(Role::TabList))
        .child(for_each(
            list,
            |tab| tab.id.clone(),
            move |id, tab| tab_view(cx, group, id, tab),
        ))
        .name(message_with(
            "creative.dock.tabs",
            "{title} tabs",
            move || [("title", title.get())],
        ))
        .style(
            style()
                .direction(Direction::Row)
                .grow(1)
                .min_width(0)
                .gap(2)
                .align_items(Align::End)
                .overflow(Overflow::Hidden),
        );
    let menu = (cx.shared().menu)(MenuInput {
        cx,
        tabs,
        maximized,
        title,
    });
    column((
        row((strip, menu)).style(
            style()
                .height(STRIP)
                .shrink(0)
                .padding_x(4)
                .gap(4)
                .align_items(Align::Center)
                .background(Color::Header)
                .border_bottom(1, Color::Hair),
        ),
        node().style(style().grow(1)),
    ))
    .role(Role::Group)
    .name(message_with(
        "creative.dock.container",
        "{title} container",
        move || [("title", title.get())],
    ))
    .hidden(move || !item.with(Item::shown))
    .style(move || item.with(Item::frame).absolute(0.0, 0.0))
    .on_key_default(move |event| match active.get() {
        Some(id) => cx.maximize_key(&id, &event),
        None => Handled::No,
    })
    .style(move || {
        let t = &current_theme();
        let p = t.get::<crate::tokens::Tokens>();
        style()
            .background(p.island)
            .border(1, if maximized.get() { p.accent } else { p.hair })
            .radius(5)
            .overflow(Overflow::Hidden)
    })
    .into_view()
}

fn tab_view<K: Clone + Eq + Hash + 'static>(
    cx: DockCx<K>,
    group: List<K>,
    id: K,
    tab: Value<DockTab<K>>,
) -> View {
    let texts = cx.shared().texts;
    let key = var(id.clone());
    let title = memo(move || tab.with(|tab| tab.title.clone()));
    let member = group.item(id.clone());
    let member_ref = member.node_ref();
    let member_state = member.state();
    cx.register_rect(id, member_ref.rect());
    let renaming = move || cx.renaming.with(|r| r.as_ref() == Some(&key.get()));
    // A renamed tab does not drag; the press still reaches the tab's selection.
    let tab_drag = use_drag::<()>(DragCoordinates::Window)
        .disabled(renaming)
        .on_update(move |update: DragUpdate<()>| {
            let id = key.get();
            let started = cx.drag.with(Option::is_some);
            if !started && update.delta.x.hypot(update.delta.y) < DRAG_THRESHOLD {
                return;
            }
            let target = cx.root.get().and_then(|root| {
                cx.current.with(|layout| {
                    drop_target(layout, root, update.pos, &id, |tab| cx.tab_rect(tab))
                })
            });
            cx.drag.set(Some(TabDrag { source: id, target }));
        })
        .on_end(move |update: DragUpdate<()>| {
            if update.phase == DragPhase::Cancel {
                cx.drag.set(None);
                return;
            }
            let target = cx.drag.with(|d| {
                d.as_ref()
                    .and_then(|d| d.target.as_ref())
                    .map(|t| t.placement.clone())
            });
            cx.drag.set(None);
            if let Some(placement) = target {
                cx.edit(|layout| layout.move_tab(&key.get(), placement));
            }
        });
    let start_rename = move || {
        cx.draft.set(title.get_untracked());
        cx.renaming.set(Some(key.get()));
    };
    let dragged = move || {
        cx.drag
            .with(|d| d.as_ref().is_some_and(|d| d.source == key.get()))
    };
    let glyph_ref = node_ref();
    let glyph_rect = glyph_ref.rect();
    let glyph = dynamic(
        move || tab.with(|tab| tab.icon.clone()),
        move |glyph| match glyph {
            Some(glyph) => icon(glyph).node_ref(glyph_ref).into_view(),
            None => node().hidden(true).into_view(),
        },
    );
    let label = show(
        move || !renaming(),
        move || text(title).style(style().shrink(1).min_width(0)),
    );
    let editor = show(renaming, move || {
        let field = node_ref();
        kivact::on_mount(move || {
            field.focus();
            field.select_all();
        });
        kivact::ui::text_input(texts.rename, cx.draft)
            .node_ref(field)
            .style(style().width(120).height(22))
            .draft(move |name| {
                cx.edit(|layout| layout.rename(&key.get(), name));
            })
            .on_edit(move |phase| {
                if phase != EditPhase::Begin {
                    cx.renaming.set(None);
                }
            })
            .return_focus(member_ref)
    });
    let close = kivact::ui::button(message_with(
        "creative.dock.close_tab",
        "Close {title}",
        move || [("title", title.get())],
    ))
    .hide_label(true)
    .icon(icon("x"))
    .variant(ButtonVariant::Ghost)
    .hidden(move || !tab.with(|tab| tab.closable))
    .style(style().size(16, 16).padding(0).radius(4))
    // Presses on the close button never start a tab drag or rename.
    .on_pointer_default(|event| {
        if event.phase == PointerPhase::Down {
            Handled::Yes
        } else {
            Handled::No
        }
    })
    .on_press(move || {
        let id = key.get();
        cx.close(|layout| layout.close(&id).into_iter().collect());
    });
    let selected = memo(move || member_state.get().selected);
    let member = member
        .role(Role::Tab)
        .name(title)
        .selected(selected)
        // A rename double-click keeps focus in the inline field.
        .on_press(move || {
            if !renaming() {
                group.choose(key.get());
            }
        });
    node()
        .part(member)
        .child((glyph, label, editor, close))
        .on_pointer_default(move |event| {
            if event.phase == PointerPhase::Down
                && event.button == Some(MouseButton::Primary)
                && event.click_count > 1
            {
                tab_drag.cancel();
                cx.drag.set(None);
                // The panel's icon maximizes, its title renames.
                if glyph_rect
                    .get()
                    .is_some_and(|rect| rect.contains(event.window_pos))
                {
                    cx.toggle_maximize(&key.get());
                } else {
                    start_rename();
                }
                return Handled::Yes;
            }
            Handled::No
        })
        // After the tab's selection, which the press reaches first.
        .part(tab_drag.source(()))
        // Before the tab's own activation keys, so Ctrl+Space is not a press.
        // Escape during a drag reaches the drag's source, which ends it.
        .on_key(move |event| match event.key {
            Key::F(2) if !renaming() => {
                start_rename();
                Handled::Yes
            }
            _ if renaming() => Handled::No,
            _ => cx.maximize_key(&key.get(), &event),
        })
        .style(move || {
            let t = &current_theme();
            let p = t.get::<crate::tokens::Tokens>();
            style()
                .direction(Direction::Row)
                .height(STRIP - 6.0)
                .padding_x(8)
                .gap(6)
                .shrink(0)
                .align_items(Align::Center)
                .radius(5)
                .color(p.muted)
                .cursor(Cursor::Grab)
                .opacity(if dragged() { 0.55 } else { 1.0 })
                .selected(style().background(p.strong).color(p.text_high))
                .hover(style().color(p.text_high))
        })
        .into_view()
}

/// Where a menu command puts the tab in another container.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum Beside {
    /// Into its tab strip.
    Tabs,
    /// Into a new container on this side of it.
    Side(DockSide),
}

/// A built-in container command.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum Builtin {
    Close,
    CloseOthers,
    CloseContainer,
    Maximize,
    /// Move the tab one place within its strip: `-1` left, `1` right.
    Shift(isize),
    /// Move the tab into a new container beside its own.
    Split(DockSide),
    /// Move the tab to the container at this index among the other containers.
    MoveTo(usize, Beside),
    /// Move the tab to a whole edge of the dock.
    MoveToEdge(DockSide),
    /// Submenu parents. They open instead of being chosen.
    MoveToMenu,
    MoveToContainer(usize),
    MoveToEdgeMenu,
}

/// The key of a container menu entry.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) enum DockCommand<C> {
    App(C),
    Builtin(Builtin),
}

const SIDES: [DockSide; 4] = [
    DockSide::Left,
    DockSide::Right,
    DockSide::Top,
    DockSide::Bottom,
];

/// The containers a tab can move to, as the container's menu lists them: every
/// container but the one of `tabs`.
fn others<'a, K: Clone + Eq + Hash>(
    layout: &'a DockLayout<K>,
    tabs: &DockTabs<K>,
) -> Vec<&'a DockTabs<K>> {
    layout
        .containers()
        .into_iter()
        .filter(|other| *other != tabs)
        .collect()
}

pub(super) fn container_menu<K, C>(input: MenuInput<K>, app: Option<AppMenu<K, C>>) -> View
where
    K: Clone + Eq + Hash + 'static,
    C: Clone + Eq + Hash + 'static,
{
    let MenuInput {
        cx,
        tabs,
        maximized,
        title,
    } = input;
    let texts = cx.shared().texts;
    let entries = memo({
        let app = app.clone();
        move || {
            let mut entries = Vec::new();
            let tabs = tabs.get();
            if let Some(app) = &app {
                entries.extend(
                    (app.entries)(&tabs)
                        .into_iter()
                        .map(|entry| entry.map_key(&DockCommand::App)),
                );
                if !entries.is_empty() {
                    entries.push(MenuEntry::separator());
                }
            }
            let maximized = maximized.get();
            let layout = cx.current.get();
            builtin_entries(&mut entries, &layout, &tabs, maximized, texts);
            entries
        }
    });
    let choose = callback(move |command| {
        let tabs = tabs.get();
        let Some(id) = tabs.active.clone() else {
            return;
        };
        match command {
            DockCommand::App(command) => {
                if let Some(app) = &app {
                    app.choose.call((id, command));
                }
            }
            DockCommand::Builtin(command) => run(cx, id, &tabs, command),
        }
    });
    let trigger = kivact::ui::button(message_with(
        "creative.dock.menu",
        "{title} panel menu",
        move || [("title", title.get())],
    ))
    .hide_label(true)
    .icon(icon("menu"))
    .variant(ButtonVariant::Ghost)
    .style(style().width(24).height(22).padding(0).shrink(0));
    kivact::ui::menu("", entries, move |value| choose.try_call(value))
        .offset(0.0)
        .attach(trigger.into_node())
        .into_view()
}

fn builtin_entries<K: Clone + Eq + Hash, C>(
    entries: &mut Vec<MenuEntry<DockCommand<C>>>,
    layout: &DockLayout<K>,
    tabs: &DockTabs<K>,
    maximized: bool,
    texts: Texts,
) {
    let item = |command: Builtin, label: Value<String>| {
        MenuEntry::item(DockCommand::Builtin(command), label.get())
    };
    let active = tabs.active_tab();
    let position = active.and_then(|a| tabs.tabs.iter().position(|t| t.id == a.id));
    let several = tabs.tabs.len() > 1;
    entries.push(
        item(Builtin::Close, texts.close_panel).disabled(!active.is_some_and(|tab| tab.closable)),
    );
    entries.push(
        item(Builtin::CloseOthers, texts.close_others).disabled(
            !tabs
                .tabs
                .iter()
                .any(|tab| Some(&tab.id) != tabs.active.as_ref() && tab.closable),
        ),
    );
    entries.push(
        item(Builtin::CloseContainer, texts.close_container)
            .disabled(!tabs.tabs.iter().any(|tab| tab.closable)),
    );
    entries.push(MenuEntry::separator());
    entries.push(
        item(
            Builtin::Maximize,
            if maximized {
                texts.restore
            } else {
                texts.maximize
            },
        )
        .shortcut(texts.maximize_shortcut.get()),
    );
    entries
        .push(item(Builtin::Shift(-1), texts.move_left).disabled(!position.is_some_and(|p| p > 0)));
    entries.push(
        item(Builtin::Shift(1), texts.move_right)
            .disabled(!position.is_some_and(|p| p + 1 < tabs.tabs.len())),
    );
    entries.push(MenuEntry::separator());
    for side in SIDES {
        entries.push(item(Builtin::Split(side), texts.split(side)).disabled(!several || maximized));
    }
    let others = others(layout, tabs);
    let mut targets = Vec::new();
    for (index, other) in others.iter().enumerate() {
        let title = other
            .active_tab()
            .map(|tab| tab.title.clone())
            .unwrap_or_default();
        let mut children = vec![item(Builtin::MoveTo(index, Beside::Tabs), texts.into_tabs)];
        children.extend(
            SIDES.map(|side| item(Builtin::MoveTo(index, Beside::Side(side)), texts.side(side))),
        );
        targets.push(MenuEntry::submenu(
            DockCommand::Builtin(Builtin::MoveToContainer(index)),
            title,
            children,
        ));
    }
    targets.push(MenuEntry::submenu(
        DockCommand::Builtin(Builtin::MoveToEdgeMenu),
        texts.edge.get(),
        SIDES
            .map(|side| item(Builtin::MoveToEdge(side), texts.side(side)))
            .into(),
    ));
    entries.push(
        MenuEntry::submenu(
            DockCommand::Builtin(Builtin::MoveToMenu),
            texts.move_to.get(),
            targets,
        )
        .disabled(maximized || (others.is_empty() && !several)),
    );
}

fn run<K: Clone + Eq + Hash + 'static>(cx: DockCx<K>, id: K, tabs: &DockTabs<K>, command: Builtin) {
    match command {
        Builtin::Close => cx.close(|layout| layout.close(&id).into_iter().collect()),
        Builtin::CloseOthers => cx.close(|layout| layout.close_others(&id)),
        Builtin::CloseContainer => cx.close(|layout| layout.close_container(&id)),
        Builtin::Maximize => cx.toggle_maximize(&id),
        Builtin::Shift(step) => {
            let Some(index) = tabs
                .tabs
                .iter()
                .position(|tab| tab.id == id)
                .and_then(|position| position.checked_add_signed(step))
            else {
                return;
            };
            cx.edit(|layout| {
                layout.move_tab(
                    &id,
                    DockPlacement::Tabs {
                        container: id.clone(),
                        index: Some(index),
                    },
                )
            });
        }
        Builtin::Split(side) => {
            cx.edit(|layout| {
                layout.move_tab(
                    &id,
                    DockPlacement::Split {
                        container: id.clone(),
                        side,
                    },
                )
            });
        }
        Builtin::MoveTo(index, beside) => {
            cx.edit(|layout| {
                let Some(container) = others(layout, tabs)
                    .get(index)
                    .and_then(|other| other.active_tab().or(other.tabs.first()))
                    .map(|tab| tab.id.clone())
                else {
                    return false;
                };
                let to = match beside {
                    Beside::Tabs => DockPlacement::Tabs {
                        container,
                        index: None,
                    },
                    Beside::Side(side) => DockPlacement::Split { container, side },
                };
                layout.move_tab(&id, to)
            });
        }
        Builtin::MoveToEdge(side) => {
            cx.edit(|layout| layout.move_tab(&id, DockPlacement::Edge(side)));
        }
        Builtin::MoveToMenu | Builtin::MoveToContainer(_) | Builtin::MoveToEdgeMenu => {}
    }
}
