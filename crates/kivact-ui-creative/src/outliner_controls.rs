//! Role: Per-row restriction toggles and expansion controls of the outliner.

use crate::editors::shared::model::EditorEvent;
use crate::outliner_data::{Restriction, RestrictionFlag, TreeItem};
use kivact::control::{Node, NodeBuilder};
use kivact::draw::Rect;
use kivact::i18n::message;
use kivact::interaction::EditSession;
use kivact::primitive::NodeExt;
use kivact::style::style;
use kivact::view::MouseButton;
use kivact::view::{A11yAction, Handled, NodeRef, PointerPhase};
use kivact::{Binding, Callback, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// A row that is mounted: its node, its rect, and which mount it is (a row
/// that is built again under the same id replaces it).
pub(super) struct Mounted {
    pub node: NodeRef,
    pub rect: Value<Option<Rect>>,
    pub generation: u64,
}
pub(super) type Rows = Rc<RefCell<HashMap<u64, Mounted>>>;

#[derive(Clone)]
pub(super) struct Context {
    pub expanded: Binding<Vec<u64>>,
    pub selected: Binding<Vec<u64>>,
    pub on_restriction: Option<Callback<EditorEvent<Restriction>>>,
    pub session: EditSession<()>,
    pub data: Value<Vec<TreeItem>>,
    pub rows: Rows,
}
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Paint {
    column: usize,
    value: bool,
    touched: Vec<u64>,
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
        context.session.once((), || {
            propose(context, column, &ids, !self::flag(&item, column))
        });
    }
}
pub(super) fn restriction(
    item: Value<TreeItem>,
    context: Context,
    column: usize,
    painting: kivact::Signal<Option<Paint>>,
) -> Node {
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
    .icon(kivact::control::icon(["eye", "pointer", "camera"][column]));
    let action_context = context.clone();
    button = button
        .focusable(false)
        .disabled(context.on_restriction.is_none())
        .checked(move || flag(&item.get(), column))
        .selected(move || flag(&item.get(), column))
        .on_a11y(A11yAction::Toggle, move || {
            toggle(&action_context, id, restriction_flag(column));
        })
        .on_pointer_default(move |event| {
            match event.phase {
                PointerPhase::Down if event.button == Some(MouseButton::Primary) => {
                    let value = !flag(&item.get(), column);
                    let touched = ids(&context, id);
                    context.session.begin(());
                    propose(&context, column, &touched, value);
                    painting.set(Some(Paint {
                        column,
                        value,
                        touched,
                    }));
                }
                PointerPhase::Move => {
                    if let Some(mut paint) = painting.get() {
                        let target = context.rows.borrow().iter().find_map(|(&id, mounted)| {
                            let rect = mounted.rect.get()?;
                            (event.window_pos.x >= rect.x
                                && event.window_pos.x <= rect.x + rect.width
                                && event.window_pos.y >= rect.y
                                && event.window_pos.y < rect.y + rect.height)
                                .then_some(id)
                        });
                        if let Some(id) = target
                            && !paint.touched.contains(&id)
                        {
                            for id in ids(&context, id) {
                                if !paint.touched.contains(&id) {
                                    paint.touched.push(id);
                                }
                            }
                            propose(&context, paint.column, &paint.touched, paint.value);
                            painting.set(Some(paint));
                        }
                    }
                }
                PointerPhase::Up => {
                    painting.set(None);
                    context.session.commit();
                }
                PointerPhase::Cancel => {
                    painting.set(None);
                    context.session.cancel();
                }
                _ => return Handled::No,
            }
            Handled::Yes
        })
        .style(style().size(20, 20).padding(0));
    button.into_node()
}
pub(super) fn expansion(item: Value<TreeItem>, context: Context) -> Node {
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
        .icon(kivact::control::icon("chevron-down"));
    button = button
        .focusable(false)
        .expanded(move || context.expanded.get().contains(&id))
        .style(style().size(18, 18).padding(0))
        .style(move || {
            style().rotate(if context.expanded.get().contains(&id) {
                0
            } else {
                -90
            })
        })
        .on_a11y(A11yAction::Toggle, move || expand(false))
        .on_pointer_default(move |event| {
            if event.phase == PointerPhase::Down && event.button == Some(MouseButton::Primary) {
                expand(event.modifiers.shift);
                return Handled::Yes;
            }
            if matches!(
                event.phase,
                PointerPhase::Up | PointerPhase::Move | PointerPhase::Cancel
            ) {
                Handled::Yes
            } else {
                Handled::No
            }
        });
    button.into_node()
}
