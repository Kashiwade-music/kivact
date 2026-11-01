//! Role: Detail pane of the selected record.

use kivact::prelude::*;

use super::state::Board;
use crate::model::RecordKey;

/// Detail of the selected record; stays mounted while the selection moves
/// (`show_some` keeps the branch on Some → Some and updates `key`).
pub fn detail(key: Value<RecordKey>) -> impl IntoView {
    let board = context::<Board>();
    let record = memo(move || {
        let k = key.get() as usize;
        board.records.with(|all| all.get(k).cloned())
    });
    column((
        text(move || {
            record.with(|r| {
                r.as_ref()
                    .map(|r| format!("{} / {}", r.service, r.summary))
                    .unwrap_or_default()
            })
        })
        .role(Role::Heading)
        .test_id("detail-title"),
        text(move || record.with(|r| r.as_ref().map(|r| r.details.clone()).unwrap_or_default()))
            .test_id("detail-body"),
    ))
    .style(
        style()
            .padding(8)
            .gap(4)
            .background(ColorToken::SurfaceAlt)
            .radius(4),
    )
}
