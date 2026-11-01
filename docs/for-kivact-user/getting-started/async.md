# 7. 非同期処理

検索やファイル読込などはすぐに終わるとは限りません。Kivact の `resource` は、状態を元にデータを取得し、処理中・成功・失敗を画面へ伝えます。

```rust,no_run
use std::time::Duration;
use kivact::prelude::*;

fn search() -> impl IntoView {
    let query = signal(String::new());
    let result = resource(
        query,
        |word| async move {
            kivact::task::sleep(Duration::from_millis(200)).await;
            if word.contains('!') {
                return Err(UiError::msg("検索語に ! は使えません"));
            }
            Ok(format!("検索語: {word}"))
        },
    );

    column((
        text_input("検索語", query),
        show(move || result.state().is_pending(), || text("検索中…")),
        show_some(
            move || result.error(),
            |error| text(move || error.get().to_string()).role(Role::Alert),
        ),
        show_some(move || result.latest(), |latest| text(latest)),
    ))
    .style(style().gap(8).padding(12))
}

fn main() -> kivact::Result<()> {
    App::new("非同期の例").run(search)
}
```

`resource` の第 1 引数は検索語の source（signal や、`move || (query.get(), page.get())` のような closure）、第 2 引数はその検索語から結果を作る非同期関数です。検索語が変わると取得し直し、古い要求の結果は採用しません。非同期関数は `Result<T, UiError>` を返します。`UiError::msg(...)` は文字列から失敗を作り、`std::error::Error` を実装する他の誤りは `?` で `UiError` に変わります。

`state()` は最新の要求の状態 `AsyncState` を返します。`Idle`（まだ始まっていない）、`Pending`（処理中）、`Ready(値)`（成功）、`Failed(誤り)`（失敗）の 4 つです。`latest()` は最後に成功した値で、取得し直している間や失敗した後も残ります。`AsyncState` は `is_pending()`・`is_ready()`・`is_failed()` と、成功の値を返す `value()`、失敗の誤りを返す `error()` を持ちます。`Resource` と `Action` は `error()` で失敗の誤り（`Option<UiError>`）を直接読めるので、失敗のときだけ出す表示は `show_some(move || result.error(), ..)` と書けます。上の例では、検索語に `!` を含めると取得が失敗し、`show_some` がそのときだけ警告文を出します。

処理中は `show(pending)`、失敗は `show_some(error)`、最後の成功値は `show_some(latest)` と、状態ごとに一つずつ並べるのが標準の書き方です。それぞれが自分の条件で出入りするので、`match` の arm を揃える必要がありません。`latest` は取得し直している間も残り、前の結果を見せたまま「検索中…」を添えられます。

クリック時にだけ処理を始める場合は `action(...)` と `action.run(())` を使います。入力を受け取る処理は同じ `action` に引数つきの closure を渡して作り、`action.run(入力)` で始めます。`run` は入力を一つ受ける一つの method で、入力のない action の入力は `()` です。closure の中で入力の method や演算を使うときは、引数の型を書きます（下の `|n: u32|`。入力の型は `run` の呼出しより先に決まる必要があるためです）。次の例の `tests` は、試験のウィンドウで仮想時計を進めて結果を待ちます。

```rust,test_harness
use std::time::Duration;
use kivact::prelude::*;

fn action_demo() -> impl IntoView {
    let work = action(|n: u32| async move {
        kivact::task::sleep(Duration::from_millis(100)).await;
        Ok(n * 2)
    });

    column((
        button("実行").on_press(move || work.run(21)),
        text(move || match work.state() {
            AsyncState::Idle => "待機中".to_string(),
            AsyncState::Pending => "処理中".to_string(),
            AsyncState::Ready(value) => format!("結果: {value}"),
            AsyncState::Failed(error) if error.is_cancelled() => "中止".to_string(),
            AsyncState::Failed(error) => format!("失敗: {error}"),
        }),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Action の例").run(action_demo)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pressing_runs_the_action() {
        let screen = kivact::test::render(action_demo);
        screen.get_by_role(Role::Button, "実行").click();
        screen.find_by_text("結果: 42");
    }
}
```

`Action` は `state()`、`latest()`、`input()`（最後に始めた処理の入力）、`retry()`、`cancel()` も持ちます。新しい `run` は処理中の前の処理を取り消します。`match` は Rust の列挙型を各状態に分ける書き方です。取り消した処理は `Failed` になり、その誤りの `is_cancelled()` が true を返します。成功・失敗・中止をすべて扱うと、処理中の画面が利用者に分かりやすくなります。

保存のように、順番を守って一つずつ実行したい処理には `.policy(ActionPolicy::Queue)` を付けます。`run` は待ち行列に入り、前の処理が終わってから始まります。`ActionPolicy::QueueLatest` は待っている処理を最新の一つだけに置き換えます。待ち行列は、部品が消えた後もしばらく処理を続け、アプリを閉じるときは処理が終わるまで最大 3 秒待ちます。

Resource と Action の失敗は `AsyncState::Failed` に保存されます。error boundary で囲んだだけでは、この失敗表示は作られません。通常イベントの `Result::Err` が boundary に届く経路とは区別し、画面で状態を `match` して表示します。

`async` の処理は UI のタスク実行器で進みます。時間のかかる同期 I/O や CPU 処理をそのまま行うと画面を止めるため、その部分を `blocking(move || ...)` に渡します。`blocking` は別スレッドで関数を実行し、結果を返す Future を返すので、`resource` や `action` の非同期関数の中で `blocking(move || ...).await` のように待って結果を受け取ります。`blocking` と `spawn` は `kivact::prelude` にも `kivact::task` にもあります。取り消しを確かめたい処理は引数つきの `blocking(move |cancel: CancelToken| ...)`（`kivact::task::CancelToken`）と書きます。新しい要求で前の要求が取り消されると `cancel.is_cancelled()` が true になり、長い処理は途中でこれを確かめて止められます。引数の型は書く必要があります（`action` の入力と同じ理由です）。状態ハンドルは別スレッドへ渡せないので、ブロッキング処理には文字列などの値を渡し、完了後に UI 側で状態を更新します。

`resource` と `action` は部品の組み立て中に作ります。イベントでは既存の `Action` を `run` してください。次の [画面のテスト](testing.md) では、テストの書き方と、仮想時計で待ち時間を進める方法を扱います。


## 自分で動かして確認する

`cargo run -p kivact-example-async` を起動し、処理を始めたときに待機の表示へ変わり、完了後に結果が表示されることを確認します。上のコードを入門 package に置いたら、待ち時間を変え、待機中にも画面が操作できることを確かめてください。
