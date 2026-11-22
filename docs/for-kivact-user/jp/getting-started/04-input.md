# ステップ 4: 入力

[English](../../en/getting-started/04-input.md)

このステップでは、テキスト欄と「Add」ボタンを加えます。何か入力するまでボタンは無効のままで、追加した後は欄を空にします。

## ここまでのプログラム

```rust,no_run
use kivact::prelude::*;

#[component]
fn header() -> impl Element {
    column((
        text("Tasks"),
        text("Things to do today"),
    ))
    .style(style().gap(4))
}

#[component]
fn task_row(title: Value<String>) -> impl Element {
    row((text("-"), text(title))).style(style().gap(8))
}

#[component]
fn task_list() -> impl Element {
    column((
        task_row("Buy milk"),
        task_row("Write the tutorial"),
        task_row("Go for a walk"),
    ))
    .style(style().gap(8))
}

fn app() -> impl IntoView {
    let completed = signal(0);
    let left = memo(move || 3 - completed.get());
    let draft = signal(String::new());
    let last_added = signal(String::new());

    let add = callback(move |_| {
        let title = draft.get().trim().to_owned();
        if !title.is_empty() {
            last_added.set(title);
            draft.set(String::new());
        }
    });

    column((
        header(),
        task_list(),
        text(move || format!("{} tasks left", left.get())),
        button("Complete one")
            .disabled(move || left.get() == 0)
            .on_press(move || completed.update(|n| *n += 1)),
        row((
            text_input(draft)
                .label("New task")
                .placeholder("What needs doing?")
                .on_submit(add)
                .style(style().width(240)),
            button("Add")
                .disabled(move || draft.with(|text| text.trim().is_empty()))
                .on_press(add),
        ))
        .style(style().gap(8)),
        text(move || {
            let title = last_added.get();
            if title.is_empty() {
                "Nothing added yet".to_owned()
            } else {
                format!("Added: {title}")
            }
        }),
    ))
    .style(style().padding(24).gap(16))
}

fn main() -> kivact::Result<()> {
    App::new("Tasks").run(app)
}
```

新しいタスクはまだ一覧に表示されません。今のところプログラムは、最後に追加されたタイトルを覚えて、下に表示するだけです。ステップ 5 で、一覧を本物のデータにします。

## 新しいこと

**`text_input(draft)`** は、1 行のテキスト欄を作ります。引数は `Signal<String>` です。欄とシグナルは双方向に結ばれています。入力するとそのシグナル `draft` が更新され、`draft.set(..)` を呼ぶと欄の表示が変わります。追加した後に欄を空にするのは、この仕組みを使います。大きな構造体の中のフィールドを扱うときは、`bind!(profile.name)` を渡すと、構造体のうちその 1 つのフィールドだけを編集できます。ステップ 5 では、タスクのチェックボックスで同じ書き方を使います。

**`.label("New task")`** は、欄に**名前**を与えます。スクリーンリーダーはこの名前を読み上げ、テストはこの名前で欄を探します。名前は画面には描かれません。そのため、欄には `.placeholder(..)` も付けます。これは、欄が空のときに表示されるヒントです。欄の横に描かれるラベルがほしいときは、代わりに `field("New task", text_input(draft))` と書きます。`field` はラベルを表示し、同じ文字列を欄の名前として与えます。フォーカスを受け取れるコントロールには、すべて名前が必要です。名前がないと、Kivact はそのコントロールが現れたときに報告します。デバッグビルドやテストでは、この報告はエラーになります。

**`.on_submit(add)`** は、欄の中で Enter キーを押したときに実行されます。**`.on_press(add)`** は、ボタンを押したときに実行されます。どちらも同じ処理を使います。`callback(move |_| ..)` は、クロージャを `Callback` に変えます。`Callback` は小さな `Copy` のハンドルで、いくつのハンドラーでも共有できます。`_` は、ハンドラーが受け取るイベントです。Enter もボタンの押下も、イベントの値は持ちません。

**`.disabled(move || ..)`** は、`text` と同じように、状態を読むクロージャを受け取ります。追加ボタンは、下書きが空か空白だけのあいだ無効になります。ここでは `draft.with(|text| ..)` で文字列をコピーせずに見ています。`draft.get()` は現在の文字列を複製します。

`add` のクロージャは、空のタイトルも確かめます。ボタンが無効のときでも、欄で Enter を押すとこのクロージャが実行されるかもしれないからです。

## 実行する

```text
cargo run
```

欄に「Water the plants」と入力します。追加ボタンが有効になります。Enter を押すか Add をクリックすると、欄が空になり、一番下の行に「Added: Water the plants」と表示されます。追加ボタンは再び無効になります。

次へ: [ステップ 5: 一覧と条件分岐](05-lists-and-conditions.md)
