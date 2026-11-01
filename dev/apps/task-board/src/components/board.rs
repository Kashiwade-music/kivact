//! Role: The Task Board's board: shared state, add form, editor, lanes and saving.
//!
//! The board: shared state in context, the add form, the editor, three lanes
//! and ordered saving through a coalescing queued action.

use super::lane::lane;
use super::search::search;
use super::task_editor::task_editor;
use crate::context::BoardCx;
use crate::model::{Board, Lane};
use crate::storage::save;
use kivact::effect_on;
use kivact::prelude::*;
use kivact::view::selector;
use std::collections::HashMap;
use std::path::PathBuf;

/// The loaded board `initial`, saved to the file in `path` on every change.
pub(super) fn board(initial: Board, path: Var<PathBuf>) -> impl IntoView {
    let board = signal(initial);
    let selected = signal(None::<u64>);
    let state = BoardCx {
        board,
        selected,
        is_selected: selector(selected),
        visits: signal(HashMap::new()),
        title: signal(String::new()),
        notes: signal(String::new()),
        search: signal(String::new()),
        notice: signal(None),
        refocus: var(None),
    };

    // One job at a time; a newer snapshot replaces the waiting one, the
    // running write is never overtaken. The App drains it (3 s) on close.
    let saver = action(move |snapshot: Board| {
        let file = path.get();
        async move {
            blocking(move || save(&file, &snapshot))
                .await
                .map_err(UiError::msg)
        }
    })
    .policy(ActionPolicy::QueueLatest);
    // Save every change (not the loaded board itself).
    effect_on(board, move |snapshot| saver.run(snapshot.clone()));
    let failed = move || saver.state().is_failed();

    let save_status = move || match saver.state() {
        AsyncState::Idle => "準備完了".to_string(),
        AsyncState::Pending => "保存中".to_string(),
        AsyncState::Ready(()) => "保存しました".to_string(),
        AsyncState::Failed(error) => format!("保存できません: {error}"),
    };

    provide(state, move || {
        column((
            search(),
            row((
                text_input("題名", state.title)
                    .placeholder("題名")
                    .style(style().width(200))
                    .test_id("task-title"),
                kivact::ui::text_area("メモ", state.notes)
                    .placeholder("メモ")
                    .style(style().width(260).height(56))
                    .test_id("task-notes"),
                button("追加")
                    .on_press(move || state.add())
                    .test_id("add-task"),
                spacer(),
                text(save_status).role(Role::Status).test_id("board-status"),
                button("保存を再試行")
                    .variant(ButtonVariant::Secondary)
                    .disabled(move || !failed())
                    .on_press(move || saver.run(board.get()))
                    .test_id("retry-save"),
            ))
            .style(style().gap(8).align_items(Align::Start)),
            show_some(state.notice, |message| {
                text(message)
                    .role(Role::Alert)
                    .style(style().color(ColorToken::Danger))
                    .test_id("notice")
            }),
            show_some(state.selected, task_editor),
            row((
                lane(Lane::Planned),
                lane(Lane::InProgress),
                lane(Lane::Done),
            ))
            .style(style().gap(8).align_items(Align::Start))
            .test_id("lanes"),
        ))
        .style(style().gap(10))
    })
}
