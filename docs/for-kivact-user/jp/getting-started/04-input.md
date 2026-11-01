# ステップ 4: 入力

[English](../../en/getting-started/04-input.md)

このステップでは、テキスト欄と追加ボタンを加えます。何か入力するまで追加ボタンは無効で、追加した後は欄を空にします。

## ここまでのプログラム

```rust,no_run
use kivact::prelude::*;

fn header() -> impl IntoView {
    column((
        text("タスク"),
        text("今日やること"),
    ))
    .style(style().gap(4))
}

fn task_row(title: &str) -> impl IntoView {
    row((text("-"), text(title))).style(style().gap(8))
}

fn task_list() -> impl IntoView {
    column((
        task_row("牛乳を買う"),
        task_row("チュートリアルを書く"),
        task_row("散歩に行く"),
    ))
    .style(style().gap(8))
}

fn app() -> impl IntoView {
    let completed = signal(0);
    let left = memo(move || 3 - completed.get());
    let draft = signal(String::new());
    let last_added = signal(String::new());

    let add = move || {
        let title = draft.get().trim().to_owned();
        if !title.is_empty() {
            last_added.set(title);
            draft.set(String::new());
        }
    };

    column((
        header(),
        task_list(),
        text(move || format!("残り {} 件", left.get())),
        button("1 件完了")
            .disabled(move || left.get() == 0)
            .on_press(move || completed.update(|n| *n += 1)),
        row((
            text_input("新しいタスク", draft)
                .placeholder("やることを入力")
                .on_submit(add)
                .style(style().width(240)),
            button("追加")
                .disabled(move || draft.with(|text| text.trim().is_empty()))
                .on_press(add),
        ))
        .style(style().gap(8)),
        text(move || {
            let title = last_added.get();
            if title.is_empty() {
                "まだ追加していません".to_owned()
            } else {
                format!("追加しました: {title}")
            }
        }),
    ))
    .style(style().padding(24).gap(16))
}

fn main() -> kivact::Result<()> {
    App::new("タスク").run(app)
}
```

追加したタスクは、まだ一覧には現れません。今のところ、プログラムは最後に追加したタイトルを覚えて、画面の下に表示するだけです。ステップ 5 で、一覧を本物のデータにします。

## 新しく出てきたもの

**`text_input(名前, draft)`** は、1 行のテキスト欄を作ります。第 2 引数は `Signal<String>` で、欄とシグナルは双方向につながります。入力すると `draft` が更新され、`draft.set(..)` を呼ぶと欄の表示が変わります。追加後に欄を空にしているのは、この仕組みです。

第 1 引数は欄の**名前**です。スクリーンリーダーがこの名前を読み上げ、テストでもこの名前で欄を探します。名前は画面には描かれません。そのため、欄が空のときに表示するヒントとして `.placeholder(..)` も付けています。

**`.on_submit(add)`** は、欄の中で Enter を押したときに実行されます。**`.on_press(add)`** は、ボタンを押したときに実行されます。どちらも同じクロージャを使っています。`add` が捕捉するのは `Copy` のシグナルハンドルだけなので、クロージャ自身も `Copy` になり、2 回渡せるからです。

**`.disabled(move || ..)`** は、`text` と同じように、状態を読むクロージャを受け取ります。追加ボタンは、入力が空か空白だけのあいだ無効になります。`draft.with(|text| ..)` は文字列をコピーせずに調べます。一方、`draft.get()` はコピーを返します。

`add` クロージャの中でも空のタイトルを確かめているのは、ボタンが無効でも、欄で Enter を押すとこのクロージャが実行されることがあるからです。

## 実行する

```text
cargo run
```

欄に「植物に水をやる」と入力します。追加ボタンが有効になります。Enter を押すか、追加ボタンをクリックすると、欄が空になり、いちばん下に「追加しました: 植物に水をやる」と表示されます。追加ボタンはまた無効になります。

次へ: [ステップ 5: 一覧と条件分岐](05-lists-and-conditions.md)
