//! Role: Root component of the Task Board: loads the saved board, then mounts it.
//!
//! Root: load the saved board on a worker, show errors with a retry, then
//! mount the board with the loaded value.

use super::board::board;
use crate::storage::load_initial;
use kivact::prelude::*;
use std::path::PathBuf;

/// The whole app. `path` is the JSON file of the board; `generated` starts
/// from that many generated tasks when the file does not exist.
pub fn app(path: PathBuf, generated: Option<usize>) -> impl IntoView {
    // Plain configuration value in a Copy handle (never read reactively).
    let path = var(path);
    let loaded = resource(
        || (),
        move |()| {
            let file = path.get();
            async move {
                blocking(move || load_initial(&file, generated))
                    .await
                    .map_err(UiError::msg)
            }
        },
    );
    scroll(
        column((
            text("Task Board  ·  Kivact Example").style(style().font_size(20).weight(600)),
            text("追加・編集・移動・削除は自動でJSONに保存します"),
            show(
                move || loaded.state().is_pending(),
                || text("読込中").role(Role::Status).test_id("load-status"),
            ),
            show_some(
                move || loaded.error(),
                move |error| {
                    row((
                        text(move || {
                            format!(
                                "読めません: {}。元ファイルを確認して再試行してください",
                                error.get()
                            )
                        })
                        .role(Role::Alert)
                        .test_id("load-error"),
                        button("読込みを再試行")
                            .variant(ButtonVariant::Secondary)
                            .on_press(move || loaded.refetch())
                            .test_id("retry-load"),
                    ))
                    .style(style().gap(8))
                },
            ),
            show_some(
                move || loaded.latest(),
                move |initial| board(initial.get_untracked(), path),
            ),
        ))
        .style(style().gap(8).padding(12)),
    )
    .style(style().grow(1))
}
