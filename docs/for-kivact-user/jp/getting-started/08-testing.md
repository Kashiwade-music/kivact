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

fn header(dark: Signal<bool>) -> impl Component {
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

fn task_row(tasks: Signal<Vec<Task>>, task: Binding<Task>) -> impl Component {
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

fn task_list(tasks: Signal<Vec<Task>>) -> impl Component {
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

fn tasks_screen(dark: Signal<bool>, saved: Vec<Task>) -> impl Component {
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

#[cfg(test)]
mod tests {
    use super::*;
    use kivact::test::Screen;

    const LOAD: Duration = Duration::from_millis(600);

    /// アプリを開き、最初の読み込みの擬似的な失敗を乗り越えます。
    fn open() -> Screen {
        let screen = kivact::test::render(app);
        screen.advance(LOAD);
        screen.get_by_role(Role::Button, "もう一度試す").click();
        screen.advance(LOAD);
        screen
    }

    fn add(screen: &Screen, title: &str) {
        screen.get_by_role(Role::TextBox, "新しいタスク").type_text(title);
        screen.get_by_role(Role::Button, "追加").click();
    }

    #[test]
    fn the_first_load_fails_and_trying_again_shows_the_tasks() {
        let screen = kivact::test::render(app);
        screen.get_by_text("タスクを読み込み中...");

        screen.advance(LOAD);
        screen.get_by_text("エラー: 保存したタスクを読み込めませんでした");

        screen.get_by_role(Role::Button, "もう一度試す").click();
        screen.advance(LOAD);
        screen.get_by_role(Role::CheckBox, "牛乳を買う");
        screen.get_by_text("残り 3 件");
    }

    #[test]
    fn adding_a_task_clears_the_field() {
        let screen = open();
        let field = || screen.get_by_role(Role::TextBox, "新しいタスク");
        assert!(screen.get_by_role(Role::Button, "追加").is_disabled());

        add(&screen, "植物に水をやる");
        screen.get_by_role(Role::CheckBox, "植物に水をやる");
        screen.get_by_text("残り 4 件");
        assert_eq!(field().value().as_deref(), Some(""));
        assert!(screen.get_by_role(Role::Button, "追加").is_disabled());
    }

    #[test]
    fn enter_adds_the_task_too() {
        let screen = open();
        screen.get_by_role(Role::TextBox, "新しいタスク").type_text("田中さんに電話する");
        screen.get_by_role(Role::TextBox, "新しいタスク").press_key(Key::Enter);
        screen.get_by_role(Role::CheckBox, "田中さんに電話する");
    }

    #[test]
    fn checking_a_task_lowers_the_count() {
        let screen = open();
        screen.get_by_role(Role::CheckBox, "牛乳を買う").click();
        assert_eq!(screen.get_by_role(Role::CheckBox, "牛乳を買う").checked(), Some(CheckedState::Checked));
        screen.get_by_text("残り 2 件");
    }

    #[test]
    fn removing_every_task_shows_the_hint() {
        let screen = open();
        screen.get_by_role(Role::Button, "牛乳を買う を削除").click();
        assert!(screen.query_by_role(Role::CheckBox, "牛乳を買う").is_none());
        screen.get_by_role(Role::Button, "チュートリアルを書く を削除").click();
        screen.get_by_role(Role::Button, "散歩に行く を削除").click();
        screen.get_by_text("やることはありません。上の欄からタスクを追加してください。");
    }
}
```

## 新しく出てきたもの

**`#[cfg(test)] mod tests`** は、`cargo test` のときだけコンパイルされます。`#[test]` を付けた関数が、それぞれ 1 つのテストです。

**`kivact::test::render(app)`** は、実際のウィンドウでアプリを組み立て、`Screen` を返します。ウィンドウはほかの作業からフォーカスを奪わずに開き、テストが終わると閉じます。画面は**仮想時計**で動くので、テストが実時間で待つことはありません。`screen.advance(LOAD)` は、時計を 600 ミリ秒進めます。仮の読み込みが終わるのに十分な時間です。

**クエリは、利用者やスクリーンリーダーと同じ方法で要素を探します。**

- `get_by_role(Role::Button, "追加")` は、役割とアクセシブルな名前でコントロールを探します。ステップ 6 ですべてのコントロールに名前を付けたのは、このためです。
- `get_by_text("残り 3 件")` は、画面上の文字を探します。
- `get_by_*` は、一致するものがない場合や複数ある場合に、読みやすいメッセージとともにテストを失敗させます。`query_by_*` は代わりに `Option` を返すので、`query_by_role(..).is_none()` のように、何かが*ない*ことを確かめるときに使います。`all_by_*` は `Vec` を返します。

**操作は本物の入力です。** `.click()` は要素を押し、`.type_text("..")` はテキスト欄に入力し、`.press_key(Key::Enter)` はキーを押します。これらは、利用者が実際にそれをできるかも確認します。無効な要素や、ほかの要素に隠れた要素をクリックすると、テストは失敗します。

**アサーション。** 要素の状態は、`.is_disabled()`、`.checked()` (`Option<CheckedState>`)、`.value()` (欄の文字) で読み取ります。判定には、普通の `assert!` と `assert_eq!` を使います。

**ヘルパーでテストを短くする。** `open()` と `add()` は、繰り返す手順をまとめた普通の関数です。テストごとに新しく `render` するので、テストどうしで状態が共有されることもありません。

## 実行する

```text
cargo test
```

テストの実行中は、ウィンドウが一瞬ずつ現れます。結果は次のようになります。

```text
running 5 tests
test tests::adding_a_task_clears_the_field ... ok
test tests::checking_a_task_lowers_the_count ... ok
test tests::enter_adds_the_task_too ... ok
test tests::removing_every_task_shows_the_hint ... ok
test tests::the_first_load_fails_and_trying_again_shows_the_tasks ... ok

test result: ok. 5 passed; 0 failed
```

最初のテストの `"残り 3 件"` を `"残り 4 件"` に変えて、もう一度実行してみてください。テストは失敗し、メッセージに画面上にあったものが一覧で示されます。Kivact は、失敗したウィンドウのスクリーンショットとアクセシビリティツリーも `target/kivact-test-failures/` に保存します。正確なパスは失敗メッセージに書かれています。確かめたら、元に戻してください。

知っておくことが 2 つあります。

- `kivact::test::render` を使うテストのバイナリでは、テストの中から `App::run` を呼べません。テストのウィンドウはすべて、1 つのイベントループを共有するからです。そのため `main` をテストから呼ぶことはできません。コンポーネント (`app`) をテストしてください。
- テストのウィンドウを開くプロセスは、同時に 1 つにしてください。複数の `cargo test` を同時に実行するのは避けましょう。

## 次に読むもの

レイアウト、状態、入力、一覧、スタイル、非同期の読み込み、テストをそなえた、完全なアプリができました。

- [How-to ガイド](../how-to-guide/README.md)は、フォームの検証、ダイアログ、メニューなどの具体的な課題を、このチュートリアルより詳しく扱います。
- [UI キット](../../../for-kivact-ui-user/jp/README.md)は、標準コンポーネントの見た目を変え、さらにコンポーネントを追加します。Tasks の既定の見た目を、キットのどれかに差し替えてみてください。
- API の全体は、`cargo doc -p kivact --open` で確認できます。
