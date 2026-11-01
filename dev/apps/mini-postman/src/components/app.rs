//! Role: Root: history loading/saving, the HTTP action and the shared context.

use super::history::history_view;
use super::request_form::request_form;
use super::response::response_view;
use crate::form::{FormState, Postman};
use crate::http::{Request, Transport};
use crate::storage::{HISTORY_LIMIT, HistoryEntry, load, save};
use kivact::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;

/// The whole app: history kept in the file `history_path`, requests sent
/// through `transport`.
pub fn postman(history_path: PathBuf, transport: Transport) -> impl IntoView {
    let form = signal(FormState::default());
    let history = signal(Vec::<HistoryEntry>::new());
    let notice = signal(None::<String>);
    let path = var(history_path);

    // Read the saved history on a worker.
    spawn(async move {
        let file = path.get();
        match blocking(move || load(&file)).await {
            Ok(entries) => history.set(entries),
            Err(error) => notice.set(Some(format!("履歴を読めません: {error}"))),
        }
    });
    // Ordered writes; only the newest waiting snapshot is kept.
    let saver = action(move |entries: Vec<HistoryEntry>| {
        let file = path.get();
        async move { Ok(blocking(move || save(&file, &entries)).await?) }
    })
    .policy(ActionPolicy::QueueLatest);
    saver.on_done(move |result| {
        if let Err(error) = result {
            notice.set(Some(format!("履歴を保存できません: {error}")));
        }
    });

    let send = action(move |request: Request| {
        let response = transport(request);
        async move { response.await.map(Arc::new).map_err(UiError::msg) }
    });
    // Completed requests (not superseded or cancelled ones) enter the history.
    send.on_done(move |result| {
        let Some(input) = send.input() else {
            return;
        };
        let outcome = match result {
            Ok(reply) => format!("HTTP {} · {} ms", reply.status, reply.elapsed_ms),
            Err(error) => format!("失敗: {error}"),
        };
        history.update(|entries| {
            let id = entries.iter().map(|entry| entry.id).max().unwrap_or(0) + 1;
            entries.insert(
                0,
                HistoryEntry {
                    id,
                    request: input,
                    result: outcome,
                },
            );
            entries.truncate(HISTORY_LIMIT);
        });
        saver.run(history.get());
    });
    let shared = Postman {
        form,
        history,
        notice,
        send,
    };

    provide(shared, || {
        scroll(
            column((
                text("Mini Postman  ·  Kivact Example").style(style().font_size(20).weight(600)),
                request_form(),
                response_view(),
                history_view(),
            ))
            .style(style().gap(10).padding(12)),
        )
        .style(style().grow(1))
    })
}
