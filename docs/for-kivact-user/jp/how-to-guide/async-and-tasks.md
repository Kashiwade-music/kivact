# コンポーネントから非同期処理を実行する

[English](../../en/how-to-guide/async-and-tasks.md)

このガイドでは、ウィンドウを固まらせずに、データの読み込み、バックグラウンドでの保存、タイマーでの待機、重い処理の UI スレッド外での実行、他のスレッドからの値の受け取りを行う方法を説明します。入門チュートリアルを読み終えていることを前提にしています。

道具は `kivact::task` にあります。ほとんどの用途は次の 4 つで足ります。

| やりたいこと | 使うもの |
|---|---|
| シグナルに応じてデータを読み込み、読み込み中・エラー・完了を表示する | `resource` |
| 利用者の操作（保存・送信・再試行）で処理を始める | `action` |
| 表示する結果のない future を動かす（タイマー、チャンネルの読み出し） | `spawn` |
| ブロックする処理や CPU を使う重い処理を実行する | `blocking` |

`resource`・`spawn`・`blocking` は `kivact::prelude` からも使えます。

## データを読み込み、読み込み中・エラー・完了を表示する

`resource(source, fetch)` は、開始時に一度 `fetch` を呼び、`source` が変わるたびにもう一度呼びます。`fetch` の結果は `AsyncState`（`Idle`・`Pending`・`Ready(値)`・`Failed(エラー)`）として扱われます。`dynamic` の中でこれを `match` し、状態ごとの表示を選びます。

```rust,no_run
use std::time::Duration;
use kivact::prelude::*;

fn app() -> impl IntoView {
    let id = signal(1u32);
    let user = resource(id, |id| async move {
        kivact::task::sleep(Duration::from_millis(300)).await;
        if id == 0 {
            Err(UiError::msg("ユーザー 0 は存在しません"))
        } else {
            Ok(format!("ユーザー #{id}"))
        }
    });

    column((
        row((
            button("前へ")
                .disabled(move || id.get() == 0)
                .on_press(move || id.update(|n| *n -= 1)),
            button("次へ").on_press(move || id.update(|n| *n += 1)),
        ))
        .style(style().gap(8)),
        dynamic(
            move || user.state(),
            |state| match state {
                AsyncState::Idle | AsyncState::Pending => text("読み込み中…"),
                AsyncState::Failed(error) => text(format!("エラー: {error}")),
                AsyncState::Ready(name) => text(name),
            },
        ),
    ))
    .style(style().padding(24).gap(12))
}

fn main() -> kivact::Result<()> {
    App::new("Resource").run(app)
}
```

次のルールを押さえておくと、思ったとおりに動きます。

- **source にはプロパティが受け取れるものを渡せます。** `Signal`、`move || (query.get(), page.get())` のようなクロージャ、定数のどれでも構いません。`resource` が追跡するのは source だけです。`fetch` や future の中で読んだシグナルは依存関係に*なりません*。
- **fetch 関数は `Result<T, UiError>` を返します。** `Err(UiError::msg("…"))` は文字列からエラーを作り、`?` は `std::error::Error` を実装する任意のエラー（`io::Error` やパースエラーなど）を `UiError` に変換します。リクエストごとにコピーが渡されるため、`T` も source の値も `Clone` でなければなりません。
- **採用されるのは最新のリクエストだけです。** 処理中に source が変わると、古い future は破棄され、その結果は表示されません。遅れて届いた古い応答が新しい結果を上書きしないよう、自分で守る必要はありません。
- **失敗は panic ではなくデータです。** fetch の失敗は `AsyncState::Failed` に入り、`error_boundary` には届きません。上の例のように自分で表示してください。

Rust のメモ: `async move` は、キャプチャした値を future に move します。future はクロージャの呼び出しより長く生きるためです。シグナルは `Copy` なハンドルなので、move してもコストはかかりません。ほかの場所でも使い続ける `String` は、先に clone する必要があります。

### 更新中も前の結果を表示し続ける

`latest()` は最後に成功した値を返します。新しいリクエストが `Pending` の間や、失敗した後も残るため、画面を空にせず、古いデータの上にスピナーを重ねて見せられます。関心ごとに 1 つずつ、`show` と `show_some` を並べます。

```rust,no_run
# use std::time::Duration;
# use kivact::prelude::*;
fn search() -> impl IntoView {
    let query = signal(String::new());
    let hits = resource(query, |word| async move {
        kivact::task::sleep(Duration::from_millis(200)).await;
        if word.contains('!') {
            return Err(UiError::msg("検索語に ! は使えません"));
        }
        Ok(format!("「{word}」の検索結果"))
    });

    column((
        text_input("検索", query),
        show(move || hits.state().is_pending(), || text("検索中…")),
        show_some(move || hits.error(), |error| {
            text(move || error.get().to_string()).role(Role::Alert)
        }),
        show_some(move || hits.latest(), |latest| text(latest)),
    ))
    .style(style().gap(8).padding(12))
}
# fn main() -> kivact::Result<()> { App::new("Search").run(search) }
```

`Resource` には次のメソッドもあります。

- `refetch()` は現在の source でもう一度取得します（「再読み込み」ボタン向け）。
- `cancel()` は処理中のリクエストを破棄します。状態は、`is_cancelled()` が `true` を返すエラーを持つ `Failed` になり、`latest()` は古い値を保ちます。
- `state().value()` と `state().error()` は、`match` なしで現在の状態の値やエラーを読みます。

## 利用者の操作で処理を始める

`resource` は source が変わったから動き、`action` は `run` を呼んだから動きます。保存、フォームの送信、削除など、副作用のある処理に使います。`AsyncState` も `latest()` も `resource` と同じです。

```rust,test_harness
use std::time::Duration;
use kivact::prelude::*;

fn save_form() -> impl IntoView {
    let name = signal(String::new());
    let save = action(ActionPolicy::Latest, |name: String| async move {
        kivact::task::sleep(Duration::from_millis(300)).await;
        if name.is_empty() {
            Err(UiError::msg("名前が空です"))
        } else {
            Ok(name)
        }
    });

    column((
        text_input("名前", name),
        button("保存")
            .disabled(move || save.state().is_pending())
            .on_press(move || save.run(name.get())),
        dynamic(move || save.state(), |state| match state {
            AsyncState::Idle => text(""),
            AsyncState::Pending => text("保存中…"),
            AsyncState::Ready(name) => text(format!("{name} を保存しました")),
            AsyncState::Failed(error) => text(format!("失敗: {error}")).role(Role::Alert),
        }),
    ))
    .style(style().gap(8).padding(12))
}

fn main() -> kivact::Result<()> {
    App::new("Action").run(save_form)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saving_shows_pending_then_the_result() {
        let screen = kivact::test::render(save_form);
        screen.get_by_role(Role::TextBox, "名前").type_text("Ada");
        screen.get_by_role(Role::Button, "保存").click();
        screen.get_by_text("保存中…");
        assert!(screen.get_by_role(Role::Button, "保存").is_disabled());
        // テストの時計は仮想なので、実際に待つ必要はありません。
        screen.advance(Duration::from_millis(300));
        screen.get_by_text("Ada を保存しました");
    }
}
```

`action` の最初の引数は、前の処理が終わらないうちに新しい `run` が来たときの扱いです（後述）。`ActionPolicy::Latest` は古い処理を取り消します。クロージャは、入力を 1 つ取る形（`|name: String|`）か、入力を取らない形（`|| async { … }`、`save.run(())` で実行）です。Rust は引数の使われ方を見る前にその型を決められないので、入力のメソッドを呼ぶときは型を書いてください。

`resource` と `action` は、イベントハンドラの中ではなく、コンポーネントの組み立て中に作ります。ハンドラの中では、すでにあるハンドルに対して `run`・`refetch`・`cancel` を呼びます。

### 利用者が続けて 2 回クリックしたら

`action` に渡す `ActionPolicy` は、前の処理が終わらないうちに新しい `run` が来たときの扱いを決めます。

| ポリシー | 動き |
|---|---|
| `ActionPolicy::Latest` | 新しい処理をすぐに始め、古い処理を取り消します。 |
| `ActionPolicy::Queue` | 処理を 1 つずつ、`run` の順に実行します。 |
| `ActionPolicy::QueueLatest` | `Queue` と同じですが、新しい `run` は待機中の処理を置き換えます。実行中の処理は中断されません。 |

```rust,no_run
# use kivact::prelude::*;
fn drafts() -> impl IntoView {
    let draft = signal(String::new());
    let save = action(ActionPolicy::QueueLatest, |text: String| async move {
        kivact::task::sleep(std::time::Duration::from_millis(500)).await;
        Ok(text.len())
    });
    save.on_done(|result| println!("saved: {result:?}"));
    column((
        text_input("下書き", draft),
        // 10 回すばやく押すと、最初の保存と最後の保存が実行されます。
        button("下書きを保存").on_press(move || save.run(draft.get())),
    ))
}
# fn main() -> kivact::Result<()> { App::new("Drafts").run(drafts) }
```

ファイルへの書き込みなど、利用者が別の操作に移っても最後まで終えたい処理には、`Queue` か `QueueLatest` を使います。キューに積んだ action は、コンポーネントがなくなった後も最大 3 秒間は処理を続け、ウィンドウを閉じるときも同じ時間だけ完了を待ちます。

`Action` のそのほかのメソッド: `retry()` は直近の実行の入力でやり直し、`cancel()` は実行中の処理を止め、`input()` は現在の状態が表す実行の入力を返し、`on_done(|result| …)` は実行が終わったとき（取り消されたときは除く）に呼ばれるハンドラを登録します。複数登録できます。

## 表示する結果のない future を動かす

`spawn` は、任意の future を UI スレッド上で実行します。future は、`spawn` を呼んだコンポーネントに所有されます。呼び出せる場所は、セットアップ、イベントハンドラ、別のタスクの中です。

```rust,no_run
use std::time::Duration;
use kivact::prelude::*;
use kivact::task::sleep;

fn copy_with_confirmation() -> impl IntoView {
    let copied = signal(false);
    column((
        button("コピー").on_press(move || {
            spawn(async move {
                kivact::platform::copy_text("hello").await?;
                copied.set(true);
                sleep(Duration::from_secs(2)).await;
                copied.set(false);
                Ok::<(), UiError>(())
            });
        }),
        show(copied, || text("コピーしました")),
    ))
}
# fn main() -> kivact::Result<()> { App::new("Spawn").run(copy_with_confirmation) }
```

- **`await` の後でシグナルに直接書き込めます。** future は UI スレッドで動くので、受け渡しの手順はありません。
- **future の戻り値は `()` か `Result<(), UiError>` です。** `Err` は、イベントハンドラが返した `Err` と同じ扱いで、タスクを所有するコンポーネントから見て最も近い `on_error` に届きます。UI の一部を `kivact::view::error_boundary(..).on_error(..)` で囲むと受け取れます。
- `spawn` は `Task` を返します。`cancel()` を呼ぶか `is_finished()` で確かめたいときだけ保持してください。
- `kivact::task::sleep(duration)` はアプリの時計で待ちます（テストでは仮想時計です。後述）。`kivact::task::timeout(duration, future)` は、future が間に合わないと `ErrorKind::Timeout` の `Err(UiError)` を返します。

繰り返し実行する処理は、`sleep` を使ったループで書きます。ループが自分では終わらない場合でも、所有者がなくなれば自動的に止まります。

```rust,no_run
# use std::time::Duration;
# use kivact::prelude::*;
fn countdown() -> impl IntoView {
    let remaining = signal(10u32);
    spawn(async move {
        while remaining.get_untracked() > 0 {
            kivact::task::sleep(Duration::from_secs(1)).await;
            remaining.update(|s| *s -= 1);
        }
    });
    text(move || format!("残り {} 秒", remaining.get()))
}
# fn main() -> kivact::Result<()> { App::new("Countdown").run(countdown) }
```

リソースを一定間隔で更新したいときは、このようなループの中で `refetch()` を呼びます。

## ブロックする処理や重い処理を UI スレッドの外に出す

非同期コードは UI スレッドで動きます。`await` の間は他の処理が進みますが、長い計算、ブロックするファイル読み込み、同期的なネットワーク呼び出し、`std::thread::sleep` は制御を譲りません。戻るまでウィンドウは描画も入力への応答も止まります。そのような処理は `blocking` に入れてください。

```rust,no_run
use kivact::prelude::*;
use kivact::task::CancelToken;

/// `limit` 未満の素数を数えます。`cancel` が true になったら途中でやめます。
fn count_primes(limit: u32, cancel: &CancelToken) -> Option<u32> {
    let mut count = 0;
    for n in 2..limit {
        if n % 4096 == 0 && cancel.is_cancelled() {
            return None;
        }
        if (2..).take_while(|d| d * d <= n).all(|d| n % d != 0) {
            count += 1;
        }
    }
    Some(count)
}

fn primes() -> impl IntoView {
    let limit = signal(200_000u32);
    let result = resource(limit, |limit| async move {
        // ここでの `limit` はただの数値なので、別のスレッドへ move できます。
        let count = blocking(move |cancel: CancelToken| count_primes(limit, &cancel)).await;
        count.ok_or_else(|| UiError::msg("中断されました"))
    });
    column((
        button("上限を 2 倍にする").on_press(move || limit.update(|n| *n *= 2)),
        show(move || result.state().is_pending(), || text("数えています…")),
        show_some(move || result.latest(), |n| text(move || format!("素数は {} 個", n.get()))),
    ))
    .style(style().gap(8).padding(12))
}
# fn main() -> kivact::Result<()> { App::new("Primes").run(primes) }
```

`blocking(f)` は `f` を小さなスレッドプールで実行し、結果の future を返します。resource、action、`spawn` したタスクの中で `await` します。

- **クロージャは `Send` でなければなりません。** 別のスレッドで動くため、`Signal`・`Value` などの UI ハンドルはキャプチャできません（コンパイラが教えてくれます）。値はクロージャの*外*で読み、ただの値を move して渡し（上の `limit`）、結果は `await` の後で書き戻します。
- **クロージャ内の panic** は、それを待っているタスクによって UI スレッドで再開され、アプリを終了させます。
- **取り消しは協調的です。** future が破棄されたとき（新しいリクエストに置き換わった、コンポーネントが破棄された）、クロージャが強制終了されることはありません。代わりに `CancelToken` が true になるので、長い処理はときどき確かめて早めに戻ってください。トークンは引数として受け取ります: `|cancel: CancelToken| …`（型注釈は必須です）。
- プールの大きさの既定値は `min(4, CPU 数)` です。`App::blocking_threads(n)` で変えられます。
- アプリの終了時は、ワーカーごとに最大 2 秒待ち、それでも止まらないジョブは見切りをつけます。

## 別のスレッドから値を受け取る

自分のコードがバックグラウンドで値を作る場合（ゲームやオーディオのエンジン、ファイル監視、独自のランタイム上のネットワーククライアントなど）は、非同期チャンネルで送り、`spawn` したタスクで読みます。`async-channel` が使えます。`Waker` を起こすチャンネルなら何でも動きます。

```rust,no_run
use std::time::Duration;
use kivact::prelude::*;

fn sensor() -> impl IntoView {
    let reading = signal(0u32);
    let (tx, rx) = async_channel::unbounded::<u32>();

    // 自分のスレッドを、セットアップで起動します。
    std::thread::spawn(move || {
        for n in 0.. {
            // `Err` は受信側がいない（コンポーネントが破棄された）ことを表します。
            if tx.send_blocking(n).is_err() {
                break;
            }
            std::thread::sleep(Duration::from_millis(500));
        }
    });

    spawn(async move {
        while let Ok(n) = rx.recv().await {
            reading.set(n);
        }
    });

    text(move || format!("測定値: {}", reading.get()))
}
# fn main() -> kivact::Result<()> { App::new("Sensor").run(sensor) }
```

チャンネルの waker が UI にターンの実行を依頼するため、利用者がウィンドウを触っていなくても値が表示されます。コンポーネントが破棄されるとタスクと `rx` も破棄され、ワーカー側の次の送信が失敗します。これがワーカーに停止を知らせる合図です。ワーカーのループは、送信の失敗で必ず終わらせてください。

UI スレッドでブロックする `recv()` を呼んだり、ワーカーが長く取り続ける `Mutex` を UI スレッドで保持したりしないでください。自前のレンダラーから絵を送り続ける方法は [キャンバスで描く](canvas.md#自前のレンダラーのフレームを表示する) を参照してください。

独自の reactor を持つ future（たとえば `tokio` のタイマーやソケット）は、そのランタイムの中で動くことを前提にしています。そのようなランタイムは専用のスレッドで動かし、結果は上のようにチャンネルで送ってください。

## 何がタスクを取り消すか

タスク、resource のリクエスト、実行中の action は、それを作ったコンポーネント（最も近い*所有者*: `show` の分岐、リストの行、ルート）に属します。UI のその部分が取り除かれると次のようになります。

- future は破棄され、次の `await` の先へ進むことはありません。
- `sleep` のタイマーは解放されます。
- `blocking` ジョブの `CancelToken` は true になります。
- キューに積んだ `Action` は、待機中の処理を 3 秒以内に終えます。

そのため、通常の非同期処理に後始末のコードは要りません。Kivact のタスクではないもの（スレッド、ファイルハンドル）を自分で始めたときは、`on_cleanup(move || …)` で解放してください。

## 待たずに非同期 UI をテストする

テストでは仮想時計を使います。`screen.advance(Duration)` は時計を進め、途中のタイマーをすべて実行します。`screen.find_by_text("…")` は、テキストが現れるまで（仮想時間で）待ちます。上の action の例に両方が出てきます。詳しくは [アプリをテストする](testing.md#時間と非同期処理) を参照してください。

## 関連項目

- [キャンバスで描く](canvas.md): ワーカーから届くフレームの表示。
- [値と要素をアニメーションさせる](animation.md): フレームごとの処理とトランジション。
- [プラットフォームのサービスを使う](platform-services.md): クリップボードやダイアログも future を返します。
- [アプリをテストする](testing.md): 仮想時計の詳細。
- API リファレンス: `cargo doc -p kivact --open` を実行し、`kivact::task` を開いてください。
