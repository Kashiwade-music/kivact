//! Role: Status line, the response (headers and body) and copying it.

use kivact::platform;

use crate::form::Postman;
use kivact::prelude::*;

/// Longest response text rendered on screen, in characters (copy uses all).
const SHOWN_CHARS: usize = 64 * 1024;

pub(super) fn response_view() -> impl IntoView {
    let p = context::<Postman>();
    let status = memo(move || match p.send.state() {
        AsyncState::Idle => "送信待ち".to_string(),
        AsyncState::Pending => "送信中".to_string(),
        AsyncState::Ready(r) => format!("HTTP {} · {} ms", r.status, r.elapsed_ms),
        AsyncState::Failed(e) if e.is_cancelled() => "キャンセルしました".to_string(),
        AsyncState::Failed(e) => format!("失敗: {e}"),
    });
    let response = memo(move || match p.send.state() {
        AsyncState::Ready(r) => format!("{}\n\n{}", r.headers, r.body),
        AsyncState::Failed(e) if !e.is_cancelled() => e.to_string(),
        _ => String::new(),
    });
    let copy = move || {
        let copied = platform::copy_text(response.get());
        spawn(async move {
            p.notice.set(Some(match copied.await {
                Ok(()) => "応答をコピーしました".into(),
                Err(error) => format!("コピーできません: {error}"),
            }));
        });
    };
    column((
        text(move || p.notice.get().unwrap_or_else(|| status.get()))
            .role(Role::Status)
            .style(style().min_height(24).weight(600))
            .test_id("postman-status"),
        scroll(
            text(move || {
                response.with(|r| match r.char_indices().nth(SHOWN_CHARS) {
                    Some((cut, _)) => format!("{}\n…（表示は先頭のみ）", &r[..cut]),
                    None => r.clone(),
                })
            })
            .test_id("response"),
        )
        .name("レスポンス（ヘッダーと本文）")
        .style(
            style()
                .height(180)
                .padding(8)
                .radius(6)
                .border(1, ColorToken::Border)
                .background(ColorToken::Surface),
        ),
        row(button("応答をコピー")
            .variant(ButtonVariant::Secondary)
            .disabled(move || response.with(String::is_empty))
            .on_press(copy)
            .test_id("copy-response")),
    ))
    .style(style().gap(8))
}
