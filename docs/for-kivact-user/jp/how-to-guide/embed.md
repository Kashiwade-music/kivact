# プラグインのウィンドウの中に Kivact の UI を開く

[English](../../en/how-to-guide/embed.md)

> **実験的。** `embed` feature は Windows と、X11 の Linux で動作します。主要なプラグインホストでの検証はまだで、macOS と Wayland は未対応です。API は予告なく変わることがあります。

別のプログラムが用意したウィンドウの中に Kivact の UI を表示するためのガイドです。具体的にはオーディオプラグイン (CLAP または VST3) のエディターを想定しています。UI は普通の Kivact のコードのままです。コンポーネント、シグナル、binding、`kivact::test` をそのまま使えます。ウィンドウの中身と UI は Kivact が担当し、音声、パラメーター、保存、プラグイン形式との橋渡しはプラグイン側が担当します。

作業は通常、二人で分担します。

| 役割 | 書くもの | このガイドで読む箇所 |
|---|---|---|
| **UI の作者** | コンポーネントとしてのエディター画面。 | [UI を書く](#ui-を書く-ui-の作者) |
| **wrapper の作者** | ホストのウィンドウにエディターを開き、閉じるプラグイン側の接着コード。 | [エディターを開く](#エディターを開く-wrapper-の作者) |

Kivact はプラグインフレームワーク (nih-plug、clack など) 用のアダプターを提供しません。wrapper の作者がフレームワークのウィンドウハンドルを `Parent` に変換し、`App::embed` を呼びます。

## feature を有効にする

```toml
[dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact", features = ["embed"] }
```

既定の `gpu` feature はそのままです。埋め込みウィンドウも、デスクトップのウィンドウと同じく GPU を使い、自動で CPU に切り替わります。

## UI を書く (UI の作者)

エディターは、デスクトップアプリのルートと同じく、ビューを返す関数です。ホストのウィンドウ、プラグイン形式、親のハンドルは何も登場しません。通常どおり書いて `kivact::test::render` でテストし、開発中は `App::run` で普通のウィンドウとして動かします。ホストへの配置は、wrapper の作者の `App::embed` が行います。

デスクトップアプリと違う規則が 2 つあります。

- **プラグインの状態は UI の外に持つ。** エディターはホストが開くたびに作られ、閉じるたびに破棄されます。パラメーター、プリセット、残すべきものはプラグインが持ちます。エディターにはその状態の `Arc` を渡します。選択、ホバー、スクロール位置はシグナルに持てますが、開くたびに最初からになります。
- **ホストの GUI スレッドを待たない。** エディターのコードは App の UI スレッドで動きます。ホストのスレッドと同じとは限りません。ホストを待つコールバックは、開閉時にデッドロックの原因になります。

### パラメーターを読み書きする

エディターに必要なものを trait として定義します。wrapper がプラグイン形式の呼び出しで実装し、テストは偽物で実装します。

```rust,test_harness
use std::sync::Arc;

use kivact::interaction::EditPhase;
use kivact::motion::{FrameRate, frame_loop};
use kivact::prelude::*;

/// エディターがプラグインに求めるもの。wrapper が実装します。
pub trait Params: Send + Sync + 'static {
    fn gain(&self) -> f64;
    fn begin_gesture(&self);
    fn set_gain(&self, value: f64);
    fn end_gesture(&self);
}

pub fn editor(params: Arc<dyn Params>) -> impl IntoView {
    let value = signal(params.gain());

    // ホストによる変更 (オートメーション、プリセット) を、表示フレームごとに追いかけます。
    let source = params.clone();
    frame_loop(move |_| value.set(source.gain())).start(FrameRate::Display);

    // 書き込みはまずプラグインへ、次に画面が読むシグナルへ行きます。
    let sink = params.clone();
    let gain = binding(
        move || value.get(),
        move |next: f64| {
            sink.set_gain(next);
            value.set(next);
        },
    );

    // ドラッグ 1 回、キー操作 1 回が、ホストの undo やオートメーションにとって 1 つのジェスチャーです。
    let gestures = params;
    column((
        slider("Gain", gain)
            .range(0.0..=1.0)
            .step(0.1)
            .on_edit(move |phase| match phase {
                EditPhase::Begin => gestures.begin_gesture(),
                EditPhase::Commit | EditPhase::Cancel => gestures.end_gesture(),
                // 今後追加される variant は ジェスチャーの終了 として扱います。
                _ => gestures.end_gesture(),
            }),
        text(move || format!("Gain: {:.2}", value.get())),
    ))
    .style(style().gap(8).padding(16))
}

fn main() {}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use std::time::Duration;

    use super::*;

    /// プラグインの代役。エディターが行ったことを記録します。
    #[derive(Default)]
    struct Fake {
        gain: Mutex<f64>,
        log: Mutex<Vec<String>>,
    }

    impl Params for Fake {
        fn gain(&self) -> f64 {
            *self.gain.lock().unwrap()
        }
        fn begin_gesture(&self) {
            self.log.lock().unwrap().push("begin".into());
        }
        fn set_gain(&self, value: f64) {
            *self.gain.lock().unwrap() = value;
            self.log.lock().unwrap().push(format!("set {value:.1}"));
        }
        fn end_gesture(&self) {
            self.log.lock().unwrap().push("end".into());
        }
    }

    #[test]
    fn the_editor_writes_gestures_and_follows_the_host() {
        let fake = Arc::new(Fake::default());
        let params = fake.clone();
        let screen = kivact::test::render(move || editor(params));

        screen.get_by_role(Role::Slider, "Gain").press_key(Key::ArrowRight);
        screen.get_by_text("Gain: 0.10");
        assert_eq!(*fake.log.lock().unwrap(), ["begin", "set 0.1", "end"]);

        // ホストがパラメーターをオートメーションで動かすと、エディターは次のフレームで追いつきます。
        *fake.gain.lock().unwrap() = 0.5;
        screen.advance(Duration::from_millis(100));
        screen.get_by_text("Gain: 0.50");
    }
}
```

呼び出しの順序は `Begin`、1 回以上の書き込み、`Commit` です。ユーザーが Escape を押したりポインターが取り消されたりした場合は、まず値が開始時に書き戻され、そのあとに `Cancel` が届きます。したがって `Cancel` ではジェスチャーを終えるだけにして、値をもう一度書き込まないでください。ホスト側の変更をきっかけにジェスチャーを始めてもいけません。

### ポーリングではなく通知で受け取る

プラグインがチャンネルなどで通知できるなら、タスクで受け取ります。タスクはコンポーネントに属し、エディターが閉じると取り消されます。`frame_loop` でのポーリングは毎フレーム 1 回読みますが、チャンネルなら変化がないあいだ費用がかかりません。

```rust,no_run
use kivact::prelude::*;

fn gain_label(incoming: async_channel::Receiver<f64>) -> impl IntoView {
    let value = signal(0.0);
    spawn(async move {
        while let Ok(next) = incoming.recv().await {
            value.set(next);
        }
    });
    text(move || format!("Gain: {:.2}", value.get()))
}
```

非同期の受信側を持つチャンネルなら何でも使えます。ここでの `async_channel` は例にすぎません。オーディオスレッドを UI で待たせてはいけません。有界または lock-free のキューに送るのは wrapper の仕事です。

### デスクトップのウィンドウとの違い

- **タイトルとフルスクリーンは効かない。** ウィンドウはホストが持ちます。`App::title` は無視され、`platform::set_fullscreen` は未対応として失敗します。
- **サイズ。** 決めるのはホストです。App の `window_size` は最初のサイズにすぎません。`min_window_size` が制限するのは自分の `platform::request_window_size` の要求であり、ホストが与えるサイズではありません。ホストに別のサイズを求める方法は[サイズ変更](#両方向のサイズ変更)を見てください。wrapper の協力がなければ、要求は `Unsupported` で失敗します。
- **閉じる操作。** 反応すべき閉じるボタンはありません。ホストがウィンドウを閉じたとき、wrapper がエディターを drop します。
- **キー。** エディターが受け取るのは、ホストのウィンドウが渡すキーです。エディターが処理しなかったキーはホストに返されず、エディターに届く前に Space などを奪うホストへの対策も Kivact にはありません。
- **パニックでエディターが止まる。** パニックはエディターを終了させ、wrapper が次に `Embedded` を呼んだときに再開されます。ホストまで届かせないでください。想定できる失敗は `error_boundary` で処理してください ([エラーや更新の原因を調べる](diagnostics.md))。

## エディターを開く (wrapper の作者)

wrapper は、プラグイン形式の呼び出しを Kivact の呼び出しに変換します。入り口は `App::embed(parent, root)` で、`Embedded` ハンドルを返します。ハンドルを drop するとエディターが閉じます。

### 親を作る

`Parent` は生のハンドル値とスケールを保存するだけです。作った時点ではハンドルを読みません。検査するのは `App::embed` です。

| ホストが渡すもの | 構築 |
|---|---|
| CLAP `clap_window.win32`、VST3 `"HWND"`、nih-plug `Win32Hwnd` | `Parent::win32(hwnd)` |
| CLAP `clap_window.x11`、VST3 `"X11EmbedWindowID"`、nih-plug `X11Window` | `Parent::x11(id)` |

```rust,no_run
use std::ffi::c_void;

use kivact::embed::Parent;

fn windows_parent(hwnd: *mut c_void, scale: f64) -> Parent {
    Parent::win32(hwnd).scale(scale)
}

fn x11_parent(window: u32, scale: f64) -> Parent {
    Parent::x11(window).scale(scale)
}
```

ホストがスケールを伝える場合 (CLAP の `set_scale`、VST3 の `setContentScaleFactor`) は、`.scale(..)` で渡します。渡さなければ OS のスケールが使われます。自分のスケールでサイズを計算した場合は、Kivact にも同じスケールを渡してください。そうしないとサイズが食い違います。

### 開く

```rust,no_run
use std::sync::Arc;

use kivact::embed::{Embedded, Parent};
use kivact::prelude::*;

# pub trait Params: Send + Sync + 'static {}
# fn editor(_: Arc<dyn Params>) -> impl IntoView { text("Editor") }
fn open(parent: Parent, params: Arc<dyn Params>) -> kivact::Result<Embedded> {
    App::new("Editor")
        .window_size(640, 480)
        .min_window_size(320, 240)
        .embed(parent, move || editor(params))
}
```

`window_size` は**論理ピクセル**で、スケールを掛けたものが最初の物理サイズになります。指定しなければ既定は 960 x 720 です。

`embed` の契約です。

- **スレッド。** ホストの GUI スレッドで呼びます。root クロージャーは `Send` でなければならず、App の UI スレッドで実行されます。root は一度だけ実行され、`embed` はその完了後に戻ります。
- **エラー。** 不正なサイズやスケールは `ErrorKind::Message` を返します。不正または null の親、OS 資源の不足、未対応の OS は `ErrorKind::Platform` を返します。root でのパニックは `embed` から再開されます。開くのに失敗したときはウィンドウが残りません。
- **ハンドル。** `Embedded` は `Send + Sync` です。エディターが開いているあいだ保持してください。複数のスレッドから setter を呼ぶなら `Arc` に入れて持ちます。

### ホストの呼び出しに応える

setter は UI スレッドへ変更を積んですぐ戻り、getter は最後に適用された値を返します。この境界でのサイズはすべて**物理ピクセル**で、スケールは物理ピクセル / 論理ピクセルです。

| ホストの呼び出し (CLAP / VST3) | wrapper がすること |
|---|---|
| 開く前の `set_scale` / `setContentScaleFactor` | スケールを保存する。 |
| 開く前の `get_size` / `getSize` | `round(論理サイズ * スケール)` を返す。 |
| `set_parent` / `attached` | 同じスケールを `Parent` に渡して `App::embed` を呼び、ハンドルを保持する。 |
| `set_size` / `onSize` | `Embedded::set_size(width, height)` |
| 開いた後の `get_size` / `getSize` | `Embedded::size()` |
| スケールの変更 | `Embedded::set_scale(scale)`。論理サイズを保ちたいなら、そのあとで新しい物理サイズをホストに求める。 |
| `adjust_size` / `checkSizeConstraint` | wrapper 自身の制限で丸める。ホストの `set_size` は `min_window_size` では制限されません。 |
| `destroy` / `removed` | ハンドルを drop する。 |

```rust,no_run
use kivact::embed::Embedded;

/// wrapper のうち、開いているエディターを保持する部分。
pub struct OpenEditor {
    embedded: Option<Embedded>,
}

impl OpenEditor {
    pub fn set_size(&self, width: u32, height: u32) {
        if let Some(editor) = &self.embedded {
            editor.set_size(width, height);
        }
    }

    pub fn size(&self) -> Option<(u32, u32)> {
        self.embedded.as_ref().map(Embedded::size)
    }

    pub fn set_scale(&self, scale: f64) {
        if let Some(editor) = &self.embedded {
            editor.set_scale(scale);
        }
    }

    /// エディターが止まった場合の理由 (プラットフォームの失敗)。
    pub fn failure(&self) -> Option<kivact::UiError> {
        self.embedded.as_ref().and_then(Embedded::error)
    }

    pub fn close(&mut self) {
        // drop するとエディターが閉じ、後始末の完了を待ちます。
        self.embedded = None;
    }
}
```

サイズが 0 の場合、プラットフォームの上限 (Windows は `i32::MAX`、X11 は `u16::MAX`) を超える場合、スケールが有限の正の数でない場合は、`tracing` に警告を出して無視されます。エディターは動き続けます。

### 両方向のサイズ変更

ユーザーやホストがウィンドウのサイズを変えたときは `set_size` を呼びます。UI が別のサイズを望むときは、**論理**サイズで `platform::request_window_size` を呼びます。要求は `Parent::on_resize_request` を通って、**物理**ピクセルで、UI スレッド上の wrapper に届きます。ホストが受理したら `true` を返してください。すると Kivact はウィンドウをリサイズします。ただしその間にホストが `set_size` を呼んでいれば、そちらが優先されます。`false` を返すと、UI の要求はプラットフォームエラーで失敗します。コールバックがなければ、要求は `Unsupported` で失敗します。

```rust,no_run
use kivact::embed::Parent;

fn resizable(parent: Parent) -> Parent {
    parent.on_resize_request(|width, height| {
        // ホストに width x height の物理ピクセルを求め、受理されたかを返します。
        // これは UI スレッドで動きます。要求はホスト自身の run loop へ送り、
        // GUI スレッドの返事は待たずに答えてください。
        let _ = (width, height);
        true
    })
}

// UI 側では、タスクや effect の中で:
async fn fit_content() -> kivact::Result<()> {
    kivact::platform::request_window_size(800, 600).await
}
```

wrapper が自分のサイズを答えるときの順序は 1 つです。**新しい論理サイズを保存し、ホストに求め、拒否されたら元のサイズに戻す。** フレームワークによっては (nih-plug)、要求の実行中にエディターのサイズを読み直します。

スケールが変わっても論理サイズを保ちたいときは、先に `set_scale` を呼び、そのあとで `round(論理サイズ * 新しいスケール)` をホストに求めます。

### 閉じる

`Embedded` はホストの GUI スレッドで drop します。App のコールバックの外では、drop は App とネイティブウィンドウの後始末が終わるのを待ちます。App が所有するスレッドごとに最大 2 秒です。その上限を超えた処理は drop が戻ったあとも続き、ホストがプラグインを安全にアンロードできるよう、Kivact が自分のコードをメモリに保持します。モジュールを自分で保持する必要はありません。自分で起動したスレッドの停止と join は、自分の責任です。

エディターが App のコールバックの中 (たとえば「閉じる」ボタン) で自分のハンドルを drop した場合、実行中のターンは最後まで進み、そのあとに後始末が行われ、以後の入力は配送されません。キューにあるアクションは消化されません。

ホストが先に親ウィンドウを破棄した場合、エディターは安全に停止し、`error()` は `None` のままです。プラットフォームの失敗は App を止め、ウィンドウを破棄し、`error()` が理由を返します。パニックも同じように App を止めますが、`error()` は返さず、次の `Embedded` の呼び出しがそれを再開します。ハンドルはその後でも drop する必要があります。長く走る `blocking` の仕事は `CancelToken` を定期的に確認して、閉じるときにその終了を待たなくて済むようにしてください。

### wrapper をテストする

`test` feature を有効にすると、`kivact::test::TestParent` が本物の親ウィンドウを開きます。埋め込み、メッセージループの実行、エディターが表示した内容のキャプチャを試せます。Windows と Linux で使えます。

```rust,no_run
# #[cfg(any(windows, target_os = "linux"))]
# fn example() -> kivact::Result<()> {
use std::time::Duration;

use kivact::embed::Embedded;
use kivact::prelude::*;
use kivact::test::TestParent;

let parent = TestParent::open(320, 240)?;
let editor: Embedded = App::new("Editor")
    .window_size(320, 240)
    .embed(parent.parent().scale(1.0), || text("Editor"))?;
parent.pump(Duration::from_millis(50));
assert_eq!(parent.capture()?.width, 320);
assert!(editor.error().is_none());
drop(editor);
# Ok(())
# }
```

## 対応状況

| 機能 | Windows | Linux X11 |
|---|---|---|
| 描画 (GPU、CPU に切り替え)、マウス、ホイール、キーボード | あり | あり (`DISPLAY` が必要。XWayland でも動作) |
| IME | あり。再変換も可 | XIM。入力メソッドが動いていなければ通常のキーボード入力。再変換は不可 |
| 読み上げソフト | あり | あり (AT-SPI) |
| クリップボード | あり | あり |
| ファイル・フォルダーのダイアログ | あり | あり (子ウィンドウをダイアログの親にする) |
| エディターへのファイルのドロップ | あり。ドロップのみ (ホバーや取消しなし) | あり (XDND)。ドロップのみ |
| タッチとペン | なし | なし |
| フルスクリーン、ウィンドウタイトル | 効かない | 効かない |
| 最大サイズ (物理) | `i32::MAX` | `u16::MAX` |
| macOS (NSView) | 未対応 | |
| Wayland (ネイティブの埋め込み) | | 未対応 |
| 未処理のキーをホストへ返す | 未対応 | 未対応 |

実際のプラグインホスト (REAPER、Bitwig Studio、Ableton Live、Cubase など)、実際の DAW での IME、プラグインウィンドウ内での読み上げソフトは、まだ検証していません。

## 関連項目

- [独自の見た目のコントロールを作る](custom-controls.md): `EditPhase` と、ジェスチャーを報告するコントロール。
- [コンポーネントから非同期処理を実行する](async-and-tasks.md): `spawn` と `blocking`。
- [エラーや更新の原因を調べる](diagnostics.md): パニックが App を終了させるしくみ。
- `cargo doc -p kivact --features embed --open`: `kivact::embed`。
