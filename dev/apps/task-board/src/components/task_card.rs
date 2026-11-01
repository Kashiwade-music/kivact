//! Role: A task card.
//! The row keeps its owner while its key stays in the lane;
//! the task itself arrives as a `Value` (no setup-time snapshot).

use crate::context::{BoardCx, Refocus};
use crate::model::{Lane, Task};
use kivact::prelude::*;
use std::sync::Arc;

pub(super) const CARD_HEIGHT: f32 = 124.0;

/// The card of task `id`; `task` follows the row's current value.
pub(super) fn task_card(id: u64, task: Value<Arc<Task>>) -> impl IntoView {
    let state = context::<BoardCx>();
    let done = memo(move || task.with(|t| t.lane == Lane::Done));
    let visits = memo(move || state.visits.with(|v| v.get(&id).copied().unwrap_or(0)));
    let right_ref = node_ref();
    let done_ref = node_ref();
    // Keep the keyboard focus on the card after it moved to another lane.
    on_mount(move || {
        if let Some((moved, target)) = state.refocus.get()
            && moved == id
        {
            state.refocus.set(None);
            match target {
                Refocus::Right if !done.get() => right_ref.focus(),
                _ => done_ref.focus(),
            }
        }
    });
    column((
        text(move || task.with(|t| t.title.clone()))
            .style(style().weight(600))
            .test_id("card-title"),
        text(move || task.with(|t| t.notes.clone())).style(style().color(ColorToken::TextMuted)),
        text(move || {
            let mark = if state.is_selected.is(&id) {
                " · 選択中"
            } else {
                ""
            };
            format!(
                "{} · 閲覧 {}{mark}",
                task.with(|t| t.lane.label()),
                visits.get()
            )
        })
        .test_id(format!("card-meta-{id}")),
        row((
            button("選択")
                .variant(ButtonVariant::Secondary)
                .on_press(move || state.select(id))
                .test_id(format!("select-{id}")),
            button("右へ")
                .variant(ButtonVariant::Secondary)
                .disabled(done)
                .on_press(move || {
                    state.refocus.set(Some((id, Refocus::Right)));
                    state.move_task(id, true);
                })
                .node_ref(right_ref)
                .test_id(format!("right-{id}")),
            {
                let checked = binding(done, move |on| {
                    state.refocus.set(Some((id, Refocus::Done)));
                    state.set_done(id, on);
                });
                checkbox("完了", checked)
            }
            .node_ref(done_ref)
            .name(move || format!("完了 {}", task.with(|t| t.title.clone())))
            .test_id(format!("done-{id}")),
        ))
        .style(style().gap(6).align_items(Align::Center)),
    ))
    .role(Role::ListItem)
    .name(move || task.with(|t| t.title.clone()))
    .selected(move || state.is_selected.is(&id))
    .test_id(format!("card-{id}"))
    .style(
        style()
            .height(CARD_HEIGHT)
            .gap(4)
            .padding(8)
            .radius(6)
            .border(1, ColorToken::Border)
            .background(ColorToken::Surface)
            .selected(style().border(2, ColorToken::Accent)),
    )
}
