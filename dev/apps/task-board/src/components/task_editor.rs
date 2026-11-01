//! Role: Editor of the selected task (mounted by `show_some` while one is selected).

use crate::context::BoardCx;
use crate::model::Lane;
use kivact::prelude::*;

pub(super) fn task_editor(id: Value<u64>) -> impl IntoView {
    let state = context::<BoardCx>();
    let lane = memo(move || {
        let id = id.get();
        state
            .board
            .with(|board| board.task(id).map_or_else(Lane::default, |t| t.lane))
    });
    let lanes = Lane::ALL
        .map(|lane| ListItem::new(lane, lane.label()))
        .to_vec();
    let selected_lane = binding(lane, move |lane| state.set_selected_lane(lane)).optional();
    row((
        text(move || {
            let id = id.get();
            state.board.with(|board| {
                board.task(id).map_or_else(
                    || "選択なし".to_string(),
                    |task| format!("選択: {} / {}", task.title, task.lane.label()),
                )
            })
        })
        .test_id("editor-label"),
        button("更新")
            .on_press(move || state.edit_selected())
            .test_id("edit-task"),
        button("左へ")
            .variant(ButtonVariant::Secondary)
            .on_press(move || state.move_selected(false))
            .test_id("move-left"),
        button("右へ")
            .variant(ButtonVariant::Secondary)
            .on_press(move || state.move_selected(true))
            .test_id("move-right"),
        kivact::ui::select("列", lanes, selected_lane).test_id("lane-select"),
        button("削除")
            .variant(ButtonVariant::Secondary)
            .on_press(move || state.delete_selected())
            .test_id("delete-task"),
    ))
    .role(Role::Toolbar)
    .name("選択中のタスク")
    .style(
        style()
            .gap(8)
            .padding(8)
            .align_items(Align::Center)
            .radius(6)
            .background(ColorToken::SurfaceAlt),
    )
    .test_id("task-editor")
}
