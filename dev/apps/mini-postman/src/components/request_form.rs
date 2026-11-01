//! Role: Method, URL, headers and body; send, retry, cancel and focus the URL.

use crate::form::{METHODS, Postman};
use crate::http::{Request, validate};
use kivact::prelude::*;

pub(super) fn request_form() -> impl IntoView {
    let p = context::<Postman>();
    // Two-way bindings to single fields of the form (no clone closures).
    let url = p.form.field(|f| &f.url, |f| &mut f.url);
    let headers = p.form.field(|f| &f.headers, |f| &mut f.headers);
    let body = p.form.field(|f| &f.body, |f| &mut f.body);
    let method = p.form.field(|f| &f.method, |f| &mut f.method).optional();
    let url_ref = node_ref();

    let submit = move |request: Request| match validate(&request) {
        Err(error) => p.notice.set(Some(error)),
        Ok(_) => {
            p.notice.set(None);
            p.send.run(request);
        }
    };
    let retry = move || {
        p.notice.set(None);
        let pending = p.send.state().is_pending();
        if !pending && p.send.input().is_some() {
            p.send.retry();
        } else if let Some(last) = p.history.with(|h| h.first().map(|e| e.request.clone())) {
            submit(last);
        } else {
            p.notice.set(Some("再試行する履歴がありません".into()));
        }
    };
    let focus_url = move || {
        url_ref.focus();
        p.notice.set(Some("URL欄へ移動しました".into()));
    };

    column((
        row((
            kivact::ui::select(
                "メソッド",
                METHODS
                    .map(|method| ListItem::new(method.to_owned(), method))
                    .to_vec(),
                method,
            )
            .test_id("method"),
            text_input("URL", url)
                .node_ref(url_ref)
                .style(style().grow(1))
                .test_id("url"),
            button("URLへ移動")
                .variant(ButtonVariant::Secondary)
                .on_press(focus_url)
                .test_id("focus-url"),
        ))
        .style(style().gap(8).align_items(Align::Center)),
        kivact::ui::text_area("ヘッダー（1行に1件: 名前: 値）", headers)
            .placeholder("Accept: application/json")
            .style(style().height(64))
            .test_id("headers"),
        kivact::ui::text_area("リクエスト本文", body)
            .placeholder("POST の本文")
            .style(style().height(64))
            .test_id("request-body"),
        row((
            button("送信")
                .on_press(move || submit(p.form.with(|f| f.to_request())))
                .test_id("send"),
            button("再試行")
                .variant(ButtonVariant::Secondary)
                .on_press(retry)
                .test_id("retry"),
            button("キャンセル")
                .variant(ButtonVariant::Secondary)
                .disabled(move || !p.send.state().is_pending())
                .on_press(move || p.send.cancel())
                .test_id("cancel"),
        ))
        .style(style().gap(8)),
    ))
    .style(style().gap(8))
}
