//! Role: One lane: a filtered, virtualized list of task cards.

use super::task_card::{CARD_HEIGHT, task_card};
use crate::context::BoardCx;
use crate::model::Lane;
use kivact::prelude::*;
use kivact::view::RowHeight;

pub(super) fn lane(lane: Lane) -> impl IntoView {
    let state = context::<BoardCx>();
    // Shared `Arc<Task>` rows; equal lists stop here (memo cutoff).
    let tasks = memo(move || {
        state
            .search
            .with(|query| state.board.with(|board| board.matching(lane, query)))
    });
    column((
        text(move || format!("{} ({}件)", lane.label(), tasks.with(Vec::len)))
            .style(style().weight(600))
            .test_id(format!("lane-{}", lane.slug())),
        virtual_list(tasks, |task| task.id, task_card)
            .row_height(RowHeight::Fixed(CARD_HEIGHT + 8.0))
            .style(style().height(400))
            .name(lane.label())
            .test_id(format!("list-{}", lane.slug())),
    ))
    .style(
        style()
            .grow(1)
            .basis(0)
            .min_width(240)
            .gap(6)
            .padding(8)
            .radius(8)
            .background(ColorToken::SurfaceAlt),
    )
}
