# 2. 設定フォームを作る

[English](../../../en/basic/getting-started/02-build-the-settings-form.md)

このステップでは「設定」画面に 2 枚のカードを置きます。プロフィールのカードには名前の入力欄と言語の select、通知のカードには 2 つのスイッチとリマインダーの選択を置きます。保存ボタンで名前を記憶し、前回の保存から名前が変わっていると、バッジでそのことを知らせます。

キットは標準コントロールのコピーを持ちません。`text_input`、`select`、`switch`、`button` は Kivact 本体のコンポーネントで、ステップ 1 のテーマがキットの見た目を与えます。キットが足すのは、標準セットにないもの（`card`、`badge`、`toggle_group`、`field_group` など）です。

## `src/main.rs` を置き換える

`theme`、`fonts`、`main` はそのままです。新しいのは `profile_card`、`notifications_card`、`settings` です。

```rust,no_run
use kivact::prelude::*;
use kivact::style::theme::Typography;
use kivact::style::{Direction, Theme, ThemeMode};
use kivact_ui_basic as basic;

fn theme(mode: ThemeMode) -> Theme {
    let theme = basic::theme(mode);
    let text = Typography {
        family: Some("Inter, 'Noto Sans JP', sans-serif".into()),
        ..theme.text().clone()
    };
    theme.with_text(text)
}

fn fonts() -> Vec<&'static [u8]> {
    Vec::new()
}

fn profile_card(
    name: Signal<String>,
    saved_name: Signal<String>,
    language: Signal<Option<&'static str>>,
) -> impl IntoView {
    let languages = vec![
        ListItem::new("en", "English"),
        ListItem::new("ja", "日本語"),
        ListItem::new("de", "Deutsch"),
    ];
    let changed = move || name.get() != saved_name.get();
    kivact::ui::card({
        basic::field_group((
            field("表示名", text_input("表示名", name)).hint("公開ページに表示されます。"),
            field("言語", select("言語", languages, language)),
        ))
    })
    .header(
        row((
            column((
                text("プロフィール")
                    .style(style().font_size(14).line_height(20.0 / 14.0).weight(500)),
                text("ほかの人にどう見えるかを決めます。").style(
                    style()
                        .line_height(1.625)
                        .color(basic::ColorName::MutedForeground),
                ),
            ))
            .style(style().gap(4).grow(1).min_width(0)),
            row({
                show(changed, || {
                    kivact::ui::badge("未保存の変更").variant(basic::BadgeVariant::Secondary)
                })
            })
            .style(style().align_self(Align::Start).shrink(0)),
        ))
        .style(style().align_items(Align::Start).gap(8)),
    )
    .style(style().width(420))
}

fn notifications_card(
    digest: Signal<bool>,
    mentions: Signal<bool>,
    reminder: Signal<Option<&'static str>>,
) -> impl IntoView {
    let reminders = vec![
        ListItem::new("daily", "毎日"),
        ListItem::new("weekly", "毎週"),
        ListItem::new("never", "なし"),
    ];
    kivact::ui::card({
        basic::field_group((
            field(
                "週間ダイジェスト",
                switch("週間ダイジェスト", digest).hide_label(true),
            )
            .hint("毎週月曜日に、1 週間のまとめが届きます。")
            .style(
                style()
                    .direction(Direction::Row)
                    .align_items(Align::Center)
                    .gap(12),
            ),
            field(
                "メンション",
                switch("メンション", mentions).hide_label(true),
            )
            .hint("自分宛てのメンションを知らせます。")
            .style(
                style()
                    .direction(Direction::Row)
                    .align_items(Align::Center)
                    .gap(12),
            ),
            basic::field_separator(),
            field(
                "リマインダー",
                kivact::ui::toggle_group("Options", reminders, reminder)
                    .variant(basic::ToggleVariant::Outline),
            ),
        ))
    })
    .header(
        column((
            text("通知").style(style().font_size(14).line_height(20.0 / 14.0).weight(500)),
            text("受け取りたいお知らせを選びます。").style(
                style()
                    .line_height(1.625)
                    .color(basic::ColorName::MutedForeground),
            ),
        ))
        .style(style().gap(4).grow(1).min_width(0)),
    )
    .style(style().width(420))
}

fn settings() -> impl IntoView {
    let name = signal(String::from("Ada"));
    let saved_name = signal(String::from("Ada"));
    let language = signal(Some("ja"));
    let digest = signal(true);
    let mentions = signal(false);
    let reminder = signal(Some("weekly"));
    column((
        basic::h3("設定"),
        basic::muted("プロフィールと通知を管理します。"),
        row((
            profile_card(name, saved_name, language),
            notifications_card(digest, mentions, reminder),
        ))
        .style(style().gap(16).align_items(Align::Start)),
        row((
            button("保存")
                .icon(icon("check"))
                .on_press(move || saved_name.set(name.get())),
            basic::muted(move || format!("保存済みの名前: {}", saved_name.get())),
        ))
        .style(style().gap(12).align_items(Align::Center)),
    ))
    .style(
        style()
            .size(pct(100), pct(100))
            .padding(24)
            .gap(16)
            .background(basic::ColorName::Background)
            .color(basic::ColorName::Foreground),
    )
}

fn main() -> kivact::Result<()> {
    let mut app = App::new("設定").theme(theme(ThemeMode::Light));
    for font in fonts() {
        app = app.font(font);
    }
    app.run(settings)
}
```

実行します。見出しの下に枠付きのカードが 2 枚横に並び、その下に保存ボタンが表示されます。次のことを試してください。

1. 「表示名」に文字を入力します。プロフィールのカードの隅に、灰色のバッジ「未保存の変更」が現れます。
2. 「保存」を押します。バッジが消え、ボタンの横の表示が新しい名前に変わります。
3. Tab キーで、コントロール間のフォーカスを移します。フォーカスされたコントロールには細い輪郭が付きます。スイッチの上で Space キーを押すと切り替わります。
4. 「言語」を開き、一覧から別の項目を選びます。
5. 「毎日」「毎週」「なし」のトグルのどれかを押します。押された状態になるのは常に 1 つだけです。

## 仕組み

- **状態は signal に置きます。** `signal(値)` は `Signal` を作ります。`Signal` は小さなハンドルで、`Copy` です。同じハンドルを、編集するコントロールにも、読み取るクロージャにも渡せます。`move` クロージャは単にそれをコピーします。名前は `Signal<String>` が持ち、`saved_name` は最後に保存した値です。二つを分けて持つので、アプリは違いを判断できます。
- **編集中の値は signal に留まります。** コントロール（`text_input`、`select`、`switch`、`toggle_group`）は signal を受け取り、ユーザーの入力や選択をそこへ書き込みます。`on_press` が `name` を `saved_name` にコピーするまで、保存は起きません。
- **`changed` はクロージャです。** `show(changed, || ..)` は、クロージャが読む signal が変わるたびにそれを呼び直し、`true` の間だけバッジを表示します。リアクティブな値には、ふつうのクロージャで足ります。
- **選択肢は `ListItem` です。** `ListItem::new(key, label)` は、自分で決めた key と表示する文字を組にします。`select` と `toggle_group` は選ばれた key を `Option<K>`（ここでは `Option<&'static str>`）として持ち、`None` は何も選ばれていない状態です。
- **スロットはクロージャを取ります。** `card().header(..)` は組み立て済みのヘッダーを取りますが、`.content(|| ..)` と `.footer(|| ..)` は、カードがマウントするときに一度だけ呼ぶクロージャを取ります。クロージャは使う signal を所有するよう `move || ..` と書きます。signal は `Copy` なので、クローンは要りません。
- **`field(label, control)`** は Kivact の関数で、見えるラベルとコントロール、必要なら `.with_hint(..)` をまとめます。ラベルを押すとコントロールが働き、スクリーンリーダーはラベルをコントロールの名前として読み上げます。中のスイッチに `.hide_label(true)` を付けているのは、ラベルを field がすでに表示しているからです。
- **`basic::field_group`** はキットの余白で field を縦に並べ、`basic::field_separator()` は field の間に区切り線を引きます。

## 次へ

[3. ダークモードとフィードバック](03-dark-mode-and-feedback.md) では、アプリの実行中にテーマを切り替え、保存したときにトーストを出し、破壊的な操作の前に確認を求めます。
