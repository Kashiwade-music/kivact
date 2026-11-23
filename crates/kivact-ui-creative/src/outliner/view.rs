//! Role: The outliner component: the hierarchy tree view, its rows, filter and drag.

use super::drop_plan::{Drag, DragEvent, DragMachine, Placement, plan_drop};
use crate::color::{ACCENT, DIM, MUTED, SELECTION, TEXT};
use crate::editors::shared::model::EditorEvent;
use crate::outliner_data::{Restriction, RestrictionFlag, TreeDrop, TreeItem};
use crate::size::ROW;
use crate::{LabelTone, OutlinerState};
use kivact::component;
use kivact::custom::Generation;
use kivact::geometry::Point;
use kivact::i18n::message;
use kivact::interaction::{
    DragCoordinates, DragPhase, DragUpdate, EditSession, Selection, TreeNode, use_drag, use_tree,
};
use kivact::state::{Callback, IntoValue, Value, callback, machine, memo, signal};
use kivact::style::{
    Align, Cursor, FontStyle, Length, Overflow, Position, color as style_color, pct, style,
};
use kivact::ui::tree_rows;
use kivact::view::{
    EditPhase, Element, Handled, IntoView, Key, KeyEvent, MouseButton, Node, PointerEvent,
    PointerPhase, View, column, icon, node, node_ref, row, show, text,
};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

type CellRenderer = kivact::view::ViewFn<Value<TreeItem>>;

/// A searchable hierarchy on [`Tree`](kivact::interaction::TreeApi).
/// It retains expansion, rename, restriction flags and drag and drop,
/// and hosts the application's cells.
///
/// The rows are one tab stop and follow the ARIA tree view (Up, Down, Home,
/// End, Right, Left, `*`, Enter). Shift and Ctrl or Cmd select a range or add.
/// F2 renames the focused row and `H` toggles its
/// visibility (so typing `h` does not jump to a label).
///
/// A double click renames the row, unless `on_activate` is set: then a double
/// click and Enter activate the item and only F2 renames. An item that is not
/// `selectable` is shown as a disabled row. The toggles at the end of a row
/// are not tab stops (they have their accessibility actions, and a press
/// toggles them; drag across rows to paint a value).
///
/// A drag moves an item. The pointer decides the place (see [`TreeDrop`]),
/// shown by a line before or after the row, or a highlighted row for "into
/// it"; Escape cancels. With `on_item_drop` the drop is reported and the
/// application moves the item; without it, the outliner keeps dragging
/// disabled.
///
/// The props: `restrictions` shows or hides the three per-item selection,
/// visibility and render controls; `on_rename` reports a submitted name (the
/// application decides whether to accept it); `on_restriction` reports
/// absolute restriction proposals (restore application items on Cancel;
/// controls are disabled without a callback); `filter` shows only the items
/// whose label contains the text (ignoring case) and their ancestors, all
/// open; `on_activate` receives the item on Enter and on a double click;
/// `on_item_drop` receives where a dragged item was dropped (see [`TreeDrop`])
/// instead of moving it in the items, and a cancelled drag reports nothing;
/// `active` is the application's current object, drawn with an accent marker
/// (it is not the selection); `reveal_active` (default: true) opens the
/// ancestors of the active item and scrolls its row into view when `active`
/// changes; `cells` are fixed-width application cells (see [`OutlinerCell`]);
/// `row_height` is the height of a row (at least 20; the default is the
/// density's row).
#[component]
pub fn outliner(
    items: Value<Vec<TreeItem>>,
    state: OutlinerState,
    #[prop(default = true)] restrictions: Value<bool>,
    on_rename: Option<Callback<(u64, String)>>,
    on_restriction: Option<Callback<EditorEvent<Restriction>>>,
    #[prop(default = String::new())] filter: Value<String>,
    on_activate: Option<Callback<u64>>,
    on_item_drop: Option<Callback<TreeDrop>>,
    #[prop(default = None::<u64>)] active: Value<Option<u64>>,
    #[prop(default = true)] reveal_active: Value<bool>,
    #[prop(default = Vec::new())] cells: Vec<OutlinerCell>,
    row_height: Option<Value<f64>>,
) -> impl Element {
    let root = column(())
        .name(message("creative.outliner.label", "Outliner"))
        .style(
            style()
                .width(pct(100))
                .min_width(0)
                .overflow(Overflow::Hidden),
        );
    let rename_edit = kivact::interaction::use_inline_edit(state.renaming);
    let has_activation = on_activate.is_some();
    let (by_id, nodes) = item_indexes(items);
    let mut tree =
        use_tree(nodes, state.expanded, Selection::Multiple(state.selection)).filter(filter);
    if let Some(callback) = on_activate {
        tree = tree.on_activate(move |value| callback.try_call(value));
    }
    let mounted: super::controls::Rows = Rc::new(RefCell::new(HashMap::new()));
    let drag = machine(Drag::Idle, || DragMachine, |_, ()| {});
    let reveal = super::reveal::observe(
        active,
        reveal_active,
        items,
        state.expanded,
        mounted.clone(),
    );
    let levels = memo(move || {
        tree.rows().with(|rows| {
            rows.iter()
                .map(|row| (row.key, row.level))
                .collect::<HashMap<_, _>>()
        })
    });
    let factory = Rc::new(RowFactory {
        by_id,
        data: items,
        rename_edit,
        generation: Rc::new(Cell::new(Generation::new())),
        mounted,
        reveal,
        on_rename,
        expanded: state.expanded,
        selected: state.selection,
        painter: restriction_painter(on_restriction),
        on_restriction,
        cells: cells
            .into_iter()
            .map(|cell| (cell.width, cell.render))
            .collect(),
        drag,
        active,
        restrictions,
        row_height,
        levels,
        item_drop: on_item_drop,
        has_activation,
    });
    let painter = factory.painter;
    let rows = tree_rows(
        tree,
        |_| (),
        move |_, entry, part| factory.build(entry, part),
    );
    root.part(tree.list()).child(rows).on_key(move |event| {
        if event.key() == Key::Escape && painter.cancel() {
            Handled::Yes
        } else {
            Handled::No
        }
    })
}

/// A fixed-width application cell after the label of an [`outliner`] row,
/// given to the outliner as `cells`.
///
/// Cells keep their width while the label shrinks, and clip overflowing
/// content. Pointer and key input in a cell stays with its controls instead of
/// selecting, dragging or renaming the row. Tab keeps normal focus navigation.
/// Non-finite or non-positive widths hide the cell.
pub struct OutlinerCell {
    pub(crate) width: Value<f64>,
    pub(crate) render: CellRenderer,
}

impl OutlinerCell {
    /// A cell `width` pixels wide whose content `render` builds for a row's item.
    pub fn new<V: IntoView>(
        width: impl IntoValue<f64>,
        render: impl Fn(Value<TreeItem>) -> V + 'static,
    ) -> Self {
        let width = width.into_value();
        let width = memo(move || {
            let width = width.get();
            if width.is_finite() && width > 0.0 {
                width
            } else {
                0.0
            }
        });
        Self {
            width,
            render: kivact::view::ViewFn::from(render),
        }
    }
}

// xtask-allow(state): identity: generation is the mount-id allocator advanced only by RowFactory::register, so a row's cleanup can tell its own mount from a newer one; row_height and has_activation are copied once from the outliner and never written; generation of registered rows
struct RowFactory {
    by_id: Value<HashMap<u64, TreeItem>>,
    data: Value<Vec<TreeItem>>,
    rename_edit: kivact::interaction::InlineEditApi<u64>,
    generation: Rc<Cell<Generation>>,
    mounted: super::controls::Rows,
    reveal: kivact::state::MachineHandle<super::reveal::Reveal>,
    on_rename: Option<Callback<(u64, String)>>,
    expanded: kivact::state::Signal<Vec<u64>>,
    selected: kivact::state::Signal<Vec<u64>>,
    painter: super::controls::Painter,
    on_restriction: Option<Callback<EditorEvent<Restriction>>>,
    cells: Vec<(Value<f64>, CellRenderer)>,
    drag: kivact::state::MachineHandle<DragMachine>,
    active: Value<Option<u64>>,
    restrictions: Value<bool>,
    row_height: Option<Value<f64>>,
    levels: Value<HashMap<u64, u32>>,
    item_drop: Option<Callback<TreeDrop>>,
    has_activation: bool,
}
impl RowFactory {
    fn build(
        &self,
        entry: Value<kivact::interaction::TreeRow<u64>>,
        part: kivact::custom::Part<kivact::interaction::TreeItemState>,
    ) -> Node {
        let entry = entry.get_untracked();
        let id = entry.key;
        let by_id = self.by_id;
        let Some(fallback) = by_id.with_untracked(|map| map.get(&id).cloned()) else {
            return node();
        };
        let item = memo(move || {
            by_id
                .with(|map| map.get(&id).cloned())
                .unwrap_or_else(|| fallback.clone())
        });
        let reference = self.register(id);
        let drag = self.drag;
        let mark = memo(move || match drag.get() {
            Drag::Dragging {
                plan: Some(plan), ..
            } if plan.over == id => Some(plan.placement),
            _ => None,
        });
        let body = self.chrome(item, entry, reference, mark);
        self.behavior(body, part, id, reference)
    }
}
fn row_label(
    item: Value<TreeItem>,
    rename_edit: kivact::interaction::InlineEditApi<u64>,
    id: u64,
) -> impl IntoView {
    show(
        move || !is_renaming(rename_edit, id),
        move || {
            text(move || item.with(|n| n.label.clone())).style(move || {
                item.with(|item| {
                    let color = if item.placeholder && item.tone == LabelTone::Normal {
                        MUTED
                    } else {
                        item.tone.color()
                    };
                    style()
                        .flex_grow(1)
                        .flex_shrink(1000)
                        .min_width(0)
                        .single_line()
                        .font_style(if item.placeholder {
                            FontStyle::Italic
                        } else {
                            FontStyle::Normal
                        })
                        .color(color)
                })
            })
        },
    )
}

fn rename_field(
    item: Value<TreeItem>,
    rename_edit: kivact::interaction::InlineEditApi<u64>,
    id: u64,
    on_rename: Option<Callback<(u64, String)>>,
    reference: kivact::view::NodeRef,
) -> impl IntoView {
    show(
        move || is_renaming(rename_edit, id),
        move || {
            node()
                .part(
                    rename_edit
                        .field(
                            id,
                            message("creative.outliner.rename", "Rename tree item"),
                            signal(item.get_untracked().label),
                        )
                        .on_commit(move |name| {
                            if let Some(callback) = on_rename {
                                callback.call((id, name));
                            }
                        })
                        .return_focus(reference)
                        .control(),
                )
                .style(style().flex_grow(1).height(22))
        },
    )
}

fn row_cells(item: Value<TreeItem>, cells: &[(Value<f64>, CellRenderer)]) -> Vec<View> {
    cells
        .iter()
        .map(|(width, render)| {
            let width = *width;
            row(render.call(item))
                .on_pointer_default(|_| Handled::Yes)
                .on_key_default(|event| {
                    // Enter/Space activate the focused child only after
                    // key bubbling. Tree/List already ignore child focus.
                    if matches!(event.key(), Key::Tab | Key::Enter | Key::Space) {
                        Handled::No
                    } else {
                        Handled::Yes
                    }
                })
                .hidden(move || width.get() <= 0.0)
                .style(move || {
                    style()
                        .align_items(Align::Center)
                        .gap(4)
                        .width(width.get())
                        .flex_shrink(0)
                        .min_width(0)
                        .overflow(Overflow::Hidden)
                })
                .into_view()
        })
        .collect::<Vec<_>>()
}

fn row_drag(
    id: u64,
    data: Value<Vec<TreeItem>>,
    mounted: super::controls::Rows,
    drag: kivact::state::MachineHandle<DragMachine>,
    item_drop: Option<Callback<TreeDrop>>,
    dragger: kivact::interaction::DragApi<u64>,
) -> kivact::interaction::DragApi<u64> {
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
    dragger
        .on_start(move |_| drag.send(DragEvent::Start(id)))
        .on_update(move |update: DragUpdate<u64>| {
            drag.send(DragEvent::Plan(if dragged(&update) {
                moved_drop(update.pos).map(|(plan, _)| plan)
            } else {
                None
            }));
        })
        .on_end(move |update: DragUpdate<u64>| {
            drag.send(if update.phase == DragPhase::End {
                DragEvent::End
            } else {
                DragEvent::Cancel
            });
            if update.phase != DragPhase::End {
                return;
            }
            if let Some((_, drop)) = dragged(&update).then(|| drop_at(update.pos)).flatten()
                && let Some(callback) = item_drop
            {
                callback.call(drop);
            }
        })
}

fn row_style(
    item: Value<TreeItem>,
    row_height: Option<Value<f64>>,
    levels: Value<HashMap<u64, u32>>,
    id: u64,
    level: u32,
    mark: Value<Option<Placement>>,
) -> kivact::style::Style {
    let height: Length = match row_height {
        Some(height) => {
            let height = height.get();
            if height.is_finite() {
                height.max(20.0).into()
            } else {
                ROW
            }
        }
        None => ROW,
    };
    let rest = style()
        .height(height)
        .padding_left(
            8 + (levels.with(|map| map.get(&id).copied()).unwrap_or(level) as i32 - 1) * 16,
        )
        .padding_right(5)
        .gap(5)
        .align_items(Align::Center)
        .cursor(Cursor::Default)
        .color(if item.get().visible { TEXT } else { DIM })
        .hover(style().background(style_color::HOVER))
        .selected(style().background(SELECTION));
    if mark.get() == Some(Placement::Into) {
        rest.background(SELECTION).outline(1, ACCENT)
    } else {
        rest
    }
}

impl RowFactory {
    fn register(&self, id: u64) -> kivact::view::NodeRef {
        let generation = self.generation.clone();
        let mounted = self.mounted.clone();
        let reveal = self.reveal;
        let reference = node_ref();
        let mut next = generation.get();
        let mount = next.next();
        generation.set(next);
        mounted.borrow_mut().insert(
            id,
            super::controls::Mounted {
                node: reference,
                rect: reference.rect(),
                generation: mount,
            },
        );
        let cleanup = mounted.clone();
        kivact::state::on_cleanup(move || {
            let mut rows = cleanup.borrow_mut();
            if rows
                .get(&id)
                .is_some_and(|row| row.generation.is_current(mount))
            {
                rows.remove(&id);
            }
        });
        kivact::state::on_mount(move || {
            reveal.send(super::reveal::Event::Mounted {
                id,
                node: reference,
            });
        });
        reference
    }
    fn chrome(
        &self,
        item: Value<TreeItem>,
        entry: kivact::interaction::TreeRow<u64>,
        reference: kivact::view::NodeRef,
        mark: Value<Option<Placement>>,
    ) -> Node {
        let rename_edit = self.rename_edit;
        let on_rename = self.on_rename;
        let expanded = self.expanded;
        let painter = self.painter;
        let selected = self.selected;
        let on_restriction = self.on_restriction;
        let data = self.data;
        let mounted = self.mounted.clone();
        let cells = self.cells.clone();
        let active = self.active;
        let restrictions = self.restrictions;
        let row_height = self.row_height;
        let levels = self.levels;
        let id = entry.key;
        let label = row_label(item, rename_edit, id);
        let edit = rename_field(item, rename_edit, id, on_rename, reference);
        let context = super::controls::Context {
            expanded,
            painter,
            selected,
            on_restriction,
            data,
            rows: mounted.clone(),
        };
        let chevron = if entry.branch.has_children() {
            super::controls::expansion(item, context.clone()).into_view()
        } else {
            // The room of the button, so the labels line up.
            column(())
                .style(style().size(18, 18).flex_shrink(0))
                .into_view()
        };
        let cells = row_cells(item, &cells);
        let line = move |at| drop_line(mark, at);
        let active_marker = column(()).hidden(move || active.get() != Some(id)).style(
            style()
                .position(Position::Absolute)
                .left(0)
                .top(0)
                .bottom(0)
                .width(2)
                .background(ACCENT),
        );
        row((
            chevron,
            icon(item.get_untracked().icon),
            label,
            edit,
            cells,
            super::controls::restriction(item, context.clone(), 1)
                .hidden(move || !restrictions.get()),
            super::controls::restriction(item, context.clone(), 0)
                .hidden(move || !restrictions.get()),
            super::controls::restriction(item, context, 2).hidden(move || !restrictions.get()),
            active_marker,
            line(Placement::Before),
            line(Placement::After),
        ))
        .node_ref(reference)
        .checked(move || active.get() == Some(id))
        .style(move || row_style(item, row_height, levels, id, entry.level, mark))
    }
    fn behavior(
        &self,
        body: Node,
        part: kivact::custom::Part<kivact::interaction::TreeItemState>,
        id: u64,
        reference: kivact::view::NodeRef,
    ) -> Node {
        let expanded = self.expanded;
        let painter = self.painter;
        let selected = self.selected;
        let on_restriction = self.on_restriction;
        let data = self.data;
        let mounted = self.mounted.clone();
        let reveal = self.reveal;
        let drag = self.drag;
        let item_drop = self.item_drop;
        let has_activation = self.has_activation;
        let rename_edit = self.rename_edit;
        let focused = reference.focused();
        let dragger = use_drag::<u64>(DragCoordinates::Window);
        let context_for_key = super::controls::Context {
            expanded,
            painter,
            selected,
            on_restriction,
            data,
            rows: mounted.clone(),
        };
        let start_rename = move || rename_edit.edit(id);
        let dragger = row_drag(id, data, mounted.clone(), drag, item_drop, dragger);
        // Consume the drag Part after the Tree Part so selection runs
        // before a drag or double-click rename can handle the press.
        let body = body
            .part(part)
            .on_key(move |e: KeyEvent| {
                let plain = !e.modifiers().ctrl && !e.modifiers().alt && !e.modifiers().meta;
                // A key from the rename field or a cell is theirs.
                if !focused.get() {
                    return Handled::No;
                }
                // Escape passes on: the list's own Escape ends a
                // restriction paint, and the drag's source a drag.
                match e.key() {
                    Key::F(2) if plain => {
                        start_rename();
                        Handled::Yes
                    }
                    Key::Char('h' | 'H') if plain => {
                        super::controls::toggle(&context_for_key, id, RestrictionFlag::Visible);
                        Handled::Yes
                    }
                    _ => Handled::No,
                }
            })
            .part({
                let input_part = kivact::custom::part(|input| input);
                kivact::custom::Part::new(input_part.state(), move |node| {
                    node.part(input_part).on_pointer(move |e: PointerEvent| {
                        if let PointerPhase::Down(press) = e.phase()
                            && press.button == MouseButton::Primary
                        {
                            reveal.send(super::reveal::Event::Press(Some(id)));
                            if press.clicks == 2 && !has_activation && !is_renaming(rename_edit, id)
                            {
                                dragger.cancel();
                                start_rename();
                                return Handled::Yes;
                            }
                        }
                        if matches!(e.phase(), PointerPhase::Up(_) | PointerPhase::Cancel) {
                            reveal.send(super::reveal::Event::Press(None));
                        }
                        Handled::No
                    })
                })
            });
        if item_drop.is_some() {
            body.part(dragger.source(id))
        } else {
            body
        }
    }
}
fn drop_line(mark: Value<Option<Placement>>, at: Placement) -> Node {
    column(())
        .hidden(move || mark.get() != Some(at))
        .style(move || {
            let base = style()
                .position(Position::Absolute)
                .left(0)
                .right(0)
                .height(2)
                .z_index(5)
                .background(ACCENT);
            if at == Placement::Before {
                base.top(0)
            } else {
                base.bottom(0)
            }
        })
}

type ItemIndexes = (Value<HashMap<u64, TreeItem>>, Value<Vec<TreeNode<u64>>>);
fn item_indexes(data: Value<Vec<TreeItem>>) -> ItemIndexes {
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
    (by_id.into_value(), nodes.into_value())
}

fn restriction_painter(
    on_restriction: Option<Callback<EditorEvent<Restriction>>>,
) -> super::controls::Painter {
    super::controls::Painter::new(crate::editors::shared::model::observed(
        EditSession::new(|_: super::controls::Paint| {}),
        on_restriction.map(|observer| {
            callback(move |phase| {
                observer.call(match phase {
                    EditPhase::Begin => EditorEvent::Begin,
                    EditPhase::Commit => EditorEvent::Commit,
                    EditPhase::Cancel => EditorEvent::Cancel,
                });
            })
        }),
    ))
}

/// Whether the item `id` is being renamed.
fn is_renaming(edit: kivact::interaction::InlineEditApi<u64>, id: u64) -> bool {
    edit.editing().with(|key| *key == Some(id))
}
