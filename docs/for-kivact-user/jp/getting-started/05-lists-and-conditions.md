# ステップ 5: 一覧と条件分岐

[English](../../en/getting-started/05-lists-and-conditions.md)

このステップでは、仮の一覧を本物にします。タスクはシグナルの中に置かれ、各行にはチェックボックスと「Remove」ボタンがあります。一覧が空のときは、ヒントを表示します。

## ここまでのプログラム

```rust,no_run
use kivact::prelude::*;

#[derive(Clone, PartialEq)]
struct Task {
    id: u32,
    title: String,
    done: bool,
}

#[component]
fn header() -> impl Element {
    column((
        text("Tasks"),
        text("Things to do today"),
    ))
    .style(style().gap(4))
}

#[component]
fn task_row(tasks: Signal<Vec<Task>>, task: Signal<Task>) -> impl Element {
    let id = task.get().id;
    row((
        checkbox(move || task.get().title, bind!(task.done)),
        button("Remove").on_press(move || tasks.update(|list| list.retain(|task| task.id != id))),
    ))
    .style(style().gap(8))
}

#[component]
fn task_list(tasks: Signal<Vec<Task>>) -> impl Element {
    column(for_each(
        tasks,
        |task| task.id,
        move |task| task_row(tasks, task),
    ))
    .style(style().gap(8))
}

fn app() -> impl IntoView {
    let tasks = signal(vec![
        Task { id: 1, title: "Buy milk".to_owned(), done: false },
        Task { id: 2, title: "Write the tutorial".to_owned(), done: false },
        Task { id: 3, title: "Go for a walk".to_owned(), done: false },
    ]);
    let next_id = var(4);
    let draft = signal(String::new());
    let left = memo(move || tasks.with(|list| list.iter().filter(|task| !task.done).count()));

    let add = callback(move |_| {
        let title = draft.get().trim().to_owned();
        if !title.is_empty() {
            let id = next_id.get();
            next_id.set(id + 1);
            tasks.update(|list| list.push(Task { id, title, done: false }));
            draft.set(String::new());
        }
    });

    column((
        header(),
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
        show(move || tasks.with(|list| !list.is_empty()), move || task_list(tasks))
            .fallback(|| text("Nothing to do. Add a task above.")),
        text(move || format!("{} left", left.get())),
    ))
    .style(style().padding(24).gap(16))
}

fn main() -> kivact::Result<()> {
    App::new("Tasks").run(app)
}
```

ステップ 3 の仮のカウンターとボタンは、もうありません。残りの数は、タスクから計算されます。

## 新しいこと

**タスクは構造体です。** `Task` は、`id`、`title`、`done` のフラグを持ちます。Kivact が一覧に入れるものには、2 つの derive が必要です。1 つ目は `Clone` です。項目を行へコピーするために使います。2 つ目は `PartialEq` です。古い項目と新しい項目を比べて、何が変わったかを調べるために使います。`tasks` は `Signal<Vec<Task>>` です。変更するには `update` を使います。`push` はタスクを追加し、`retain` は条件に合うタスクだけを残します。これでタスクを 1 つ削除できます。

**`var`** は、画面が監視しない、普通の書き換え可能な箱です。`next_id` は、重複しない id を配ります。値を変えても何も更新されません。これは、`add` のクロージャだけが使うカウンターに、まさに必要な性質です。画面に表示するものには `signal` を使います。

**`for_each(items, key, build_row)`** は、項目ごとに 1 つの行を作ります。

- `items` は、一覧を持つシグナルです。
- `key` は、項目を識別する値を返します。ここでは `task.id` です。行はキーで対応づけられるので、2 番目のタスクを削除すると、Kivact はその行だけを取り除き、ほかの行はそれが持つ状態も含めて残します。
- `build_row` は、新しいキーごとに一度だけ呼ばれます。その項目の `Signal<Task>` を受け取ります。それを読むと現在の項目が得られ、それに書き込むと一覧が書き換わります。

`column(for_each(..))` は、行を縦に並べます。シグナルの代わりに読み取り専用の `Value<Vec<Task>>` を渡すと、各行は読み取り専用の `Value<Task>` を受け取ります。元の項目をチェックボックスで編集できるのは、書き込み可能な一覧のシグナルがあるからです。キーに項目の現在の配列の添字を使ってはいけません。項目を削除したり並べ替えたりすると、その添字は変わってしまうからです。

**行は 2 つの props を持つコンポーネントです。** `task_row` は、一覧と行のタスクを受け取ります。本体は行ごとに一度だけ実行され、本体は読んだ値を追跡しません。そのため、ここでの `task.get().id` は、Remove のハンドラーのために id を一度だけ読みます。`move || task.get().title` のようなクロージャの中での読み取りは、後の変更に従います。

**チェックボックスを 1 つのフィールドに結びつける。** `checkbox(label, value)` は、`Signal<bool>` を編集するチェックボックスを表示します。行の `done` フラグは一覧の中にあるので、`bind!(task.done)` は、そのタスクのそのフィールドだけを指す `Signal<bool>` を作ります。チェックボックスをクリックすると一覧に書き込まれ、`left` の memo が数え直されます。

`checkbox` はラベルとしてクロージャを受け取るので、ラベルはタイトルに従います。ラベルは描画もされ、スクリーンリーダーが読み上げる名前にもなります。

**`task_row` が `tasks` を受け取る理由。** Remove ボタンは、行の項目だけでなく一覧全体を変える必要があるので、行には一覧のシグナルが必要です。`Signal` は `Copy` なので、下に渡してもコストはかかりません。

**`show(condition, view).fallback(other)`** は、条件が真のあいだ `view` を表示し、偽のあいだ `other` を表示します。条件は、ほかの変化する props と同じように、シグナルを読むクロージャです。ここでは、タスクが 1 つ以上あるときに一覧を表示し、1 つもないときにヒントを表示します。表示が切り替わるとき、Kivact は新しい方の分岐を作り、古い方を破棄します。

## 実行する

```text
cargo run
```

3 つのタスクが、それぞれチェックボックスと Remove ボタン付きで表示され、一番下に「3 left」と表示されます。「Buy milk」にチェックを入れると、数が「2 left」に減ります。欄でタスクを追加すると、一覧の最後に表示されます。すべてのタスクを削除すると、一覧は「Nothing to do. Add a task above.」に置き換わります。

次へ: [ステップ 6: スタイルとアクセシビリティ](06-style-and-accessibility.md)
