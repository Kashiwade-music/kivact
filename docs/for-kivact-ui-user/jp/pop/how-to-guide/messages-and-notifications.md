# メッセージと通知を出す

[English](../../../en/pop/how-to-guide/messages-and-notifications.md)

何が起きたかを伝える手段をまとめた手引きです。対象のそばに出すインライン・アラート、自動で消えるトースト、ベルボタンの奥にある通知履歴、表示するものがないあいだの読み込み中・空のプレースホルダーを扱います。

どれもアプリの状態で動きます。キットはメッセージを描くだけで、いつ出すか、履歴を保存するか、既読にするかを決めることはありません。

## ページにアラートを出す

`kivact::ui::alert(title)` は、タイトル、任意の説明、任意の操作を持つインラインの枠です。色は標準の `kivact::interaction::Tone` で決まります。

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::Tone;
use kivact_ui_pop as pop;

fn payment_status(failed: Signal<bool>) -> impl IntoView {
    show(failed, || {
        {let tone=kivact::state::IntoValue::into_value(Tone::Danger); kivact::ui::alert("Payment failed").tone(tone).icon(kivact::view::icon(move || kivact::style::Icon::named(match tone.get() { kivact::interaction::Tone::Danger=>"alert-circle", kivact::interaction::Tone::Warning=>"alert-triangle",kivact::interaction::Tone::Success=>"circle-check",_=>"info-circle"}))).description("Your card was declined. Update it and try again.").actions(button("Update card").variant(kivact::ui::ButtonVariant::Outline))}
    })
}
# fn main() {}
```

アラートを出したり消したりするには `show(condition, ..)` を使います。アラートは `Role::Alert` を持つので、現れるとスクリーンリーダーが読み上げます。閉じるボタンはありません。閉じられるようにしたいときは、`.actions(..)` にシグナルを消すボタンを置きます。使える色調は `Neutral`（既定）、`Info`、`Success`、`Warning`、`Danger`、`Accent` です。

## トーストを出す

トーストは、画面の隅に現れて数秒で消える短いメッセージです。2 つの部品が組になって動きます。キューにあたる `toaster` と、それを描く `toast_region` です。`toaster` は setup で一度だけ作り、`toast_region` はルートの近くに 1 つ置き、メッセージはどこからでも push します。

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact::interaction::{Toast, Tone, toaster};
use kivact::ui::{toast_region};
use kivact_ui_pop as pop;
use std::time::Duration;

fn app() -> impl IntoView {
    let toaster = toaster().max_visible(3);
    column((
        button("Save").on_press(move || {
            toaster.push(
                Toast::new("Saved")
                    .tone(Tone::Success)
                    .body("Your draft is stored.")
                    .duration(Some(Duration::from_secs(4))),
            );
        }),
        toast_region(toaster),
    ))
    .style(style().gap(12).padding(24))
}

fn main() -> kivact::Result<()> {
    App::new("Toasts")
        .theme(pop::theme(ThemeMode::Light))
        .run(app)
}
```

- `Toast::new(title)` が 1 行目で、`.body(text)` が 2 行目、`.tone(Tone::..)` がアイコンと色を決めます。Pop テーマは、色調のアイコン付きの角丸のカードを右下に描きます。
- `.duration(Some(d))` は表示時間を決めます（既定は 5 秒）。`.duration(None)` なら閉じるまで残ります。ポインターやキーボードフォーカスが載っているあいだは、時間が止まります。
- `.action(label, handler)` はトーストにボタン（たとえば「元に戻す」）を足します。押すとハンドラーが実行され、トーストが閉じます。
- `.icon(|| view)` は色調のアイコンを置き換えます。たとえば `.icon(|| icon("gift"))` です。
- `max_visible(n)` は同時に表示する数を制限します。toaster を作った場所で呼びます。
- リージョンは画面ごとではなく、アプリの一部です。ルートに 1 つ置き、そこで `toaster()` を一度だけ呼び、push する側のコードには `Toaster`（`Copy`）を渡します。

## 通知の履歴を持たせる

`pop::notification_menu(trigger, items).on_mark_read(mark_read)` は、トリガー（通常はベルのボタン）から通知の一覧をポップアップで出します。未読の項目があるあいだ、トリガーに点が出ます。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_pop as pop;

fn bell() -> impl IntoView {
    let items = signal(vec![
        pop::NotificationItem {
            unread: true,
            hue: pop::Hue::Mint,
            ..pop::NotificationItem::new(1, "Order paid", "Order 1042 was paid.", "2 min ago")
        },
        pop::NotificationItem::new(2, "Report ready", "The weekly report is ready.", "Yesterday"),
    ]);
    pop::notification_menu(
        button("Notifications")
            .icon(icon("bell"))
            .hide_label(true)
            .variant(kivact::ui::ButtonVariant::Ghost),
        items,
    ).on_mark_read(move || items.update(|list| list.iter_mut().for_each(|item| item.unread = false)))
}
# fn main() {}
```

メニューが伝えるのは「すべて既読にする」という要求だけです。コールバックはアプリのもので、一覧の `unread` を書き換える必要があります。そのあともポップアップは開いたままです。新しい通知を追加するには、シグナルに項目を push します。各項目の `id` は変わらないものにしてください。既読状態が変わっても行が保たれます。文言（"Notifications"、"Mark all as read"、"Unread notifications"）は英語です。`pop.notifications.title`、`pop.notifications.mark_read`、`pop.notifications.unread` のキーを `App::messages` で差し替えます。

## 読み込み中・空であることを示す

データを読み込んでいるあいだは、何も出さずにおくのではなくプレースホルダーを見せます。データがないときは、それを伝えて次の操作を案内します。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_pop as pop;

fn orders(loading: Signal<bool>, count: Signal<usize>) -> impl IntoView {
    column((
        show(loading, || kivact::ui::skeleton().lines(3)),
        show(move || !loading.get() && count.get() == 0, || {
            kivact::ui::empty_state("No orders yet").icon(pop::empty_illustration())
                .description("Orders appear here as soon as customers check out.")
                .actions(button("Create an order").variant(kivact::ui::ButtonVariant::Primary))
        }),
        show(loading, || row((kivact::ui::spinner(), text("Loading orders")))
            .style(style().gap(8))),
    ))
    .style(style().gap(16).padding(24))
}
# fn main() {}
```

- `kivact::ui::skeleton().lines(n)` は、文字が入る場所に仮のバーを描きます。
- `kivact::ui::spinner()` と `kivact::ui::spinner().variant(pop::SpinnerLook::Dots)` は小さなアニメーション付きのローダーで、スクリーンリーダー向けに "Loading" という名前が付きます（メッセージ `spinner.loading`）。
- `kivact::ui::empty_state(title).icon(pop::empty_illustration())` は、イラスト、説明、任意の操作を描きます。

読み込みの開始と完了（データが届いたら `loading` を更新するタスク）はコアの話題で、非同期処理のコアのガイドで扱います。

## どれを選ぶか

| 状況 | 使うもの |
| --- | --- |
| 読まないと先に進めない | 原因のそばに `alert` |
| たった今起きたことを知らせたい | トースト |
| 席を外していたあいだの出来事 | `notification_menu` |
| まだ見せるものがない | `skeleton`、`spinner`、`empty_state` |
| ユーザーが今すぐ決める必要がある | 標準の `kivact::ui::dialog` |

## 関連

- [業務画面で数値・状態・レコードを見せる](status-and-figures.md)
- [日付と期間を選ばせる](dates-and-calendars.md): カレンダーの文言のキーについて。
- トーストの API は `cargo doc -p kivact --open` の `kivact::ui::Toast`、`Toaster`、`toast_region`、Pop 全体のコンポーネントは `cargo doc -p kivact-ui-pop --open` で確認できます。

メッセージのログには標準の `chat_thread`、`chat_message(author)`、`chat_composer`、`typing_indicator` を使います。`PopBubble` はコンパクトな吹き出しです。`.sender(..)` は著者名とは独立した参加者の役割、`.content(..)` は任意のビュー、`.time(..)` はアプリが整形した時刻です。段落のグループは `ChatBubbleGroup::First`、`Middle`、`Last` を持つメッセージを並べ、最初にアバター、最後に時刻を置きます。共有のログが末尾への追従を持ち、履歴、ストリーミング、revision はアプリが持ちます。`pop::thread` は選択、未読数、プレビューを持つ会話の一覧行として残ります。標準の `code_block` は行番号、ハイライト、折り返し、コピーのフィードバックを提供します。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_pop as pop;
fn conversation() -> impl IntoView {
    let draft=signal(String::new());
    let sent=signal(Vec::<String>::new());
    kivact::ui::chat_thread((
        kivact::ui::chat_message("Haruka").content(text("The report is ready."))
            .time("09:12").variant(pop::PopBubble),
        for_index(sent,|_,message|kivact::ui::chat_message("You")
            .sender(kivact::ui::ChatSender::User)
            .content(text(message)).variant(pop::PopBubble)),
    )).composer(kivact::ui::chat_composer(draft)
        .on_submit(move|message|sent.update(|sent|sent.push(message))))
}
fn source() -> impl IntoView {
    kivact::ui::code_block("let answer = 42;".to_owned())
        .title("Example").language("Rust").line_numbers(true).wrap(true)
}
```
