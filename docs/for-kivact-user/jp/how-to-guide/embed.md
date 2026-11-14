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

use kivact::motion::FrameRate;
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
- **閉じる操作。** 反応すべき閉じるボタンはありません。ホストがウィンドウを閉じたとき、wrapper がエディターを drop します。埋め込みの App では `AppHandle::close` は `ErrorKind::Unsupported` を返します。代わりに `Embedded` ハンドルを drop してください。
- **ウィンドウは 1 つ。** エディターが入る 1 つのウィンドウはホストが持ちます。`platform::capabilities().get().multi_window` は false で、`window` の宣言は `ErrorKind::Unsupported` を報告して開きません（[ウィンドウが使えない環境](windows.md#ウィンドウが使えない環境) を参照）。
- **入力の注入はできない。** DevTools はエディターを調べられますが、`inject_input` は `UNSUPPORTED`（-32015）で断られます。embed の port は自分の入力状態を持ちます。
- **キー。** エディターが受け取るのは、ホストのウィンドウが渡すキーです。既定では、エディターが処理しなかったキーはエディターに残ります。`Parent::forward_unhandled_keys`（[キーとホスト](#キーとホスト) を参照）を付けると、そのキーと release がホストへ返ります。エディターに届く前に Space などを奪うホストへの対策も Kivact にはありません。
- **パニックでエディターが止まる。** パニックはエディターを終了させ、`Embedded::status` が `ErrorKind::Panicked` として報告します。wrapper へ巻き戻ることはないので、C ABI のホストが abort することはありません。ハンドルのメソッドはその後も普通に戻り、drop すると残りが閉じられます。想定できる失敗は `error_boundary` で処理してください ([エラーや更新の原因を調べる](diagnostics.md))。

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
- **エラー。** 不正なサイズやスケールは `ErrorKind::Message` を返します。不正または null の親、OS 資源の不足、未対応の OS は `ErrorKind::Platform` を返します。root または開く途中でのパニックは `ErrorKind::Panicked` を返し、ウィンドウを残しません。そのあとのパニックはエディターを止め、`Embedded::status` が報告します。Kivact が呼び出し元へ巻き戻すことはありません。
- **ハンドル。** `Embedded` は `Send + Sync` です。エディターが開いているあいだ保持してください。複数のスレッドから setter を呼ぶなら `Arc` に入れて持ちます。

### ホストの呼び出しに応える

setter は UI スレッドへ変更を積んですぐ戻り、getter は最後に適用された値を返します。この境界でのサイズはすべて**物理ピクセル**で、スケールは物理ピクセル / 論理ピクセルです。

| ホストの呼び出し (CLAP / VST3) | wrapper がすること |
|---|---|
| 開く前の `set_scale` / `setContentScaleFactor` | スケールを保存する。 |
| 開く前の `get_size` / `getSize` | `round(論理サイズ * スケール)` を返す。 |
| `set_parent` / `attached` | 同じスケールを `Parent` に渡して `App::embed` を呼び、ハンドルを保持する。 |
| `show` / `hide` (CLAP `gui.show` / `gui.hide`) | `Embedded::show()` / `Embedded::hide()`。適用済みの状態は `Embedded::is_visible()` で読む。 |
| `set_size` / `onSize` | `Embedded::set_size(width, height)`。確定待ちのリサイズ要求にも答える。 |
| 開いた後の `get_size` / `getSize` | `Embedded::size()` |
| スケールの変更 | `Embedded::set_scale(scale)`。論理サイズを保ちたいなら、そのあとで新しい物理サイズをホストに求める。 |
| `adjust_size` / `checkSizeConstraint` | wrapper 自身の制限で丸める。ホストの `set_size` は `min_window_size` では制限されません。 |
| `destroy` / `removed` | ハンドルを drop する。 |
| OS が表示スケールを変えた (Windows) | `Parent::on_notice` が `EmbedNotice::ScaleChanged(scale)` を渡す。`scale()` を読み、`round(論理サイズ * スケール)` をホストに求めて論理サイズを保つ。 |
| エディターが止まった (親の破棄、失敗、パニック) | `Parent::on_notice` が `EmbedNotice::Stopped` を渡す。`status()` を読む。 |

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

    /// エディターが開いている場合の停止理由。`Running`、`Closed`（ホストが親を破棄）、
    /// `Failed(error)`、`Panicked(error)`。
    pub fn status(&self) -> Option<kivact::embed::EmbedStatus> {
        self.embedded.as_ref().map(Embedded::status)
    }

    pub fn close(&mut self) {
        // drop するとエディターが閉じ、後始末の完了を待ちます。
        self.embedded = None;
    }
}
```

サイズが 0 の場合、プラットフォームの上限 (Windows は `i32::MAX`、X11 は `u16::MAX`) を超える場合、スケールが有限の正の数でない場合は、`tracing` に警告を出して無視されます。エディターは動き続けます。

### ポーリングではなく通知を受け取る

`Parent::on_notice` は、OS が適用スケールを変えたときと、エディターが止まったときに wrapper を起こします。通知は起こすためのもので、状態ではありません。`status()` や `scale()` を読んで何が起きたかを知ります。コールバックを用意するまでは、ホストの idle timer (CLAP の `timer-support`、VST3 の run loop) で `status()` をポーリングしてください。

```rust,no_run
use std::sync::Arc;

use kivact::embed::{EmbedNotice, Parent};
use kivact::embed::EmbedStatus;

/// エディターに反応する wrapper の部分。
#[derive(Clone)]
pub struct Wake {
    /// ホストのメインスレッドへメッセージを送る (どのチャンネルでもよい)。
    post: Arc<dyn Fn() + Send + Sync>,
}

fn notices(parent: Parent, wake: Wake) -> Parent {
    parent.on_notice(move |notice| {
        // ここでは待たず、あとでホストに実行させるだけにします。
        keep_logical_size(notice);
        let post: &(dyn Fn() + Send + Sync) = &*wake.post;
        post();
    })
}

/// ホストが新しい物理サイズを求めて論理サイズを保つべきか。
fn keep_logical_size(notice: EmbedNotice) -> bool {
    matches!(notice, EmbedNotice::ScaleChanged(_))
}

/// ホストのメインスレッドで、起きたあとに: エディターが動いているか。
fn alive(status: EmbedStatus) -> bool {
    status == EmbedStatus::Running
}
```

コールバックはホストの GUI スレッドを待たず、パニックしてもいけません。X11 ではエディタースレッドの終了処理から呼ばれることがあり、ハンドルの drop で始めた close は drop したスレッドから呼ばれることがあります。`ScaleChanged` を送るのは Windows だけです。X11 のスケールは Xft の 1 つだけなので、モニターを移しても変わりません。

### 表示と非表示

CLAP は `set_parent` のあとに `gui.show` / `gui.hide` を呼びます。VST3 と
standalone のウィンドウは表示した状態で開きます。ホストが非表示でエディターを
始める場合は `Parent::hidden()` で開き、あとから `Embedded::show` /
`Embedded::hide` を呼びます。どちらも冪等で、他の setter と同じくキューに積まれます。
非表示のエディターは App（task、timer、signal）を動かしますが、何も描きません。
frame loop は動かず、`platform::is_surface_active()` は `show` まで false です。
非表示にするとキーボードフォーカスは親へ戻り、押されているポインターは取り消されます。
キューに積まれた呼び出しは、UI スレッドが適用するまで `is_visible` に反映されません。

```rust,no_run
use kivact::embed::{Embedded, Parent};

fn hidden_until_shown(parent: Parent) -> Parent {
    parent.hidden()
}

fn show(editor: &Embedded) {
    editor.show();
}
```

### キーとホスト

キーの**押下**が未処理であるとは、フォーカスされたテキスト入力、フォーカスの移動、
IME やコピー/貼り付け、`on_key` ハンドラー（失敗したハンドラーは処理済みと数える）、
既定動作（Enter/Space の活性化、Escape の破棄）のどれも使わなかった場合です。
release はその押下に従います。

既定では未処理のキーはエディターに残ります。`Parent::forward_unhandled_keys` を
付けると、押下とその release が親ウィンドウへ返ります。Windows では元の
`WM_KEYDOWN`/`WM_KEYUP` を親へ post し（JUCE のエディターと同じ）、X11 では合成の
`KeyPress`/`KeyRelease` を送ります（`XSendEvent`。合成イベントを無視するツールキットもあります）。
Space などのショートカットを取り戻したいホストが opt-in します。テキスト入力と
エディター自身のショートカットが先に働くことは変わりません。

```rust,no_run
use kivact::embed::Parent;

fn forwarding(parent: Parent) -> Parent {
    parent.forward_unhandled_keys()
}
```

判定は同期です。Kivact はネイティブの押下ごとに probe し（ネイティブメッセージが
戻る前に配送と turn を済ませる）、ホストはユーザーが押した順にキーを受け取ります。
App のコードがすでにスタック上にあるとき（App のハンドラーが開いたモーダルなど）は
probe せず、その押下は返しません。wrapper のコールバックの中でホストの GUI スレッドを
待ってはいけません。

### 両方向のサイズ変更

ユーザーやホストがウィンドウのサイズを変えたときは `set_size` を呼びます。UI が別のサイズを望むときは、**論理**サイズで `platform::request_window_size` を呼びます。要求は `Parent::on_resize_request` を通って、**物理**ピクセルで、UI スレッド上の wrapper に届きます。ホストがしたことを返してください。

| 戻り値 | 意味 | そのあと |
|---|---|---|
| `ResizeAnswer::Accepted` | ホストがその場で受理した (CLAP `request_resize` が true)。 | Kivact が自分でウィンドウをリサイズする。あとから `set_size` が来ればそちらが優先。UI の要求はすぐ成功する。 |
| `ResizeAnswer::Pending` | ホストが後で決め、`set_size` で確定する (VST3 `resizeView` → `onSize`)。 | それまでウィンドウのサイズは変わらない。ホストのサイズが要求と一致すれば成功、違えばホストが選んだサイズを名指すプラットフォームエラー、新しい要求があれば `Cancelled`、2 秒で `Timeout`。 |
| `ResizeAnswer::Refused` | ホストが断った。 | 要求はプラットフォームエラーで失敗する。 |

コールバックがなければ、要求は `Unsupported` で失敗します。

```rust,no_run
use kivact::embed::{Parent, ResizeAnswer};

fn resizable(parent: Parent) -> Parent {
    parent.on_resize_request(|width, height| {
        // ホストに width x height の物理ピクセルを求めます。これは UI スレッドで
        // 動くので、要求はホスト自身の run loop へ送り、GUI スレッドの返事は
        // 待たずに答えてください。
        let _ = (width, height);
        ResizeAnswer::Pending
    })
}

// UI 側では、タスクや effect の中で:
async fn fit_content() -> kivact::Result<()> {
    kivact::platform::request_window_size(800, 600).await
}
```

wrapper が自分のサイズを答えるときの順序は 1 つです。**新しい論理サイズを保存し、ホストに求め、拒否されたら元のサイズに戻す。** フレームワークによっては (nih-plug)、要求の実行中にエディターのサイズを読み直します。

`Pending` のとき、ホストが確定するまで子ウィンドウはリサイズされません。ホストのサイズ通知 (`onSize`、X11 では post した CLAP の `request_resize` が true を返したあと) から `Embedded::set_size` を呼んでください。要求はその呼び出しで解決します。`Pending` と答えて確定しないホストでは、UI の要求は 2 秒でタイムアウトし、子は元のサイズのままです（安全な側）。

スケールが変わっても論理サイズを保ちたいときは、先に `set_scale` を呼び、そのあとで `round(論理サイズ * 新しいスケール)` をホストに求めます。

### 閉じる

`Embedded` はホストの GUI スレッドで drop します。App のコールバックの外では、drop は App とネイティブウィンドウの後始末が終わるのを待ちます。App が所有するスレッドごとに最大 2 秒です。その上限を超えた処理は drop が戻ったあとも続き、ホストがプラグインを安全にアンロードできるよう、Kivact が自分のコードをメモリに保持します。モジュールを自分で保持する必要はありません。自分で起動したスレッドの停止と join は、自分の責任です。

エディターが App のコールバックの中 (たとえば「閉じる」ボタン) で自分のハンドルを drop した場合、実行中のターンは最後まで進み、そのあとに後始末が行われ、以後の入力は配送されません。キューにあるアクションは消化されません。

ホストが先に親ウィンドウを破棄した場合、エディターは安全に停止し、`status()` は `EmbedStatus::Closed` を返します。プラットフォームの失敗は App を止め、ウィンドウを破棄し、`status()` は理由つきの `EmbedStatus::Failed` を返します。パニックも同じように止め、`status()` はパニックのメッセージつきの `EmbedStatus::Panicked` を返します。どの呼び出しも drop もそれを再開しません。ハンドルはその後でも drop する必要があります。長く走る `blocking` の仕事は `CancelToken` を定期的に確認して、閉じるときにその終了を待たなくて済むようにしてください。

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
assert_eq!(editor.status(), kivact::embed::EmbedStatus::Running);
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
| 追加のウィンドウ | なし | なし |
| DevTools の入力の注入 | なし（`UNSUPPORTED`） | なし（`UNSUPPORTED`） |
| フルスクリーン、ウィンドウタイトル | 効かない | 効かない |
| 最大サイズ (物理) | `i32::MAX` | `u16::MAX` |
| macOS (NSView) | 未対応 | |
| Wayland (ネイティブの埋め込み) | | 未対応 |
| 未処理のキーをホストへ返す | opt-in (`Parent::forward_unhandled_keys`) | opt-in、ホスト依存 (`XSendEvent`) |
| エディターの表示・非表示 | 可 (`Embedded::show`/`hide`、`Parent::hidden`) | 可 |
| ホストの確定を待つリサイズ | 可 (`ResizeAnswer::Pending`) | 可 |
| OS の表示スケールの変更 | `EmbedNotice::ScaleChanged` | なし (Xft のスケールは 1 つ) |

実際のプラグインホスト (REAPER、Bitwig Studio、Ableton Live、Cubase など)、実際の DAW での IME、プラグインウィンドウ内での読み上げソフトは、まだ検証していません。

## 埋め込みのテーマ

埋め込みの `App::theme` は固定のテーマを受け取ります。呼び出し元スレッドの reactive な
テーマは `UiError::Unsupported` になります。動的なテーマは `embed` に渡す `Send` な
root factory の中でシグナルと `provide_theme` を作ります。デスクトップの `run` と試験では、
`App::theme` の reactive なテーマは root owner の下で評価されます。

## 関連項目

- [独自の見た目のコントロールを作る](custom-controls.md): `EditPhase` と、ジェスチャーを報告するコントロール。
- [コンポーネントから非同期処理を実行する](async-and-tasks.md): `spawn` と `blocking`。
- [エラーや更新の原因を調べる](diagnostics.md): パニックが App を終了させるしくみ。
- [プラグインのエディタを専用のウィンドウに表示する](foreign-windows.md): 逆向きの用途で、ホストアプリが別のライブラリの UI を埋め込む。
- `cargo doc -p kivact --features embed --open`: `kivact::embed`。
