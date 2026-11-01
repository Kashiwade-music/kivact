//! Role: One row of the record list.

use kivact::prelude::*;
use kivact::ui::ControlSize;

use super::state::Board;
use crate::model::RecordKey;

/// One record row. The key is fixed for the row's lifetime (keyed list), so
/// the row reads its record once through the shared context.
pub fn record_row(key: RecordKey) -> impl IntoView {
    let board = context::<Board>();
    let label = board.records.with(|all| {
        let r = &all[key as usize];
        format!("{} · {}", r.service, r.summary)
    });
    let is_selected = memo(move || board.selected.get() == Some(key));
    row((
        text(format!("#{key}")).style(
            style()
                .width(72)
                .color(ColorToken::TextMuted)
                .text_wrap(false),
        ),
        text(label).style(style().grow(1).text_wrap(false)),
        button("Open")
            .variant(ButtonVariant::Secondary)
            .size(ControlSize::Small)
            .on_press(move || board.selected.set(Some(key)))
            .test_id(format!("record-open-{key}")),
    ))
    .role(Role::ListItem)
    .name(format!("Record {key}"))
    .selected(is_selected)
    .style(
        style()
            .gap(8)
            .padding_x(8)
            .height(36)
            .align_items(Align::Center)
            .selected(style().background(ColorToken::Selection)),
    )
}
