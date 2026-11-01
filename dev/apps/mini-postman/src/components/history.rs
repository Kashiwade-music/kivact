//! Role: Sent requests; choosing one restores the whole form.

use crate::form::{FormState, Postman};
use crate::storage::HistoryEntry;
use kivact::prelude::*;

fn label(entry: &HistoryEntry) -> String {
    format!(
        "{} {} — {}",
        entry.request.method, entry.request.url, entry.result
    )
}

pub(super) fn history_view() -> impl IntoView {
    let p = context::<Postman>();
    column((
        text("送信履歴（選ぶとフォームへ戻します）"),
        scroll(
            column(for_each(
                p.history,
                |entry| entry.id,
                move |id, entry| {
                    button(move || entry.with(label))
                        .variant(ButtonVariant::Secondary)
                        .on_press(move || {
                            entry.with(|e| p.form.set(FormState::from_request(&e.request)))
                        })
                        .style(style().justify_content(Justify::Start))
                        .test_id(format!("history-{id}"))
                },
            ))
            .style(style().gap(4)),
        )
        .role(Role::List)
        .name("送信履歴")
        .style(style().height(200))
        .test_id("history"),
    ))
    .style(style().gap(6))
}
