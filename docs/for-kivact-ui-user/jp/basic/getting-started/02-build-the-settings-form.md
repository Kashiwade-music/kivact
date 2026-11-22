# 2. 設定フォームを作る

[English](../../../en/basic/getting-started/02-build-the-settings-form.md)

このステップでは「設定」画面に 2 つのカードを置きます。プロフィールのカードには名前の入力欄と言語の select を、通知のカードには 2 つのスイッチとリマインダーの選択肢を置きます。保存ボタンは名前を記憶し、名前が前回の保存から変わっているときは、バッジでそのことを知らせます。

キットは標準のコントロールのコピーを持ちません。`text_input`、`select`、`switch`、`button` は Kivact 本体から来ており、ステップ 1 のテーマがキットの見た目を与えます。キットは、標準セットにないもの（`card`、`badge`、`toggle_group`、`field_group` など）を足します。

## `src/main.rs` を置き換える

`theme`、`fonts`、`main` は変わりません。新しいのは `profile_card`、`notifications_card`、`settings` です。

```rust,no_run
use kivact::prelude::*;
use kivact::style::{FlexDirection, FontFamily, Theme, ThemeMode, font};
use kivact_ui_basic as basic;

fn theme(mode: ThemeMode) -> Theme {
    basic::theme(mode).set(&font::FAMILY_VAR, FontFamily::from("Inter, sans-serif"))
}

fn fonts() -> Vec<&'static [u8]> {
    Vec::new()
}

#[component]
fn profile_card(
    name: Signal<String>,
    saved_name: Signal<String>,
    language: Signal<Option<&'static str>>,
) -> impl Element {
    let languages = vec![
        ListItem::new("en", "English"),
        ListItem::new("ja", "Japanese"),
        ListItem::new("de", "German"),
    ];
    let changed = move || name.get() != saved_name.get();
    kivact::ui::card({
        basic::field_group((
            field("Display name", text_input(name))
                .hint("Shown on your public page."),
            field("Language", select(languages, language)),
        ))
    })
    .header(
        row((
            column((
                text("Profile").style(style().font_size(14).line_height(20.0 / 14.0).font_weight(500)),
                text("Choose how others see you.").style(
                    style()
                        .line_height(1.625)
                        .color(basic::MUTED_FOREGROUND),
                ),
            ))
            .style(style().gap(4).flex_grow(1).min_width(0)),
            row({
                show(changed, || {
                    kivact::ui::badge("Unsaved changes").variant(basic::BadgeVariant::Secondary)
                })
            })
            .style(style().align_self(Align::Start).flex_shrink(0)),
        ))
        .style(style().align_items(Align::Start).gap(8)),
    )
    .style(style().width(420))
}

#[component]
fn notifications_card(
    digest: Signal<bool>,
    mentions: Signal<bool>,
    reminder: Signal<Option<&'static str>>,
) -> impl Element {
    let reminders = vec![
        ListItem::new("daily", "Daily"),
        ListItem::new("weekly", "Weekly"),
        ListItem::new("never", "Never"),
    ];
    kivact::ui::card({
        basic::field_group((
            field(
                "Weekly digest",
                switch("Weekly digest", digest).hide_label(true),
            )
            .hint("A summary of your week, every Monday.")
            .style(
                style()
                    .flex_direction(FlexDirection::Row)
                    .align_items(Align::Center)
                    .gap(12),
            ),
            field("Mentions", switch("Mentions", mentions).hide_label(true))
                .hint("Tell me when someone mentions me.")
                .style(
                    style()
                        .flex_direction(FlexDirection::Row)
                        .align_items(Align::Center)
                        .gap(12),
                ),
            basic::field_separator(),
            field(
                "Reminders",
                kivact::ui::toggle_group("Options", reminders, reminder)
                    .variant(basic::ToggleVariant::Outline),
            ),
        ))
    })
    .header(
        column((
            text("Notifications").style(style().font_size(14).line_height(20.0 / 14.0).font_weight(500)),
            text("Choose what you want to hear about.").style(
                style()
                    .line_height(1.625)
                    .color(basic::MUTED_FOREGROUND),
            ),
        ))
        .style(style().gap(4).flex_grow(1).min_width(0)),
    )
    .style(style().width(420))
}

#[component]
fn settings() -> impl Element {
    let name = signal(String::from("Ada"));
    let saved_name = signal(String::from("Ada"));
    let language = signal(Some("en"));
    let digest = signal(true);
    let mentions = signal(false);
    let reminder = signal(Some("weekly"));
    column((
        kivact::ui::heading("Settings", 3),
        basic::muted("Manage your profile and notifications."),
        row((
            profile_card(name, saved_name, language),
            notifications_card(digest, mentions, reminder),
        ))
        .style(style().gap(16).align_items(Align::Start)),
        row((
            button("Save")
                .icon(icon("check"))
                .on_press(move || saved_name.set(name.get())),
            basic::muted(move || format!("Saved name: {}", saved_name.get())),
        ))
        .style(style().gap(12).align_items(Align::Center)),
    ))
    .style(
        style()
            .size(pct(100), pct(100))
            .padding(24)
            .gap(16)
            .background(basic::BACKGROUND)
            .color(basic::FOREGROUND),
    )
}

fn main() -> kivact::Result<()> {
    App::new("Settings")
        .theme(theme(ThemeMode::Light).with_fonts(fonts()))
        .run(settings)
}
```

実行します。見出しの下に、枠付きのカードが横に 2 枚並び、その下に Save のボタンが見えるはずです。次を試してください。

1. 「Display name」に文字を入力します。プロフィールのカードの隅に、secondary のバッジ「Unsaved changes」が現れます。
2. Save を押します。バッジが消え、ボタンの横の行が新しい名前を表示します。
3. Tab キーでコントロールからコントロールへフォーカスを移します。フォーカスされたコントロールには細い輪郭が付きます。スイッチの上で Space キーを押すと切り替わります。
4. 「Language」を開き、一覧から別の項目を選びます。
5. Daily、Weekly、Never のトグルのどれかを押します。押された状態のままになるのは 1 つだけです。

## 仕組み

- **状態は signal に置きます。** `signal(value)` は `Signal` を作ります。`Signal` は小さなハンドルで、`Copy` です。同じハンドルを、編集するコントロールにも、読むクロージャにも渡せます。`move` のクロージャは、それをコピーするだけです。`Signal<String>` が名前を持ち、`saved_name` は最後に保存した値です。アプリが両者の違いを判断できるように、別々に持ちます。
- **編集中の値は signal に留まります。** コントロール（`text_input`、`select`、`switch`、`toggle_group`）は signal を受け取り、利用者が入力・選択するたびにそこへ書き込みます。`on_press` が `name` を `saved_name` にコピーするまで、保存は起きません。
- **`changed` はクロージャです。** `show(changed, || ..)` は、読んでいる signal が変わるたびに `changed` を呼び直し、それが `true` のあいだだけバッジを配置します。リアクティブな値には、普通のクロージャで足ります。
- **選択肢は `ListItem` です。** `ListItem::new(key, label)` は、自分で決めたキーと表示する文字を組にします。`select` と `toggle_group` は、選ばれたキーを `Option<K>`（ここでは `Option<&'static str>`）で持ちます。`None` は、何も選ばれていない状態です。
- **表示する内容は、そのまま渡します。** 標準のカードの `.header(..)`、`.content(..)`、`.footer(..)` はビューを受け取ります。開くたびに内容を作り直すスロットはクロージャを受け取り、各ビルダーがその違いを説明します。
- **Kivact の `field(label, control)`** は、見えるラベルとコントロール、任意の `.hint(..)` を組にします。ラベルを押すとコントロールが働き、スクリーンリーダーはラベルをコントロールの名前として読み上げます。中のスイッチに `.hide_label(true)` を付けているのは、ラベルを field がすでに表示しているからです。
- **`basic::field_group`** は、キットの間隔で欄を積み重ね、`basic::field_separator()` は欄のあいだに罫線を引きます。

## 次へ

[3. ダークモードとフィードバック](03-dark-mode-and-feedback.md) では、アプリの実行中にテーマを切り替え、保存したときにトーストを出し、破壊的な操作の前に確認を求めます。
