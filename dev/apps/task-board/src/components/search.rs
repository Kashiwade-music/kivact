//! Role: Search field that filters every lane by title or notes.

use crate::context::BoardCx;
use kivact::prelude::*;

/// Filter every lane by title or notes.
pub(super) fn search() -> impl IntoView {
    let board = context::<BoardCx>();
    text_input("検索", board.search)
        .placeholder("検索（題名・メモ）")
        .test_id("task-search")
        .style(style().min_width(240))
}
