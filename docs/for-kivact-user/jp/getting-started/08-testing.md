# ステップ 8: テスト

[English](../../en/getting-started/08-testing.md)

最後のステップでは、人が使うのと同じ方法でアプリを操作する自動テストを書きます。入力し、クリックし、画面に表示されているものを読み取ります。読み込み中とエラーの状態、タスクの追加、チェック、削除、そして空の状態をテストします。

## 準備

テストには `kivact::test` を使います。これは `test` feature の背後にあります。`Cargo.toml` で、テスト用にだけ有効にします。

```toml
[dev-dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact", features = ["test"] }
```

## テストつきのプログラム

ステップ 7 のプログラムの末尾に、`tests` モジュールを加えたものです。`#[cfg(test)]` より上は変わっていません。

```rust,test_harness
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

#[cfg(test)]
mod tests {
    use super::*;
    use kivact::test::Screen;

    const LOAD: Duration = Duration::from_millis(600);

    /// Opens the app and gets past the simulated failure of the first load.
    fn open() -> Screen {
        let screen = kivact::test::render(app);
        screen.advance(LOAD);
        screen.get_by_role(Role::Button, "Try again").click();
        screen.advance(LOAD);
        screen
    }

    fn add(screen: &Screen, title: &str) {
        screen.get_by_role(Role::TextBox, "New task").type_text(title);
        screen.get_by_role(Role::Button, "Add").click();
    }

    #[test]
    fn the_first_load_fails_and_trying_again_shows_the_tasks() {
        let screen = kivact::test::render(app);
        screen.get_by_text("Loading tasks...");

        screen.advance(LOAD);
        screen.get_by_text("Error: Could not read the saved tasks");

        screen.get_by_role(Role::Button, "Try again").click();
        screen.advance(LOAD);
        screen.get_by_role(Role::CheckBox, "Buy milk");
        screen.get_by_text("3 left");
    }

    #[test]
    fn adding_a_task_clears_the_field() {
        let screen = open();
        let field = || screen.get_by_role(Role::TextBox, "New task");
        assert!(screen.get_by_role(Role::Button, "Add").is_disabled());

        add(&screen, "Water the plants");
        screen.get_by_role(Role::CheckBox, "Water the plants");
        screen.get_by_text("4 left");
        assert_eq!(field().value().as_deref(), Some(""));
        assert!(screen.get_by_role(Role::Button, "Add").is_disabled());
    }

    #[test]
    fn enter_adds_the_task_too() {
        let screen = open();
        screen.get_by_role(Role::TextBox, "New task").type_text("Call Sam");
        screen.get_by_role(Role::TextBox, "New task").press_key(Key::Enter);
        screen.get_by_role(Role::CheckBox, "Call Sam");
    }

    #[test]
    fn checking_a_task_lowers_the_count() {
        let screen = open();
        screen.get_by_role(Role::CheckBox, "Buy milk").click();
        assert_eq!(screen.get_by_role(Role::CheckBox, "Buy milk").checked(), Some(CheckedState::Checked));
        screen.get_by_text("2 left");
    }

    #[test]
    fn removing_every_task_shows_the_hint() {
        let screen = open();
        screen.get_by_role(Role::Button, "Remove Buy milk").click();
        assert!(screen.query_by_role(Role::CheckBox, "Buy milk").is_none());
        screen.get_by_role(Role::Button, "Remove Write the tutorial").click();
        screen.get_by_role(Role::Button, "Remove Go for a walk").click();
        screen.get_by_text("Nothing to do. Add a task above.");
    }
}
```

## 新しく出てきたもの

**`#[cfg(test)] mod tests`** は、`cargo test` のときだけコンパイルされます。`#[test]` を付けた関数が、それぞれ 1 つのテストです。

**`kivact::test::render(app)`** は、App 全体を `arvane-virtual` のバックエンドで組み立て、`Screen` を返します。手元では、描画されたフレームを実ウィンドウへミラー表示できます。ミラー表示は任意で、環境変数 `KIVACT_TEST_MIRROR=off` で無効にできます。検証する App や入力経路は同じです。

この例の `sleep` は**仮想時計**で進みます。`screen.advance(LOAD)` は時計を 600 ミリ秒進め、仮の読み込みを完了させます。実際のネットワークや背景処理まで仮想時計で完了するわけではないため、そのような処理をテストするときは、結果が届いたことを待ってから確認します。

**クエリは、利用者やスクリーンリーダーと同じ方法で要素を探します。**

- `get_by_role(Role::Button, "追加")` は、役割とアクセシブルな名前でコントロールを探します。ステップ 6 ですべてのコントロールに名前を付けたのは、このためです。
- `get_by_text("残り 3 件")` は、画面上の文字を探します。
- `get_by_*` は、一致するものがない場合や複数ある場合に、読みやすいメッセージとともにテストを失敗させます。`query_by_*` は代わりに `Option` を返すので、`query_by_role(..).is_none()` のように、何かが*ない*ことを確かめるときに使います。`all_by_*` は `Vec` を返します。

**操作は App の入力経路を通ります。** `.click()` は要素を押し、`.type_text("..")` はテキスト欄に入力し、`.press_key(Key::Enter)` はキーを押します。これらは、利用者が実際にそれをできるかも確認します。無効な要素や、ほかの要素に隠れた要素をクリックすると、テストは失敗します。

**アサーション。** 要素の状態は、`.is_disabled()`、`.checked()` (`Option<CheckedState>`)、`.value()` (欄の文字) で読み取ります。判定には、普通の `assert!` と `assert_eq!` を使います。

**ヘルパーでテストを短くする。** `open()` と `add()` は、繰り返す手順をまとめた普通の関数です。テストごとに新しく `render` するので、テストどうしで状態が共有されることもありません。

## 実行する

```text
cargo test
```

ミラー表示を有効にしている場合は、ウィンドウが一瞬ずつ現れることがあります。ターミナルには次のような結果が表示されます。

```text
running 5 tests
test tests::adding_a_task_clears_the_field ... ok
test tests::checking_a_task_lowers_the_count ... ok
test tests::enter_adds_the_task_too ... ok
test tests::removing_every_task_shows_the_hint ... ok
test tests::the_first_load_fails_and_trying_again_shows_the_tasks ... ok

test result: ok. 5 passed; 0 failed
```

最初のテストの `"残り 3 件"` を `"残り 4 件"` に変えて、もう一度実行してみてください。テストは失敗し、メッセージに画面上にあったものが一覧で示されます。Kivact は、失敗した描画フレームとアクセシビリティツリーも `target/kivact-test-failures/` に保存します。正確なパスは失敗メッセージに書かれています。確かめたら、元に戻してください。

知っておくことが 2 つあります。

- テストでは `render(app)` に view を作る関数を渡します。画面を描画するテストのバイナリでは、テストドライバーが App のイベントループを管理するため、`App::run` や `main` をテストから呼びません。
- テストのプロセスは同時に 1 つにします。Kivact のチェックアウト内では `dev/tools/window-lock.sh -- cargo test` を使うと、ほかの worktree と共通のウィンドウロックを使えます。`KIVACT_TEST_MIRROR=off` の場合も同じです。

## 次に読むもの

レイアウト、状態、入力、一覧、スタイル、非同期の読み込み、テストをそなえた、完全なアプリができました。

- [How-to ガイド](../how-to-guide/README.md)は、フォームの検証、ダイアログ、メニューなどの具体的な課題を、このチュートリアルより詳しく扱います。
- [UI キット](../../../for-kivact-ui-user/jp/README.md)は、標準コンポーネントの見た目を変え、さらにコンポーネントを追加します。Tasks の既定の見た目を、キットのどれかに差し替えてみてください。
- API の全体は、`cargo doc -p kivact --open` で確認できます。
