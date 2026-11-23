# メッセージと通知を出す

[English](../../../en/pop/how-to-guide/messages-and-notifications.md)

この手引きでは、何が起きたかを人に伝える方法を扱います。対象のそばに出すインラインのアラート、自動で消えるトースト、ベルボタンの奥にある通知の履歴、表示するものがないあいだの読み込み中や空のプレースホルダーです。どれも、アプリの状態で動きます。キットはメッセージを描くだけで、いつ出すか、履歴を保存するか、既読にするかは決めません。

## ページにアラートを出す

`kivact::ui::alert(title)` は、タイトル、任意の説明、任意の操作を持つインラインの枠です。色は標準の `kivact::interaction::Tone` で決まります。

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::Tone;
use kivact_ui_pop as pop;

#[component]
fn payment_status(failed: Signal<bool>) -> impl IntoView {
    show(failed, || {
        {let tone=kivact::state::IntoValue::into_value(Tone::Danger); kivact::ui::alert("Payment failed").tone(tone).icon(kivact::view::icon(move || kivact::style::Icon::named(match tone.get() { kivact::interaction::Tone::Danger=>"alert-circle", kivact::interaction::Tone::Warning=>"alert-triangle",kivact::interaction::Tone::Success=>"circle-check",_=>"info-circle"}))).description("Your card was declined. Update it and try again.").actions(button("Update card").variant(kivact::ui::ButtonVariant::Outline))}
    })
}
# fn main() {}
```

`show(condition, ..)` を使うと、アラートをページに出し、また消せます。アラートは `Role::Alert` を持つので、現れたときにスクリーンリーダーが読み上げます。閉じるボタンはありません。閉じられるようにしたいなら、`.actions(..)` に、シグナルを消すボタンを置いてください。色調は `Neutral`（既定）、`Info`、`Success`、`Warning`、`Danger`、`Accent` です。

## トーストを出す

トーストは、画面の隅に現れて数秒後に消える短いメッセージです。2 つの部品が組になって働きます。キューである `use_toaster()`（トーストを積む先）と、それを描く `toast_region` です。`use_toaster()` は setup で一度作り、リージョンはルートの近くに 1 つ置いて、どこからでもメッセージを push します。

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact::interaction::{Toast, Tone, use_toaster};
use kivact::ui::{toast_region};
use kivact_ui_pop as pop;
use std::time::Duration;

fn app() -> impl IntoView {
    let toaster = use_toaster().max_visible(3);
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

- `Toast::new(title)` が 1 行目です。`.body(text)` は 2 行目を足し、`.tone(Tone::..)` はアイコンと色を決めます。Pop テーマは、色調のアイコンを右下に置いた角丸のカードを描きます。
- `.duration(Some(d))` は表示する時間を決めます（既定は 5 秒）。`.duration(None)` は閉じるまで残します。ポインターかキーボードのフォーカスが載っているあいだは止まります。
- `.action(label, handler)` は、トーストにボタンを足します（たとえば「元に戻す」）。押すとハンドラーが動き、トーストが閉じます。
- `.icon(|| view)` は色調のアイコンを置き換えます。たとえば `.icon(|| icon("gift"))` です。
- `max_visible(n)` は同時に表示する数を制限します。`use_toaster()` を呼ぶ場所で指定します。
- リージョンは 1 つの画面ではなく、アプリの一部です。ルートに 1 つ置き、そこで `use_toaster()` を一度呼び、push するコードには `ToasterApi`（`Copy`）を渡します。

## 通知の履歴を持たせる

`pop::notification_menu(trigger, items).on_mark_read(mark_read)` は、トリガー（通常はベルのボタン）から、通知の一覧をポップアップで出します。未読の項目があるあいだ、トリガーに点が出ます。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_pop as pop;

#[component]
fn bell() -> impl Element {
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

メニューが伝えるのは「すべて既読にする」という操作だけです。コールバックはアプリが書き、一覧の `unread` を変える必要があります。そのあとも、ポップアップは開いたままです。新しい通知を足すには、項目をシグナルに push します。各項目には変わらない `id` を与えてください。そうすると、既読状態が変わっても行が保たれます。文言（"Notifications"、"Mark all as read"、"Unread notifications"）は英語です。キー `pop.notifications.title`、`pop.notifications.mark_read`、`pop.notifications.unread` を `App::messages` で差し替えてください。

## 読み込み中・空であることを示す

データを読み込んでいるあいだは、何も出さずにおくのではなく、プレースホルダーを見せます。データがないあいだは、それを伝え、次の操作を案内します。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_pop as pop;

#[component]
fn orders(loading: Signal<bool>, count: Signal<usize>) -> impl Element {
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

- `kivact::ui::skeleton().lines(n)` は、文字が現れる場所に仮のバーを描きます。
- `kivact::ui::spinner()` と `kivact::ui::spinner().variant(pop::SpinnerVariant::Dots)` は、小さなアニメーションのローダーです。スクリーンリーダー向けに "Loading" という名前が付きます（メッセージ `spinner.loading`）。
- `kivact::ui::empty_state(title).icon(pop::empty_illustration())` は、イラスト、説明、任意の操作を描きます。

読み込みを始めて終える方法（データが届いたときに `loading` を設定するタスク）は中核の話題で、非同期処理の中核のガイドで扱います。

## どれを選ぶか

| 状況 | 使うもの |
| --- | --- |
| 利用者が読むまで先に進めない | 原因のそばの `alert` |
| たった今起きたことを確認させたい | トースト |
| 利用者が離れていたあいだに起きたこと | `notification_menu` |
| まだ表示するものがない | `skeleton`、`spinner`、`empty_state` |
| 利用者がいま決める必要がある | 標準の `kivact::ui::dialog` |

## 関連

- [状態と数値を表示する](status-and-figures.md)
- [日付と期間を選ぶ](dates-and-calendars.md)。カレンダーの文言のキーは、こちらを参照してください。
- トーストの API は、`cargo doc -p kivact --open` で `kivact::interaction::Toast`、`kivact::interaction::ToasterApi`、`kivact::ui::toast_region` を見てください。Pop のコンポーネントの全体は `cargo doc -p kivact-ui-pop --open` です。

メッセージのログには、標準の `chat_thread`、`chat_message(author)`、`chat_composer`、`typing_indicator` を使います。`ChatBubbleVariant` は、コンパクトな会話の吹き出しを選びます。`.sender(..)` は著者とは独立した送り手、`.content(..)` はビューを受け取り、`.time(..)` はアプリが整形した時刻を保ちます。段落のまとまりは、個々のメッセージを `ChatBubbleGroup::First`、`Middle`、`Last` で描いて作ります。最初にアバター、最後に時刻を置きます。共有のログが末尾への追従を持ち、メッセージの履歴、ストリーミング、revision はアプリが持ちます。`pop::thread` は、選択、未読数、プレビューを持つ会話の一覧の行のままです。標準の `code_block` は、行番号、ハイライト、折り返し、コピーの表示を提供します。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_pop as pop;
#[component]
fn conversation() -> impl Element {
    let draft=signal(String::new());
    let sent=signal(Vec::<String>::new());
    kivact::ui::chat_thread((
        kivact::ui::chat_message("Haruka").content(text("The report is ready."))
            .time("09:12").variant(pop::ChatBubbleVariant),
        for_index(sent,|_,message|kivact::ui::chat_message("You")
            .sender(kivact::ui::ChatSender::User)
            .content(text(message)).variant(pop::ChatBubbleVariant)),
    )).composer(kivact::ui::chat_composer(draft)
        .on_submit(move|message|sent.update(|sent|sent.push(message))))
}
#[component]
fn source() -> impl Element {
    kivact::ui::code_block("let answer = 42;".to_owned())
        .title("Example").language("Rust").line_numbers(true).wrap(true)
}
```
