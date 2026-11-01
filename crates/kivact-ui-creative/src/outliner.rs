//! Role: Outliner of the Creative kit: a consumer-owned hierarchy tree view.
pub(crate) mod view_state;

use crate::Color;
use crate::LabelTone;
use kivact::control::{Node, NodeBuilder, column, icon, node, row, text};
use kivact::draw::Point;
use kivact::i18n::message;
use kivact::interaction::{
    DragCoordinates, DragPhase, DragUpdate, EditSession, ListSelection, use_drag, use_tree,
};
use kivact::primitive::NodeExt;
use kivact::style::current_theme;
use kivact::style::{Align, ColorToken, Cursor, Overflow, Position, pct, style};
use kivact::ui::{EditPhase, TreeNode};
use kivact::view::HandlerResult;
use kivact::view::{
    Handled, IntoValue, IntoView, Key, KeyEvent, MouseButton, PointerEvent, PointerPhase, View,
    callback, for_each, node_ref, show,
};
use kivact::{Callback, Value, memo, signal};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

#[path = "outliner_controls.rs"]
mod controls;

use crate::editors::shared::model::EditorEvent;
pub use crate::outliner_data::{Restriction, RestrictionFlag, TreeDrop, TreeItem};

/// Which part of a row the pointer is over during a drag.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Placement {
    Before,
    After,
    Into,
}

/// The row under the dragged pointer and the part of it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Plan {
    over: u64,
    placement: Placement,
}

/// The drop for `dragged` released over `over`, `fraction` of the way down
/// that row (0 at its top edge). `None` when the item cannot go there.
fn plan_drop(
    items: &[TreeItem],
    dragged: u64,
    over: u64,
    fraction: f32,
) -> Option<(Plan, TreeDrop)> {
    let parent_of = |id: u64| {
        items
            .iter()
            .find(|item| item.id == id)
            .map(|item| item.parent)
    };
    // Not on itself, and not into its own subtree.
    let mut at = Some(over);
    for _ in 0..=items.len() {
        match at {
            Some(id) if id == dragged => return None,
            Some(id) => at = parent_of(id)?,
            None => break,
        }
    }
    let placement = if fraction < 1.0 / 3.0 {
        Placement::Before
    } else if fraction > 2.0 / 3.0 {
        Placement::After
    } else {
        Placement::Into
    };
    let children = |parent: Option<u64>| {
        items
            .iter()
            .filter(|item| item.parent == parent && item.id != dragged)
            .map(|item| item.id)
            .collect::<Vec<_>>()
    };
    let drop = match placement {
        Placement::Into => TreeDrop {
            id: dragged,
            target: Some(over),
            after: children(Some(over)).last().copied(),
        },
        Placement::After | Placement::Before => {
            let parent = parent_of(over)?;
            let siblings = children(parent);
            let at = siblings.iter().position(|id| *id == over)?;
            TreeDrop {
                id: dragged,
                target: parent,
                after: match placement {
                    Placement::After => Some(over),
                    _ => at.checked_sub(1).map(|before| siblings[before]),
                },
            }
        }
    };
    Some((Plan { over, placement }, drop))
}

type CellRenderer = Rc<dyn Fn(Value<TreeItem>) -> View>;

/// A searchable hierarchy on [`Tree`](kivact::interaction::Tree).
/// It retains expansion, rename, restriction flags and drag and drop,
/// and hosts the application's cells.
///
/// The rows are one tab stop and follow the ARIA tree view (Up, Down, Home,
/// End, Right, Left, `*`, Enter). Shift and Ctrl or Cmd select a range or add.
/// F2 renames the focused row and `H` toggles its
/// visibility (so typing `h` does not jump to a label).
///
/// A double click renames the row, unless
/// [`on_item_activate`](Self::on_item_activate) is set: then a double click
/// and Enter activate the item and only F2 renames. An item that is not
/// `selectable` is shown as a disabled row. The toggles at the end of a row
/// are not tab stops (they have their accessibility actions, and a press
/// toggles them; drag across rows to paint a value).
///
/// A drag moves an item. The pointer decides the place (see [`TreeDrop`]),
/// shown by a line before or after the row, or a highlighted row for "into
/// it"; Escape cancels. With [`on_item_drop`](Self::on_item_drop) the drop is
/// reported and the application moves the item; without it, the outliner
/// keeps dragging disabled.
pub struct Outliner {
    root: Node,
    items: Value<Vec<TreeItem>>,
    state: crate::OutlinerState,
    on_rename: Option<Callback<(u64, String)>>,
    on_restriction: Option<Callback<EditorEvent<Restriction>>>,
    filter: Value<String>,
    restrictions: Value<bool>,
    activation: Option<Callback<u64>>,
    item_drop: Option<Callback<TreeDrop>>,
    active: Value<Option<u64>>,
    reveal_active: Value<bool>,
    cells: Vec<(f32, CellRenderer)>,
    row_height: Option<Value<f32>>,
}
/// An outliner of `items` with the open, selected and renamed items of
/// `state`.
#[track_caller]
pub fn outliner(items: impl IntoValue<Vec<TreeItem>>, state: crate::OutlinerState) -> Outliner {
    Outliner {
        root: column(())
            .name(message("creative.outliner.label", "Outliner"))
            .style(
                style()
                    .width(pct(100))
                    .min_width(0)
                    .overflow(Overflow::Hidden),
            ),
        items: items.into_value(),
        state,
        on_rename: None,
        on_restriction: None,
        filter: String::new().into_value(),
        restrictions: true.into_value(),
        activation: None,
        item_drop: None,
        active: None::<u64>.into_value(),
        reveal_active: true.into_value(),
        cells: Vec::new(),
        row_height: None,
    }
}
impl Outliner {
    /// Show or hide the three per-item selection, visibility and render
    /// controls (constant or reactive).
    #[track_caller]
    pub fn restrictions(mut self, enabled: impl IntoValue<bool>) -> Self {
        self.restrictions = enabled.into_value();
        self
    }
    /// Report a submitted name. The application decides whether to accept it.
    pub fn on_rename<R: HandlerResult>(mut self, f: impl Fn((u64, String)) -> R + 'static) -> Self {
        self.on_rename = Some(callback(f));
        self
    }
    /// Report absolute restriction proposals; restore application items on Cancel.
    /// Selected rows use one value, opposite to the pressed/focused row's initial flag.
    /// Painting accumulates affected IDs without changing that value. Single clicks,
    /// keys and accessibility actions emit Begin, Request, Commit. Controls are
    /// disabled without a callback.
    pub fn on_restriction<R: HandlerResult>(
        mut self,
        f: impl Fn(EditorEvent<Restriction>) -> R + 'static,
    ) -> Self {
        self.on_restriction = Some(callback(f));
        self
    }
    /// Show only the items whose label contains the text (ignoring case) and
    /// their ancestors, all open; the expansion binding is preserved.
    pub fn filter(mut self, filter: impl IntoValue<String>) -> Self {
        self.filter = filter.into_value();
        self
    }
    /// Call `f` with the item on Enter and on a double click, instead
    /// of renaming on a double click. F2 still renames.
    pub fn on_item_activate<R: HandlerResult>(mut self, f: impl Fn(u64) -> R + 'static) -> Self {
        self.activation = Some(callback(f));
        self
    }
    /// Report where a dragged item was dropped (see [`TreeDrop`]) instead of
    /// moving it in the items. A cancelled drag (Escape, or a drag too short
    /// to be one) reports nothing.
    pub fn on_item_drop<R: HandlerResult>(mut self, f: impl Fn(TreeDrop) -> R + 'static) -> Self {
        self.item_drop = Some(callback(f));
        self
    }
    /// The application's current object, which is not the selection: its row
    /// is drawn with an accent marker and publishes `checked`.
    pub fn active(mut self, active: impl IntoValue<Option<u64>>) -> Self {
        self.active = active.into_value();
        self
    }
    /// Whether a change of [`active`](Self::active) opens the ancestors of
    /// the item (by writing the expansion binding) and scrolls its row into
    /// view (default: true; constant or reactive, read when `active`
    /// changes). A change that follows a press on a row of this outliner is
    /// left alone, and so is the value at mount.
    #[track_caller]
    pub fn reveal_active(mut self, reveal: impl IntoValue<bool>) -> Self {
        self.reveal_active = reveal.into_value();
        self
    }
    /// Append a fixed-width application cell after the label and before restrictions.
    /// Cells keep their width while the label shrinks, and clip overflowing content.
    /// Pointer and key input in a cell stays with its controls instead of selecting,
    /// dragging or renaming the row. Tab keeps normal focus navigation.
    /// Non-finite or non-positive widths hide the cell.
    pub fn cell<V: IntoView>(
        mut self,
        width: f32,
        render: impl Fn(Value<TreeItem>) -> V + 'static,
    ) -> Self {
        let width = if width.is_finite() && width > 0.0 {
            width
        } else {
            0.0
        };
        self.cells
            .push((width, Rc::new(move |item| render(item).into_view())));
        self
    }
    /// The height of a row (at least 20). The default is the density's row.
    pub fn row_height(mut self, height: impl IntoValue<f32>) -> Self {
        self.row_height = Some(height.into_value());
        self
    }
}
impl NodeBuilder for Outliner {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Outliner {
    fn into_view(self) -> View {
        let Outliner {
            root,
            items,
            state:
                crate::OutlinerState {
                    expanded,
                    selection: selected,
                    renaming: rename,
                },
            on_rename,
            on_restriction,
            filter,
            restrictions,
            activation,
            item_drop,
            active,
            reveal_active,
            cells,
            row_height,
        } = self;
        let data = items;
        let renaming = rename.into_value();
        let draft = signal(String::new());
        kivact::effect(move || {
            if let Some(id) = renaming.get() {
                draft.set(data.with_untracked(|v| {
                    v.iter()
                        .find(|n| n.id == id)
                        .map_or_else(String::new, |n| n.label.clone())
                }));
            }
        });
        let by_id = memo(move || {
            data.with(|items| {
                items
                    .iter()
                    .map(|item| (item.id, item.clone()))
                    .collect::<HashMap<_, _>>()
            })
        });
        let nodes = memo(move || {
            data.with(|items| {
                items
                    .iter()
                    .map(|item| {
                        TreeNode::new(item.id, item.parent, item.label.clone())
                            .disabled(!item.selectable)
                    })
                    .collect::<Vec<_>>()
            })
        });
        let has_activation = activation.is_some();
        let mut tree = use_tree(nodes, expanded, ListSelection::Multiple(selected)).filter(filter);
        if let Some(callback) = activation {
            tree = tree.on_activate(move |value| callback.try_call(value));
        }
        let mounted: controls::Rows = Rc::new(RefCell::new(HashMap::new()));
        let generation = Rc::new(Cell::new(0u64));
        // The row that a primary press is on, from its press to its release.
        let pressed = Rc::new(Cell::new(None::<u64>));
        let plan = signal(None::<Plan>);
        // An item to scroll to when its row appears (its ancestors opened).
        let pending = signal(None::<u64>);
        // The effect keeps its own handles on the rows and the press.
        {
            let mounted = mounted.clone();
            let pressed = pressed.clone();
            let first = Cell::new(true);
            kivact::effect(move || {
                let target = active.get();
                if first.replace(false) || !reveal_active.get_untracked() {
                    return;
                }
                pending.set(None);
                let picked = pressed.get();
                let Some(id) = target.filter(|id| picked != Some(*id)) else {
                    return;
                };
                let next = data.get_untracked();
                let mut open = expanded.get_untracked();
                let mut closed = Vec::new();
                let mut parent = next.iter().find(|n| n.id == id).and_then(|n| n.parent);
                for _ in 0..next.len() {
                    let Some(ancestor) = parent.and_then(|p| next.iter().find(|n| n.id == p))
                    else {
                        break;
                    };
                    if !open.contains(&ancestor.id) {
                        closed.push(ancestor.id);
                    }
                    parent = ancestor.parent;
                }
                if !closed.is_empty() {
                    open.extend(closed);
                    expanded.set(open);
                }
                // A row that is not there yet scrolls when it mounts.
                let node = mounted.borrow().get(&id).map(|row| row.node);
                match node {
                    Some(node) => node.scroll_into_view(),
                    None => pending.set(Some(id)),
                }
            });
        }
        let painting = signal(None::<controls::Paint>);
        let session = crate::editors::shared::model::observed(
            EditSession::new(|()| {}),
            on_restriction.map(|observer| {
                callback(move |phase| {
                    observer.call(match phase {
                        EditPhase::Begin => EditorEvent::Begin,
                        EditPhase::Commit => EditorEvent::Commit,
                        EditPhase::Cancel => EditorEvent::Cancel,
                    });
                })
            }),
        );
        let levels = memo(move || {
            tree.rows().with(|rows| {
                rows.iter()
                    .map(|row| (row.key, row.level))
                    .collect::<HashMap<_, _>>()
            })
        });
        let rows = for_each(tree.rows(), |entry| (entry.key, entry.has_children), {
            let mounted = mounted.clone();
            move |_, entry| {
                let entry = entry.get_untracked();
                let id = entry.key;
                let Some(fallback) = by_id.with_untracked(|map| map.get(&id).cloned()) else {
                    return node();
                };
                let item = memo(move || {
                    by_id
                        .with(|map| map.get(&id).cloned())
                        .unwrap_or_else(|| fallback.clone())
                });
                let dragger = use_drag::<u64>(DragCoordinates::Window);
                let reference = node_ref();
                let focused = reference.focused();
                let mount = generation.get() + 1;
                generation.set(mount);
                mounted.borrow_mut().insert(
                    id,
                    controls::Mounted {
                        node: reference,
                        rect: reference.rect(),
                        generation: mount,
                    },
                );
                let cleanup = mounted.clone();
                kivact::on_cleanup(move || {
                    let mut rows = cleanup.borrow_mut();
                    if rows.get(&id).is_some_and(|row| row.generation == mount) {
                        rows.remove(&id);
                    }
                });
                kivact::on_mount(move || {
                    if pending.get() == Some(id) {
                        pending.set(None);
                        reference.scroll_into_view();
                    }
                });
                let start_rename = move || {
                    rename.set(Some(id));
                    draft.set(item.get_untracked().label);
                };
                let label = show(
                    move || renaming.get() != Some(id),
                    move || {
                        text(move || item.with(|n| n.label.clone())).style(move || {
                            let theme = current_theme();
                            let p = theme.get::<crate::tokens::Tokens>();
                            item.with(|item| {
                                let color = if item.placeholder && item.tone == LabelTone::Normal {
                                    p.muted
                                } else {
                                    item.tone.color(&p)
                                };
                                style()
                                    .grow(1)
                                    .shrink(1000)
                                    .min_width(0)
                                    .single_line()
                                    .italic(item.placeholder)
                                    .color(color)
                            })
                        })
                    },
                );
                let edit = show(
                    move || renaming.get() == Some(id),
                    move || {
                        let input_ref = node_ref();
                        kivact::on_mount(move || {
                            input_ref.focus();
                            input_ref.select_all();
                        });
                        kivact::ui::text_input(
                            message("creative.outliner.rename", "Rename tree item"),
                            draft,
                        )
                        .node_ref(input_ref)
                        .style(style().grow(1).height(22))
                        .draft(move |name| {
                            if let Some(callback) = on_rename {
                                callback.call((id, name));
                            }
                        })
                        .on_edit(move |phase| {
                            if phase != kivact::ui::EditPhase::Begin {
                                rename.set(None);
                            }
                        })
                        .return_focus(reference)
                    },
                );
                let context = controls::Context {
                    expanded,
                    session,
                    selected,
                    on_restriction,
                    data,
                    rows: mounted.clone(),
                };
                let chevron = if entry.has_children {
                    controls::expansion(item, context.clone())
                } else {
                    // The room of the button, so the labels line up.
                    column(()).style(style().size(18, 18).shrink(0))
                };
                let cells = cells
                    .iter()
                    .filter(|(width, _)| *width > 0.0)
                    .map(|(width, render)| {
                        row(render(item))
                            .on_pointer_default(|_| Handled::Yes)
                            .on_key_default(|event| {
                                // Enter/Space activate the focused child only after
                                // key bubbling. Tree/List already ignore child focus.
                                if matches!(event.key, Key::Tab | Key::Enter | Key::Space) {
                                    Handled::No
                                } else {
                                    Handled::Yes
                                }
                            })
                            .style(
                                style()
                                    .align_items(Align::Center)
                                    .gap(4)
                                    .width(*width)
                                    .shrink(0)
                                    .min_width(0)
                                    .overflow(Overflow::Hidden),
                            )
                            .into_view()
                    })
                    .collect::<Vec<_>>();
                let mark = memo(move || {
                    plan.get()
                        .filter(|plan| plan.over == id)
                        .map(|plan| plan.placement)
                });
                let line = move |at: Placement| {
                    column(())
                        .hidden(move || mark.get() != Some(at))
                        .style(move || {
                            let t = &current_theme();
                            let base = style()
                                .position(Position::Absolute)
                                .left(0)
                                .right(0)
                                .height(2)
                                .z_index(5)
                                .background(t.get::<crate::tokens::Tokens>().accent);
                            if at == Placement::Before {
                                base.top(0)
                            } else {
                                base.bottom(0)
                            }
                        })
                };
                let active_marker = column(()).hidden(move || active.get() != Some(id)).style(
                    style()
                        .position(Position::Absolute)
                        .left(0)
                        .top(0)
                        .bottom(0)
                        .width(2)
                        .background(Color::Accent),
                );
                let body = row((
                    chevron,
                    icon(item.get_untracked().icon),
                    label,
                    edit,
                    cells,
                    controls::restriction(item, context.clone(), 1, painting)
                        .hidden(move || !restrictions.get()),
                    controls::restriction(item, context.clone(), 0, painting)
                        .hidden(move || !restrictions.get()),
                    controls::restriction(item, context, 2, painting)
                        .hidden(move || !restrictions.get()),
                    active_marker,
                    line(Placement::Before),
                    line(Placement::After),
                ))
                .node_ref(reference)
                .checked(move || active.get() == Some(id))
                .style(move || {
                    let t = &current_theme();
                    let p = t.get::<crate::tokens::Tokens>();
                    let rest = style()
                        .height(row_height.map_or(p.row, |height| {
                            let height = height.get();
                            if height.is_finite() {
                                height.max(20.0)
                            } else {
                                p.row
                            }
                        }))
                        .padding_left(
                            8 + (levels
                                .with(|map| map.get(&id).copied())
                                .unwrap_or(entry.level) as i32
                                - 1)
                                * 16,
                        )
                        .padding_right(5)
                        .gap(5)
                        .align_items(Align::Center)
                        .cursor(Cursor::Default)
                        .color(if item.get().visible { p.text } else { p.dim })
                        .hover(style().background(ColorToken::Hover))
                        .selected(style().background(p.selection));
                    if mark.get() == Some(Placement::Into) {
                        rest.background(p.selection).ring(1, p.accent)
                    } else {
                        rest
                    }
                });
                let context_for_key = controls::Context {
                    expanded,
                    session,
                    selected,
                    on_restriction,
                    data,
                    rows: mounted.clone(),
                };
                let released = pressed.clone();
                let pressed = pressed.clone();
                let rows = mounted.clone();
                let drop_at = move |pos: Point| {
                    let (over, fraction) = rows.borrow().iter().find_map(|(&over, row)| {
                        let rect = row.rect.get_untracked()?;
                        rect.contains(pos)
                            .then(|| (over, (pos.y - rect.y) / rect.height))
                    })?;
                    data.with_untracked(|items| plan_drop(items, id, over, fraction))
                };
                let drop_at = Rc::new(drop_at);
                let moved_drop = drop_at.clone();
                let dragged = |update: &DragUpdate<u64>| update.delta.x.hypot(update.delta.y) > 4.0;
                let dragger = dragger
                    .on_update(move |update: DragUpdate<u64>| {
                        plan.set(if dragged(&update) {
                            moved_drop(update.pos).map(|(plan, _)| plan)
                        } else {
                            None
                        });
                    })
                    .on_end(move |update: DragUpdate<u64>| {
                        plan.set(None);
                        if update.phase != DragPhase::End {
                            return;
                        }
                        if let Some((_, drop)) =
                            dragged(&update).then(|| drop_at(update.pos)).flatten()
                            && let Some(callback) = item_drop
                        {
                            callback.call(drop);
                        }
                    });
                // Consume the drag Part after the Tree Part so selection runs
                // before a drag or double-click rename can handle the press.
                let body = body
                    .part(tree.item(entry.key))
                    .on_key(move |e: KeyEvent| {
                        let plain = !e.modifiers.ctrl && !e.modifiers.alt && !e.modifiers.meta;
                        // A key from the rename field or a cell is theirs.
                        if !focused.get() {
                            return Handled::No;
                        }
                        // Escape passes on: the list's own Escape ends a
                        // restriction paint, and the drag's source a drag.
                        match e.key {
                            Key::F(2) if plain => {
                                start_rename();
                                Handled::Yes
                            }
                            Key::Char('h' | 'H') if plain => {
                                controls::toggle(&context_for_key, id, RestrictionFlag::Visible);
                                Handled::Yes
                            }
                            _ => Handled::No,
                        }
                    })
                    .part(kivact::primitive::part(|input| input).on_pointer(
                        move |e: PointerEvent| {
                            let primary = e.button == Some(MouseButton::Primary);
                            if e.phase == PointerPhase::Down && primary {
                                pressed.set(Some(id));
                                if e.click_count == 2
                                    && !has_activation
                                    && renaming.get() != Some(id)
                                {
                                    dragger.cancel();
                                    start_rename();
                                    return Handled::Yes;
                                }
                            }
                            if matches!(e.phase, PointerPhase::Up | PointerPhase::Cancel) {
                                released.set(None);
                            }
                            Handled::No
                        },
                    ));
                if item_drop.is_some() {
                    body.part(dragger.source(id))
                } else {
                    body
                }
            }
        });
        root.part(tree.list())
            .child(rows)
            .on_key(move |event| {
                if event.key == Key::Escape && session.cancel() {
                    painting.set(None);
                    Handled::Yes
                } else {
                    Handled::No
                }
            })
            .into_view()
    }
}
