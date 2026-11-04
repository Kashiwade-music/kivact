# ステップ 2: フォームを組み立てる

[English](../../../en/pop/getting-started/02-build-the-form.md)

このステップでは、1 つだけだった入力欄を本物のフォームに育てます。ヒント付きの名前欄とメール欄、プランの切り替え、規約のチェックボックス、キット色のボタン 2 つを追加します。

## プログラム

`src/main.rs` を次の内容に置き換えます。

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_pop as pop;

fn sign_up() -> impl IntoView {
    let name = signal(String::new());
    let email = signal(String::new());
    let plan = signal(Some("team"));
    let terms = signal(false);

    pop::panel((
        text("新規アカウント").style(style().font_size(20).weight(600)),
        field(
            "名前",
            text_input("名前", name).placeholder("お名前を入力"),
        )
        .hint("請求書に表示されます。"),
        field(
            "メールアドレス",
            text_input("メールアドレス", email).placeholder("you@example.com"),
        ),
        field(
            "プラン",
            radio_group(
                "プラン",
                vec![
                    ListItem::new("solo", "ソロ"),
                    ListItem::new("team", "チーム"),
                    ListItem::new("business", "ビジネス"),
                ],
                plan,
            )
            .orientation(Orientation::Horizontal)
            .variant(pop::Segmented),
        )
        .hint("プランはあとから変更できます。"),
        checkbox("利用規約に同意します", terms),
        row((
            button("キャンセル").variant(pop::ButtonTone::Soft),
            button("アカウントを作成").variant(pop::ButtonTone::Accent),
        ))
        .style(style().gap(8).justify_content(Justify::End)),
    ))
    .style(style().width(420).margin(24).gap(16))
}

fn main() -> kivact::Result<()> {
    App::new("New account")
        .theme(pop::theme(ThemeMode::Light))
        .run(sign_up)
}
```

実行します。ラベル付きの欄が 2 つ並び、名前欄の下には控えめな色のヒントが出ます。3 つに分かれた丸い切り替え（チームが選択済み）、チェックボックス、右寄せのボタン 2 つ（灰色のキャンセルと黄色の作成）が見えるはずです。セグメントをクリックするか、切り替えにフォーカスを当てて矢印キーを押すと、プランが変わります。チェックボックスはクリックか Space で切り替えられます。

## 各部分の役割

**`field(label, control)`** は標準コンポーネントです。コントロールの上にラベルを描き、`.hint(..)` で下に控えめな説明を足します。ラベルはコントロールのアクセシブルな名前にもなるので、スクリーンリーダーは「名前」と読み上げます。次のステップでは、同じビルダーに `.error(..)` を足します。

**Rust メモ: `signal(Some("team"))`。** ラジオグループは何も選ばれていない状態を持てるので、プランのシグナルには選択肢のキーを `Option` で包んで持たせます。キーには `Clone + Eq + Hash` を満たす任意の型を使えます。ここでは文字列スライス、あとで自分の `enum` にもできます。

**`radio_group(..).variant(pop::Segmented)`** はラジオグループをセグメントコントロールとして描きます。セグメントコントロールは選択肢を必ず 1 つ選ぶので、タブではなくラジオグループです。`.orientation(Orientation::Horizontal)` で選択肢を横に並べます。`.variant(..)` はキットが標準コンポーネントに見た目を足す方法で、コンポーネント自体は同じまま、描き方だけが変わります。

**`button(..).variant(pop::ButtonTone::Accent)`** はキット固有のボタンの色調を選びます。Pop には `Accent`（黄色）、`Pink`、`Soft`、`Glass`、`Dark` があります。色調を付けなければ、コアのチュートリアルで使った `kivact::ui::ButtonVariant::Primary`、`Secondary`、`Outline`、`Ghost`、`Link` が Pop の色で描かれます。ボタンには色調と形を両方付けられます。`.variant(pop::Slant)` を足すと平行四辺形になります。`button("アカウントを作成").variant(pop::ButtonTone::Pink).variant(pop::Slant)` を試してみてください。

シグナル `name`、`email`、`plan`、`terms` がフォームの状態です。まだどこからも読まれていません。次のステップで使います。

次へ: [ステップ 3: 検証して送信する](03-validate-and-submit.md)
