# ステップ 7: 非同期処理

[English](../../en/getting-started/07-async.md)

このステップでは、アプリが起動したときに保存済みのタスクを読み込みます。読み込みには少し時間がかかり、失敗することもあるので、アプリは読み込み中のメッセージ、再試行ボタン付きのエラー、またはタスク一覧のいずれかを表示します。

## ここまでのプログラム

```rust,no_run
use std::time::Duration;

use kivact::prelude::*;

const LOGO: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><rect width="24" height="24" rx="6" fill="#336699"/><path d="M6 12.5l4 4 8-9" stroke="white" stroke-width="2.5" fill="none"/></svg>"##;

#[derive(Clone, PartialEq)]
struct Task {
    id: u32,
    title: String,
    done: bool,
}

#[component]
fn header(dark: Signal<bool>) -> impl Element {
    row((
        image(LOGO).name("Tasks logo").style(style().size(32, 32)),
        column((
            text("Tasks").role(Role::Heading).level(1).style(style().font_size(28).font_weight(700)),
            text("Things to do today").style(style().color(color::TEXT_MUTED)),
        ))
        .style(style().gap(2).flex_grow(1)),
        switch("Dark mode", dark),
    ))
    .style(style().gap(12).align_items(Align::Center))
}

#[component]
fn task_row(tasks: Signal<Vec<Task>>, task: Signal<Task>) -> impl Element {
    let id = task.get().id;
    row((
        checkbox(move || task.get().title, bind!(task.done)),
        button("Remove")
            .name(move || format!("Remove {}", task.get().title))
            .variant(ButtonVariant::Ghost)
            .on_press(move || tasks.update(|list| list.retain(|task| task.id != id))),
    ))
    .role(Role::ListItem)
    .style(style().gap(8).align_items(Align::Center).justify_content(Justify::SpaceBetween))
}

#[component]
fn task_list(tasks: Signal<Vec<Task>>) -> impl Element {
    column(for_each(
        tasks,
        |task| task.id,
        move |task| task_row(tasks, task),
    ))
    .role(Role::List)
    .name("Task list")
    .style(
        style()
            .gap(8)
            .padding(space::MD)
            .border_radius(radius::MD)
            .background(color::SURFACE_ALT)
            .border(1, color::BORDER),
    )
}

#[component]
fn tasks_screen(dark: Signal<bool>, saved: Vec<Task>) -> impl Element {
    let next_id = var(saved.iter().map(|task| task.id).max().unwrap_or(0) + 1);
    let tasks = signal(saved);
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
        header(dark),
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
        text(move || format!("{} left", left.get())).style(style().color(color::TEXT_MUTED)),
    ))
    .style(
        style()
            .padding(24)
            .gap(16)
            .width(pct(100))
            .height(pct(100))
            .background(color::SURFACE)
            .color(color::TEXT),
    )
}

/// Pretends to read the saved tasks from disk. The first attempt fails so that
/// you can see the error state; every later attempt succeeds.
async fn load_tasks(attempt: u32) -> kivact::Result<Vec<Task>> {
    kivact::task::sleep(Duration::from_millis(600)).await;
    if attempt == 1 {
        return Err(UiError::msg("Could not read the saved tasks"));
    }
    Ok(vec![
        Task { id: 1, title: "Buy milk".to_owned(), done: false },
        Task { id: 2, title: "Write the tutorial".to_owned(), done: false },
        Task { id: 3, title: "Go for a walk".to_owned(), done: false },
    ])
}

#[component]
fn loading_screen(dark: Signal<bool>) -> impl Element {
    // Counts the loads, so that the fake loader can fail only the first one.
    let attempts = var(0);
    let saved = resource((), move |()| {
        attempts.update(|n| *n += 1);
        load_tasks(attempts.get())
    });
    async_view(saved, move |tasks| tasks_screen(dark, tasks))
        .loading(|| text("Loading tasks...").role(Role::Status).style(style().padding(24)))
        .fallback(|error, retry| {
            column((
                text(format!("Error: {error}")).role(Role::Alert),
                button("Try again").on_press(retry),
            ))
            .style(style().padding(24).gap(12))
        })
}

fn app() -> impl IntoView {
    let dark = signal(false);
    provide_theme(
        move || if dark.get() { Theme::dark() } else { Theme::light() },
        move || loading_screen(dark),
    )
}

fn main() -> kivact::Result<()> {
    App::new("Tasks").run(app)
}
```

`tasks_screen` は、タスクを作る代わりに、読み込まれたタスクを受け取るようになりました。画面のそれ以外の部分は、ステップ 6 から変わりません。

## 新しいこと

**`resource(source, load)`** は、非同期の処理を開始し、その進行状況を状態として保持します。

- `source` は、処理の入力です。Kivact は、最初に一度だけ処理を開始し、`source` が変わるたびに、もう一度開始します。`source` にシグナルを使うと、変更のたびに新しいリクエストが作られ、まだ実行中のリクエストは破棄されます。ここでの `source` は `()` で、これは変わりません。そのため、処理は、下のエラーの画面の `retry`、または `saved.refetch()` で頼んだときにだけ、もう一度実行されます。
- `load` は、現在の source の値を受け取り、`kivact::Result<T>` の future を返します。成功なら `Ok(value)`、失敗なら `Err(UiError::msg("..."))` です。`std::error::Error + Send + Sync + 'static` を実装するエラーに `?` を使うと、それは `UiError` に変換されます。

`load_tasks` は、ファイルの読み込みやサーバーへの問い合わせの代わりです。`kivact::task::sleep(..).await` は、ウィンドウを止めずに待ちます。そのため、ウィンドウは反応し続けます。大きなファイルの読み込みなど、遅い同期的な処理は、`kivact::task::blocking(..)` に入れます。

すべての状態を見られるように、偽の読み込みは最初の試行で失敗します。`attempts` は、ステップ 5 の `var` で、読み込みが呼ばれるたびに増えます。実験が終わったら、`if attempt == 1` の確認を取り除いてください。

**`async_view(resource, ready)`** は、処理の状態を表示します。`<Suspense>` とエラーの境界を合わせたようなものです。

- `.loading(..)` は、まだ何も届いていないあいだに表示する view を作ります。
- `ready` は、処理が成功したら、その値から view を作ります。ここでは、読み込まれたタスクから、タスク画面全体を作ります。
- `.fallback(|error, retry| ..)` は、処理が失敗したときに表示する view を作ります。`retry` は、処理をもう一度開始する `Callback<()>` です。そのまま `on_press` に渡します。

`.loading(..)` と `.fallback(..)` を省くと、`async_view` は標準の読み込み表示と、再試行ボタン付きのエラーメッセージを表示します。

`ready` の view は、その結果が表示されているあいだ生きています。`tasks_screen` は、データが届いたときに `tasks` のシグナルを作り、「Try again」は新しい読み込みを開始します。それによって、新しい結果から新しい画面が作り直されます。再読み込みをまたいで残す必要のある状態は、`async_view` の外に置き、そのハンドルを渡してください。

状態を自分で扱いたいときは、`saved.state()` が `AsyncState`（`Idle`、`Pending`、`Failed`、`Ready`）を返します。[コンポーネントから非同期の処理を実行する](../how-to-guide/async-and-tasks.md) で、その使い方を説明しています。

**状態のメッセージのアクセシブルなロール。** `Role::Status` は読み込み中のメッセージに、`Role::Alert` はエラーに付けます。そうすると、表示されたときにスクリーンリーダーが読み上げます。

## 実行する

```text
cargo run
```

ウィンドウには、約半秒のあいだ「Loading tasks...」と表示されます。その後、「Error: Could not read the saved tasks」と「Try again」ボタンが表示されます。そのボタンを押すと、読み込み中のメッセージがもう一度表示され、その後、ステップ 6 で作ったタスク一覧が、一番下に「3 left」と共に表示されます。

次へ: [ステップ 8: テスト](08-testing.md)
