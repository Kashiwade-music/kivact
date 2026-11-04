# ステップ 7: 非同期処理

[English](../../en/getting-started/07-async.md)

このステップでは、アプリの起動時に保存済みのタスクを読み込みます。読み込みには時間がかかり、失敗することもあるので、読み込み中のメッセージ、再試行ボタンつきのエラー、タスク一覧のどれかを表示します。

## ここまでのプログラム

```rust,no_run
use std::time::Duration;

use kivact::prelude::*;
use kivact::style::provide_theme;

const LOGO: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><rect width="24" height="24" rx="6" fill="#336699"/><path d="M6 12.5l4 4 8-9" stroke="white" stroke-width="2.5" fill="none"/></svg>"##;

#[derive(Clone, PartialEq)]
struct Task {
    id: u32,
    title: String,
    done: bool,
}

fn header(dark: Signal<bool>) -> impl IntoView {
    row((
        image(LOGO).name("タスクのロゴ").style(style().size(32, 32)),
        column((
            text("タスク").role(Role::Heading).style(style().font_size(28).weight(700)),
            text("今日やること").style(style().color(ColorToken::TextMuted)),
        ))
        .style(style().gap(2).grow(1)),
        switch("ダークモード", dark),
    ))
    .style(style().gap(12).align_items(Align::Center))
}

fn task_row(tasks: Signal<Vec<Task>>, task: Binding<Task>) -> impl IntoView {
    let id = task.get_untracked().id;
    let done = bind!(task.done);
    row((
        checkbox(move || task.get().title, done),
        button("削除")
            .name(move || format!("{} を削除", task.get().title))
            .variant(kivact::ui::ButtonVariant::Ghost)
            .on_press(move || tasks.update(|list| list.retain(|task| task.id != id))),
    ))
    .role(Role::ListItem)
    .style(style().gap(8).align_items(Align::Center).justify_content(Justify::SpaceBetween))
}

fn task_list(tasks: Signal<Vec<Task>>) -> impl IntoView {
    column(for_each(
        tasks,
        |task| task.id,
        move |task| task_row(tasks, task),
    ))
    .role(Role::List)
    .name("タスク一覧")
    .style(
        style()
            .gap(8)
            .padding(12)
            .radius(8)
            .background(ColorToken::SurfaceAlt)
            .border(1, ColorToken::Border),
    )
}

fn tasks_screen(dark: Signal<bool>, saved: Vec<Task>) -> impl IntoView {
    let next_id = var(saved.iter().map(|task| task.id).max().unwrap_or(0) + 1);
    let tasks = signal(saved);
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
        header(dark),
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
        text(move || format!("残り {} 件", left.get())).style(style().color(ColorToken::TextMuted)),
    ))
    .style(
        style()
            .padding(24)
            .gap(16)
            .width(pct(100))
            .height(pct(100))
            .background(ColorToken::Surface)
            .color(ColorToken::Text),
    )
}

/// 保存済みのタスクをディスクから読み込むふりをします。エラーの状態を確かめられる
/// よう、最初の試行は失敗し、以降の試行は成功します。
async fn load_tasks(attempt: u32) -> kivact::Result<Vec<Task>> {
    kivact::task::sleep(Duration::from_millis(600)).await;
    if attempt == 1 {
        return Err(UiError::msg("保存したタスクを読み込めませんでした"));
    }
    Ok(vec![
        Task { id: 1, title: "牛乳を買う".to_owned(), done: false },
        Task { id: 2, title: "チュートリアルを書く".to_owned(), done: false },
        Task { id: 3, title: "散歩に行く".to_owned(), done: false },
    ])
}

fn loading_screen(dark: Signal<bool>, attempt: Signal<u32>) -> impl IntoView {
    let saved = resource(attempt, load_tasks);
    dynamic(
        move || saved.state(),
        move |state| match state {
            AsyncState::Idle | AsyncState::Pending => text("タスクを読み込み中...")
                .role(Role::Status)
                .style(style().padding(24))
                .into_view(),
            AsyncState::Failed(error) => column((
                text(format!("エラー: {error}")).role(Role::Alert),
                button("もう一度試す").on_press(move || attempt.update(|n| *n += 1)),
            ))
            .style(style().padding(24).gap(12))
            .into_view(),
            AsyncState::Ready(tasks) => tasks_screen(dark, tasks).into_view(),
        },
    )
}

fn app() -> impl IntoView {
    let dark = signal(false);
    let attempt = signal(1);
    let theme = memo(move || if dark.get() { Theme::dark() } else { Theme::light() });
    provide_theme(theme, move || loading_screen(dark, attempt))
}

fn main() -> kivact::Result<()> {
    App::new("タスク").run(app)
}
```

`tasks_screen` は、タスクを自分で作る代わりに、読み込んだタスクを受け取るようになりました。画面のほかの部分は、ステップ 6 から変わっていません。

## 新しく出てきたもの

**`resource(source, load)`** は、非同期の処理を開始し、その進み具合を状態として保持します。

- `source` は、処理への入力となるリアクティブな値で、ここでは `attempt` シグナルです。Kivact は最初に一度処理を開始し、`source` が変わるたびに、もう一度開始します。処理の実行中に `source` が変わった場合、その古い処理の結果は捨てられます。
- `load` は、現在の `source` の値を受け取り、`kivact::Result<T>` を返す `async` 関数です。成功なら `Ok(値)`、失敗なら `Err(UiError::msg("..."))` を返します。`std::error::Error` を実装する任意のエラーには `?` が使え、`UiError` に変換されます。

`load_tasks` は、ファイルの読み込みやサーバーの呼び出しの代わりです。`kivact::task::sleep(..).await` はウィンドウを止めずに待つので、待っているあいだもウィンドウは操作できます。`load` には、`fn` もクロージャも使えます。大きなファイルの読み込みのように時間のかかる同期処理は、`kivact::task::blocking(..)` に任せてください。

すべての状態を確かめられるよう、この仮の読み込みは最初の試行で失敗します。試し終わったら、`if attempt == 1` の確認を削除してください。

**`saved.state()`** は `AsyncState` を返します。次のどれかです。

- `Idle` または `Pending`: まだ何も終わっていません。
- `Failed(error)`: 処理がエラーを返しました。
- `Ready(value)`: 処理が値とともに成功しました。

**`dynamic(source, build)`** は、値に応じて表示するものを選びます。`source` が変わるたびに、Kivact は古い view を破棄し、新しい値で `build` を呼びます。ここでは状態ごとに専用の view があり、`match` によって、すべての状態を扱っているかを Rust が確かめます。分岐ごとに返す view の種類が違うため、どれも `.into_view()` で終え、共通の型にそろえています。

分岐の中で作った状態は、その分岐が生きているあいだだけ残ります。`tasks_screen` は、データが届いたときに `tasks` シグナルを作ります。「もう一度試す」を押すと新しい読み込みが始まり、その結果から新しい画面が組み立てられます。

**状態メッセージの役割。** 読み込み中のメッセージには `Role::Status`、エラーには `Role::Alert` を付けています。表示されたときに、スクリーンリーダーが読み上げます。

## 実行する

```text
cargo run
```

ウィンドウに約半秒のあいだ「タスクを読み込み中...」と表示され、その後「エラー: 保存したタスクを読み込めませんでした」と「もう一度試す」ボタンが表示されます。ボタンを押すと、また読み込み中のメッセージが出て、その後ステップ 6 で作ったタスク一覧と、いちばん下の「残り 3 件」が表示されます。

次へ: [ステップ 8: テスト](08-testing.md)
