# ステップ 5: 一覧と条件分岐

[English](../../en/getting-started/05-lists-and-conditions.md)

このステップでは、仮の一覧を本物にします。タスクをシグナルに持たせ、各行にチェックボックスと削除ボタンを付け、一覧が空のときはヒントを表示します。

## ここまでのプログラム

```rust,no_run
use kivact::prelude::*;

#[derive(Clone, PartialEq)]
struct Task {
    id: u32,
    title: String,
    done: bool,
}

fn header() -> impl IntoView {
    column((
        text("タスク"),
        text("今日やること"),
    ))
    .style(style().gap(4))
}

fn task_row(tasks: Signal<Vec<Task>>, task: Binding<Task>) -> impl IntoView {
    let id = task.get_untracked().id;
    let done = bind!(task.done);
    row((
        checkbox(move || task.get().title, done),
        button("削除").on_press(move || tasks.update(|list| list.retain(|task| task.id != id))),
    ))
    .style(style().gap(8))
}

fn task_list(tasks: Signal<Vec<Task>>) -> impl IntoView {
    column(for_each(
        tasks,
        |task| task.id,
        move |task| task_row(tasks, task),
    ))
    .style(style().gap(8))
}

fn app() -> impl IntoView {
    let tasks = signal(vec![
        Task { id: 1, title: "牛乳を買う".to_owned(), done: false },
        Task { id: 2, title: "チュートリアルを書く".to_owned(), done: false },
        Task { id: 3, title: "散歩に行く".to_owned(), done: false },
    ]);
    let next_id = var(4);
    let draft = signal(String::new());
    let left = memo(move || tasks.with(|list| list.iter().filter(|task| !task.done).count()));

    let add = move || {
        let title = draft.get().trim().to_owned();
        if !title.is_empty() {
            let id = next_id.get();
            next_id.set(id + 1);
            tasks.update(|list| list.push(Task { id, title, done: false }));
            draft.set(String::new());
        }
    };

    column((
        header(),
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
        show(move || tasks.with(|list| !list.is_empty()), move || task_list(tasks))
            .fallback(|| text("やることはありません。上の欄からタスクを追加してください。")),
        text(move || format!("残り {} 件", left.get())),
    ))
    .style(style().padding(24).gap(16))
}

fn main() -> kivact::Result<()> {
    App::new("タスク").run(app)
}
```

ステップ 3 の仮のカウンターとボタンはなくなりました。残り件数は、タスクから数えるようになっています。

## 新しく出てきたもの

**タスクは構造体です。** `Task` は `id`、`title`、`done` を持ちます。一覧に入れる型には、`derive` が 2 つ必要です。行へ項目をコピーするために `Clone`、古い項目と新しい項目を比べて何が変わったかを調べるために `PartialEq` です。`tasks` は `Signal<Vec<Task>>` で、`update` で変更します。`push` はタスクを追加し、`retain` は条件に合うタスクだけを残すので、削除に使えます。

**`var`** は、画面が監視しない普通の可変変数です。`next_id` は、重複しない id を配ります。変更しても画面は更新されません。`add` クロージャだけが使うカウンターには、それが適しています。画面に表示するものには、`signal` を使ってください。

**`for_each(items, key, row)`** は、項目ごとに 1 行を作ります。

- `items` は、一覧を持つシグナルです。
- `key` は、項目を見分ける値を返します。ここでは `task.id` です。行はキーで対応づけられるので、2 番目のタスクを削除すると、Kivact はその行だけを取り除き、ほかの行は、その状態ごと残します。
- `row` は、新しいキーごとに一度だけ呼ばれます。`Binding<Task>` を 1 つ受け取ります。現在の項目を reactive に読み書きするハンドルです。`task.get()` で読みます。

`column(for_each(..))` は、行を縦に並べます。

**チェックボックスを 1 つのフィールドにバインドする。** `checkbox(名前, バインディング)` は、`bool` を編集するチェックボックスを作ります。行の `done` フラグは一覧の中にあるので、
`bind!(task.done)`（または `task.project(|task| &task.done, |task| &mut task.done)`）
で、この id のタスクの、そのフィールドだけへの双方向バインディングを作ります。チェックボックスをクリックすると一覧へ書き込まれ、`left` が数え直されます。`field` の最初のクロージャはフィールドを読み、2 つ目は書き込み用の参照を返します。

`checkbox` は名前にクロージャを受け取れるので、ラベルはタイトルに追従します。この名前は、スクリーンリーダーが読み上げる名前でもあります。

**`task_row` が `tasks` を受け取る理由。** 削除ボタンは、行の項目ではなく一覧全体を変更するので、行には一覧のシグナルが必要です。`Signal` は `Copy` なので、渡してもコストはかかりません。

**`show(条件, view).fallback(別の view)`** は、条件が真のあいだ `view` を、偽のあいだ `fallback` を表示します。条件は、ほかのリアクティブなプロパティと同じく、シグナルを読むクロージャです。ここでは、タスクが 1 件以上あれば一覧を、なければヒントを表示します。分岐が切り替わると、Kivact は新しい分岐を作り、古い分岐を破棄します。

## 実行する

```text
cargo run
```

チェックボックスと削除ボタンのついた 3 つのタスクと、いちばん下の「残り 3 件」が表示されます。「牛乳を買う」にチェックを入れると、「残り 2 件」になります。入力欄からタスクを追加すると、一覧の最後に現れます。すべてのタスクを削除すると、一覧が「やることはありません。上の欄からタスクを追加してください。」に置き換わります。

次へ: [ステップ 6: スタイルとアクセシビリティ](06-style-and-accessibility.md)
