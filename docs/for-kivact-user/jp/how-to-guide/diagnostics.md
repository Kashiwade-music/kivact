# エラーや更新の原因を調べる

[English](../../en/how-to-guide/diagnostics.md)

ウィンドウが開かない、ボタンを押しても何も起きない、エラーが消えてしまったように見える、値が変わったのに誰が変えたのか分からない。そんなときに使うガイドです。入門チュートリアルを読んでいることを前提にしています。

## 最短の方法: tracing の出力を有効にする

Kivact は、実行中に捕まえた問題をすべて [`tracing`](https://docs.rs/tracing) クレート経由で報告します。ただし Kivact 自身は何も表示しません。subscriber を入れるまでは、失敗したハンドラーや誤用されたシグナルのエラーは画面にもコンソールにも出ません。アプリケーションに subscriber を追加してください。

```toml
[dependencies]
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

```text
fn main() -> kivact::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    App::new("Demo").run(app)
}
```

`tracing-subscriber` は Kivact の依存ではなくアプリケーション側の依存なので、好みの subscriber を選べます。`EnvFilter` を使うと、起動時に表示する内容を選べます。

```text
RUST_LOG=kivact=debug cargo run              # bash
$env:RUST_LOG = "kivact=debug"; cargo run    # PowerShell
```

Kivact が出力するものはすべて target が `kivact` です。ただしレイアウトの lint だけは `kivact::layout` です。

| レベル | target | 内容 |
|---|---|---|
| `error` | `kivact` | エラーが既定の出力先 (default sink) に届いた。誰も処理しなかったということです (後述)。エラーの種類とソースの位置が出ます。 |
| `warn` | `kivact` | プラットフォーム側の問題。たとえばウィンドウより長く生きたバックグラウンドスレッドなど。 |
| `warn` | `kivact::layout` | レイアウトの lint。テキストのはみ出し、折り返し、切り取り (デバッグビルドのみ)。 |
| `info` | `kivact` | プラットフォームにできないこと。起動時に一度だけ (`the platform lacks: ...`)。 |
| `debug` | `kivact` | 失敗にはせず無視した誤用。破棄済みシグナルへの書き込みなど。 |
| `trace` | `kivact` | ターンごとに 1 行 (イベント数、書き込み、再実行、ウィンドウへの書き込み) と、そのターンのメモ。 |

最初は既定 (エラーのみ) で動かし、1 つの問題を追うときだけ `kivact` のレベルを上げてください。

## `App::run` が返すもの

`App::run` はウィンドウが閉じると戻ります。`Err` を返すのは次の 2 つの場合だけで、どちらも UI が作られる前です。

- `ErrorKind::Message`: ウィンドウの設定が不正。サイズが 0 だったり、`min_window_size` が `window_size` より大きかったりする場合です。
- `ErrorKind::Platform`: ホストを起動できなかった。たとえばウィンドウを開けなかった場合です。

どのエラーにもメッセージがあり、分かる場合は原因になったコードの位置も持っています。

```rust,no_run
use kivact::ErrorKind;
use kivact::prelude::*;

fn main() {
    let result = App::new("Demo").window_size(800, 600).run(|| text("Hello"));
    if let Err(error) = result {
        match error.kind() {
            ErrorKind::Message => eprintln!("ウィンドウの設定が不正です: {error}"),
            ErrorKind::Platform => eprintln!("ウィンドウを開けませんでした: {error}"),
            _ => eprintln!("{error:?}"),
        }
        if let Some(at) = error.location() {
            eprintln!("発生位置: {at}");
        }
        std::process::exit(1);
    }
}
```

`ErrorKind` は `#[non_exhaustive]` なので、`match` には `_` の腕が必要です。`main` から `kivact::Result<()>` を返す場合は、Rust がエラーを `Debug` 形式 (`UiError(Message: ... at src/main.rs:12)`) で表示し、終了コード 1 で終わります。

実行中のアプリの内部で起きたエラーが `run` を `Err` で終わらせることはありません。次の節のとおりに扱われます。

## コールバック内のエラーはどこへ行くか

Kivact はコンポーネントの setup、イベントハンドラー (`on_press`、`on_key` など)、`effect`、タスク、クリーンアップ、`on_mount` であなたのコードを実行します。これらはそれぞれ境界になっていて、失敗のしかたによって扱いが変わります。

- **ハンドラーが `Err` を返す。** ハンドラーは `()` か `Result<(), UiError>` を返せるので、あらゆる `std::error::Error` に `?` が使えます。エラーは `?` の位置とともに報告されます。
- **コードがパニックする。** パニックはバグです。Kivact は捕まえません。Rust の通常のメッセージとともに、アプリが終了します (テストなら失敗します)。
- **Kivact の関数を誤用する。** ハンドラーの中で `signal` を呼ぶ、`memo` の中でシグナルに書き込む、所有者が破棄されたあとのハンドルを使う、などもバグです。Kivact は呼び出しの位置で `kivact misuse: UiError(<kind>: ... at file:line)` というメッセージとともにパニックします (下の表)。

報告されたエラーは、失敗したコードより上にある一番近い `error_boundary` に渡されます。境界は `on_error`、`fallback`、またはその両方を実行します。境界がなければ、エラーは**既定の出力先**に届きます。`tracing::error!` の 1 行と、直近 256 件のエラーを持つリングバッファです。アプリは止まりません。

```rust,test_harness
use kivact::ErrorKind;
use kivact::prelude::*;
use kivact::view::error_boundary;

fn export() -> Result<(), UiError> {
    // `?` はあらゆる `std::error::Error` を `UiError` に変換します。
    std::fs::read_to_string("does-not-exist.txt")?;
    Ok(())
}

fn app() -> impl IntoView {
    column((
        // 境界なし: エラーは既定の出力先へ行きます。
        button("Export").on_press(export),
        // fallback つきの境界は、子を差し替えて再試行を用意します。
        error_boundary(|| {
            button("Risky").on_press(|| -> Result<(), UiError> {
                Err(UiError::msg("the disk is full"))
            })
        })
        .fallback(|error, reset| {
            column((
                text(format!("Failed: {error}")),
                button("Retry").on_press(move || reset.run()),
            ))
        }),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Errors").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_reach_the_sink_or_the_boundary() {
        let screen = kivact::test::render(app);

        screen.get_by_role(Role::Button, "Export").click();
        let errors = screen.errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].kind(), &ErrorKind::External);
        assert!(errors[0].location().is_some());

        screen.get_by_role(Role::Button, "Risky").click();
        screen.get_by_text("Failed: the disk is full");
        // 境界が処理したので、出力先に 2 つ目のエラーは届きません。
        assert_eq!(screen.errors().len(), 1);

        screen.get_by_role(Role::Button, "Retry").click();
        screen.get_by_role(Role::Button, "Risky");
    }
}
```

知っておくと役に立つ点が 3 つあります。

- fallback の `reset` 関数は子を新しく作り直します。そのため境界の内側で作った状態は最初からやり直しになります。
- `on_error` だけの境界は、子を画面に残したままあなたの関数を呼びます。どちらのハンドラーも持たない境界は、エラーを親の境界へ渡します。
- 画像やフォントの失敗は障害ではなく診断です。境界の内側でも既定の出力先 (`ErrorKind::Asset`) へ行き、そのノードは何も描かず、アプリの残りは動き続けます。


## 実行中のアプリで直近のエラーを表示する

`kivact::diagnostics::errors()` は、既定の出力先のリングバッファにあるエラーを古い順に返します。テストの `Screen::errors()` も同じリングバッファを読みます。結果はその時点の写しで、何も購読しません。「問題を報告」ダイアログを開くときのように、必要になったときに読んでください。

```rust,no_run
use kivact::prelude::*;

fn report_button() -> impl IntoView {
    let summary = signal(String::new());
    column((
        button("問題を報告").on_press(move || {
            let lines: Vec<String> = kivact::diagnostics::errors()
                .iter()
                .map(|error| format!("{:?}: {error}", error.kind()))
                .collect();
            summary.set(lines.join("; "));
        }),
        text(move || summary.get()),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Report").run(report_button)
}
```

アプリのコードの外 (そのスレッドで App が動いていないとき) では、一覧は空です。

## よくあるエラーと対処

`error.kind()` で分岐するか、tracing の行を読んでください。出会うことが多い種類は次のとおりです。

| 種類 | 原因 | 対処 |
|---|---|---|
| `SetupOnly { api }` | `signal`、`memo`、`effect`、`callback` など setup 用の関数を、ハンドラー、タスク、メモの中から呼んだ。 | ハンドルはコンポーネントの setup で作り、ハンドラーではそのハンドルを使います。 |
| `WriteInPureContext` | `memo` の中や、プロパティを計算するクロージャー (`text(move \|\| ...)`) の中でシグナルに書き込んだ。 | 書き込まずに値を導出します。副作用が必要なら `effect` を使います。 |
| `BorrowedWrite` | シグナル自身の `with` や `update` のクロージャーの中で、そのシグナルに書き込んだ (または読んだ)。 | `with` から判断だけを返し、戻ってから書き込みます。 |
| `Disposed { .. }` | ハンドルを作ったコンポーネントが消えたあとに、そのハンドルを使った。エラーにはハンドルと所有者が作られた位置が入ります。 | コンポーネントより長生きする場所にハンドルを保持しないか、`try_get` で読みます。 |
| `WrongApp` | 別の App、またはすでに存在しない App のハンドルを使った。 | ハンドルは、それを作った App の中だけで使います。 |
| `Cycle { path }` | effect や memo が互いに依存して循環している。`path` に作成位置が並びます。 | 循環を断ちます。片方を `get_untracked` で読むか、2 つを 1 つにまとめます。 |
| `DuplicateKey` | `for_each` の 2 行が同じキーを返した。あとの行は捨てられます。 | 項目ごとに一意のキーを返します。 |
| `Unsupported` | 設定がそのノードの種類やプラットフォームに適用できない。メッセージにその設定が書かれ、ノードはその設定なしで mount されます。 | 設定を外すか、適用できるノードを使います。 |
| `Platform` | クリップボード、ダイアログ、フォーカスの要求が失敗した。 | 呼び出し側で失敗を扱います。システムによっては普通に起こる結果です。 |
| `Asset` | 画像やフォントを使えなかった。メッセージにファイルと理由が書かれます。 | パスや形式を直します。 |

誤用の種類 (`SetupOnly`、`WriteInPureContext`、`BorrowedWrite`、`Disposed`、`WrongApp`、memo の `Cycle`) は、このパニックのメッセージに現れます。`try_*` 関数は `Disposed` と `WrongApp` を、パニックではなく `Err` として返します。収束しない effect は `Cycle` を出力先に報告します。

失敗にはせず無視されるものが 1 つあり、`debug` レベルにだけ出ます。存在しなくなったシグナルへの `signal.set(..)` は何もせず、その書き込みはターンのメモとして記録されます (`ErrorKind::WriteAfterDispose`)。シグナルのハンドルをスレッドやグローバルに持ち出し、コンポーネントが消えたあとに書き込むと起きます。(そのようなシグナルへの `signal.update(..)` は、クロージャーに渡す値がないため、同じ種類のエラーとして失敗します。)

## デバッグビルドで増える検査

デバッグビルド (`--release` なしの `cargo run`) では検査が増えます。リリースビルドでは費用がかかりません。

- ロールを持たずフォーカスも受けられないノードの `on_press` は、`Unsupported` として一度だけ報告されます。キーボードや読み上げソフトの利用者が届かないためです。`button(..)` を使うか、`.role(..)` と `.focusable(true)` を足してください。
- テーマのアイコンセットにない名前の `icon("name")` は `Asset` として報告され、何も描きません。
- レイアウトの lint は target `kivact::layout` で警告します。対象は、テキストが箱からはみ出す、ボタンのような 1 行のコントロールで折り返す、中央に置かれるはずの箱の中心からずれる、省略記号なしで切り取られる場合です。警告の行にはノードのソース位置が入ります。
- 同じプロパティを 2 回設定したスタイルは、ターンのメモに記録されます (最後の値が勝ちます)。
- 消えた項目に対する `Signal::item` のバインディングへの書き込みは、`WriteAfterDispose` の診断として記録されます。

## 値が更新された理由を調べる

テキストが想定外の値を表示している、変わるはずのないものが変わる、という場合はノードに聞いてください。`kivact::diagnostics::why` は `NodeRef` を受け取り、動的なプロパティごとに、どの effect がそれを設定しているか、その effect がどのシグナルとメモを読んだか、そしてそれぞれの作成位置と最後の書き込み位置を説明します。値そのものは記録されず、型とソース位置だけです。

```rust,no_run
use kivact::diagnostics::why;
use kivact::prelude::*;

fn app() -> impl IntoView {
    let count = signal(0);
    let doubled = memo(move || count.get() * 2);
    let label = node_ref();

    column((
        text(move || format!("doubled: {}", doubled.get())).node_ref(label),
        button("+1").on_press(move || count.update(|n| *n += 1)),
        button("Why?").on_press(move || {
            // ノードが mount されていないときは `None` です。
            if let Some(explanation) = why(&label) {
                eprintln!("{explanation}");
            }
        }),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Why").run(app)
}
```

`+1` を 2 回押してから `Why?` を押すと、次のような報告が出ます。

```text
node 5 Text "doubled: 4"
  Content set by the effect created at src/main.rs:10:9
    reads memo i32 created at src/main.rs:6:19
      reads signal i32 created at src/main.rs:5:17, last written at src/main.rs:12:48 (turn 2)
```

上から読みます。テキストの `Content` は 1 つの effect が設定しています。その effect はメモを読み、メモはシグナルを読んでいて、そのシグナルを最後に書いたのは、たぶんあなたが見たかった行です。変化しないプロパティは出ません (`all props are static`)。effect に結びついていないノードは、`show` の分岐やリストの行のように、ノードごと作り直される仕組みでしか更新されません。

テストでは `NodeRef` は不要です。見つけた要素に `Element::why()` を呼べば同じ情報が得られます。

```rust,test_harness
use kivact::prelude::*;

fn app() -> impl IntoView {
    let count = signal(0);
    let doubled = memo(move || count.get() * 2);
    column((
        text(move || format!("doubled: {}", doubled.get())).test_id("doubled"),
        button("+1").on_press(move || count.update(|n| *n += 1)),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Why").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_text_depends_on_the_memo_which_depends_on_the_signal() {
        let screen = kivact::test::render(app);
        screen.get_by_role(Role::Button, "+1").click();
        screen.get_by_role(Role::Button, "+1").click();

        let why = screen.get_by_test_id("doubled").why();
        let content = &why.props[0];
        assert_eq!(content.prop, "Content");
        assert_eq!(content.reads[0].kind, "memo");
        let signal = &content.reads[0].reads[0];
        assert_eq!((signal.kind, signal.ty), ("signal", "i32"));
        assert!(signal.last_write.is_some());
    }
}
```

## ターンの中で何が起きたかを読む

Kivact は入力をターン単位で処理します。イベントが届き、ハンドラーがシグナルに書き込み、effect と memo が再実行され、変更がウィンドウへ送られます。テストでは `Screen::trace()` が直近のターンを `TurnRecord` として返します。ターンのきっかけ (`causes`)、値を変えたすべての書き込み (`writes`。型と書き込んだ行)、再実行された memo と effect (`reruns`)、ウィンドウへ送られた変更の数 (`ops`)、所要時間です。クリック 1 回で想定より多くの更新が起きるときは、書き込みと再実行を見れば、どのシグナルが発端か分かります。

```rust,test_harness
use kivact::prelude::*;

fn app() -> impl IntoView {
    let count = signal(0);
    column((
        text(move || format!("count: {}", count.get())),
        button("+1").on_press(move || count.update(|n| *n += 1)),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Trace").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_click_writes_the_counter_once() {
        let screen = kivact::test::render(app);
        screen.get_by_role(Role::Button, "+1").click();

        let turns = screen.trace();
        let click = turns
            .iter()
            .find(|turn| turn.causes.iter().any(|cause| cause.starts_with("Activate")))
            .expect("the click started a turn");
        let writes: Vec<_> = click.writes.iter().filter(|w| w.ty == "i32").collect();
        assert_eq!(writes.len(), 1);
        assert!(click.reruns.iter().any(|run| run.kind == "render"));
        assert!(screen.errors().is_empty());
    }
}
```

同じターンの記録は `trace` レベルで tracing にも書かれます。実行中のアプリでも `RUST_LOG=kivact=trace` を指定すれば見られます。

`Screen::stats()` はノード、所有者、タイマーの数を返します。これは Kivact 自体の変更のためのものです。アプリのテストには `errors()`、`trace()`、各種クエリを使い、カウンターは使わないでください。

## 「何も起きない」の裏にある典型的なミス

- **テキストが更新されない。** `text(count.get())` は setup の間にシグナルを一度だけ読み、そのスナップショットを表示します。クロージャー `text(move || count.get())` か、シグナルやメモそのものを渡してください。
- **コンポーネントの動きが二重になる。** effect でシグナルを別のシグナルへコピーする (`effect(move || other.set(count.get()))`) と、真実の源が 2 つになります。2 つ目の値は `memo` で導出してください。
- **読んでいる最中の書き込みが失敗する。** `signal.with(|v| signal.set(..))` は借用の中で書き込んでいます。`with` の中で新しい値を計算して返し、そのあとで書き込んでください。
- **エラーが一向に表示されない。** subscriber が入っていません。最初の節を見てください。

## 関連項目

- [アプリをテストする](testing.md): `Screen`、クエリ、レイアウトの問題でテストを失敗させるレイアウト検査。
- [再利用できるコンポーネントを書く](custom-components.md): 自分のコンポーネントの誤用を報告する方法。
- `cargo doc -p kivact --open`: `kivact::diagnostics` (`ErrorKind`、`why`、`TurnRecord`) と `kivact::test`。
