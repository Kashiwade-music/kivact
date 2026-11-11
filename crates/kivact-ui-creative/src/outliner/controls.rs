//! Role: Per-row restriction toggles and expansion controls of the outliner.

use crate::editors::shared::model::EditorEvent;
use crate::outliner_data::{Restriction, RestrictionFlag, TreeItem};
use kivact::custom::Generation;
use kivact::geometry::Rect;
use kivact::i18n::message;
use kivact::interaction::EditSession;
use kivact::view::Element;

use kivact::state::{Binding, Callback, Value};
use kivact::style::style;
use kivact::view::MouseButton;
use kivact::view::{A11yAction, ButtonPress, Handled, NodeRef, PointerPhase};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// A row that is mounted: its node, its rect, and which mount it is (a row
/// that is built again under the same id replaces it).
pub(super) struct Mounted {
    pub node: NodeRef,
    pub rect: Value<Option<Rect>>,
    pub generation: Generation,
}
pub(super) type Rows = Rc<RefCell<HashMap<u64, Mounted>>>;

#[derive(Clone)]
pub(super) struct Context {
    pub expanded: Binding<Vec<u64>>,
    pub selected: Binding<Vec<u64>>,
    pub on_restriction: Option<Callback<EditorEvent<Restriction>>>,
    pub painter: Painter,
    pub data: Value<Vec<TreeItem>>,
    pub rows: Rows,
}
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Paint {
    column: usize,
    value: bool,
    touched: Vec<u64>,
}

/// One restriction paint: a drag across rows that sets one flag on each row
/// it touches, as one edit. The paint is the payload of the edit's active
/// phase, so it exists exactly while the edit does.
#[derive(Clone, Copy)]
pub(super) struct Painter {
    session: EditSession<Paint>,
}

impl Painter {
    pub(super) fn new(session: EditSession<Paint>) -> Self {
        Painter { session }
    }

    /// Begin the paint and propose its first rows; nothing happens (and
    /// `false` returns) while an edit runs.
    fn start(self, context: &Context, paint: Paint) -> bool {
        if self.session.is_active() {
            return false;
        }
        self.session.begin(paint.clone());
        propose(context, paint.column, &paint.touched, paint.value);
        true
    }

    /// The pointer is over the row `over`: add its rows and propose again.
    fn extend(self, context: &Context, over: u64) {
        let Some(mut paint) = self.session.with_initial(Paint::clone) else {
            return;
        };
        if paint.touched.contains(&over) {
            return;
        }
        for id in ids(context, over) {
            if !paint.touched.contains(&id) {
                paint.touched.push(id);
            }
        }
        propose(context, paint.column, &paint.touched, paint.value);
        self.session.replace_initial(paint);
    }

    /// The button is released: end the paint and commit its edit.
    fn finish(self) {
        self.session.commit();
    }

    /// End the paint and cancel its edit; whether a paint ran.
    pub(super) fn cancel(self) -> bool {
        self.session.cancel()
    }

    /// A single toggle (a key or an assistive action) as one edit.
    fn once(self, paint: Paint, write: impl FnOnce()) {
        self.session.once(paint, write);
    }
}

fn flag(item: &TreeItem, column: usize) -> bool {
    match column {
        0 => item.visible,
        1 => item.selectable,
        _ => item.rendered,
    }
}
fn restriction_flag(column: usize) -> RestrictionFlag {
    [
        RestrictionFlag::Visible,
        RestrictionFlag::Selectable,
        RestrictionFlag::Rendered,
    ][column]
}
fn ids(context: &Context, id: u64) -> Vec<u64> {
    let selected = context.selected.get_untracked();
    if selected.contains(&id) {
        selected
    } else {
        vec![id]
    }
}
fn propose(context: &Context, column: usize, ids: &[u64], value: bool) {
    if let Some(callback) = context.on_restriction {
        callback.call(EditorEvent::Request(Restriction {
            ids: ids.to_vec(),
            flag: restriction_flag(column),
            value,
        }));
    }
}
pub(super) fn toggle(context: &Context, id: u64, flag: RestrictionFlag) {
    let column = match flag {
        RestrictionFlag::Visible => 0,
        RestrictionFlag::Selectable => 1,
        RestrictionFlag::Rendered => 2,
    };
    if let Some(item) = context
        .data
        .with_untracked(|items| items.iter().find(|item| item.id == id).cloned())
    {
        let ids = ids(context, id);
        let value = !self::flag(&item, column);
        let paint = Paint {
            column,
            value,
            touched: ids.clone(),
        };
        context
            .painter
            .once(paint, || propose(context, column, &ids, value));
    }
}
pub(super) fn restriction(
    item: Value<TreeItem>,
    context: Context,
    column: usize,
) -> impl kivact::view::Element {
    let id = item.get_untracked().id;
    let mut button = kivact::ui::button(
        [
            message("creative.outliner.visibility", "Tree visibility"),
            message(
                "creative.outliner.selection_restriction",
                "Tree selection restriction",
            ),
            message(
                "creative.outliner.render_visibility",
                "Tree render visibility",
            ),
        ][column],
    )
    .variant(kivact::ui::ButtonVariant::Secondary)
    .hide_label(true)
    .icon(kivact::view::icon(["eye", "pointer", "camera"][column]));
    let action_context = context.clone();
    button = button
        .with_node(move |node| node.focusable(false))
        .with_node(move |node| node.disabled(context.on_restriction.is_none()))
        .with_node(move |node| node.checked(move || flag(&item.get(), column)))
        .with_node(move |node| node.selected(move || flag(&item.get(), column)))
        .with_node(move |node| {
            node.a11y_action(A11yAction::Toggle, move |_| {
                toggle(&action_context, id, restriction_flag(column));
            })
        })
        .with_node(move |node| {
            node.on_pointer_default(move |event| {
                match event.phase() {
                    PointerPhase::Down(ButtonPress {
                        button: MouseButton::Primary,
                        ..
                    }) => {
                        let value = !flag(&item.get(), column);
                        let touched = ids(&context, id);
                        let paint = Paint {
                            column,
                            value,
                            touched,
                        };
                        if !context.painter.start(&context, paint) {
                            return Handled::Yes;
                        }
                    }
                    PointerPhase::Move => {
                        let target = context.rows.borrow().iter().find_map(|(&id, mounted)| {
                            let rect = mounted.rect.get()?;
                            (event.window_pos().x >= rect.x
                                && event.window_pos().x <= rect.x + rect.width
                                && event.window_pos().y >= rect.y
                                && event.window_pos().y < rect.y + rect.height)
                                .then_some(id)
                        });
                        if let Some(id) = target {
                            context.painter.extend(&context, id);
                        }
                    }
                    PointerPhase::Up(_) => context.painter.finish(),
                    PointerPhase::Cancel => {
                        context.painter.cancel();
                    }
                    _ => return Handled::No,
                }
                Handled::Yes
            })
        })
        .with_node(move |node| node.style(style().size(20, 20).padding(0)));
    button
}
pub(super) fn expansion(item: Value<TreeItem>, context: Context) -> impl kivact::view::Element {
    let id = item.get_untracked().id;
    let expand = move |subtree: bool| {
        let next = context.data.get_untracked();
        let mut open = context.expanded.get_untracked();
        let value = !open.contains(&id);
        let mut ids = vec![id];
        if subtree {
            for _ in 0..next.len() {
                let more = next
                    .iter()
                    .filter(|n| n.parent.is_some_and(|p| ids.contains(&p)) && !ids.contains(&n.id))
                    .map(|n| n.id)
                    .collect::<Vec<_>>();
                if more.is_empty() {
                    break;
                }
                ids.extend(more);
            }
        }
        open.retain(|id| !ids.contains(id));
        if value {
            open.extend(ids);
        }
        context.expanded.set(open);
    };
    let mut button = kivact::ui::button(message("creative.outliner.expand", "Expand tree item"))
        .variant(kivact::ui::ButtonVariant::Secondary)
        .hide_label(true)
        .icon(kivact::view::icon("chevron-down"));
    button = button
        .with_node(move |node| node.focusable(false))
        .with_node(move |node| node.expanded(move || context.expanded.get().contains(&id)))
        .with_node(move |node| node.style(style().size(18, 18).padding(0)))
        .with_node(move |node| {
            node.style(move || {
                style().rotate(if context.expanded.get().contains(&id) {
                    0
                } else {
                    -90
                })
            })
        })
        .with_node(move |node| node.a11y_action(A11yAction::Toggle, move |_| expand(false)))
        .with_node(move |node| {
            node.on_pointer_default(move |event| {
                if matches!(
                    event.phase(),
                    PointerPhase::Down(ButtonPress {
                        button: MouseButton::Primary,
                        ..
                    })
                ) {
                    expand(event.modifiers().shift);
                    return Handled::Yes;
                }
                if matches!(
                    event.phase(),
                    PointerPhase::Up(_) | PointerPhase::Move | PointerPhase::Cancel
                ) {
                    Handled::Yes
                } else {
                    Handled::No
                }
            })
        });
    button
}
