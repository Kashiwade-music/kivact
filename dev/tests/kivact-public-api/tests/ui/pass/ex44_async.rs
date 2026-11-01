#![allow(dead_code, unused_variables)]
// Usage example: async data and actions
use kivact::prelude::*;
use kivact::view::PointerEvent;
use kivact::style::Color;
use kivact::draw::Draw;
use kivact::draw::Rect;
use kivact::view::Handled;
use kivact::Callback;
use kivact::view::PointerId;
use kivact::draw::Size;
include!("../support/prelude_stubs.rs");

fn search() -> impl IntoView {
    let query = signal(String::new());
    // source（query）が変わるたびに fetch し直す。受理する結果は最新の要求だけ
    let results = resource(move || query.get(), |q: String| async move {
        if q.is_empty() {
            return Ok(Vec::new());
        }
        Ok(blocking(move || search_index(&q)).await?) // Send が必要なのはここだけ
    });
    column((
        kivact::control::node().name("検索").part(kivact::interaction::use_text_edit(query).control()),
        show(move || results.state().is_pending(), || text("検索中…")),
        show_some(
            move || match results.state() {
                AsyncState::Failed(e) => Some(e),
                _ => None,
            },
            |e| text(move || e.get().to_string()).role(Role::Alert),
        ),
        for_each(move || results.latest().unwrap_or_default(), |r| r.id, |_, r| text(move || r.get().title)),
    ))
}

fn request_form(form: Signal<Form>) -> impl IntoView {
    let send = action(|req: Request| async move { Ok(blocking(move || http::send(req)).await?) }); // 最新の要求を優先
    let status = memo(move || match send.state() {
        AsyncState::Idle => "送信待ち".to_string(),
        AsyncState::Pending => "送信中".into(),
        AsyncState::Ready(r) => format!("HTTP {}", r.status),
        AsyncState::Failed(e) if e.is_cancelled() => "キャンセルしました".into(),
        AsyncState::Failed(e) => format!("失敗: {e}"),
    });
    row((
        { let label = ("送信").into_value(); node().role(Role::Button).focusable(true).name(label).child(text(label)) }.on_press(move || send.run(form.get().into_request())),
        { let label = ("再試行").into_value(); node().role(Role::Button).focusable(true).name(label).child(text(label)) }.disabled(move || send.input().is_none() || send.state().is_pending()).on_press(move || send.retry()),
        { let label = ("キャンセル").into_value(); node().role(Role::Button).focusable(true).name(label).child(text(label)) }.disabled(move || !send.state().is_pending()).on_press(move || send.cancel()),
        text(status),
    ))
}

fn main() {
    let _ = search;
    let _ = request_form;
}
