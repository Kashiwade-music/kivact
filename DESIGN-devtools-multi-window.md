# 設計書: 複数ウィンドウ・DevTools・暗黙の状態マシンの解消

| 項目 | 内容 |
|---|---|
| 作業ブランチ | `feat/kashiwade/devtools-multi-window` |
| 基準 commit | `96e22b6b`（`main`、#10 visual parity） |
| 対になる文書 | [作業指示書](WORK-ORDER-devtools-multi-window.md)（手順・担当・検証）、[状態マシン修正の付録](WORK-ORDER-appendix-state-machines.md) |
| 読者 | 実装を担当する AI（とその指揮役）。この文書だけで「何を作るか」が決まるように書く。手順は作業指示書に書く |

この文書は作業ブランチの間だけ置く。merge 前に、恒久的に残す内容を `docs/for-kivact-developer/` の各文書と利用者ガイドへ移し、この文書と作業指示書と付録は削除する（作業指示書の最終段階に含める）。

---

## 0. 用語

| 用語 | 意味 |
|---|---|
| App | 一つの `kivact::App`。Runtime（signal・owner・turn）は App に一つ |
| window | OS の最上位ウィンドウ一つ。App は 1 個以上の window を持つ。最初の window を main window と呼ぶ |
| `WindowId` | window の識別子。core が採番する。main window は常に `WindowId::MAIN`（= 1） |
| target | DevTools が調べる側のアプリ（`devtools` feature を付けてビルドしたもの） |
| agent | target の中で動く DevTools の server（crate `kivact-devtools-agent`） |
| client | agent に接続するもの。DevTools アプリ・MCP server・Python の 3 種類 |
| protocol | agent と client の間の JSON-RPC 2.0 の語彙（crate `kivact-devtools-protocol`） |
| 暗黙の状態マシン | `architecture.md` の S-1〜S-9 に反する状態の持ち方。定義は 5 章 |

---

## 1. 決定事項（すべて確定済み。実装中に変更しない）

| # | 項目 | 決定 |
|---|---|---|
| D-1 | 進め方 | 複数ウィンドウ・DevTools・状態マシンの解消を、一つのブランチ・一つの PR で行う。複数ウィンドウを先に作る |
| D-2 | DevTools の配置 | 別プロセス。target 内の agent が protocol の server になり、DevTools アプリ・MCP・Python の 3 つが同じ protocol の client になる |
| D-3 | 有効化 | facade の cargo feature `devtools` だけで決める。release ビルドでも使える。有効な release ビルドは起動時に警告を出す。feature なしでは agent・socket・core の devtools module は入らない。engine と runtime の口は常にコンパイルするが、設定されない限り何もしない（4.10 節） |
| D-4 | 起動キー | F12 / Ctrl+Shift+I で開く。Ctrl+Shift+C で pick mode から開く。アプリのショートカットより先に処理する。キーは `.kivact/devtools.json` で変えられる |
| D-5 | ファイルの置き場所 | target の workspace root の `.kivact/`。review は `.kivact/review/<session>/`、Python script は `.kivact/devtools/scripts/`、設定は `.kivact/devtools.json`。`.kivact/` は最初に file を書くときに作る（起動では作らない）。例外: discovery file は利用者ごとの directory に置く（4.2.2 節） |
| D-6 | review の書き出し | 一つの review model から JSON（正本、schema と版番号付き）と Markdown を書き出す |
| D-7 | 更新理由 | 人向けはグラフィカル（時間軸のレーン・原因グラフ・画面上の点滅とヒートマップ）。MCP と Python には構造化した JSON（nodes と edges）で返す |
| D-8 | MCP | 最初の段階から作る。tool 名は protocol の method 名と同じ |
| D-9 | Python | 型付きの package。VS Code（Pylance / pyright）で補完と型検査が効く。DevTools の Scripts パネルからも実行できる |
| D-10 | 採用する機能 | 前回の提案の A 群・B 群のすべてと MCP（4.5 節の一覧）。review の再読込みと 3D 表示は作らない |
| D-11 | window の位置 | 相対的な意図だけ（`Auto` / `CenterOnParent` / `NextToParent`）。座標は公開しない |
| D-12 | 入力の注入 | MCP と Python から許す（調査用）。注入は virtual backend と同じ正規化の経路を通る。契約の試験は `cargo test` の E2E に置く |
| D-13 | スタイル編集の範囲 | このノードだけ／同じ呼び出し位置の全ノード／theme token の 3 つ |
| D-14 | 値の表示 | engine の prop（確定値）は表示する。signal は型名と位置だけ（値は記録しない） |
| D-15 | 状態マシン | 存在する暗黙の状態マシンをすべて解消する（5 章）。xtask の検査を強めて、残りが無いことを機械的に確かめる |
| D-16 | window を前に出す | 新しい公開 API `kivact::platform::focus_window()` を足す（呼んだ owner の window を前面に出す。最善の努力で、capability は無い）。DevTools アプリは `focus_requested` の通知でこれを呼ぶ。開いている window の `open` に true を書き直しても何も起きない |
| D-17 | embed での入力の注入 | embed の target では `inject_input` を `UNSUPPORTED`（-32015）で断る。embed の port（win32・X11）の入力処理を `WindowInput` へ移すことは、この変更では行わない（7 章） |
| D-18 | 描画の調査の表示 | paint flashing（再描画した箱の点滅）と flex/grid の重ね表示は作る。compositor の layer の境界の表示（layer borders）は作らない（7 章） |

---

## 2. 実装方針の適用

利用者（オーナー）の実装方針と、それをこの変更でどう守るか。

| 方針 | この変更での具体化 |
|---|---|
| Rust 未経験の React・SolidJS 開発者が書ける公開 API | `window(title, open).content(|| view)` は React の Portal・Solid の `<Portal>` と同じ考え方で説明できる（中身は宣言した場所の context を引き継ぎ、別の OS ウィンドウに出る）。開閉は `dialog(title, open)` と同じ binding。アプリ作者が DevTools のために書くコードは無い（Cargo.toml の app の feature に `devtools = ["kivact/devtools"]` を足すだけ） |
| わかりやすく統一的な API | window の API は `dialog` と同じ形（タイトルと binding が位置引数、`.content` は factory、構造の選択は `.mode(WindowMode)`。`DialogMode` と同じ規則）。閉じる要求の handler は `overlay` と同じ署名の `on_dismiss(DismissReason)`。overlay と違い、window は既定で自分で閉じる（`dialog` と同じ。`Handled::No` で閉じ、`Yes`・`PreventDefault`・失敗では開いたまま）。DevTools の検索語彙は `kivact::test` と同じ（`get_by_role` / `query_by_role` / `find_by_role` / `all_by_role` / `*_by_text` / `*_by_test_id`）で、protocol・MCP・Python の 3 つで名前が同じ |
| JavaScript・Python 開発者が容易に書ける | Python package は標準ライブラリだけで動き、型ヒントと docstring が付く。`connect()` は引数なしで動く（引数は keyword だけ）。低水準の method は `app.rpc` に分け、手書きの API（`App`・`Window`・`Element`・`Recording`）と名前がぶつからない。設定ファイルは JSON で、JSON Schema（GitHub の URL）を `$schema` に書けば VS Code で補完が効く。導入は `cargo install --git …` と `pip install "… @ git+…"` で、Kivact の repo の外でも使える |
| 内部が統一的で単純 | main window も追加の window も同じ経路で開く。engine の per-window の部分は今の `Model` をそのまま「一つの window の model」として使い、App 全体の部分（event の列・書き込み数・asset・motion の時計・frame の要求・DevTools の記録）だけを `AppModel` へ出す。入力の正規化は native（winit）・virtual・DevTools の注入で一つの型（`arvane_shell::WindowInput`）を使う。注入は他の入力と同じ列を通る（a11y の操作も `InjectedInput` の一つ） |
| 長期保守性 | protocol の型の正本は Rust の型一つ。JSON Schema・Python の型・MCP の input schema・error code の一覧はすべて xtask が生成し、生成物が古ければ `cargo xtask check` が落ちる。生成器は対応していない serde の属性を拒む。版の規則（wire の削除・改名・型の変更で `PROTOCOL_VERSION` を上げる）を protocol crate と `architecture.md` の checklist に書く |
| strategy.md を簡単に遂行できる | DevTools は 1 章「更新理由を追える仕組み」、G-05、P-03、P-08、9 章「診断」の指標を直接満たす。複数ウィンドウは主対象（業務・開発ツールの inspector、クリエイティブ編集ツールの tool window）に必要 |
| crate 分割（境界がはっきりしているときだけ） | 新しい crate は 5 つ（4.1 節）。それぞれの境界の理由を 4.1 節に書く。複数ウィンドウのためには crate を作らない（既存の境界の中身が変わるだけ）。DevTools アプリの各パネルは module であり crate にしない |
| 暗黙の状態マシンをすべて解消 | 新しく作る状態はすべて明示的な enum か Machine（3.11 節・4.3 節）。既存の 674 か所の例外の印を全件分類し、直すものは付録に修正内容を書いた。xtask の検査を強め、構造体単位の例外の印を無効にする |

---

## 3. 複数ウィンドウ

### 3.1 公開 API

置き場所は `kivact::view`（`overlay` と同じ。中身を host の tree に置く構造的な view だから）。`window` は prelude に入れる（`dialog` と同じ扱い）。型 `WindowMode`・`WindowPlacement` は `kivact::view` から出し、prelude にも入れる（`DialogMode`・`Placement` と同じ。構造の選択は `.mode(XxxMode)` という既存の規則に合わせる）。

```rust
/// A window of this App: while `open` is true its content is mounted and
/// shown in its own OS window. The content inherits the contexts of the
/// place where `window` is declared, like a React portal.
#[track_caller]
pub fn window(title: impl IntoText, open: impl IntoBinding<bool>) -> Window;

impl Window {
    /// The content, built again each time the window opens (API-6 factory).
    pub fn content<V: IntoView>(self, content: impl IntoViewFn<V>) -> Self;
    /// The size the window opens with, in logical pixels (default 640 × 480).
    pub fn size(self, width: impl Into<f64>, height: impl Into<f64>) -> Self;
    /// The smallest size the user can resize the window to.
    pub fn min_size(self, width: impl Into<f64>, height: impl Into<f64>) -> Self;
    /// What kind of window it is (structural, default `WindowMode::Normal`).
    pub fn mode(self, mode: WindowMode) -> Self;
    /// Where it opens relative to the window it is declared in (structural,
    /// default `WindowPlacement::Auto`).
    pub fn placement(self, placement: WindowPlacement) -> Self;
    /// Called when the user asks to close the window (title bar button,
    /// Alt+F4). Return `()` (or `Handled::No`) to let it close: `open` is set
    /// to false. `Handled::Yes` and `Handled::PreventDefault` keep it open, as
    /// they skip every default action; so does a failed handler (its error is
    /// reported). Unlike an overlay, a window closes itself, like `dialog`.
    pub fn on_dismiss<R: HandlerResult>(self, f: impl Fn(DismissReason) -> R + 'static) -> Self;
}

impl IntoView for Window { /* … */ }
```

利用例（ガイドに載せる例。React 開発者に向けて「Portal と同じ」と説明する）:

```rust
use kivact::prelude::*;

fn editor() -> impl IntoView {
    let settings_open = signal(false);
    column((
        button("Settings").on_press(move || settings_open.set(true)),
        window("Settings", settings_open)
            .size(480, 360)
            .mode(WindowMode::Tool)
            .placement(WindowPlacement::CenterOnParent)
            .content(|| text("Settings go here")),
    ))
}
```

規則:

1. `title` は reactive（`IntoText`）。変わると OS のタイトルも変わる。
2. `size` と `min_size` は開くときの値（定数）。開いた後の大きさは利用者が変える。`window` の view を mount するとき一度だけ `geometry::check_window_sizes(Some(size), min_size, ("Window::size", "Window::min_size"), at)` で確かめる。失敗したら宣言した owner に `ErrorKind::Message` で報告し、その window は開かない（`open` を読む effect を作らない。binding は書き換えない）。この検査は `App::window_size`・`App::min_window_size` と同じ関数を使う（今の `kivact-core/src/launch/settings.rs:202` の `check_window_sizes` を `kivact-core/src/geometry.rs` へ移し、`launch` と `view` の両方から使う。`view` は module の規則で `launch` を参照できないため。関数は error の message に出す API の名前を引数 `names: (&'static str, &'static str)` で受け取る）。
3. `mode` と `placement` は構造 option（定数）。
4. `open` が true の間だけ OS window が存在する。false にすると中身を unmount し、window を閉じる。宣言した owner が破棄されたときも閉じる（G-01）。すでに開いている window の `open` に true を書き直しても何も起きない（前面には出ない。前面に出すには、その window の中から `platform::focus_window()` を呼ぶ。3.1.1）。
5. 利用者が閉じる操作をすると `on_dismiss(DismissReason::WindowClose)` を呼ぶ。`on_dismiss` が無いときと、返り値が `Handled::No`（`()` を含む）のときだけ `open` に false を書く。`Handled::Yes`・`Handled::PreventDefault`・失敗（`Err`。報告する）では開いたまま（key の既定の動作と同じ規則。`client/dispatch/key.rs:54, 78-80`）。overlay と違い、window は既定で自分で閉じる（`dialog` と同じ）。
6. main window を閉じる操作は今と同じく App を終了する（`rt.request_exit()`）。main window の閉じる操作の取り消しはこの変更では作らない（7 章）。
7. `Capabilities.multi_window` が false の環境（embed、mobile）で `open` が true になると、`ErrorKind::Unsupported` の `UiError` を宣言した owner に報告し、window は開かない。binding は書き換えない。報告は window の宣言ごとに一度（`WindowSlot` が `ReportOnce` を持つ）。
8. `platform` の環境の問い合わせ（`window_size()`、`scale_factor()`、`is_fullscreen()`、`safe_area()`、`keyboard_inset()`、`theme_hint()`、`is_surface_active()`、`is_motion_reduced()`、`pointer_settings()`）と、window に向けた service（`request_window_size`、`set_fullscreen`、`focus_window`、ファイル選択の親）は、呼び出した owner が属する window を対象にする。`callback` の中の `platform` の呼び出しは、その callback を作った owner の window を対象にする（`callback` の規則）。window の中の操作で使う service（ファイル選択の親、`request_window_size`）は、その window の content の中で作った handler・callback から呼ぶ。
9. focus は window ごとにある。`NodeRef::focus()` はその node の window の中で focus を移す。

`DismissReason` に variant `WindowClose` を足す（`kivact-types/src/semantics/values.rs`。doc に「A window closes itself unless its `on_dismiss` keeps it open.」の 1 文を含める）。

#### 3.1.1 `platform::focus_window`

```rust
/// Bring the window of the calling owner to the front and give it the
/// keyboard focus. Best effort: the platform may refuse (Windows may only
/// flash the taskbar entry; Wayland has no way to raise a window). Needs no
/// capability. `ErrorKind::Unsupported` in an embedded editor.
#[track_caller]
pub fn focus_window() -> impl Future<Output = Result<(), UiError>> + 'static;
```

`request_window_size` と同じ道を通る: `kivact-types` の `PlatformRequest::FocusWindow` → `HostRequest::Platform { window, request }` → `ShellWindow::perform` → `ShellWindow::focus()` → winit の `Window::focus_window()`。virtual は呼ばれた回数を `WindowProbe::focus_requests()` に記録する。embed の window は `ShellError::unsupported` を返す。DevTools アプリは `focus_requested` の通知を受けたらこれを呼ぶ（4.5.1）。

### 3.2 値の型（`kivact-types`）

新しい leaf module `crates/kivact-types/src/window.rs`。`kivact-types` の lib.rs から再公開する。`[state_fields] value_modules` に `"kivact-types::window"` を足す。

```rust
//! Role: Window identities and the request to open one.

use std::num::NonZeroU64;
use crate::Size;

/// The identity of one window of an App. The App's first window is `MAIN`;
/// the core numbers further windows 2, 3, ... and never reuses a number.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WindowId(NonZeroU64);

impl WindowId {
    /// The App's first window.
    pub const MAIN: WindowId = WindowId(NonZeroU64::MIN);
    /// The id with number `raw`, or `None` for 0.
    pub fn new(raw: u64) -> Option<WindowId> { NonZeroU64::new(raw).map(WindowId) }
    /// The number of this id.
    pub fn get(self) -> u64 { self.0.get() }
}

/// What kind of window a window is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum WindowMode {
    /// An ordinary window with its own taskbar entry.
    #[default]
    Normal,
    /// A utility window that stays above its parent and has no taskbar entry.
    Tool,
    /// A window that blocks input to its parent while it is open.
    Modal,
}

/// Where a window opens, relative to its parent window.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum WindowPlacement {
    /// Where the platform puts it (the main window: centered on the primary
    /// monitor's work area, as today).
    #[default]
    Auto,
    /// Centered over the parent window.
    CenterOnParent,
    /// Next to the parent window: to its right, or to its left when the right
    /// side has no room, or `Auto` when neither has room.
    NextToParent,
}

/// A request to open one window.
#[derive(Clone, Debug, PartialEq)]
pub struct WindowRequest {
    pub id: WindowId,
    pub title: String,
    /// Logical size.
    pub size: Size,
    pub min_size: Option<Size>,
    pub mode: WindowMode,
    pub placement: WindowPlacement,
    /// The window the request was declared in; `None` only for the main window.
    pub parent: Option<WindowId>,
}

impl WindowRequest {
    /// The request for the App's first window: `WindowId::MAIN`, a `Normal`
    /// window at the platform's default place, without a parent.
    pub fn main(title: impl Into<String>, size: Size) -> WindowRequest {
        WindowRequest {
            id: WindowId::MAIN,
            title: title.into(),
            size,
            min_size: None,
            mode: WindowMode::Normal,
            placement: WindowPlacement::Auto,
            parent: None,
        }
    }
}
```

main window の要求も `WindowRequest` で渡す。`arvane-shell` の `ShellOptions` は消す（開く要求の型が二つになるため）。`arvane_native::backend(main: WindowRequest)`、`VirtualOptions` は `title`・`size` の代わりに `main: WindowRequest` を持つ。

`Capabilities`（`crates/kivact-types/src/env.rs`）に 2 つの field を足す。既定は false。

```rust
/// More windows than the first can be opened.
pub multi_window: bool,
/// The platform honours `WindowPlacement` other than `Auto` (false on Wayland).
pub window_placement: bool,
```

値: native（winit）は `multi_window: true`、`window_placement` は Wayland のとき false・それ以外 true（判定は既存の `display/placement.rs` の Wayland 判定を使う）。virtual は `VirtualOptions.capabilities` に従う（既定の `VirtualOptions` では両方 true にする）。embed（win32・x11）は両方 false。`platform::capabilities()` の doc は「fixed for the App's life, except while DevTools emulates touch」とし、能力の一覧に `multi_window`・`window_placement` を足す。

`PlatformRequest`（`crates/kivact-types/src/platform.rs`）に `FocusWindow`（doc: `/// Bring the window to the front and give it the keyboard focus (best effort).`）を足す。

### 3.3 shell の SPI（`arvane-shell`）

一つの handler が、loop 全体と複数の window を相手にする形に変える。

```rust
// crates/arvane-shell/src/window.rs （既存 file。ShellHandler / Backend / SteppedBackend を置き換える）

/// What one backend event is about.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventTarget {
    /// The loop as a whole: SaveState, Wake, Suspended, Resumed, MemoryWarning.
    Loop,
    /// One window: every other event.
    Window(WindowId),
}

impl EventTarget {
    /// The target of `event` for a loop whose window is `window`: `Loop` for
    /// SaveState, Wake, Suspended, Resumed and MemoryWarning, else `Window(window)`.
    pub fn of(event: &ShellEvent, window: WindowId) -> EventTarget;
}

/// The running loop as the handler sees it during one callback.
pub trait ShellLoop {
    /// An open window, or `None` once it is closed.
    fn window(&mut self, id: WindowId) -> Option<&mut dyn ShellWindow>;
    /// The open windows, ascending by id.
    fn windows(&self) -> Vec<WindowId>;
    /// Open a window. It becomes visible when the current callback returns.
    /// `ShellError::unsupported` where `Capabilities.multi_window` is false.
    fn open_window(&mut self, request: WindowRequest) -> Result<(), ShellError>;
    /// Destroy a window. No event for it follows, and input queued for it
    /// (`inject`) is dropped without `ShellEvent::Injected`. The handler
    /// releases what it holds for the window (its presenter) before calling this.
    fn close_window(&mut self, id: WindowId);
    /// Queue input for a window; it goes through that window's `WindowInput`
    /// like the platform's own input and is delivered right after the current
    /// callback returns, followed by `ShellEvent::Injected { request }` once
    /// all of it was delivered.
    fn inject(&mut self, id: WindowId, inputs: Vec<InjectedInput>, request: RequestId);
    /// The loop clock, comparable to `ShellControl::WakeAt`.
    fn now_ns(&self) -> u64;
    /// The waker of this loop (the same one `ShellHandler::start` received).
    fn waker(&self) -> ShellWaker;
    /// What the loop can do (`multi_window`, `window_placement`, ...).
    fn capabilities(&self) -> kivact_types::Capabilities;
}

pub trait ShellHandler {
    /// Called once, after the backend opened the main window (`WindowId::MAIN`).
    fn start(&mut self, shell: &mut dyn ShellLoop, waker: ShellWaker) -> Result<ShellControl, ShellError>;
    /// One ordered event. After `ShellControl::Exit` or an error, none follows.
    fn event(&mut self, target: EventTarget, event: ShellEvent, shell: &mut dyn ShellLoop) -> Result<ShellControl, ShellError>;
    /// Called exactly once for every handler whose `start` was called; the
    /// windows are still open during the call.
    fn close(&mut self, shell: &mut dyn ShellLoop);
}

pub trait Backend {
    fn run(self: Box<Self>, handler: &mut dyn ShellHandler) -> Result<(), ShellError>;
}

pub trait SteppedBackend {
    /// Open the main window and call `handler.start` before returning.
    fn start(self: Box<Self>, handler: Box<dyn ShellHandler>) -> Result<Box<dyn SteppedLoop>, ShellError>;
}
```

`ShellWindow`（既存 trait）はそのまま一つの window を表す。次の method を一つ足す:

```rust
/// Bring the window to the front and give it the keyboard focus. Best effort:
/// `Ok` also when the platform only flashes the taskbar entry.
/// `ShellError::unsupported` where a window cannot be raised (an embedded editor).
fn focus(&mut self) -> Result<(), ShellError>;
```

各 backend の `perform` は `PlatformRequest::FocusWindow` を `self.focus().map(|()| PlatformResult::Done)` にする。`ShellWindow::capabilities()` は window から分かる値だけを返す。`multi_window`・`window_placement` を含む loop の値は `ShellLoop::capabilities()` が正本で、engine は `HostInfo.capabilities` をこちらから作る（GPU の値は `arvane` が足す。3.6）。

main window の要求も `WindowRequest`（`id: WindowId::MAIN`、`parent: None`）で渡す。`ShellOptions` は消す。`StepOutcome::Idle.redraw_requested` は「どれか一つの window が描画を求めている」を意味する。

`InjectedInput`（`crates/arvane-shell/src/input.rs`、新規の leaf module）:

```rust
/// Input a tool (the DevTools agent) asks the loop to deliver, before the
/// platform-independent normalization.
#[derive(Clone, Debug, PartialEq)]
pub enum InjectedInput {
    /// Move the mouse to a logical window position.
    MouseMove(kivact_types::Point),
    MouseDown(kivact_types::MouseButton),
    MouseUp(kivact_types::MouseButton),
    /// Wheel lines at the current mouse position.
    Wheel { lines_x: f64, lines_y: f64 },
    /// Set the held modifier keys (delivered as `ShellEvent::Modifiers`, the
    /// way both backends report modifiers today). `kivact_types::Key` has no
    /// modifier keys, so an injected chord is: Modifiers(set), key down/up,
    /// Modifiers(empty).
    Modifiers(kivact_types::Modifiers),
    /// One raw key, IME or file-drop input (the `pipeline::RawInput` vocabulary).
    Raw(crate::pipeline::RawInput),
    /// Leave the window with the mouse.
    MouseLeave,
    /// An assistive-technology request for one node.
    Accessibility { node: u64, request: kivact_types::A11yRequest },
}
```

`Accessibility` の規則: `ShellLoop::inject` は他の入力と同じ列で順に配る。配る前に、その window の受理済みの a11y tree で検証する（`arvane_shell::Accessibility::request_action(node, request)` / native は `PlatformTree::request_action`。どちらも内部は `a11y::action::accept`）。`Some(event)` なら `ShellEvent::Accessibility` としてそのまま配り、`None`（node が無い、非表示、無効、宣言されていない action）なら配らない（`Injected` は配る）。`WindowInput::normalize` はこの variant を扱わず空の列を返す（検証は loop の仕事）。これで a11y の step も他の step と順番が保たれる。

`pipeline::RawInput`・`RawKey`・`PlatformKey`・`RawIme`・`RawDrop` に `Clone` と `PartialEq` を derive する（今は `Debug` だけ）。

`ShellEvent` に `Injected { request: RequestId }` を足す（`ShellLoop::inject` の入力をすべて配った後に、その window に 1 回配る。engine は `HostEvent::InjectDelivered { window, request }` にする。CONTRACT X7 に `window` を足したのは意図的: 閉じた window の event を捨てる 3.7 の規則と同じ形にする）。inject の識別は effect の `RequestId`（`kivact_types::RequestId`）を使い、別の番号を作らない。注入した入力を配るために loop を起こす bit（`WakeReasons`）は作らない: `inject` は handler の callback の中（UI thread）で呼ばれ、入力は callback が返った直後に配る（native では `pending_show` を処理するのと同じ場所）。

`WindowInput`（`crates/arvane-shell/src/window_input.rs`、新規の leaf module）: 一つの window の入力正規化の状態を一つにまとめる。今は native の `Runner` と virtual の `Input` が同じ部品（`Mouse`、`ModifierState`、`ClickCounter`、`Contacts`、`KeyPipeline`、`DropSession`）をそれぞれ別に持っている。これを一つの型にし、native・virtual・注入の 3 経路が同じ型を使う。IME の driver（`ImeDriver`）は入れない: native では今どおり window ごとの `NativeInput.ime`（Windows の subclass と `preceding_event` が使う）が持ち主で、virtual では window ごとの入力の記録が持つ。`normalize` は IME の入力のとき、呼び出し側が渡す `&mut ImeDriver` を使う。

```rust
/// The input normalization of one window: the same for the native port, the
/// virtual backend and injected input. The IME driver stays with its owner
/// (the native window's `NativeInput`, the virtual window's input record).
#[derive(Default)]
pub struct WindowInput {
    mouse: Mouse,
    modifiers: ModifierState,
    clicks: ClickCounter,
    contacts: Contacts,
    keys: KeyPipeline,
    dnd: DropSession,
}

impl WindowInput {
    pub fn new() -> Self;   // Self::default()
    /// Normalize one injected or raw input into the events to deliver, in order
    /// (each already passed through `platform`, so clicks are counted once).
    /// `double_click` is the platform's double-click time and distance, as
    /// `ClickCounter::count` takes it.
    pub fn normalize(&mut self, input: InjectedInput, now_ns: u64, double_click: (u64, (f64, f64)),
        ime: &mut ImeDriver) -> Vec<ShellEvent>;
    /// The mouse event of `phase` at `position` (`None`: where the mouse last was), as
    /// `Mouse::event` makes it, already passed through `platform`; `None` while the
    /// mouse has no position.
    pub fn mouse(&mut self, phase: kivact_types::PointerPhase, position: Option<kivact_types::Point>,
        now_ns: u64, double_click: (u64, (f64, f64))) -> Option<ShellEvent>;
    /// The platform's held modifiers: `ShellEvent::Modifiers` only when they changed
    /// (`ModifierState::update`).
    pub fn modifiers_changed(&mut self, modifiers: kivact_types::Modifiers) -> Option<ShellEvent>;
    /// Stamp modifiers on any other platform event (key, wheel, touch, file, IME,
    /// Redraw, ...) and count its clicks, returning it to deliver. Call it once per event:
    /// what `mouse`, `normalize` and `focus_lost` return has passed through it already.
    pub fn platform(&mut self, event: ShellEvent, now_ns: u64, double_click: (u64, (f64, f64))) -> ShellEvent;
    /// The window lost focus: release keys, contacts and the mouse buttons, and
    /// return the release events to deliver.
    pub fn focus_lost(&mut self) -> Vec<ShellEvent>;
    /// The mouse position inside the window (`Mouse::window_position`); `None` outside it.
    pub fn mouse_position(&self) -> Option<kivact_types::Point>;
    /// The last mouse position inside or outside the window (`Mouse::position`).
    pub fn last_mouse_position(&self) -> Option<kivact_types::Point>;
    pub fn modifiers(&self) -> kivact_types::Modifiers;
    /// The drop session the native port feeds platform drag events into.
    pub fn dnd_mut(&mut self) -> &mut DropSession;
    /// The contact allocator the native port feeds touch events into.
    pub fn contacts_mut(&mut self) -> &mut Contacts;
    /// The key pipeline the native port feeds keyboard events into.
    pub fn keys_mut(&mut self) -> &mut KeyPipeline;
}
```

作業指示書に、今の `Runner::input`・`preceding_event`・`script::enqueue`・`Script::raw` のどの行がどの `WindowInput` の method に移るかを書く。

### 3.4 native（`arvane-native`）

- `port/winit/runtime.rs` は 1000 code 行の上限に近い（約 954 行）。新しい処理はここへ足さず、次の新しい leaf module に置く。module の参照が循環しないよう、型の置き場所を次のとおりにする（xtask は module 間の参照の循環を拒む）:
  - `port/winit/window.rs`（新規）: 今 `runtime.rs` にある `WinitWindow`（90-112 行）と `Surface`（78-86 行）とその `impl`（`ShellWindow` の実装を含む）をここへ移す。他の 3 つの新しい module と `runtime.rs` はこれを使う。
  - `port/winit/open.rs`: `pub(super) fn create_window(el: &ActiveEventLoop, request: &WindowRequest, parent: Option<&WinitWindow>, waker: &ShellWaker, started: Instant) -> Result<WinitWindow, ShellError>`。今の `resumed` の window 作成部分（属性、位置、`NativeAccessibility::new`、`NativeInput::new`、monitor、subclass、frame period）をここへ移す。mode と placement の扱いもここ。
  - `port/winit/windows.rs`: `pub(super) struct WindowTable { by_id: BTreeMap<WindowId, NativeWindow>, ids: HashMap<winit::window::WindowId, WindowId>, pending_show: Vec<WindowId>, placement: bool }` と、`NativeWindow { window: WinitWindow, input: WindowInput, injected: VecDeque<(Vec<InjectedInput>, RequestId)> }`。`placement` は `resumed` で一度だけ Wayland の判定（`display/placement.rs:30-36`。`ActiveEventLoop` が要る）をした結果で、`Capabilities.window_placement` に使う。
  - `port/winit/shell_loop.rs`: `ShellLoop` の実装 `NativeLoop<'a> { el: &'a ActiveEventLoop, table: &'a mut WindowTable, started: Instant, waker: &'a ShellWaker }`。
  - `open`・`windows` を使うのは `runtime.rs` と `shell_loop.rs`、`shell_loop` を使うのは `runtime.rs` だけ（新しい module は `runtime.rs` を参照しない。逆向きの参照を作らない）。
- `Runner` の field を整理する: `run: Run` を `phase: LoopPhase` に置き換える。window の表は phase の payload に入れる（表が意味を持つのは Running と Stopped だけなので、S-3）: `enum LoopPhase { Starting, Running(WindowTable), Stopped { table: WindowTable, failure: Option<ShellError> }, Unopened(ShellError) }`。`mouse`・`modifiers`・`dnd`・`clicks`・`contacts`・`keys` は `NativeWindow.input: WindowInput` へ移す（`WinitWindow.input: Rc<NativeInput>` はそのまま残る。こちらは OS の設定・IME・mailbox の事実で、`WindowInput` とは別の事実）。
- `window_event` は `table.ids` で `WindowId` を引き、知らない id は捨てる。その window の `WindowInput` で正規化し、`handler.event(EventTarget::Window(id), event, &mut loop)` を呼ぶ。マウス・modifier の platform event は `WindowInput::mouse`・`modifiers_changed` を通し、それ以外の window の event だけを `platform` に一度通す（正規化した event をもう一度 `platform` に通すと click を二重に数える）。
- `user_event` は全 window について、file drag・環境の変化・a11y action を処理する。
- 注入された入力（`injected`）は、`callback` が返った直後（`pending_show` を処理するのと同じ場所）に、各 window の `injected` が空になるまで取り出して配る。その window の入力を `WindowInput::normalize` に通し（`Accessibility` は window の a11y tree で検証してから配る。3.3）、batch の最後に `ShellEvent::Injected { request }` を配る。配った handler がさらに `inject` したら、同じ場所で続けて配る。loop を起こす専用の bit は作らない。
- `open_window` は `create_window` を呼び、`pending_show` に積む。`callback` が返った後、`pending_show` の window を `set_visible(true)` にする（a11y adapter が先にあること、という今の規則を保つ）。
- mode（Windows）: `Tool` は `with_owner_window(親 HWND)` と `with_skip_taskbar(true)`。`Modal` は `with_owner_window(親 HWND)` と、開いている間 `親.set_enable(false)`（閉じる前に `set_enable(true)`）。X11: `Tool` は `with_x11_window_type(vec![WindowType::Utility])`、`Modal` は `vec![WindowType::Dialog]`。Wayland: mode は `Normal` と同じに扱う。入力の modal 性は backend に頼らず engine でも守る（3.7 節）。
- placement: `display/placement.rs` に `pub(crate) fn place_relative(parent_outer: Rect, size: Size, work_area: Rect, placement: WindowPlacement) -> Option<Point>` を足す（`NextToParent` は親の右、入らなければ左、どちらも入らなければ `None` = Auto）。Wayland は今と同じく位置を指定しない。
- `WinitWindow::focus()` は winit の `Window::focus_window()` を呼ぶ。
- 終了時: `run_app` が返った後、`handler.close(&mut loop)` を一回呼ぶ（今の「一つの window を閉じる」処理を置き換える）。

### 3.5 virtual（`arvane-virtual`）

- `VirtualShell` を、loop 全体の状態 `VirtualPlatform`（clipboard、dialogs、opened、revealed、responders、fail_next、saved_state、capabilities、now）と、window ごとの `VirtualWindow`（env、title、min_size、ime、ime_text、cursor、soft_keyboard、frame、gpu、display、headless a11y、occluded、failing、surface、cpu_frame、redraw、requested_sizes、focus_requests）に分ける。`ShellWindow` を実装するのは `VirtualWindow`（`VirtualPlatform` への `Rc` を持つ）。`VirtualBackend::new` は `platform: Rc<VirtualPlatform>` と空の window の表 `Rc<RefCell<BTreeMap<WindowId, VirtualWindow>>>` を作り、`Probe` はこの二つの `Rc` を持つ。main の `VirtualWindow` は `start` で mirror を開いた後に表示先（`Display::Mirrored` か `Display::Headless`）を決めて作り、表に入れる（表示先を後から書き換えない）。`Probe::window(id)` は start の前は `None`。
- `Input` を `VirtualInput { queue: VecDeque<Item>, clock: Clock, saved_state, windows: BTreeMap<WindowId, VirtualWindowInput> }`（loop に一つ）と、window ごとの `VirtualWindowInput { input: WindowInput, ime: ImeDriver }`（3.3 節）に分ける。`Running` の window の表と `VirtualInput.windows` は同じ window の集合で、`open_window`・`close_window` が一緒に足し・外す。`Item` は次のとおり:

```rust
pub(crate) enum Item {
    Respond(Responder), Time(u64), Clipboard(String), Dialog(Option<Vec<PathBuf>>), FailEffect(ShellError),
    /// Delivered with EventTarget::Loop: SaveState, Suspended, Resumed, MemoryWarning.
    LoopEvent(ShellEvent),
    /// Sets every open window's `env.pointer`, then `EnvironmentChanged` to each (ascending id).
    Pointer(kivact_types::PointerSettings),
    Window(WindowId, WindowItem),
}
pub(crate) enum WindowItem {
    Event(ShellEvent), Resize(Size), Scale(f64), Period(u64), Reduced(bool), Dark(bool),
    Locale(Option<String>), SafeArea(Insets), Surface(bool), Occluded(bool), FailPresents(u32),
    Action(u64, A11yRequest),
}
```

  `Item::Window` の window が表に無ければ（閉じた後）その項目は step のときに捨てる。`KeyboardInset` は `WindowItem::Event` としてその window の env に入れる。
- `Script` は loop 全体の操作（`saved_state`、`save_state`、`respond`、`set_now`、`set_clipboard`、`push_file_dialog`、`push_files_dialog`、`fail_next_effect`、`fail_next_effect_unsupported`、`pointer_settings`、`is_empty`）を持ち、`Script::window(&self, id: WindowId) -> WindowScript` が window ごとの入力（`input(event)`、`raw`、`mouse`、`contact`、`focus`、`accessibility_action`、`resize`、`set_scale`、`set_frame_period`、`set_reduced_motion`、`set_dark`、`set_locale`、`set_safe_area`、`set_keyboard_inset`、surface 操作、`set_occluded`、`fail_presents`）を持つ。今の `Script` の window ごとの method は `Script::window(WindowId::MAIN)` の method に移す（同名、同じ振る舞い）。`Script::input(event)` は残し、`SaveState`・`Suspended`・`Resumed`・`MemoryWarning` は loop の項目（`EventTarget::Loop`）として積み、それ以外の event は `Script::window(WindowId::MAIN).input(event)` と同じにする（今の試験の振る舞いを保つ）。`WindowScript::mouse(phase, position)` は `WindowInput::mouse` の event を積む。
- `Probe::window(&self, id) -> Option<WindowProbe>`（今の window ごとの読み取り。閉じた・知らない window は `None`。`Script::window(id)` は常に `WindowScript` を返し、閉じた window の項目は捨てられる）と `Probe::windows(&self) -> Vec<WindowId>`。`WindowProbe::has_window()` は今の `Probe::has_window` と同じ「その window が mirror の実 window に映っているか」。loop の読み取り（`now_ns`、`woken`、`clipboard`、`opened_paths`、`revealed_paths`）は `Probe` に残す。
- `Running` は同じ window の表を持つ。`Step::Redraw` は redraw を求めている全 window に id の昇順で `Redraw` を配る。
- `open_window` は `VirtualWindow` を作り（mirror が有効なら mirror window も開く。`TestDisplay` はすでに複数 window に対応している）、`AccessibilityActivated` をその window への項目として queue の末尾に積む（callback の中から handler は呼べない）。`close_window` は mirror を閉じて取り除き、その window に積まれた注入の項目を捨てる。
- 注入（`inject`）は `WindowInput::normalize` で正規化し（`Accessibility` は `Accessibility::request_action` で検証してから）、`Item::Window(id, WindowItem::Event(..))` として queue に積み、最後に `ShellEvent::Injected { request }` を積む。
- `VirtualLoop::capabilities()` は main の `VirtualWindow` の `ShellWindow::capabilities()`（`multi_window`・`window_placement` は `VirtualOptions.capabilities` の値）。

### 3.6 arvane

- `Driver<H>` は `presenters: BTreeMap<WindowId, Presenter>` を持つ。main window の presenter は `start` で作る。
- `arvane::Handler` を次に変える:

```rust
pub trait Handler {
    fn start(&mut self, windows: &mut Windows<'_>, waker: ShellWaker) -> Result<ShellControl, ShellError>;
    fn event(&mut self, target: EventTarget, event: ShellEvent, windows: &mut Windows<'_>) -> Result<ShellControl, ShellError>;
    fn close(&mut self, windows: &mut Windows<'_>);
}

/// The open windows with their presenters, during one callback.
pub struct Windows<'a> { shell: &'a mut dyn ShellLoop, presenters: &'a mut BTreeMap<WindowId, Presenter>, /* env, observer */ }

impl<'a> Windows<'a> {
    pub fn get(&mut self, id: WindowId) -> Option<Window<'_>>;
    pub fn ids(&self) -> Vec<WindowId>;
    /// Make a presenter (and start its GPU when preferred), then open the shell window.
    pub fn open(&mut self, request: WindowRequest) -> Result<(), ShellError>;
    /// Shut the presenter down, then close the shell window.
    pub fn close(&mut self, id: WindowId);
    /// Forward to `ShellLoop::inject`.
    pub fn inject(&mut self, id: WindowId, inputs: Vec<InjectedInput>, request: RequestId);
    pub fn now_ns(&self) -> u64;
    /// `Capabilities { gpu: arvane_compose::GPU_BUILT, ..self.shell.capabilities() }`:
    /// the same GPU override as today's `Window::capabilities`; `multi_window` and
    /// `window_placement` are the loop's values (`ShellLoop::capabilities`).
    pub fn capabilities(&self) -> Capabilities;
}
```

- `Wake(GPU_STEP)` は全 presenter に `prepare` を呼ぶ。`SurfaceDestroyed` は対象 window の presenter だけに。
- `Observer` は `Rc<RefCell<dyn FnMut(WindowId, &RendererChange)>>` に変え、すべての presenter の `Notes` が同じものを共有する（後から開く window の presenter にも渡せるように）。
- presenter の GPU の起動に要る waker は `ShellLoop::waker()` から得る（`Driver` に保存しない。後から埋める slot を作らない）。
- GPU device は window ごと（presenter ごと）のままにする。device の共有はこの変更では行わない（7 章）。
- `EventTarget`・`InjectedInput`・`Windows` は `arvane` の root から、`ShellLoop` は `arvane::spi` から再公開する（engine は `arvane` だけを通して使う）。

### 3.7 engine（`kivact-engine`）

構造:

- 今の `Model`（`model/aggregate.rs`）を「一つの window の model」として使う。中身の処理は基本的に変えない。
- App 全体で一つの事実は `Model` の外へ出す:
  - `events`: `EventSink`（新規。`Rc<RefCell<EventQueue>>` の newtype で、`push(&self, HostEvent)` などを `&self` で提供する）。各 `Model` は同じ `EventSink` の clone を持つ。`self.events.push(..)` の呼び出しはそのまま動く。
  - `assets`: `Rc<RefCell<ArvaneAssets>>` を全 `Model` で共有する（field の型を `RefCell<ArvaneAssets>` から変える）。
  - `writes`（revision と統計）: `AppModel` へ移す。
  - motion の時計: `Model.frame.motion` を消し、`AppModel` が持つ（下の「frame」）。
  - core の frame の要求: `Model.frame.frames` には engine 自身の要求だけを残し、core の要求は `AppModel.client_frames` にだけ置く（下の「frame」）。
- `EventSink` と、全 window の `Model` が共有する `Shared { events: EventSink, assets: Rc<RefCell<ArvaneAssets>> }` は `model/events.rs` に置く（`model/aggregate.rs` の `Model::new` と `model/app_model.rs` の両方が使うので、どちらか一方に置くと module の参照が循環する）。
- revision: 今の `Model::commit_revision`（`model/apply.rs:51-63`）は自分で revision を計算して event の列に境界を積む。共有の列では window ごとに呼べないので、`Model::prepare_commit(&mut self)`（pack child orders から reconcile drawables まで。境界を積まない）に変え、`Writer::commit` が全 window の `prepare_commit` → revision を 1 つ決めて `events.commit(revision)` を 1 回 → `writes.commit(revision)` → 全 window の `sync()` の順に行う。
- 新しい `AppModel`（`model/app_model.rs`。`impl_model` の規則により `model/` の下に置く）:

```rust
pub(crate) struct AppModel {
    windows: BTreeMap<WindowId, Model>,
    events: EventSink,
    writes: Writes,
    assets: Rc<RefCell<ArvaneAssets>>,
    /// Window changes the host performs after the turn, in order.
    window_changes: Vec<WindowChange>,
    /// The App's one motion clock (the runtime's "one clock"); `FrameTick.clock` carries it.
    motion: MotionClock,
    /// Whether the App's motion clock runs: it stops while no window can draw.
    clock: AppClock,
    /// The frames the core asked for; only the tick-source window's `Model::tick` reads them.
    client_frames: FrameRequests,
    /// The window the OS gives the keyboard focus (`WindowInfo.focused`).
    focused_window: Option<WindowId>,
}

pub(crate) enum WindowChange {
    Open(WindowRequest),
    Close(WindowId),
}

/// Whether the App's motion clock runs.
pub(crate) enum AppClock {
    Running,
    /// No window can draw since `since_ns` (host ns).
    Stopped { since_ns: u64 },
}
```

`AppModel` の field はすべて `pub(crate)`（host が直接読み書きし、`windows` と `client_frames` を同時に借りる）。

- 「node がどの window にあるか」は別の表を持たず、持ち主の `Model` から導く: `AppModel::window_of(node)` は `self.windows.iter().find(|(_, m)| m.tree.get(node).is_some()).map(|(id, _)| *id)`（window は数個）。exit の演出中の node も `Model` が tree に持つ間は見つかる。
- `Writer` と `Reader` は `Rc<RefCell<AppModel>>` を持つ（今の `Rc<RefCell<Model>>` を置き換える）。node を受け取る method は `window_of` で window を引いて、その `Model` に対して今と同じ処理をする。
- `Writer` の変更:
  - `create(&self, window: WindowId, id: NodeId, kind: NodeKind)`（window を足す）。
  - `insert` は親と子が同じ window であることを `expect("invariant: …")` で確かめる。
  - `open_window(&self, request: WindowRequest)`: その場で新しい `Model` を `windows` に入れ（core はこの turn の中でその window の node を作れる）、`WindowChange::Open` を積む。
  - `close_window(&self, id: WindowId)`: `WindowChange::Close` を積む（`Model` は host が turn の後で取り除く）。
  - `set_window(&self, window: WindowId, title: String, min_size: Option<Size>)`。
  - `effect(&self, request: RequestId, effect: HostRequest)` は形を変えない。window は `HostRequest::Platform { window, request }` と `HostRequest::Inject { window, .. }` の中にだけ書く（同じ事実を 2 か所に書かない）。node や window を持たない request の行き先は次のとおり（`effect` 以外の `Writer` の method も同じ）:
    - `Snapshot(node)`・`DrawableExport`・`SetDrawableGeometry`・`drawable_request`・`import_external`・`drop_external` は、その node／drawable／image を使う node の window の `Model`（GPU device は window ごとなので、別の window の image は使えない）。
    - `RegisterFont` は共有の asset に 1 回登録し、全 window の `Model` の `env.fonts` を更新する。
    - `ReducedMotion` は `Host.reduced_motion_override` に 1 回入れ、全 window の env に掛ける。
    - `forget_adopted_pointer`・`reserve_touch_gesture` は全 window の `Model`（pointer の id は window をまたいで一意）。
  - `blur(&self, window: WindowId)`。`focus(target)` は node から window を引く。
  - `request_frame(&self, at_ns: u64)` は `AppModel.client_frames` に入れる。host は、tick の源の window（下の frame の規則）の `Model::tick` にだけこれを見せる。
  - `commit` は全 window をまとめて一つの revision にする。
- `Host`（`host/pump.rs`）:
  - App 全体の field: `client`、`phase`、`app: Rc<RefCell<AppModel>>`、`engine`、`next_wake`、`close`、`reduced_motion_override`、`presence`、`frame_log`、`accessibility_noted`。
  - window ごとの field は新しい `HostWindow`（`host/window.rs`）へ: `last_present`、`environment`、`surface`、`soft_keyboard`、`ime_allowed`、`adapter`、`last_stats`、`mode: WindowMode`、`parent: Option<WindowId>`。`LastPresent`（`pump.rs:40-47`）と `Surface`・`Unavailable`（`pump.rs:100-108`）の定義も `host/window.rs` へ移す（`pump.rs` がそれを使う。逆の参照を作らない）。modal の子の数のような counter は持たない（下の modal の規則のとおり、毎回 `HostWindow` の `mode` と `parent` から導く）。
  - `Host { windows: BTreeMap<WindowId, HostWindow>, .. }`。
- `HostEvent` の変更（`contract/events.rs`）:
  - `Routed<T>` に `pub window: WindowId` を足す（Pointer、Wheel、File、Key の全部に効く）。
  - 次の variant に `window: WindowId` を足す（名前付き field の variant にする。今 tuple か unit の variant は次の形に変える）: `FocusChanged { window, node, region, visible }`、`HoverChanged { window, target }`、`MousePosition { window, position: Option<Point> }`、`FrameTick { window, now_ns, estimated_present_ns, clock }`、`EnvChanged { window, environment, surface_active }`、`CloseRequested { window }`、`HostFailure { window, revision, reason }`、`GpuAdapterChanged { window, adapter: Option<GpuAdapter> }`、`ModifiersChanged { window, modifiers: Modifiers }`。
  - 新しい variant: `WindowOpened { window: WindowId, environment: Environment, surface_active: bool }`、`WindowFailed { window: WindowId, reason: String }`、`InjectDelivered { window: WindowId, request: RequestId }`（`ShellEvent::Injected` から）、`AppMotion { drawable: bool, frame_period_ns: u64 }`（motion の入力が変わったとき 1 回。下の frame）。
- `HostRequest::Platform` を `Platform { window: WindowId, request: PlatformRequest }` に変える。
- 開く途中の window: core は open を求めた turn からその window を開いたものとして扱う（env は親の複製）。host は turn の後で実際に開く。host は、知らない id の `Close` を無視する。core は、すでに閉じた window を指す event（`window` を持つ variant と `Routed` の入力。`FrameTick` と `AppMotion` を除く）を捨てる。
- turn の後の処理（`pump` の最後）: `window_changes` を順に実行する。`Open` は `windows.open(request)`、成功したら `HostWindow` を作り `WindowOpened` を積む（新しい window の `Model` の環境の初期化は `start_window` 関数が `begin` と同じ読み方で行う）、失敗したら `Model` を取り除き `WindowFailed` を積む。`Close` は `windows.close(id)` と `Model`・`HostWindow` の削除。
- 入口の順: `Host::handle` の入口は (1) DevTools の起動キー、(2) pick（その window が pick 中なら）、(3) modal の親への入力を捨てる、(4) 今の処理、の順（(1)(2) は 4.4.2 の engine の DevTools の口。この節で作るのは (3) と (4)）。
- modal window: `mode == Modal` の window が開いている間、その親 window への Pointer・Wheel・Key・Ime・File・Accessibility（DevTools が注入する `InjectedInput::Accessibility` を含む）の入力を (3) で捨てる。「親に開いた modal の子があるか」は `Host.windows` の各 `HostWindow` の `mode` と `parent` から毎回導く（別の counter を持たない）。
- frame: motion の時計は App に一つ（runtime の「There is one clock」）。`Model.frame.motion` を消し、`AppModel.motion`・`AppModel.clock` に置く。host は `pump` の各回の最後に「描画できる window が一つでもあるか」（どれかの `Host::drawable(id)` が真）を毎回導き、`Running` → どれも描画できない で `Stopped { since_ns: now }`、`Stopped` → どれかが描画できる で `motion.offset_ns += now - since_ns; motion.resumed_ns = now; Running` にする。`FrameTick.clock` はこの `AppModel.motion`。window ごとの engine の transition の時刻ずらし（`shift_state_times` など）は今どおり window の `Model` が行う。host は App の motion の入力（描画できる window の有無、tick の源の window の `frame_period_ns`）が変わったとき `HostEvent::AppMotion { drawable, frame_period_ns }` を 1 回積む（`Host.app_motion: Sent<..>` で重複を省く）。tick の源は、描画できる window のうち id が最小のもの（毎回 `Host` が導く。保存しない）。`Model::tick(&mut self, now, client: Option<&mut FrameRequests>)` は、源の window にだけ `Some(&mut app.client_frames)` を渡す。源の `Model` は自分の期限と `client` の最小で tick し、`now` 以下の `client` の要求を消す。源でない `Model` は `None`（core の要求を見ない）。`FrameTick` は源の tick のときだけ host が積む（`Model::tick` は積むべき刻印を返す）。描画できる window が無い間、core の要求は `AppModel` に残る。host の `control` の frame の期限は、描画できる window の `Model` の期限（tick の源の window は `client_frames` を含む）の最小で、描画できる window が無ければ無い。core は `FrameTick.window` を motion には使わない。
- focus: OS の focus がある window は `AppModel.focused_window`（host が `ShellEvent::Focused(true/false)` で書く）を正本にし、`Reader::focused_window()` で読む（`WindowInfo.focused`）。`Focused(false)` で今の `key_down_targets` の消去と pointer の取り消しを、その window の `Model` についてだけ行う。

### 3.8 core（`kivact-core`）

- `ViewNodes`（`view/tree.rs`）: `root: NodeId` を `roots: BTreeMap<WindowId, NodeId>` に変える。window の root の持ち主は `ViewNodes.roots` だけ（`WindowRecord` に root・owner を持たない）。window の owner は `ViewNodes::owner_of(root)`（root の `CoreNode.owner`）で得る。`root_owner()` は field にせず、呼び出し元は `rt.root()`（App の root owner）を使う。`CoreNode` に window は持たない: node の window は mount 中は `Mcx.window`、それ以外は `ViewState::tree(rt).reader().window_of(id)`（engine の持ち主の `Model` が正本）。`ViewNodes::create(kind, owner, window)` は engine の `Writer::create(window, id, kind)` に window を渡す。
- `Mcx`（`view/descriptor/description.rs`）に `window: WindowId` を足す。`fragment()`・`host(kind)` は `self.window` で node を作る。`mount_in(rt, owner, window, build)`。flow の `Site` は `frag` の window を使う。
- `mount_root` を 2 つに分ける: App 全体の context を入れる `install_app_contexts(rt, owner, settings)` と、window の root を作り中身を mount する `mount_window_root(rt, window, owner, build, at)`。main window もこの関数で作る。
- window の記録: `view/state.rs` に（`EnvState`・`FocusKeys`・`ViewState` と同じ file。別の file にすると module の参照が循環する）`pub(crate) struct WindowRecord { declared_in: Owner, env: Rc<EnvState>, focus: FocusKeys, hover: Cell<PointerObservation<FocusTarget>>, mouse: Cell<PointerObservation<crate::geometry::Point>>, title: Value<String>, min_size: Option<Size>, dismiss: Option<Handler<DismissReason>>, open: Binding<bool>, mode: WindowMode }`（main の record は `declared_in` が root owner、`open` は常に true の binding、`title` は App の title、`min_size` は `App::min_window_size`）。`hover`・`mouse` は新しい window では `PointerObservation::Plain(None)` と `PointerObservation::Reactive(owner.signal(None, at))`（今の `ViewState::new` と同じ）。`ViewState.windows: RefCell<BTreeMap<WindowId, WindowRecord>>` と、id の採番 `next_window: Cell<u64>`（S-8 の ID）。`WindowScope` と `current_window` も `view/state.rs` に置く。`ViewState.min_window_size` は消す（main の `WindowRecord.min_size` を読む）。`view::pointer_position()` は呼び出した owner の window（`current_window`）の記録を読む。`modifiers` と `pointer_modality` は App に一つのまま `ViewState` に残す（どの window の `ModifiersChanged` も同じ値を書く）。
- capabilities: `ViewState.capabilities`（定数）と `capabilities_value` の 2 つをやめ、`host_capabilities: Signal<Capabilities>`（`HostInfo.capabilities` が初期値。host の event だけが書く）と、その上に作る `capabilities: Value<Capabilities>`（memo）にする。DevTools の touch emulation は memo の中で `host_capabilities` の上に当てる（`devtools_hooks::wrap_capabilities`。feature が無いときは `host_capabilities` をそのまま返す）。memo には何も書かない。読むところは untracked で読む。
- motion: `ViewState.app_motion: Signal<AppMotion>`（`AppMotion { drawable: bool, frame_period_ns: u64 }`。`HostEvent::AppMotion` が書く）を置く。`MotionEnv.surface_active` と `frame_period_ns` はこの signal の投影（memo）にする（main の `EnvState` から作らない）。`reduced_motion` は OS の設定なので main の env のまま。
- window の context: 各 window の root owner に `WindowScope(WindowId)` を `provide` する（main は root owner）。`pub(crate) fn current_window(owner) -> WindowId` は context から読む。`platform` の env と service はこれで window を決める。
- `window()` の view（`view/window_view.rs`、新規）: `Show` と同じく `open` を読む Internal effect を持つ。true になったら: id を採番し、宣言した owner の子 owner を先に作り、その owner で親 window の env の複製の `EnvState` と `FocusKeys` を作り、`Writer::open_window`、`mount_window_root`（子 owner）、`Writer::set_window`。false になったら（または owner の破棄で）: root の部分木を外し、`Writer::close_window`、`WindowRecord` を消す。
- `HostEvent::CloseRequested { window }`: main なら今どおり `rt.request_exit()`。それ以外は `WindowRecord.dismiss` を呼び、`Handled::No`（または `on_dismiss` が無い）のときだけ `open` に false を書く。
- `HostEvent::WindowOpened`: その window の `EnvState` を更新する。`WindowFailed`: 宣言した owner に `ErrorKind::Platform` を報告し、`open` に false を書く。
- `EnvChanged`・`FocusChanged`・`HoverChanged`・`MousePosition` は `window` の記録に対して処理する。`FrameTick` は今どおり App の motion に使い、`AppMotion` は `app_motion` に書く。
- `Inspect`（`launch/inspect.rs`）: `root()` を `roots() -> Vec<(WindowId, NodeId)>`（`ViewNodes.roots` から作る）に、`app_view()` を削除（呼び出し元なし）、`check_tree` は全 root から辿る。
- `key_route` の既定先は、その event の window の root の最初の子（無ければ root）。`escape_dismisses`・`modal_boundary`・`admitted` は event の window の model に問い合わせる（engine の Reader が node から window を引く）。
- trace（`client/trace.rs`）は全 root を辿る。
- `HostState` の統計はそのまま（App 全体）。

### 3.9 kivact-test

- `Screen` を「App と、その中の一つの window」を指す handle にする: `pub struct Screen { app: Rc<TestApp>, window: WindowId }`。`TestApp` に今の `Screen` の中身（host、launch、record、actions、name、at、artifacts、contacts）を移す。`Screen` は `Clone`。`Contact` は `window: WindowId` を持つ（`contact_*` はその window へ送り、handle の window と違えば `self.fail("contact belongs to another window")`）。
- 新しい method:
  - `pub fn window(&self, title: &str) -> Screen` — その title の window が開くまで仮想時間を進めて待つ（`find_by_*` と同じ時間上限）。見つからなければ、開いている window の title の一覧を出して失敗する。その title の window が 2 つ以上あれば `get_by_*` と同じく失敗する（"2 windows are titled …"）。
  - `pub fn windows(&self) -> Vec<Screen>` — 開いている window（id の昇順）。
  - `pub fn is_open(&self) -> bool` — この handle の window がまだ開いているか。
- 既存の method は `self.window` を対象にする。`render`・`render_with` が返す `Screen` は main window を指す。App 全体の method（`errors`、`trace`、`stats`、`advance`、`run_until_idle`、`now`、`wait_for`）はどの handle から呼んでも同じ。`script()` は handle の window に結び付いた `Script` を返す（window ごとの method は `arvane_virtual::Script::window(self.window)` へ、loop の method は今どおり）。`has_window()` は `is_mirrored()` に改名する（「その window が mirror の実 window に映っているか」。`windows()`・`window()` と紛らわしいため）。
- `window()` の戻り値は一時値なので、そこから取った `Element` を `let` に入れると E0716 になる。window の handle は先に変数に入れる（`let settings = screen.window("Settings"); settings.get_by_role(..).click();`）。ガイドに 1 文書く。
- 閉じた window の handle で window 向けの method を呼ぶと、「window <id> is closed」で失敗する（title は閉じた後に読めない）。
- 失敗時の成果物は window ごとに `<test>-w<id>.png` と `.txt` を書く（main は今どおり `<test>.png`）。
- `ArvaneTestHost`（engine の `host/testing/test_host.rs`）は window ごとの操作に `WindowId` を受け取る（`semantics(window)`、`frame(window)`、`hit(window, point)`、`raw_input(window, ..)`、`mouse(window, ..)`、`send(window, event)`、`request_close(window)`、`scale(window)` など）。
- `Facts::new(inspect, window)` はその window の root から辿る。

### 3.10 embed

- `port/embed`（win32・x11）の loop は `open_window` に `ShellError::unsupported("windows beyond the embedded editor")` を返す。`Capabilities.multi_window = false`。
- `ShellLoop` を実装する（window は一つだけ。`windows()` は `[WindowId::MAIN]`）。実装は `port/embed/session.rs` に一つだけ作り、win32・x11 の両方が使う（`EmbedLoop`）。
- `inject` は入力を捨て、`Injected { request }` だけを callback が返った後に配る。core は embed の target の `inject_input` に `code::UNSUPPORTED` を返す（DevTools の入力の注入は embed では行わない。embed の port を `WindowInput` に載せ替える作業はこの変更に含めない。7 章）。
- `focus()` は `ShellError::unsupported`。

### 3.11 新しく導入する状態（すべて明示的な型）

| 持ち主 | 型 | 状態 |
|---|---|---|
| native `Runner` | `LoopPhase` | `Starting`、`Running(WindowTable)`、`Stopped { table, failure }`、`Unopened(err)` |
| native `WindowTable` | 表の中の有無 | 開いている window だけが入る。閉じた window の状態は持たない |
| virtual `Running`・`VirtualInput` | 2 つの表の有無 | 同じ window の集合（`open_window`・`close_window` が同時に変える） |
| engine `Host` | `HostWindow` の有無 | 開いて `WindowOpened` を出した window だけが入る |
| engine `AppModel` | `window_changes` | turn の後で実行する変更の列（Mailbox と同じ性質の一回きりの列） |
| engine `AppModel` | `AppClock` | `Running`（どれかの window が描画できる）、`Stopped { since_ns }` |
| engine `AppModel` | `client_frames` | core が求めた frame の時刻（tick の源の window が消す。源が変わっても要求は失われない） |
| core `WindowRecord` | 表の中の有無 | `open` が true で mount した window だけが入る |
| core の `window()` の view | `enum WindowArm { Closed, Open { id: WindowId } }` | root と owner は `ViewNodes` にだけ持つ（二重に持たない） |

開いている途中・閉じている途中という中間の状態は作らない。core は open を求めた turn から開いたものとして扱い、実際に OS の window を開くのは host の turn の後の処理である（3.7 の「開く途中の window」の規則で、失敗と遅れて届く event を扱う）。

### 3.12 E2E の契約（`dev/tests/kivact-public-api/tests/e2e/windows.rs`、新規）

次の振る舞いをそれぞれ試験にする（試験の名前は振る舞いの文にする）:

1. `open` を true にすると title の window が開き、`screen.window(title)` で中身を検索できる。
2. `open` を false にすると window が閉じ、handle の `is_open()` が false になる。
3. 宣言した `show` の枝が消えると window も閉じる。
4. 中身が宣言した場所の context（`provide`、theme、locale）を引き継ぐ。
5. 利用者の閉じる操作（`screen.window(title).request_close()`）で、`on_dismiss` が `()` を返すか未指定なら `open` が false になり、`Handled::PreventDefault` を返すと開いたまま、`Err` を返すと開いたまま error が報告される。
6. main window の閉じる操作は App を終了する（既存の契約を保つ）。
7. window ごとに focus と Tab 移動が独立している。
8. `mode(WindowMode::Modal)` の window が開いている間、親 window の click が handler に届かない。支援技術の Activate（`accessibility_action`）も親に届かない。
9. `platform::window_size()` が、呼び出した window の大きさを返す。
10. `RenderOptions::single_window()` の構成では `ErrorKind::Unsupported` が報告され、window は開かない。
11. 2 つの window で同じ signal を読む text が、同じ turn で両方更新される。
12. main の window の surface を外して（`Script::window(WindowId::MAIN)` の surface 操作）も、開いている別の window の `animated` は進み続け、main の surface を戻したときに値が巻き戻らない。

`platform::focus_window()` は virtual では記録するだけなので、E2E の契約にしない（`arvane` の embed の試験が「embed は `Unsupported`」を確かめ、実機の動きは手動確認の表に載せる）。

---

## 4. DevTools

### 4.1 構成と crate

```
target アプリ（facade feature "devtools"）
  kivact-core の devtools module（Call を自分で処理する。core・engine・runtime の内部を読む）
    └─ kivact-devtools-agent（server: TCP、thread、discovery、設定、review と trace の保存、DevTools アプリの起動、PNG と base64）
          ⇅ JSON-RPC 2.0（loopback TCP、1 行 1 message、token）
    ├─ kivact-devtools      （DevTools アプリ。Kivact + kivact-ui-basic）── kivact-devtools-client
    ├─ kivact-devtools-mcp  （MCP stdio server）                          ── kivact-devtools-client
    └─ python/kivact-devtools（Python package。標準ライブラリだけ）
すべての client と agent は kivact-devtools-protocol の型を共有する（Python は生成した型）
```

| crate | 場所 | 種類 | 依存（workspace 内） | 境界の理由 |
|---|---|---|---|---|
| `kivact-devtools-protocol` | `crates/kivact-devtools-protocol` | lib | なし（外部は serde、serde_json） | 4 者（agent・アプリ・MCP・Python 生成）が共有する通信の語彙。I/O を持たない純粋な値と純粋関数だけ（review の Markdown 化と locator、Chrome trace の JSON 化、discovery directory の計算、error code、日時の文字列） |
| `kivact-devtools-agent` | `crates/kivact-devtools-agent` | lib | protocol（外部は serde_json、png、tracing） | socket・thread・file・process を扱う server。Kivact の内部を知らない。core は `take_incoming` で受け取った `Call` を自分で処理し、`Responder::send` で答える。core が socket・JSON・PNG・file を知らずに済む |
| `kivact-devtools-client` | `crates/kivact-devtools-client` | lib | protocol | discovery の一覧・接続・型付きの呼び出し・通知の受信。アプリと MCP が共有し、二重実装を避ける |
| `kivact-devtools` | `crates/kivact-devtools` | bin | kivact（`default-features = false`）、kivact-ui-basic、client、protocol | DevTools の GUI。公開 API と kit だけで作る（P-07 の dogfooding）。パネルは module |
| `kivact-devtools-mcp` | `crates/kivact-devtools-mcp` | bin | client、protocol | MCP の仕様への追従と stdio。GUI を link しない |

Python package は `python/kivact-devtools/`（crate ではない）。protocol の生成器は `dev/xtask` の新しい module（4.2.8）。

feature の配線:

- `crates/kivact-core/Cargo.toml`: `devtools = ["dep:kivact-devtools-agent", "dep:kivact-devtools-protocol"]`（両方 optional 依存）。
- `crates/kivact/Cargo.toml`: `devtools = ["kivact-core/devtools"]`。
- `kivact-core/src/lib.rs`: `#[cfg(feature = "devtools")] mod devtools;`（公開しない。facade からも再公開しない。アプリ作者が書くコードは無い）。
- アプリ作者は自分の `Cargo.toml` に `[features] devtools = ["kivact/devtools"]` を書き、`cargo run --features devtools` で動かす（release ビルドに agent を入れないため。利用者ガイドの手順）。
- `[visible_to]`: `kivact-devtools-protocol = ["kivact-devtools-agent", "kivact-devtools-client", "kivact-core", "kivact-devtools", "kivact-devtools-mcp", "kivact-public-api-tests"]`、`kivact-devtools-agent = ["kivact-core"]`、`kivact-devtools-client = ["kivact-devtools", "kivact-devtools-mcp", "kivact-devtools-agent", "kivact-public-api-tests"]`（agent と E2E は client を dev-dependency で使う）。2 つの binary は誰も依存しないので、アプリ・例の並びに `kivact-devtools = []`、`kivact-devtools-mcp = []` を書く。
- `[module_rules.kivact-core.uses]`: `devtools = ["task", "motion", "style", "state", "geometry", "sync", "view", "platform", "diagnostics", "client"]` を足し、`launch` の値の列に `"devtools"` と `"geometry"`、`app` の値の列に `"geometry"` を足す（`geometry` は `check_window_sizes` を移すため。3.1 の 2.）。`client` の列には `"devtools"` を足さない（`client` は `devtools` を参照せず、`view/devtools_hooks.rs` の hook を呼ぶ）。`view` の列には足さない（`view` は `devtools` を参照しない、をこの規則で機械的に守る）。`devtools` は `launch` を参照しない（`ClientMode`・`AppSettings`・`Inspect` を使わない。4.4.1）。
- DevTools アプリ自身は `devtools` feature を付けずにビルドする（自分自身を調べない）。

### 4.2 protocol

#### 4.2.1 transport

- TCP の `127.0.0.1`。port は OS が選ぶ（bind に `127.0.0.1:0`）。
- 1 行に 1 つの JSON（UTF-8、区切りは `\n`）。1 行の上限は 64 MiB。超えたら error（`INVALID_REQUEST`）を返して接続を閉じる。
- JSON-RPC 2.0。request は `{"jsonrpc":"2.0","id":<number|string>,"method":"<name>","params":{...}}`、response は `result` か `error`、agent からの通知は `id` の無い `{"jsonrpc":"2.0","method":"<name>","params":{...}}`。
- 最初の request は必ず `initialize`。agent は次の順に確かめる: `params.token` が discovery file の token と違えば `BAD_TOKEN` を返して閉じる。`params.protocol` が `PROTOCOL_VERSION` と違えば `VERSION_MISMATCH`（message に target の版と client の版）を返して閉じる。`initialize` の前の他の method は `NOT_INITIALIZED`（接続は閉じない。10 秒以内に `initialize` が来ない接続は agent が閉じる）。client は `InitializeResult.protocol` も確かめ、違えば `ClientError::Protocol`／`DevtoolsError` にする。
- 版の規則: wire の型の field の削除・改名・型の変更、method・通知の削除・改名は `PROTOCOL_VERSION`（`app.rs` の `pub const PROTOCOL_VERSION: u32`）を上げる。method・通知・`Option`／`Vec` の field の追加は上げない。生成器は `PROTOCOL_VERSION` の値を schema の `x-protocol` と Python の `PROTOCOL_VERSION` に写す。
- 1 つの target に同時に複数の client が接続できる（DevTools アプリ・MCP・Python）。target に掛ける状態（highlight、pick、overlays、forced states、override、emulation、時間の倍率、記録）は target に一つで、全 client が共有する（後の書き込みが勝つ）。

error code（名前は `crates/kivact-devtools-protocol/src/code.rs` の `pub const <名前>: i32`。ほかの task は `code::NODE_NOT_FOUND` のように書く）:

| 名前 | code | 意味 |
|---|---|---|
| `PARSE_ERROR` | -32700 | JSON として読めない |
| `INVALID_REQUEST` | -32600 | JSON-RPC として不正 |
| `METHOD_NOT_FOUND` | -32601 | 知らない method |
| `INVALID_PARAMS` | -32602 | params が不正 |
| `INTERNAL_ERROR` | -32603 | agent の内部の失敗 |
| `BAD_TOKEN` | -32001 | token が違う |
| `NOT_INITIALIZED` | -32002 | まだ initialize していない |
| `VERSION_MISMATCH` | -32003 | protocol の版が違う |
| `NODE_NOT_FOUND` | -32010 | node が無い（unmount 済みを含む） |
| `WINDOW_NOT_FOUND` | -32011 | window が無い |
| `NO_MATCH` | -32012 | query が 0 件（`get_*`、`find_*`） |
| `MULTIPLE_MATCHES` | -32013 | query が 2 件以上（`get_*`、`find_*`） |
| `TIMED_OUT` | -32014 | 時間切れ（`find_*`） |
| `UNSUPPORTED` | -32015 | この target では使えない（例: `multi_window` が無い、GPU が無い、embed の target の `inject_input`（`-32015`。core が入力を渡す前に断る）） |

`RpcError::new(code: i32, message: impl Into<String>) -> RpcError`（`data` は `None`）。

#### 4.2.2 discovery

- directory: 環境変数で決める（`cfg` を使わない）。`LOCALAPPDATA` があれば `%LOCALAPPDATA%\kivact\devtools`、なければ `XDG_RUNTIME_DIR` があれば `$XDG_RUNTIME_DIR/kivact/devtools`、なければ `HOME` があれば `$HOME/.cache/kivact/devtools`、なければ `std::env::temp_dir()/kivact-devtools`（空の値は無いものとして扱う）。計算は protocol crate の純粋関数 `discovery_dir_from(var, temp_dir)` と `discovery_file_name(pid, port)`（環境変数と temp dir は呼び出し側が渡す）。agent と client はこの関数に `std::env::var_os` と `std::env::temp_dir()` を渡す。Python だけは同じ規則の写しを持ち、試験で同じ 4 つの場合を確かめる。
- file: `<pid>-<port>.json`（一つの process に複数の App があっても port で区別できる）。中身は `DiscoveryFile`（4.2.4）。`<pid>-<port>.json.tmp` に書いてから rename する。directory は agent が最初に書くときに作る。
- agent は終了時に自分の file を消す。client は一覧を作るとき、各 file の port へ 100ms の上限で接続を試し、接続を拒まれた（`ConnectionRefused`）file を消して一覧に入れない（時間切れの file は消さずに一覧から外す）。
- agent を起動する条件（core が判断する）: (1) `devtools` feature があり、(2) 環境変数 `KIVACT_DEVTOOLS` が `0` ではなく、(3) process の実行 file の名前（拡張子を除く）が `kivact-devtools` ではなく（DevTools アプリ自身を調べない。`--all-features` のビルドでも DevTools アプリに agent が入らないように）、(4) App が native の window（`App::run`）か embed（`App::embed`）で動いている。`kivact::test` の App では、`RenderOptions::devtools()`（feature `devtools` のときだけある）を指定したときだけ起動する（他の試験の振る舞いを変えないため）。
- 環境変数 `KIVACT_DEVTOOLS_TARGET` に discovery file の path を入れると、client はその target に接続する（DevTools アプリが Scripts を実行するときに設定する。利用者も使える）。

#### 4.2.3 型の書き方の規則（生成器が読める形）

protocol の型はすべて `crates/kivact-devtools-protocol/src/` の leaf module に、次の形だけで書く。生成器（4.2.8）はこれ以外の形を見つけると失敗する。

1. 名前付き field の `pub struct`（すべての field が `pub`）。`#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]`（設定の型 `KeyConfig`・`DevtoolsConfig` だけは `Default` も derive する。生成器は derive の中身を見ない）。
2. unit variant だけの `pub enum`。`#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]` と `#[serde(rename_all = "snake_case")]`。
3. 名前付き field の variant（または unit variant）の `pub enum`。`#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]` と `#[serde(tag = "kind", rename_all = "snake_case")]`。
4. `pub type <Name> = u64;`（id の別名。`NodeId`、`WindowId`、`OwnerId`）。
5. field の型は次の文法だけ: `bool`、`u32`、`u64`、`i64`、`f64`、`String`、`Option<T>`、`Vec<T>`、この crate の型の名前（`T` はこの文法の型。`Option<Vec<String>>` も書ける）。map、tuple、generic、`Box` は使わない。
6. `Option<T>` の field には `#[serde(default, skip_serializing_if = "Option::is_none")]`、`Vec<T>` の field には `#[serde(default, skip_serializing_if = "Vec::is_empty")]` を付ける。
7. すべての型と field に英語の `///` を付ける（Python の docstring と JSON Schema の description になる）。
8. derive は `use serde::{Deserialize, Serialize};` の後に裸の名前で書く（xtask の trusted derive の規則）。
9. serde の attribute は 2・3・6 の形と、`DevtoolsConfig.schema` の `#[serde(default, rename = "$schema", skip_serializing_if = "Option::is_none")]` の 1 つだけ。`rename`・`flatten`・`skip`・`untagged`・`alias` など、それ以外の serde の attribute と、必要な attribute が無い enum・field は、生成器が file と型と field の名前を付けて error にする（serde が実際に送る形と schema・Python がずれないように）。
10. method と通知の variant 名と marker 名は、wire 名を `_` で切り、各語の先頭だけを大文字にしてつなぐ（`get_a11y_tree` → `GetA11yTree`、`get_ime_state` → `GetImeState`、`get_by_test_id` → `GetByTestId`）。

生成器が読まない file（wire の型ではない）: `lib.rs`、`rpc.rs`、`time.rs`、`review_markdown.rs`、`discovery.rs`、`trace.rs`、`schema.rs`。`code.rs` は型ではなく `pub const NAME: i32 = N;` だけを読む（4.2.8）。`app.rs` の `PROTOCOL_VERSION` の値も読む。

名前の規則（VOC-3）: protocol の型は Kivact の型と同じ名前にしない。`kivact::interaction::TreeNode` と区別するため protocol では `ElementInfo`、core の style の層 `StyleLayer` と区別するため `StyleLayerInfo`、runtime の `Stage` と区別するため `TurnStage` と書く。DevTools アプリは `use kivact_devtools_protocol as p;` で `p::Role` のように protocol の型を書く（prelude の `Role`・`Rect`・`Size` とぶつけない）。

#### 4.2.4 型の一覧

以下は確定した型である。作業指示書のとおりにそのまま書く。説明の英語 doc comment は作業指示書に書く。

`common.rs`:

```rust
pub type NodeId = u64;
pub type WindowId = u64;
pub type OwnerId = u64;
pub struct SourceLocation { pub file: String, pub line: u32, pub column: u32 }
pub struct Point { pub x: f64, pub y: f64 }
pub struct Size { pub width: f64, pub height: f64 }
pub struct Rect { pub x: f64, pub y: f64, pub width: f64, pub height: f64 }
pub struct Empty {}
/// Mirrors kivact_types::Role variant by variant, in the same order.
pub enum Role {
    Generic, Group, Text, Heading, Button, CheckBox, ComboBox, ListBox, ListItem, Option, TextBox, Image,
    Slider, ProgressBar, Alert, Status, List, Dialog, AlertDialog, Menu, MenuItem, Tab, TabList, ScrollView,
    Canvas, Link, Toolbar, Tooltip, RadioGroup, RadioButton, Switch, TabPanel, Splitter, Table, Row, Cell,
    ColumnHeader, Tree, TreeItem, Meter, SpinButton, MenuBar, MenuItemCheckBox, MenuItemRadio, Separator,
    Grid, GridCell, RowGroup, Navigation, Log, Article, Main, Region,
}
/// Mirrors kivact_types::A11yAction variant by variant, in the same order.
pub enum A11yAction { Activate, Focus, Increment, Decrement, Expand, Collapse, Toggle, Select, ScrollIntoView,
    SetValue, ShowMenu, Dismiss }
pub enum NodeKind { Root, Box, Text, Image, Overlay, Contents }
pub enum InteractiveState { Hovered, Pressed, Focused, FocusVisible, Disabled, Selected }
pub enum MouseButton { Primary, Secondary, Middle }
pub enum WindowMode { Normal, Tool, Modal }
pub enum BuildKind { Debug, Release }
```

`Role`（`crates/kivact-types/src/semantics/values.rs`）と `A11yAction`（同じ file）の variant は、書く前にその file で確かめて同じ順・同じ名前で写す（上の列は基準 commit で確かめた）。`MouseButton` は `kivact_types::MouseButton` の最初の 3 つと同じ名前（`Back`・`Forward` は注入しない）。`Modifier` の型は無い（modifier は chord の文字列で表す）。

`app.rs`:

```rust
/// The wire version; see 4.2.1 for when to bump it.
pub const PROTOCOL_VERSION: u32 = 1;
pub struct AppInfo { pub name: String, pub pid: u64, pub kivact_version: String, pub build: BuildKind,
    pub exe: String, pub cwd: String, pub workspace_root: String }
pub struct WindowInfo { pub id: WindowId, pub title: String, pub bounds: Rect, pub scale: f64,
    pub focused: bool, pub mode: WindowMode, pub parent: Option<WindowId> }
pub struct WindowList { pub windows: Vec<WindowInfo> }
pub enum ClientKind { Devtools, Mcp, Python, Other }
pub struct ClientInfo { pub name: String, pub version: String, pub kind: ClientKind }
pub struct InitializeParams { pub protocol: u32, pub token: String, pub client: ClientInfo }
pub struct InitializeResult { pub protocol: u32, pub app: AppInfo }
pub struct DiscoveryFile { pub protocol: u32, pub pid: u64, pub app_name: String, pub kivact_version: String,
    pub port: u32, pub token: String, pub started_unix_ms: u64, pub workspace_root: String }
pub enum Topic { Tree, Turns, Frames, Pick, Errors, Devtools, Scripts, Windows }
pub struct SubscribeParams { pub topics: Vec<Topic> }
```

`PROTOCOL_VERSION` の `const` は型ではないので生成器は 4.2.3 の形としては読まないが、値だけは読む（B4）。`Topic::Focus` は無い（focus の変化を知らせる通知は作らない。`get_focus` で読む）。DevTools アプリは `initialize` の直後に全 topic を購読する。

`tree.rs`:

```rust
pub struct ElementInfo { pub id: NodeId, pub window: WindowId, pub parent: Option<NodeId>,
    pub children: Vec<NodeId>, pub kind: NodeKind, pub role: Option<Role>, pub name: Option<String>,
    pub text: Option<String>, pub test_id: Option<String>, pub source: Option<SourceLocation>,
    pub hidden: bool, pub leaving: bool, pub update_count: u32 }
pub struct GetTreeParams { pub window: WindowId, pub root: Option<NodeId>, pub depth: Option<u32> }
pub struct TreeSnapshot { pub window: WindowId, pub root: NodeId, pub revision: u64, pub nodes: Vec<ElementInfo> }
pub struct TreeChanged { pub window: WindowId, pub revision: u64, pub upserted: Vec<ElementInfo>, pub removed: Vec<NodeId> }
```

`WindowInfo.mode` は `kivact::view::WindowMode` と同じ variant の `WindowMode`（`.mode(WindowMode)` の builder の値）。`WindowInfo.bounds` は window の大きさだけを表す（`x` と `y` は常に 0。位置は公開しない。D-11）。`ElementInfo.text` は Text node の内容の先頭 80 文字。`update_count` は記録を始めてから、その node の prop が変わった回数（記録していなければ 0）。

`node.rs`:

```rust
pub struct GetNodeParams { pub node: NodeId }
pub struct BoxModel { pub margin: Rect, pub border: Rect, pub padding: Rect, pub content: Rect }
pub struct CallFrame { pub location: SourceLocation, pub app_code: bool }
pub enum StyleOrigin { Devtools, State, Selected, Disabled, Caller, Recipe, Base, ThemeDefault, Inherited }
pub struct StyleProperty { pub name: String, pub value: String, pub overridden: bool }
pub struct StyleLayerInfo { pub origin: StyleOrigin, pub name: Option<String>, pub source: Option<SourceLocation>,
    pub properties: Vec<StyleProperty> }
pub struct SemanticsInfo { pub role: Option<Role>, pub name: Option<String>, pub description: Option<String>,
    pub value: Option<String>, pub states: Vec<String>, pub actions: Vec<A11yAction>, pub labelled_by: Vec<NodeId>,
    pub described_by: Vec<NodeId>, pub controls: Vec<NodeId>, pub focusable: bool }
pub struct HandlerInfo { pub event: String }
pub struct WhyDep { pub kind: String, pub ty: String, pub created_at: SourceLocation,
    pub name_hint: Option<String>, pub last_write: Option<SourceLocation>, pub last_write_turn: Option<u64>,
    pub reads: Vec<WhyDep> }
pub struct WhyProp { pub prop: String, pub effect_at: SourceLocation, pub reads: Vec<WhyDep> }
pub struct NodeDetail { pub node: ElementInfo, pub bounds: BoxModel, pub call_chain: Vec<CallFrame>,
    pub styles: Vec<StyleLayerInfo>, pub computed: Vec<StyleProperty>, pub semantics: SemanticsInfo,
    pub handlers: Vec<HandlerInfo>, pub states: Vec<InteractiveState>, pub why: Vec<WhyProp> }
```

- `call_chain`: node の作成位置（`CoreNode.at`）を先頭に、node の owner から root へ向かって owner の作成位置を並べる。同じ位置が続くときは一つにまとめる。
- `app_code`: path の `/` 区切りの部分に `kivact`・`kivact-*`・`arvane*` の crate 名（`crates/<name>/` または cargo registry の `<name>-<version>/`）を含まないとき true。DevTools は最初の `app_code: true` の frame を「アプリのコード」として表示する（Zed の inspector で問題になった「位置が UI 部品の中を指す」への対策）。
- `styles`: 上ほど優先が高い順に並べる（4.4.5）。
- `computed`: 確定した値（engine の effective paint、計算した layout、継承後の text style）を名前と値の文字列で。

`query.rs`（`kivact::test` と同じ規則で探す: role は `Some(name)` なら名前の完全一致で、role の無い node は `Generic` として比べる、text は Text node の内容の完全一致、test id は完全一致。見えている node だけ）:

```rust
pub struct RoleQuery { pub window: WindowId, pub within: Option<NodeId>, pub role: Role, pub name: Option<String>,
    pub timeout_ms: Option<u32> }
pub struct TextQuery { pub window: WindowId, pub within: Option<NodeId>, pub text: String, pub timeout_ms: Option<u32> }
pub struct TestIdQuery { pub window: WindowId, pub within: Option<NodeId>, pub id: String, pub timeout_ms: Option<u32> }
pub struct FoundNode { pub node: ElementInfo }
pub struct MaybeNode { pub node: Option<ElementInfo> }
pub struct FoundNodes { pub nodes: Vec<ElementInfo> }
```

`RoleQuery.name` が `None` なら名前を比べない（role だけで探す。`get_*`・`find_*` は 2 件以上で `MULTIPLE_MATCHES`）。`Some("")` は名前が空の node だけに合う。`timeout_ms` は `find_*` だけが使う（既定 5000）。`get_*`・`query_*`・`all_*` は無視する。`within` が無い node なら `NODE_NOT_FOUND`、`window` が無ければ `WINDOW_NOT_FOUND`。`find_*` の待っている間にその window が閉じたら `WINDOW_NOT_FOUND`。

`overlay.rs`:

```rust
pub struct HighlightParams { pub node: Option<NodeId>, pub layout: bool }
pub struct OverlaySettings { pub paint_flashing: bool, pub layout_lint: bool, pub update_heatmap: bool,
    pub tab_order: bool }
pub struct StartPickParams { pub window: WindowId }
pub enum PickPhase { Hover, Select, Cancel }
pub struct PickEvent { pub window: WindowId, pub node: Option<NodeId>, pub phase: PickPhase }
pub struct ScreenshotParams { pub window: WindowId, pub node: Option<NodeId> }
pub struct Image { pub png_base64: String, pub width: u32, pub height: u32 }
```

- `HighlightParams.node` が `None` なら消す。node が無ければ `NODE_NOT_FOUND`。`layout` が true なら、強調した node の箱に加えて、その子の箱の輪郭と、子の間の隙間の塗りも出す（flex／grid の重ね表示。Computed・Layout の toggle）。
- `OverlaySettings` の paint flashing は、最新の turn の `SyncRecord.redrawn` の node の箱の border を点滅させる（engine は damage の矩形を持たない）。compositor の layer の境界は表示しない（7 章）。
- `StartPickParams.window` は必須（pick を始める window）。

`style.rs`:

```rust
pub enum OverrideScope { Node { node: NodeId }, CallSite { location: SourceLocation }, Token { token: String } }
pub struct SetStyleOverrideParams { pub scope: OverrideScope, pub property: String, pub value: String }
pub struct StyleOverride { pub id: u64, pub scope: OverrideScope, pub property: String, pub value: String }
pub struct StyleOverrideList { pub overrides: Vec<StyleOverride> }
pub struct ClearStyleOverridesParams { pub id: Option<u64> }
pub struct ForceStateParams { pub node: NodeId, pub states: Vec<InteractiveState> }
pub enum TokenKind { Color, Length, Number, Text }
pub struct ThemeToken { pub name: String, pub kind: TokenKind, pub value: String }
pub struct ThemeTokenList { pub tokens: Vec<ThemeToken> }
```

`updates.rs`:

```rust
pub struct RecordingSettings { pub updates: bool, pub frames: bool, pub input: bool }
pub enum TurnStage { Begin, Reports, Dispatch, Async, Commit, Mounts, UserEffects, Output }
pub struct StageTime { pub stage: TurnStage, pub start_ns: u64, pub duration_ns: u64 }
pub enum CauseKind { Input, Timer, Task, Mount, Devtools, SignalWrite, MemoRun, EffectRun, PropChange, Layout, Paint, Frame }
pub struct CauseNode { pub id: u32, pub kind: CauseKind, pub label: String, pub ty: Option<String>,
    pub name_hint: Option<String>, pub source: Option<SourceLocation>, pub node: Option<NodeId>,
    pub window: Option<WindowId>, pub count: u32, pub wasted: bool }
pub struct CauseEdge { pub source: u32, pub target: u32 }
pub enum LayoutReason { Pass, Style, Viewport }
pub struct LayoutPass { pub window: WindowId, pub reasons: Vec<LayoutReason>, pub measured: u32, pub laid_out: u32 }
pub enum Renderer { Cpu, Gpu }
pub struct FrameInfo { pub window: WindowId, pub end_ns: u64, pub work_ns: u64, pub scene_ns: u64,
    pub raster_ns: u64, pub present_ns: u64, pub chunks_rastered: u32, pub pixels_rastered: u64,
    pub damage_pixels: u64, pub renderer: Renderer, pub frame_period_ns: u64, pub over_budget: bool }
pub struct WastedWork { pub equal_writes: u32, pub equal_memos: u32, pub unchanged_props: u32 }
pub struct TurnGraph { pub turn: u64, pub started_ns: u64, pub duration_ns: u64, pub stages: Vec<StageTime>,
    pub nodes: Vec<CauseNode>, pub edges: Vec<CauseEdge>, pub changed_nodes: Vec<NodeId>,
    pub layout: Vec<LayoutPass>, pub frames: Vec<FrameInfo>, pub wasted: WastedWork, pub truncated: bool }
pub struct ListTurnsParams { pub since: Option<u64>, pub limit: Option<u32>, pub wait_ms: Option<u32> }
pub struct TurnList { pub turns: Vec<TurnGraph> }
pub struct ListFramesParams { pub window: Option<WindowId>, pub since: Option<u64>, pub limit: Option<u32> }
pub struct FrameList { pub frames: Vec<FrameInfo> }
```

時刻の規則: protocol の時刻（名前が `_ns` で終わり、長さではないもの: `TurnGraph.started_ns`・`StageTime.start_ns`・`FrameInfo.end_ns`・`InputLogEntry.at_ns`・`FocusMove.at_ns`・`SnapshotInfo.taken_ns`・`TaskInfo.deadline_ns`）は、すべて App の loop の時計（`Runtime::now_ns` と同じ。native は `ShellLoop::now_ns`、試験は仮想時計）の絶対値である。長さ（`duration_ns`・`work_ns`・`scene_ns`・`raster_ns`・`present_ns`・`frame_period_ns`）は実時間の長さ。wall clock の時刻は review の `created`・`updated`（RFC 3339）だけが持つ。

履歴の method の共通の規則: `list_turns`・`list_frames`・`list_inputs`・`list_errors` は `since: Option<u64>`（その値より後だけを返す）と `limit: Option<u32>`（最新の件数）で区切る。`since` の意味は、turns は turn の番号、errors と inputs は `seq`、frames は `end_ns`。返す順は古いものから。`since` も `limit` も無ければ保持している全部。

`ListTurnsParams.limit`: `since` より新しい turn のうち新しい方から `limit` 件を、turn 番号の昇順で返す（`since` が無く `limit: 1` なら最新の 1 件。Python の `Recording` と DevTools の E2E は、`limit: 1` で最新の turn 番号を読む）。

`ListTurnsParams.wait_ms`: `since` より後の完成した graph が無いとき、最大 `wait_ms` ミリ秒待ってから答える（待つ間に完成した graph があれば、すぐ答える。時間が切れたら空の `TurnList` で答える。error にしない）。`find_*` の `timeout_ms` と同じ期限の仕組みで待つ（4.9 の `Pending::Turns`）。

原因グラフの作り方（core の devtools module が turn の終わりに作る。4.4.3）:

1. turn の cause ごとに `Input`／`Timer`／`Task`／`Mount`／`Devtools` の node を 1 つ作る。種類は runtime の `CauseTag` から決める（文字列を読み解かない。4.4.3）。DevTools が処理した call は `Devtools`（label は `devtools <method 名>`）。cause を持たない書き込みや再評価は、cause の node の無い根の node になる。
2. 値を変えた書き込みごとに `SignalWrite`、再評価ごとに `MemoRun`／`EffectRun`。runtime の edge（書き込み → 汚れた購読者、memo → 再評価した購読者）を `CauseEdge` にする。各書き込みと再評価は、その時点の cause に属する。
3. render effect が host node の prop を変えたら `PropChange`（`node` と prop 名）を作り、effect からの edge を張る。
4. layout と frame は turn の後で起きるので、1〜3 までの graph を、その turn の commit の revision と一緒に「frame を待つ graph」として保存する。後の turn の終わりに engine の記録（`SyncRecord` と `FrameRecord`。どちらも revision を持つ）を取り出し、revision が同じ `SyncRecord` から `Layout`／`Paint` の node を足す。graph は、その revision の `FrameRecord` が来たとき（`Frame` の node を足す）か、その revision より新しい revision の `SyncRecord` か `FrameRecord` が来たとき（その revision では frame を描かなかった）に完成させ、`turn_recorded` を通知する。`list_turns` は完成した graph だけを返す（直前の操作の graph は 1〜2 turn 遅れて完成するので、client は `wait_ms` を付けて呼ぶか、`turn_recorded` を待つ）。`stop_recording` は、止める前に engine の記録を取って待っている graph に足し、残った graph を（`Frame` の node なしで）すべて完成させて `turn_recorded` を出し、その後で engine と runtime の記録を止める（止めた直後の `list_turns` に最後の操作の turn が入る）。`start_recording` は、記録していない状態から呼ぶと、前の記録で残った「frame を待つ graph」を捨てる。
5. 同じ `kind` と `source` と `window` の node が 8 個以上あれば 1 つにまとめ、`count` に数を入れる（for_each の 30 行は「×30」の 1 個になる）。
6. 値を変えなかった書き込み・同じ値を返した memo・変化の無い prop の書き込みは `wasted: true` にして `WastedWork` に数える（詳細な記録のときは、値を変えなかった書き込みも runtime が記録する）。
7. node が 2000 個を超えたとき、または runtime の詳細な記録の `dropped` が 0 でないとき、残りは捨てて `truncated: true`。
8. `name_hint` は agent が `source` の行を読んで、`let` の後の識別子（`mut` を除く）を取り出したもの。取れなければ `None`。

`perf.rs`:

```rust
pub struct Counters { pub live_reactive: u64, pub live_owners: u64, pub live_objects: u64, pub live_constants: u64,
    pub tree_nodes: u64, pub timers: u64, pub frame_loops: u64, pub animations: u64, pub turns: u64,
    pub outstanding_blocking: u64 }
pub struct SnapshotInfo { pub id: u64, pub taken_ns: u64, pub owners: u32, pub objects: u32 }
pub struct OwnerEntry { pub id: OwnerId, pub parent: Option<OwnerId>, pub source: SourceLocation }
pub struct ObjectEntry { pub id: u64, pub owner: OwnerId, pub name: String, pub source: SourceLocation }
pub struct DiffSnapshotsParams { pub before: u64, pub after: u64 }
pub struct SnapshotDiff { pub added_owners: Vec<OwnerEntry>, pub removed_owners: Vec<OwnerEntry>,
    pub added_objects: Vec<ObjectEntry>, pub removed_objects: Vec<ObjectEntry> }
pub struct OwnerNode { pub id: u64, pub label: String, pub location: Option<SourceLocation>,
    pub nodes: Vec<NodeId>, pub children: Vec<OwnerNode> }
pub struct OwnerTree { pub roots: Vec<OwnerNode> }
pub struct ExportTraceParams { pub path: Option<String> }
pub struct ExportedFiles { pub paths: Vec<String> }
```

- `get_owners { window }` は、その window の owner の木を返す（Elements の「owner tree への切り替え」が使う）。`OwnerNode.nodes` はその owner が作った host node。`label` は owner の作成位置の `"<file の名前>:<行>"`（例 `counter.rs:12`。runtime は owner の種類を持たないので位置で名付ける。T12.1）。
- `export_trace` は記録した turn と frame を Chrome Trace Event Format（JSON の `traceEvents` 配列。Perfetto で開ける）で書く。JSON の文字列は protocol の `chrome_trace(turns, frames)`（純粋関数。`trace.rs`）が作る。書く場所は agent の `write_trace`（`path` は workspace root からの相対。絶対 path ならそのまま。既定は `.kivact/traces/<session>-<n>.json`。`n` は 1 から数え、既存の file は上書きしない）。

`a11y.rs`:

```rust
pub struct A11yNode { pub node: NodeId, pub parent: Option<NodeId>, pub children: Vec<NodeId>,
    pub semantics: SemanticsInfo, pub bounds: Rect }
pub struct A11yTree { pub window: WindowId, pub nodes: Vec<A11yNode> }
pub enum AuditRule { MissingName, LowContrast, SmallTarget, FocusableWithoutRole, DuplicateTestId, UnlabelledImage }
pub struct AuditFinding { pub rule: AuditRule, pub node: NodeId, pub message: String,
    pub value: Option<f64>, pub limit: Option<f64> }
pub struct AuditReport { pub window: WindowId, pub findings: Vec<AuditFinding> }
pub enum FocusReason { Pointer, Keyboard, Programmatic }
pub struct FocusMove { pub at_ns: u64, pub before: Option<NodeId>, pub after: Option<NodeId>, pub reason: FocusReason }
pub struct FocusInfo { pub window: WindowId, pub node: Option<NodeId>, pub visible: bool, pub history: Vec<FocusMove> }
pub struct TabOrder { pub window: WindowId, pub nodes: Vec<NodeId> }
pub struct WindowParams { pub window: WindowId }
```

監査の規則（すべて core の devtools module で計算する）:

| 規則 | 判定 |
|---|---|
| `MissingName` | role が操作できる部品の role（`kivact_types::Role` の Button・Link・CheckBox・Switch・RadioButton・TextBox・ComboBox・Slider・Tab・MenuItem・TreeItem・Option。名前が少し違う variant は、`crates/kivact-types/src/semantics/values.rs` の `Role` の同じ意味の variant を使う）のどれかで、name が空 |
| `LowContrast` | Text node の文字色と、祖先を辿って最初に見つかる不透明な背景色のコントラスト比が 4.5 未満（文字の大きさが 18px 以上、または 14px 以上で weight 700 以上なら 3.0 未満）。`value` に比、`limit` に閾値 |
| `SmallTarget` | focus できる node か on_press を持つ node の大きさが 24 × 24 未満（`Capabilities.touch` が true なら 44 × 44 未満）。`value` に短い辺 |
| `FocusableWithoutRole` | focus できるのに role が無い |
| `DuplicateTestId` | 同じ window に同じ test id が 2 つ以上 |
| `UnlabelledImage` | role が Image で name が空 |

`FocusReason` は、core の focus request（`NodeRef::focus` など）と一致すれば `Programmatic`、そうでなく engine の `FocusChanged` の `visible` が true なら `Keyboard`、それ以外は `Pointer`。focus の履歴は window ごとに最新 100 件。

`layout.rs`:

```rust
pub enum LayoutRuleKind { OverflowOwnBox, OverflowParent, Wrapped, OffCenter, ClippedWithoutEllipsis }
pub struct LayoutFindingInfo { pub node: NodeId, pub rule: LayoutRuleKind, pub amount: f64,
    pub text_rect: Rect, pub limit_rect: Rect }
pub struct LayoutLint { pub window: WindowId, pub findings: Vec<LayoutFindingInfo> }
```

`input.rs`:

```rust
pub enum InputStep {
    Click { node: NodeId }, MouseMove { x: f64, y: f64 }, MouseDown { button: MouseButton },
    MouseUp { button: MouseButton }, Wheel { lines_x: f64, lines_y: f64 },
    Key { key: String }, Text { text: String },
    A11y { node: NodeId, action: A11yAction, value: Option<String> },
}
pub struct InjectInputParams { pub window: WindowId, pub steps: Vec<InputStep> }
pub enum InputSource { Platform, Injected }
pub enum HandlerOutcome { Handled, NotHandled, PreventDefault, Failed }
pub struct HandlerCall { pub node: NodeId, pub event: String, pub outcome: HandlerOutcome }
pub struct InputLogEntry { pub seq: u64, pub at_ns: u64, pub window: WindowId, pub kind: String, pub detail: String,
    pub source: InputSource, pub target: Option<NodeId>, pub route: Vec<NodeId>, pub handlers: Vec<HandlerCall> }
pub struct InputLog { pub entries: Vec<InputLogEntry> }
pub struct ListInputsParams { pub since: Option<u64>, pub limit: Option<u32> }
pub struct ImeState { pub window: WindowId, pub node: Option<NodeId>, pub composing: bool,
    pub preedit: Option<String>, pub cursor_area: Option<Rect>, pub purpose: String }
```

- `Wheel { lines_x, lines_y }`: 1 line は論理 20 px、`lines_y` が正なら下へ scroll する。
- protocol の `MouseButton` は `kivact_types::MouseButton` の同じ名前の variant に写す。
- `Click { node }` は node の矩形の中心へ `MouseMove`・`MouseDown(Primary)`・`MouseUp(Primary)` を順に注入する。node が見えていない・大きさ 0・window の外なら `NODE_NOT_FOUND`。
- `Key.key` は 4.2.7 の chord の書き方（`"Enter"`、`"Ctrl+Enter"`、`"Shift+Tab"`、`"F5"`。`F1`〜`F24`、1 文字、`Key` の unit variant の名前）。core は `convert::parse_chord` で読み、chord の modifiers を `InjectedInput::Modifiers` で押し、key を押して離し、最後に `InjectedInput::Modifiers(Modifiers::default())` で離す。読めない chord は `INVALID_PARAMS`（message に書き方の一覧）。`Text` は文字ごとに key を注入する（`kivact::test` の `type_text` と同じ規則）。
- `A11y.action` は `kivact_types::A11yAction` と同じ variant。`SetValue` は `value` が要る（無ければ `INVALID_PARAMS`）。`Dismiss` は `DismissReason::Accessibility`。他の action は `value` を無視する。a11y の注入は他の入力と同じ `InjectedInput::Accessibility` の経路を通る。
- embed の target は `inject_input` を `UNSUPPORTED` で断る（embed の入力経路は `WindowInput` ではないため。7 章）。
- 入力の log は記録中（`RecordingSettings.input`）だけ、最新 500 件。`seq` は log に入れた順の番号（1 から）。

`motion.rs`・`tasks.rs`・`console.rs`・`emulation.rs`:

```rust
// motion.rs
pub enum AnimationKind { Animated, Timeline, FrameLoop }
pub struct AnimationInfo { pub id: u64, pub kind: AnimationKind, pub source: Option<SourceLocation>,
    pub owner: OwnerId, pub running: bool, pub paused: bool, pub decorative: bool }
pub struct AnimationList { pub animations: Vec<AnimationInfo>, pub time_scale: f64 }
pub struct TimeScaleParams { pub scale: f64 }
// tasks.rs
pub enum TaskKind { Task, Resource, Action, Timer }
pub enum AsyncPhase { Idle, Pending, Ready, Failed }
pub struct TaskInfo { pub id: u64, pub kind: TaskKind, pub source: Option<SourceLocation>, pub owner: Option<OwnerId>,
    pub phase: AsyncPhase, pub deadline_ns: Option<u64> }
pub struct TaskList { pub tasks: Vec<TaskInfo>, pub outstanding_blocking: u32 }
// console.rs
pub struct ErrorEntry { pub seq: u64, pub kind: String, pub message: String, pub source: Option<SourceLocation>,
    pub turn: Option<u64>, pub handled: bool }
pub struct ErrorList { pub errors: Vec<ErrorEntry> }
pub struct ListErrorsParams { pub since: Option<u64>, pub limit: Option<u32> }
// emulation.rs
pub enum ThemeEmulation { Light, Dark }
pub enum PseudoLocale { Off, Accented, Long, Rtl }
pub struct Emulation { pub theme: Option<ThemeEmulation>, pub reduced_motion: Option<bool>,
    pub locale: Option<String>, pub pseudo_locale: PseudoLocale, pub scale: Option<f64>, pub touch: Option<bool> }
```

- `set_time_scale`: 0 は一時停止、1 は通常、0.25 などは遅回し。上限 4。範囲外は `INVALID_PARAMS`。
- `Emulation` の各 `None` は「上書きしない」。`theme` は host の `theme_hint` を上書きする（`platform::theme_hint()` に従って theme を選ぶアプリにだけ効く。DevTools の画面にこの注意を表示する）。`scale` は window の DPI の倍率を上書きする（論理の大きさが変わる）。`touch` は `Capabilities.touch`（core の `capabilities` の投影）と `PointerSettings.primary = Coarse` を上書きする。
- pseudo-locale（`message()` と `message_with()` を通る文字列だけに効く。直接 `text("…")` に書いた文字列は変わらない＝埋め込みの文字列を見つけられる）:
  - `Accented`: ASCII の英字をアクセント付きの字に置き換える（`a→á`、`e→é`、`i→í`、`o→ó`、`u→ú`、`A→Á`、`E→É`、`I→Í`、`O→Ó`、`U→Ú`、`c→ç`、`n→ñ`、`y→ý`）。前後を `［` と `］` で囲む。`{name}` の置換位置は変えない。
  - `Long`: `Accented` に加え、文字数の 40%（切り上げ）だけ `~` を末尾（`］` の前）に足す。
  - `Rtl`: 文字列の前に U+202E（RIGHT-TO-LEFT OVERRIDE）、後に U+202C を付ける。

`review.rs`:

```rust
pub enum ReviewTarget {
    Node { node: NodeId, role: Option<Role>, name: Option<String>, test_id: Option<String>,
        source: Option<SourceLocation>, call_chain: Vec<CallFrame>, bounds: Rect, states: Vec<InteractiveState> },
    Window {},
}
pub struct StyleChange { pub scope: OverrideScope, pub property: String, pub before: Option<String>, pub after: String }
pub struct Comment { pub id: String, pub text: String, pub created: String }
pub struct ReviewItem { pub id: String, pub window_title: String, pub window: WindowId, pub target: ReviewTarget,
    pub style_changes: Vec<StyleChange>, pub image: Option<String>, pub comments: Vec<Comment> }
pub struct ReviewDocument { pub schema: String, pub app: AppInfo, pub session: String, pub created: String,
    pub updated: String, pub environment: Emulation, pub items: Vec<ReviewItem> }
pub struct AddCommentParams { pub window: WindowId, pub node: Option<NodeId>, pub text: String, pub screenshot: bool }
pub struct EditCommentParams { pub comment: String, pub text: String }
pub struct RemoveCommentParams { pub comment: String }
pub struct ExportedReview { pub json: String, pub markdown: String }
```

- `schema` は常に `"kivact-devtools-review/1"`。日時は UTC の RFC 3339（`2026-11-08T05:32:00Z`）。protocol crate に純粋関数 `fn rfc3339_utc(unix_ms: u64) -> String` を置く（外部 crate を使わない。days-from-civil の逆算）。
- `ReviewItem.id` は `C1`、`C2`、…（document の中で一意。消しても再利用しない）。`Comment.id` は item ごとの番号 `C1.1`、`C1.2`、…（消しても再利用しない）。
- 同じ node（または同じ window）へのコメントは同じ `ReviewItem` に足す。
- `style_changes` は、その node に掛かっている override（4.4.5）をコメントを足した時点で写したもの。
- `image` は `images/<item id>.png`（review の directory からの相対 path）。同じ item の新しい screenshot は同じ path を上書きする。
- session の directory は `.kivact/review/<YYYYMMDD-HHMMSS>-<pid>-<port>/`（UTC。一つの process に複数の App があっても衝突しない）。`ReviewStore` は変更のたびに `review.json` と `review.md` の両方を書く。`export_review` は params `Empty` で、結果は 2 つの file の path（`ExportedReview`）。
- `add_comment` の node が無ければ `NODE_NOT_FOUND`。item の `ReviewTarget::Node` は作成時の写しなので、node が消えても item とコメントは残り、`edit_comment`・`remove_comment` は使える。DevTools アプリの Review パネルは、今の tree に無い node の item に「(removed)」を出し、強調の操作を無効にする。
- protocol の `review_markdown.rs` に `pub fn to_markdown(&ReviewDocument) -> String` と `pub fn locator(item: &ReviewItem) -> Option<String>`。`locator` は `kivact::test` の書き方の Rust の式を返す: test id があれば `get_by_test_id("<id>")`、なければ role と name で `get_by_role(Role::<Rust の variant 名>, "<name>")`（name が無ければ `""`）、role も無ければ `None`（window へのコメントも `None`）。window が main（id 1）なら `screen.` を、それ以外なら `screen.window("<window_title>").` を前に付ける。Markdown の `Query:` 行もアプリの「locator をコピー」も、この同じ関数を使う。Markdown に書く role は Rust の variant 名（`CheckBox`）。

`scripts.rs` と通知の一覧:

```rust
pub struct ShowTableParams { pub title: String, pub columns: Vec<String>, pub rows: Vec<Vec<String>> }
```

| 通知 | params | topic | いつ |
|---|---|---|---|
| `tree_changed` | `TreeChanged` | Tree | commit で tree が変わったとき（window ごと） |
| `turn_recorded` | `TurnGraph` | Turns | 記録中の turn が完成したとき |
| `frame_recorded` | `FrameInfo` | Frames | 記録中の frame を出したとき |
| `pick` | `PickEvent` | Pick | pick mode の hover・select・cancel |
| `error_reported` | `ErrorEntry` | Errors | 新しい error が記録されたとき |
| `focus_requested` | `Empty` | Devtools | DevTools がすでに接続しているのに F12 などが押されたとき |
| `script_table` | `ShowTableParams` | Scripts | script が `show_table` を呼んだとき |
| `windows_changed` | `WindowList` | Windows | window が開いた・閉じた・title が変わったとき |

その他の protocol の file（型を持たない）:

- `code.rs`: 4.2.1 の error code の `pub const <名前>: i32`。
- `rpc.rs`: `RpcRequest`・`RpcResponse`・`RpcError` と、request・response・通知の行の読み書き（T1.2）。
- `call.rs`・`notify.rs`: 4.2.6 の macro と、その呼び出し。
- `config.rs`: 4.2.7 の `KeyConfig`・`DevtoolsConfig`（wire の型でもあるので 4.2.3 の形で書く）。
- `time.rs`・`trace.rs`・`discovery.rs`・`review_markdown.rs`: 純粋関数だけ。

#### 4.2.5 method の一覧

method 名は MCP の tool 名・Python の method 名と同じ（D-8）。全 60 個。

| method | params | result | 説明 |
|---|---|---|---|
| `initialize` | `InitializeParams` | `InitializeResult` | 最初に必ず呼ぶ（agent が答える） |
| `subscribe` | `SubscribeParams` | `Empty` | 通知の topic を選ぶ（置き換え。agent が答える） |
| `get_app_info` | `Empty` | `AppInfo` | |
| `get_config` | `Empty` | `DevtoolsConfig` | 既定を埋めた設定（agent が答える。DevTools アプリは設定 file を自分で読まずこれを使う） |
| `list_windows` | `Empty` | `WindowList` | |
| `get_tree` | `GetTreeParams` | `TreeSnapshot` | |
| `get_node` | `GetNodeParams` | `NodeDetail` | |
| `get_by_role` / `query_by_role` / `find_by_role` / `all_by_role` | `RoleQuery` | `FoundNode` / `MaybeNode` / `FoundNode` / `FoundNodes` | `name` が `None` なら名前を比べない |
| `get_by_text` / `query_by_text` / `find_by_text` / `all_by_text` | `TextQuery` | 同上 | |
| `get_by_test_id` / `query_by_test_id` / `find_by_test_id` / `all_by_test_id` | `TestIdQuery` | 同上 | |
| `highlight` | `HighlightParams` | `Empty` | `node: None` で消す |
| `set_overlays` | `OverlaySettings` | `Empty` | 全 window の重ね表示 |
| `get_overlays` | `Empty` | `OverlaySettings` | 今の重ね表示の設定（DevTools アプリは写しを持たずこれを読む。`get_recording` と同じく target が持つ事実） |
| `start_pick` / `stop_pick` | `StartPickParams` / `Empty` | `Empty` | |
| `screenshot` | `ScreenshotParams` | `Image` | |
| `set_style_override` | `SetStyleOverrideParams` | `StyleOverride` | |
| `clear_style_overrides` | `ClearStyleOverridesParams` | `Empty` | `id: None` で全部 |
| `list_style_overrides` | `Empty` | `StyleOverrideList` | |
| `force_state` | `ForceStateParams` | `Empty` | `states: []` で解除 |
| `list_theme_tokens` | `Empty` | `ThemeTokenList` | main window の root の theme |
| `start_recording` | `RecordingSettings` | `Empty` | 記録を始める（記録中なら設定を置き換える。記録していない状態からなら、前の記録の「frame を待つ graph」を捨てる） |
| `stop_recording` | `Empty` | `Empty` | 先に待っている graph をすべて完成させて `turn_recorded` を出し、それから止める |
| `get_recording` | `Empty` | `RecordingSettings` | 今の記録の設定（すべて false なら記録していない）。DevTools アプリはこれを読んで表示する（写しを持たない） |
| `list_turns` | `ListTurnsParams` | `TurnList` | 記録した turn（最新 1000 件まで保持）。`wait_ms` で完成を待てる |
| `list_frames` | `ListFramesParams` | `FrameList` | 記録した frame（window ごとに最新 3000 件） |
| `get_counters` | `Empty` | `Counters` | |
| `take_snapshot` | `Empty` | `SnapshotInfo` | 生きている owner と object の一覧を保存（最新 10 個） |
| `diff_snapshots` | `DiffSnapshotsParams` | `SnapshotDiff` | |
| `get_owners` | `WindowParams` | `OwnerTree` | window の owner の木 |
| `export_trace` | `ExportTraceParams` | `ExportedFiles` | |
| `get_a11y_tree` | `WindowParams` | `A11yTree` | |
| `audit_a11y` | `WindowParams` | `AuditReport` | |
| `get_focus` | `WindowParams` | `FocusInfo` | |
| `get_tab_order` | `WindowParams` | `TabOrder` | |
| `lint_layout` | `WindowParams` | `LayoutLint` | `kivact::test` の `check_layout` と同じ既定の `LayoutCheck` |
| `inject_input` | `InjectInputParams` | `Empty` | 注入した入力の turn が終わってから返す。window が閉じたら `WINDOW_NOT_FOUND`。embed は `UNSUPPORTED` |
| `list_inputs` | `ListInputsParams` | `InputLog` | |
| `get_ime_state` | `WindowParams` | `ImeState` | |
| `list_animations` | `Empty` | `AnimationList` | |
| `set_time_scale` | `TimeScaleParams` | `Empty` | |
| `list_tasks` | `Empty` | `TaskList` | |
| `list_errors` | `ListErrorsParams` | `ErrorList` | |
| `get_emulation` / `set_emulation` | `Empty` / `Emulation` | `Emulation` | `set_emulation` は全体を置き換える |
| `get_review` | `Empty` | `ReviewDocument` | |
| `add_comment` | `AddCommentParams` | `ReviewItem` | `node: None` は window へのコメント |
| `edit_comment` / `remove_comment` | `EditCommentParams` / `RemoveCommentParams` | `ReviewItem` / `Empty` | |
| `export_review` | `Empty` | `ExportedReview` | `review.json` と `review.md` の path |
| `show_table` | `ShowTableParams` | `Empty` | script の結果を DevTools に表示 |

#### 4.2.6 method と通知の宣言（protocol crate の `call.rs`・`notify.rs`）

method の一覧は一か所（macro の呼び出し 1 つ）に書く。macro が次を生成する:

- `pub enum Call`（`#[serde(tag = "method", content = "params")]`、各 variant に `#[serde(rename = "<method>")]`）。JSON-RPC の request の `method` と `params` をそのまま読める。
- `pub enum Reply`（variant ごとに result の型。`fn into_value(self) -> serde_json::Value`）。
- marker の struct と `pub trait Method { const NAME: &'static str; type Params; type Result; }` の実装（client の型付き呼び出し `call::<GetTree>(params)` に使う）。

variant 名と marker 名は 4.2.3 の 10 の規則で wire 名から決まる。呼び出しの全文（60 個）は作業指示書 T1.2 に書く。形:

```rust
protocol_methods! {
    /// Start a session (the first request on a connection).
    Initialize => "initialize" (InitializeParams) -> InitializeResult;
    /// Choose the notification topics of this connection.
    Subscribe => "subscribe" (SubscribeParams) -> Empty;
    // ... every row of 4.2.5, in that order ...
}
protocol_notifications! {
    /// The element tree of a window changed.
    TreeChanged => "tree_changed" (TreeChanged) in Tree;
    // ... every row of the notification table ...
}
```

macro の実装（`macro_rules!`）は作業指示書に全文を書く。`parse_call` は、知らない method を `METHOD_NOT_FOUND`、params の型違いを `INVALID_PARAMS` にする。`Empty` を params に使う method は、client が `params` を省いても読める。

#### 4.2.7 設定ファイル（`.kivact/devtools.json`）

```rust
pub struct KeyConfig { pub open: Option<Vec<String>>, pub pick: Option<Vec<String>> }
pub struct DevtoolsConfig { pub keys: Option<KeyConfig>, pub app: Option<String>, pub python: Option<String>,
    pub scripts_dir: Option<String>, pub editor: Vec<String>, pub schema: Option<String> }
```

（`schema` の field は `#[serde(default, rename = "$schema", skip_serializing_if = "Option::is_none")]`。生成器は Python の型には出さず、設定の schema にだけ `"$schema"` の property として出す。）

- 既定: `keys.open = ["F12", "Ctrl+Shift+I"]`、`keys.pick = ["Ctrl+Shift+C"]`、`app` は無し（4.3 の探し方）、`python` は無し（4.7.4 の探し方）、`scripts_dir = ".kivact/devtools/scripts"`、`editor = ["code", "--goto", "{file}:{line}:{column}"]`（空なら既定。`{file}` は workspace root からの絶対 path に置き換える。DevTools アプリが `std::process::Command` で起動する）。
- `keys.open` と `keys.pick` は別々に既定へ戻る: `None` は既定、`[]` は無効（`open` だけを書いても `pick` は既定のまま）。
- path の値（`app`・`python`・`scripts_dir`）は workspace root からの相対 path（絶対 path も可）。
- `editor` の command が `std::io::ErrorKind::NotFound` で起動できなかったら、`args[0]` に `.cmd` を付けて 1 回だけ試す（Windows の `code` は `code.cmd` で、std の `Command` は `.exe` 以外の拡張子を補わない。OS の分岐を書かずに済む）。それも失敗したら、失敗の文と設定の `editor` の書き方を status bar に出す。
- key の書き方: `Ctrl`・`Shift`・`Alt`・`Meta` を `+` でつなぎ、最後に key の名前。名前は、`F1`〜`F24` なら `kivact_types::Key::F(n)`、1 文字なら `Key::Char(c)`、それ以外は `Key` の unit variant の名前（`Enter`、`Tab`、`Escape` など。`crates/kivact-types/src/input.rs` の `Key`）。比べるとき `Key::Char` は大文字と小文字を区別しない（Shift を押すと文字が大文字になるため）。読めない chord は `tracing::warn!` を出してその 1 つだけ捨てる（core の `convert::parse_chord` が読む）。
- file の先頭に `"$schema": "https://raw.githubusercontent.com/Kashiwade-music/kivact/main/crates/kivact-devtools-protocol/schema/devtools-config.schema.json"` を書けば VS Code で補完が効く（生成した schema。agent はこの key を無視する）。
- file が無い・読めない・JSON として不正なら既定を使い、不正なときは target の `tracing::warn!` に理由を出す。
- protocol crate の `DevtoolsConfig` に、既定の値を返す `open_keys()`・`pick_keys()`・`scripts_dir()`・`editor_command()` と、既定を埋めた写しを返す `with_defaults()`（`get_config` の答え）を持たせる。

#### 4.2.8 生成器（`cargo xtask devtools-codegen`）

- 置き場所: `dev/xtask/src/devtools_codegen.rs`（`mod` と `use` だけ。中身は `devtools_codegen/` の leaf module）。既存の `syn` を使い、新しい依存を足さない。
- 入力: `crates/kivact-devtools-protocol/src/**/*.rs` の型（4.2.3 の形）と、`protocol_methods!`・`protocol_notifications!` の呼び出しの中身、`app.rs` の `PROTOCOL_VERSION` の値、`code.rs` の `pub const NAME: i32 = N;`。
- 出力（commit する）:
  1. `crates/kivact-devtools-protocol/schema/kivact-devtools.schema.json`: JSON Schema（draft 2020-12）。`$defs` にすべての型、`x-protocol` に `PROTOCOL_VERSION`、`x-methods` に method の一覧（name、params、result、description）、`x-notifications` に通知の一覧、`x-errors` に error code の一覧（name、code）。
  2. `crates/kivact-devtools-protocol/schema/devtools-config.schema.json`: `DevtoolsConfig` だけの schema（`$schema` key を許す）。
  3. `python/kivact-devtools/src/kivact_devtools/_generated.py`: 型と `Methods` class と `ErrorCode`（4.7.2）。
- `cargo xtask devtools-codegen` は書き出す。`cargo xtask devtools-codegen --check` は生成して比べ（改行の `\r\n` は `\n` として比べる）、違えば失敗する。`cargo xtask check` の local check に `devtools-codegen` を加え（`check_cli.rs` の `RULES`）、`--check` と同じ比較を**一回だけ**する（module ごとには走らせない）。
- 型の対応: `bool → boolean`、整数 → `integer`（u は `minimum: 0`）、`f64 → number`、`String → string`、`Option<T>` → required から外す、`Vec<T> → array`、unit enum → `string` の `enum`、tag 付き enum → `oneOf`（各 variant は `kind` の `const` を持つ object）、別名 → その型、他の型 → `$ref`。

### 4.3 agent（`kivact-devtools-agent`）

公開する API（core だけが使う）:

```rust
/// What the core passes to `Agent::start`. `app.workspace_root` is passed empty;
/// the agent finds the workspace root, reads `.kivact/devtools.json` and fills it in.
pub struct AgentConfig { pub app: AppInfo }

/// What arrived since the last `take_incoming`.
pub enum Incoming {
    /// A call to answer through its `Responder`.
    Call(Call, Responder),
    /// A connection that had finished `initialize` ended; `remaining` is the
    /// number of initialized connections that are still open.
    Closed { remaining: usize },
}

/// Where the reply of one call goes. Every call must be answered exactly once
/// (`send` consumes it); a client gives up on an unanswered call after 30 s.
pub struct Responder { /* request id, writer of the connection */ }
impl Responder { pub fn send(self, result: Result<Reply, RpcError>); }

pub struct Agent { /* listener thread, peers, receiver of incoming, review store, source cache */ }
impl Agent {
    /// Bind, write the discovery file and start the listener thread. `wake` is
    /// called from other threads when something arrives.
    pub fn start(config: AgentConfig, wake: Box<dyn Fn() + Send + Sync>) -> std::io::Result<Agent>;
    /// Everything that arrived since the last time, in order, on the UI thread.
    /// The caller runs the calls after releasing its borrow of the agent (a
    /// call's handler uses the agent again: review, name hints, notifications).
    pub fn take_incoming(&mut self) -> Vec<Incoming>;
    /// Send a notification to the connections subscribed to its topic.
    pub fn notify(&self, notification: Notification);
    /// Whether any connection subscribed to `topic`.
    pub fn wants(&self, topic: Topic) -> bool;
    /// Notify the connected DevTools app (`focus_requested`), or start one.
    pub fn open_devtools(&mut self) -> Result<(), String>;
    /// The configuration as read (no defaults filled in).
    pub fn config(&self) -> &DevtoolsConfig;
    pub fn discovery_path(&self) -> &std::path::Path;
    pub fn workspace_root(&self) -> &std::path::Path;
    /// The app as announced to clients (`workspace_root` filled in): the answer of `get_app_info`.
    pub fn app(&self) -> &AppInfo;
    /// The review of this session (owned here; saved on every change).
    pub fn review(&mut self) -> &mut ReviewStore;
    /// The identifier after `let` on the line at `location`, if any.
    pub fn name_hint(&mut self, location: &SourceLocation) -> Option<String>;
    /// Write a trace file (4.2.4 `export_trace`) and return its path.
    pub fn write_trace(&self, contents: &str, path: Option<&str>) -> std::io::Result<std::path::PathBuf>;
}

/// `image.rs`: bytes for `Image.png_base64`.
pub fn encode_png(width: u32, height: u32, rgba: &[u8]) -> std::io::Result<Vec<u8>>;
pub fn base64(bytes: &[u8]) -> String;
```

core は agent に `Target` のような trait を実装しない。core は `take_incoming` の `Incoming::Call` の `Call` を自分で処理して `Responder::send` で答える。

内部の構造:

- listener thread: blocking `accept`。`Agent` は停止用の `std::sync::mpsc::Sender<()>` を持ち、listener thread は `Receiver` を持つ。`accept` が返るたびに `try_recv()` を見て、値が来ているか `Disconnected` なら終わる（flag を持たない）。`Drop` は `()` を送ってから自分の port に 1 回接続して `accept` を返させ、listener thread だけを join する。
- connection ごとの reader thread: 1 行読んで JSON-RPC として解釈する。`initialize`・`subscribe`・`get_config` はこの thread で答える（UI thread を要らない）。それ以外は `Incoming::Call(Call, Responder)` を `std::sync::mpsc` で UI thread へ送り、`wake()` を呼ぶ。読み込みが終わった（EOF・失敗）か書き込みが失敗したら、`Peer` を表から消し、`initialize` を終えた connection だったときだけ `Incoming::Closed { remaining }` を送って `wake()` を呼ぶ。
- connection の状態: reader thread は `enum Session { AwaitingInitialize, Ready }` を持つ。UI thread と reader が共有する表は `Peers`（`Mutex<BTreeMap<ConnectionId, Peer>>`）で、`Peer { client: ClientInfo, topics: Vec<Topic>, writer }`。`initialize` が通ったら reader thread が `Peer` を入れて `Session::Ready` にする。`subscribe` はその `Peer.topics` を書き換える。`notify`・`wants` は `topics` を、`open_devtools` は `client.kind` を読む。
- writer: `TcpStream::try_clone` した stream を `Arc<Mutex<TcpStream>>` で包み、1 message を 1 行で書く（書き込みの timeout は 5 秒）。書き込みに失敗した stream は `shutdown(Both)` する（reader thread が終わって、上の後始末が一か所で走る）。
- `Drop`: 停止を知らせて listener thread を join し、表の各 `Peer` の stream を `shutdown(Both)` する（reader thread は read が終わって自分で終わる。join しない）。最後に discovery file を消す。
- `ReviewStore`: `.kivact/review/<session>/review.json` と `review.md` に、変更のたびに書く（`tmp` に書いて rename）。`session` は `<YYYYMMDD-HHMMSS>-<pid>-<port>`（UTC）。画像は同じ directory の `images/` に置く。directory は最初に書くときに作る。
- workspace root: `std::env::current_dir()` から上へ辿り、最初に `.kivact` directory を持つもの、無ければ最初に `Cargo.lock` を持つもの。どちらにも無ければ `std::env::current_exe()` の directory から同じように辿る（DAW に埋め込まれた editor は DAW の cwd を持つため）。それでも無ければ current dir。`.kivact/` は最初に file を書くとき（review、trace）に作る。作れなくても agent は動く（review と trace の書き出しが error を返す）。
- DevTools アプリの探し方（`open_devtools`）: (1) 設定の `app`、(2) 環境変数 `KIVACT_DEVTOOLS_APP`、(3) `std::env::current_exe()` と同じ directory の `kivact-devtools` + `std::env::consts::EXE_SUFFIX`、(4) PATH の `kivact-devtools`。引数は `--target <discovery path>` だけ（pick は core が key の window で先に始めている。4.4.1）。kind が `Devtools` の connection がすでにあれば、起動せず `focus_requested` を通知する。
- 起動に失敗したら `Err("DevTools app not found. Install it with `cargo install --git https://github.com/Kashiwade-music/kivact kivact-devtools` or set \"app\" in .kivact/devtools.json")` を返す（core が `UiError` として一度だけ報告する）。
- DevTools が target に掛けた状態は全 client で共有する（4.2.1）。`Incoming::Closed { remaining: 0 }` を受けたら、core はそれらを既定に戻す（4.4.1）。review は残す。
### 4.4 target 側の口（core・engine・runtime・shell・style）

#### 4.4.1 core の devtools module（`crates/kivact-core/src/devtools.rs` と `devtools/`、feature `devtools` のときだけ）

- module の分け方（module の参照が循環しないように。向きは一方向: `devtools_state` ← `devtools_apply` ← `devtools_hooks` ← view の各所。`devtools` は上の 3 つと `view` を参照する）:
  - `view/devtools_state.rs`（`view` の leaf。feature のときだけ）: `DevtoolsState`（下）と、hook と `devtools/*` が共有するデータの型。この module は `crate::view` の他の module も `crate::devtools` も参照しない。
  - `view/devtools_apply.rs`（`view` の leaf。feature のときだけ）: override・emulation を `Style`・`Theme`・文字列に当てる関数（`override_style`、`node_overrides`、`apply_token_overrides`、`pseudo_locale`、`call_sites`）。参照するのは `devtools_state` だけ。
  - `view/devtools_hooks.rs`（`view` の leaf。**feature に関わらず常にコンパイルする**）: view の各所から呼ばれる hook（4.4.5・4.4.6）。どの hook も、feature の有無で同じ signature を持ち、本体だけを `#[cfg(feature = "devtools")]`（data を積む・読む小さな関数）と `#[cfg(not(feature = "devtools"))]`（何もしない・入力をそのまま返す）に分ける。hook の呼び出し側に `cfg` を書かない（両方のビルドで呼び出しが同じ形になる）。
  - `devtools.rs` と `devtools/*.rs`（core の top-level module。feature のときだけ）: protocol の method の処理と turn の処理。`view` と上の 3 つを参照してよい。`view` は `devtools` を参照しない。`launch::runtime` が `devtools` を呼ぶ。**`devtools/*` は `crate::launch` を参照しない**（`ClientMode`・`AppSettings`・`Inspect` を使わない。launch が必要な値を引数で渡す。`get_counters` は `Inspect::stats` と同じ式を `ViewState`・`HostState`・`MotionState`・`TaskState` から直接計算する）。
  - `[module_rules.kivact-core.uses]` に `devtools` の行を足す（T0.1）ので、この向きは xtask が検査する。
- `DevtoolsState`（runtime の `Extensions` に、feature があれば必ず入れる。extension が無いと `Runtime::ext` が panic するため）は `enum DevtoolsState { Off, Running(Box<Inspector>) }`。起動の条件（4.2.2）を満たさないか起動に失敗したら `Off`。`Off` の App は signal も field も何も持たない。`Running` を作るときだけ `Inspector` と signal を作る:
  - `Inspector { agent: RefCell<Agent>, can_inject: bool, hooks: HookData, session: SessionData }`。
  - `HookData`: view の hook が読み書くもの。記録の設定、この turn の prop の書き込み、override の表、forced states、emulation、focus の履歴、最後の focus request、input の log。
  - `SessionData`: `devtools/*` だけが使うもの。答えを待つ呼び出し、原因グラフの組み立て（待っている graph、完成した graph）、frame の履歴、highlight、pick、重ね表示の設定、snapshot、client へ最後に送った内容の写し、id の採番。
  - hook は `let DevtoolsState::Running(i) = DevtoolsState::of(rt) else { return <入力のまま> };` で始める（例外は `wrap_capabilities` だけ: App の構築中に呼ばれるので、`DevtoolsState` を作った memo の中で読む。T11.1）。
  - agent の借用は `Inspector::notify`・`wants` の 1 つの文の中で終える（借りたまま他の関数を呼ばない）。
- 起動: `Launch::start` が `Runtime::new` の前に `ThreadWaker` を clone し、`devtools::start::agent_slot(title, allowed, can_inject, waker)` で agent を起動する。`allowed` と `can_inject` は launch が計算して渡す（`allowed`: `ClientMode::Window`・`Embedded` なら true、`Test` なら `AppSettings.devtools_in_tests`。`can_inject`: `Embedded` 以外は true）。成功なら `Running`、起動の条件を満たさないか失敗なら `Off`（失敗は `tracing::warn!`）。build closure の `Extensions` に `DevtoolsState` を入れる（`launch/runtime.rs:113-150`）。release ビルド（`!cfg!(debug_assertions)`）なら `tracing::warn!(target: "kivact::devtools", "the DevTools agent is enabled in a release build")`。
- 呼び出しの処理（cause を正しく付けるため、turn の中で行う）:
  - `Launch::turn` は `advance_clock` の直後に `devtools::take_incoming(&rt)`（`agent.take_incoming()` が返す `Incoming` の列を取る。agent の借用はここで終わる）を呼び、「event が無ければ早く返す」判定に「呼び出しが無い」を加える。`rt.begin_turn` の後、event を配る前に `devtools::run_incoming(&rt, &turn, incoming)`。
  - `run_incoming` は `Incoming::Call(call, reply)` ごとに、`turn.stage(Stage::Dispatch)`、`turn.cause_with(CauseTag::Devtools, format!("devtools {}", call.method()))`、`rt.root().run_handler(|| target::dispatch(rt, call, reply))` の順で行う。root owner の handler として動くので、`spawn`・signal の書き込み・`context` が使える。DevTools の書き込みは、その呼び出しの cause に属する（最初の入力の cause にならない）。`Incoming::Closed { remaining }` は、`remaining == 0` なら下の「reset」を行う。最後に `turn.stabilize()`。
  - 何かの call が残っている（答えを待つ）か、`run_incoming` が signal を書いたとき、`rt.request_turn()` を呼ぶ。
- turn の終わり: `turn.finish(..)` が返った**直後**に `devtools::after_turn(&rt)`（`finish` がこの turn の `TurnRecord` と `TurnDetail` を確定するため、before だと一つ前の turn を読む）。`after_turn` の中身（この順）: window が閉じたことの後始末と `windows_changed`、`tree_changed`（Tree を購読している connection があるときだけ）、待っている `find_*` と `list_turns` の処理、記録中なら原因グラフ、重ね表示の作り直し、新しい error の通知。`after_turn` は engine に revision を作らせる書き込みをしない（`set_devtools_overlay` などは次の turn の前に host が読む）。`inject_input` の答えは `HostEvent::InjectDelivered` を受けたとき（`host_event`）に返す。
- 設定の持ち主: pick の window、記録の設定、emulation の持ち主は core の `DevtoolsState` である。engine の `DevtoolsHost` と runtime の詳細な記録にある同じ値は、core が `Writer`・`Runtime` の setter で一方向に書き写す適用先であり、engine と runtime はそれを読むだけで書き換えない（S-4 の正本は core）。
- 状態の共有と reset: DevTools が target に掛けた状態（highlight、pick、overlays、forced、overrides、emulation、時間の倍率、記録）は target に一つで、すべての client が共有する（後の書き込みが勝つ）。agent が `Incoming::Closed { remaining: 0 }`（最後の client が切れた）を渡したら、core はそれらをすべて既定に戻す（pick を止める、highlight と重ね表示を消す、override と forced を消す、emulation を空にする、時間の倍率を 1 にする、記録を止める）。review は残す。
- engine の `HostEvent::DevtoolsKey`・`DevtoolsPick`・`InjectDelivered` は dispatch（`client/dispatch/delivery.rs`）から `devtools_hooks::host_event(rt, event)` へ渡す。`client` は `devtools` を参照しない（`devtools` が `client` を参照する向きだけ）ので、hook はこの event を `SessionData.events`（`DevtoolsEvent` の列）に積むだけで、`Launch::turn` が event を配り終えた後に `devtools::run_events(&rt)` が root owner の handler として処理する（feature が無いときは hook が何もしない。engine は key と pick を設定されたときだけ出す）。`DevtoolsKey { window, pick: true }` は、core が先にその window で pick を始め（`pick` を `Some(window)` にして `Writer::set_pick`）、それから DevTools を開く（すでに接続していれば `focus_requested`）。
- 各 method の処理は `devtools/` の leaf module に分ける: `target.rs`（`dispatch` と、leaf ごとの `route` を順に呼ぶ表）、`tree.rs`、`node.rs`、`query.rs`、`overlay.rs`、`styles.rs`、`updates.rs`、`perf.rs`、`a11y.rs`、`input.rs`、`motion.rs`、`console.rs`、`emulation.rs`、`review.rs`、`scripts.rs`、`convert.rs`（Kivact の型 → protocol の型）、`start.rs`（agent の起動と engine へ渡す key）、`turn.rs`（`take_incoming`・`run_incoming`・`run_events`・`after_turn`・reset）。PNG の符号化・base64・trace の書き出しは agent（`kivact_devtools_agent::{encode_png, base64}`、`Agent::write_trace`）と protocol（`chrome_trace`）の仕事で、core には置かない。

#### 4.4.2 engine の口（すべて常にコンパイルする。`Writer`・`Reader` の `#[doc(hidden)]` method と理由付きの `internal-contract`）

| 口 | 形 | 内容 |
|---|---|---|
| 重ね表示 | `Writer::set_devtools_overlay(window, DevtoolsOverlay)` | `Host::present` の `model.composition(..)` の後、`window.present(..)` の前に、専用の `LayerKey::Devtools` の layer を作り、**root の layer（index 0）の items の最後に `LayerItem::Layer` で参照させる**（参照が無い layer は描かれない）。hit・layout・a11y・アプリの snapshot に入らない |
| pick | `Writer::set_pick(Option<WindowId>)` | 有効な window の Pointer・Wheel を `Host::handle` の入口で横取りし、modal を無視した hit test の結果を `HostEvent::DevtoolsPick { window, node, phase }` で出す。click で `Select`、Esc で `Cancel` |
| 起動キー | `Writer::set_devtools_keys(DevtoolsKeys { open, pick })` | Key Down が一致したら routing の前に横取りし `HostEvent::DevtoolsKey { window, pick: bool }`（key が届いた window と、pick 用の key かどうか）を出す。`Char` の比較は大文字小文字を区別せず、`Key::Unidentified` のときは `KeyInput.code` の文字で比べる |
| 記録 | `Writer::set_engine_recording(syncs: bool, frames: bool)`、`Reader::take_engine_records() -> EngineRecords` | 記録の有無と記録は、全 window の `Model` と `Host` が共有する `EngineLog`（`Off` か `Recording`、一つの列挙型）にだけ置く。`Model::sync`・`Model::layout` が `SyncRecord { window, revision, reasons, measured, laid_out, redrawn }` を、`Host::present` が `FrameRecord`（時間、`FrameStats`、その frame が描いた最新の revision）を積む。`EngineRecords` は core が毎 turn 取り出す受け渡しの列（sync 4096 件、frame 512 件。超えたら古いものから捨てて `dropped` に数える）。履歴（window ごと 3000 件）は core だけが持つ。無効なら何もしない |
| 注入 | `HostRequest::Inject { window, inputs: Vec<InjectedInput> }` | turn の後で `windows.inject(window, inputs, request)`（`request` は `Writer::effect` に渡した `RequestId`）。a11y action も `InjectedInput::Accessibility` として同じ列で順に配る（別の経路を作らない） |
| 注入の完了 | `HostEvent::InjectDelivered { window, request }` | `ShellEvent::Injected { request }` を受けたら出す。core は `inject_input` の答えをこれで返す |
| 型の再公開 | `kivact_engine::contract` から `InjectedInput`（`arvane::InjectedInput`。T5.2 が arvane の root で再公開している）・`RawInput`・`RawKey`・`PlatformKey`・`RawIme`・`RawDrop`（arvane-shell の型。core は arvane-shell に依存できない。`pipeline::Raw*` の型は arvane が再公開していないので、engine が arvane-shell に依存して、理由付きの `internal-contract` で再公開する） | |
| 箱の幾何 | `Reader::box_model(id) -> Option<BoxModelRects>` | margin・border・padding・content の矩形（window 座標）。`kivact-layout` の `NodeLayout` に `padding: Insets`、`border: Insets`、`margin: Insets` を足して taffy の結果から埋める |
| 確定値 | `Reader::effective_paint(id)`、`Reader::computed_text(id)`、`Reader::layout_props(id)` | 今 test host が `Model` から直接読んでいるものを Reader から読めるようにする |
| window | `Reader::windows()`、`Reader::window_of(id)`、`Reader::root(window)`、`Reader::focused_window()` | T6.1 が足す（OS の focus がある window は `AppModel.focused_window`） |
| hit | `Reader::hit_any(window, point) -> Option<NodeId>` | modal を無視する hit test |
| focus・pointer | `Reader::focused(window)`、`Reader::tab_order(window) -> Vec<FocusTarget>`、`Reader::pointer_state(id) -> (bool, bool)` | `pointer_state` は `Model::is_hovered`・`is_pressed`（hover と press） |
| レイアウト崩れ | `Reader::layout_findings(window, &LayoutCheck) -> Vec<LayoutFinding>` | `check/` を `cfg(debug_assertions, testing)` から外して常にコンパイルする |
| IME | `Reader::ime_state(window) -> ImeStateInfo` | |
| emulation | `HostRequest::Emulate(EngineEmulation { theme_hint, scale, touch })` | 全 window の環境に掛け、`EnvChanged` を出し直す。`touch` は `PointerSettings.primary` を `Coarse` にするだけ。`Capabilities.touch` は engine が上書きしない（core の `capabilities` の memo が掛ける） |
| 時間の倍率 | `HostRequest::TimeScale(f64)` | 各 window の `Model` の frame clock に倍率を掛ける（engine が動かす transition・smooth scroll に効く。`kivact_types::TimeScale`） |

`Host::handle` の入口の順（X26）: (1) 起動キー、(2) pick（その window が pick 中なら）、(3) modal の親への入力（Accessibility を含む）の破棄、(4) 今の処理。(3) は T6.1 が書き、(1)(2) は T10.1 が (3) の前に置く。

#### 4.4.3 runtime の口

| 口 | 内容 |
|---|---|
| `Runtime::set_detailed_trace(bool)` | `true` の間だけ、hidden な `TurnDetail`（cause・write・run・edge・stage の時刻）を記録する。`false` なら今と同じ（cost も同じ）。`Runtime::take_turn_details() -> Vec<TurnDetail>` が終わった turn の分を取り出す（`#[doc(hidden)]`、facade から再公開しない）。list ごとの上限は 2000 で、超えた分は `TurnDetail.dropped` に数える。`TraceWrite`・`TraceRun`・`TurnRecord` は変えない |
| cause の種類 | `pub enum CauseTag { Input, Timer, Task, Mount, Devtools }`（hidden）と `TurnGuard::cause_with(tag, label)`（`cause(String)` を置き換える）。core の event は `Input`、runtime の timer・task は `Timer`・`Task`、event が無く mount だけが due の turn は `Mount`、DevTools の呼び出しは `Devtools`。`TurnRecord.causes` の文字列の形は変えない。write と run の cause は「その時点の最後の cause」で、cause がまだ無いか上限で捨てた後なら `None`（0 にしない） |
| `Runtime::owner_info(OwnerId) -> Option<OwnerInfo { parent, created_at }>` | |
| `Runtime::live_snapshot() -> LiveSnapshot` | 生きている owner（id、parent、作成位置）と object（id、owner、名前、作成位置）。`Object` に `Owner::store` の名前を保存する。`OwnerId::raw()`・`NodeId::raw()`（`u64`）で protocol の id にする |
| `Runtime::reported() -> Vec<Reported>` | error と診断の ring を発生順に（`Reported { seq, handled, error }`）。`seq` は `Diag.next_seq` が ring に入れるときに振る通し番号。`errors()` は今の形のまま |
| `MotionState::animations() -> Vec<AnimInfo>`、`MotionState::set_time_scale(rt, f64)`、`MotionState::time_scale() -> f64`（今の倍率） | motion の時計に倍率（`kivact_types::TimeScale`）を掛ける（Animated・Timeline・FrameLoop）。timer と入力の時間には掛けない |
| `TaskState::list() -> Vec<TaskEntry>` | timer に作成位置と owner を保存する |

#### 4.4.4 shell・native の口

- `ShellLoop::inject`（3.3）。native は `NativeWindow.injected` に積み、callback が返った後（`pending_show` を処理するのと同じ場所）に `WindowInput::normalize` に通して配る。専用の wake の bit は作らない。

#### 4.4.5 スタイルの層と override

- 層の記録: core の `StyleLayer` を `Rc<dyn Fn() -> Style>` から `#[derive(Clone)] struct StyleLayer { f: Rc<dyn Fn() -> Style>, at: Option<&'static Location<'static>>, origin: LayerOrigin }` に変える。`#[derive(Clone)] enum LayerOrigin { Base, BaseRecipe(RecipeName), Recipe(RecipeName), Caller }`、`type RecipeName = Rc<dyn Fn() -> &'static str>`（DevTools が調べるときにだけ呼ぶ関数）。base の層の群（`Base` と `BaseRecipe`）は、今 `Common.base_styles: usize` が数えている「先頭の base の層」と同じものなので、`base_styles` の field は消し、`Common::base_count()` が `origin` から数える（同じ事実を二重に持たない）。`NodeBuilder::style`・`Node::style`・`Node::base_style` に `#[track_caller]` を付ける。
- 層の保存: `bind_style` は層を effect の closure に動かすので、DevTools が起動しているとき（`devtools_hooks::is_running(rt)`）だけ、`ViewNodes::set_style_layers(id, layers.clone())` で `CoreNode.style_layers`（feature のときだけある field）に写す。`styles::layers` はこれを読む。
- recipe の名前: `kivact-ui` の `Recipe`（`recipes.rs:80`）を `struct Recipe<S> { f: Arc<RecipeFn<S>>, name: &'static str }` にする。`From<F>` は `name: ""` で作り、`pub(crate) fn named(self, name) -> Self` を足す。`recipe_table!`（`recipes.rs:115`）は、既定の recipe と、kit が置き換えた recipe（`Recipe::from(f(prev))` の箇所）の両方で `.named(concat!(stringify!(<table>), ".", stringify!(<field>)))` を付ける（置き換えても名前が残る）。名前は theme を読まずに決まるので、`with_recipe_style`・`with_base_recipe_style`（`layer.rs:15, 31`）は `let name = pick(UiRecipes::default_table()).name();` を求めて `Rc::new(move || name)` を `RecipeName` として渡す（`recipe_table!` の `default_table` を `pub(crate) fn default_table() -> &'static Self` にする）。258 か所の呼び出しは変えない。
- 層の表示（`get_node` の `styles`）: 上から `Devtools`（override）、`State`（hover などの差分。forced なら印）、`Disabled`、`Selected`、`Caller`（呼び出し順の逆）、`Recipe`、`Base`。`ThemeDefault` と `Inherited` は出さない（core が層として持っていない）。各層の property は `kivact_style::internal::describe_style(&Style) -> Vec<(&'static str, String)>`（新規。`classify_properties!` が値も返すようにする。値は `Debug` の文字列）で作る。state の差分は `describe_state_styles(&Style)` で取る。下の層で上書きされた property は `overridden: true`。
- override の適用:
  - node と呼び出し位置: `bind_style`（`view/mount/style.rs`）は node ごとに一度、`StyleHook::new(rt, owner, id, at)` を作る。`StyleHook` は、その node に当たる override の `Value`（`memo`）と、その node の forced states の `Value`（`memo`）を持つ（表全体を読む `Signal` を style effect が直接読まない。どの node の override を編集しても、他の node の style effect は再実行されない）。style effect は `hook.apply(merge_layers(&layers))` と `Interactive { forced: hook.forced(), .. }` を使う。feature が無いときの `StyleHook` は何も持たず、`apply` は入力をそのまま返す。
  - 呼び出し位置の override の `location` は、その node の `call_chain` の最初の `app_code: true` の frame（無ければ先頭）。node の作成位置と owner の作成位置の列（`devtools_apply::call_sites`）のどれかがその位置と一致する node に当てる。
  - theme token: `install_theme` に渡す theme を、token override を当てた memo で包む（root と `provide_theme` の両方がここを通る）。`scope` が `Token` の override は、`property` が `token` と同じ名前でなければ `-32602`、`value` は `ThemeToken.kind` の書き方で検査する（`Color` は色、`Length`・`Number` は数、`Text` は文字列）。
  - 優先順位: 呼び出し位置 < node（両方あれば node が勝つ）。
- override の property と値の書き方（`set_style_override`、`scope` が `Node`・`CallSite` のとき）:

| property | 値の例 | `Style` の setter |
|---|---|---|
| `width`、`height`、`min_width`、`min_height`、`max_width`、`max_height` | `120`、`50%`、`auto` | 同名 |
| `padding`、`padding_top`、`padding_right`、`padding_bottom`、`padding_left` | `12` | 同名 |
| `margin` と 4 辺 | `8` | 同名 |
| `gap`、`row_gap`、`column_gap` | `8` | 同名 |
| `grow`、`shrink`、`opacity` | `1`、`0.5` | 同名 |
| `background`、`color`、`border_color` | `#2563eb`、`rgb(37 99 235)`、`Accent`（`ColorToken` の名前） | 同名 |
| `border_width`、`radius`、`font_size`、`line_height` | `1`、`6`、`14`、`1.5` | 同名 |
| `weight` | `400`、`700` | 同名 |
| `text_align` | `start`、`center`、`end` | 同名 |
| `display` | `flex`、`grid`、`contents` | 同名 |

  表に無い property や読めない値は error `-32602`（message に受け付ける書き方を書く）。
- forced states: `kivact_style::internal::Interactive` に `forced: ForcedStates`（hover・pressed・focus・focus_visible・disabled・selected の bit の集合）を足す。`compile_style` は forced の状態の差分を base に merge し、その状態を `StateStyles` から外す。disabled と selected の forced は `state.disabled`・`state.selected` に OR する。

#### 4.4.6 i18n と emulation の口

- locale の上書き: `mount_root` の root locale を `devtools_hooks::wrap_locale(owner, locale)` で包む（emulation の locale があればそれ、無ければ元の値の `memo`。feature が無いときは入力のまま）。
- pseudo-locale: `message()` と `message_with()` の memo の中で、文字列を決めた後に `devtools_hooks::pseudo(text)` を通す（4.2.4 の規則。feature が無いときは入力のまま）。
- reduced motion は既存の `HostRequest::ReducedMotion` を使う。theme・scale・touch は `HostRequest::Emulate`。
- touch の上書きで `Capabilities` が変わる: core は host の値を `ViewState.host_capabilities: Signal<Capabilities>`（host の event からだけ書く。T7.1）に持ち、`capabilities: Value<Capabilities>` は `devtools_hooks::wrap_capabilities(owner, host_capabilities)` が作る memo にする（emulation の `touch` を `Capabilities.touch` に掛ける。書き込みは無い。`ViewState::new` は App の構築中に呼ばれ、extension がまだ無いので、`DevtoolsState` は memo の中（最初に読まれたとき）で読む。feature が無いときは `memo(|| host_capabilities.get())` のまま）。engine は `Capabilities.touch` を上書きしない。

### 4.5 DevTools アプリ（`kivact-devtools`）

#### 4.5.1 起動と接続

- `kivact-devtools [--target <discovery file>]`。target は MCP・Python・`KIVACT_DEVTOOLS_TARGET`・`screen.devtools_target()` と同じ「discovery file の path」で、フラグ名も `--target` に揃える。`--target` が無ければ、discovery の一覧から target を選ぶ画面を出す（1 つしかなければそれに接続）。`--pick` は無い: Ctrl+Shift+C で DevTools が開くときは、target の core が先にそのキーを押した window の pick を始めている（4.4.2）ので、アプリは `pick` の通知に従うだけでよい。
- 接続の状態は明示的な enum で、**型はすべて `state.rs` に置く**（`connection.rs` や `source.rs` に enum を置くと module が循環する）。持つのは plain な data だけ（`Session` の signal はここに入れない。signal と resource は setup の中でしか作れず、接続の確立は task の中で起きるため）:

  ```rust
  /// What the app is connected to (plain data, so it can be a `dynamic` source).
  #[derive(Clone, PartialEq)]
  pub enum Connection {
      /// Listing the running targets (the first screen without `--target`).
      Searching,
      /// The list to choose from (empty: nothing is running).
      Choosing(Vec<(PathBuf, p::DiscoveryFile)>),
      Connecting(PathBuf),
      Connected(Target),
      /// The connection ended or could not be opened.
      Lost { path: PathBuf, app_name: String, reason: String },
  }
  #[derive(Clone, PartialEq)]
  pub struct Target {
      pub source: Source,
      pub app: p::AppInfo,
      pub windows: Vec<p::WindowInfo>,       // from list_windows at connect time
      pub config: Rc<p::DevtoolsConfig>,     // from get_config at connect time
      pub path: Option<PathBuf>,             // the discovery file; None for a test fixture
  }
  ```

- 接続している間だけ意味を持つ data は `Session` に入れる（S-3）。`Session` は **Copy の handle の束**で、`Connected(Target)` の枝が作られるときの setup の中で `Session::new(target, connection_signal)` が一度だけ作る（枝が変われば `Session` ごと捨てられ、`Session::new` が起こした task も止まる）:

  ```rust
  #[derive(Clone, Copy)]
  pub(crate) struct Session {
      pub(crate) target: Var<Target>,                    // written once, never changes while the branch lives
      pub(crate) connection: Signal<Connection>,         // to move to Lost
      pub(crate) windows: Signal<Vec<p::WindowInfo>>,
      pub(crate) inspected: Signal<Inspected>,
      pub(crate) turns: Signal<Vec<p::TurnGraph>>,       // newest last, at most 1000
      pub(crate) frames: Signal<Vec<p::FrameInfo>>,      // at most 3000
      pub(crate) errors: Signal<Vec<p::ErrorEntry>>,     // at most 1000
      pub(crate) tables: Signal<Vec<p::ShowTableParams>>,// show_table results, at most 50
      pub(crate) panel: Signal<Panel>,
      pub(crate) popped: Signal<Vec<Panel>>,             // panels shown in their own window
      pub(crate) status: Signal<Option<String>>,         // the last failure to show in the status bar
      pub(crate) detail: Resource<Option<p::NodeDetail>>,
      pub(crate) recording: Resource<p::RecordingSettings>,
      pub(crate) overrides: Resource<p::StyleOverrideList>,
      pub(crate) emulation: Resource<p::Emulation>,
      pub(crate) overlays: Resource<p::OverlaySettings>, // get_overlays
      pub(crate) review: Resource<p::ReviewDocument>,
  }
  ```

  `set_overlays` は全体を置き換えるので、Updates・Performance・Accessibility の 3 つの panel が互いの toggle を消さないよう、`Session::set_overlay(edit)` が「`overlays` の今の値（読み込み前は全部 false）を取り、`edit` で 1 つ変え、全体を `set_overlays` で送り、`overlays.refetch()`」を行う。値の持ち主は agent で、アプリは写しを持たない（`get_overlays` の resource で読む。他の client と同時に使うと最後の書き込みが勝つ）。toggle は resource の値を表示し、読み込み中は無効にする。

  調べている window とその tree と選んだ node は一つの enum にする（window を変えたら tree と選択も一緒に変わるので、別々に持つと補修の書き込みが要る）。選んだ node を別の window の pick で指すとき、その tree を読み込む間に選択を覚えておく slot が要るので `Loading` も `select` を持つ:

  ```rust
  pub(crate) enum Inspected {
      Loading { window: p::WindowId, select: Option<p::NodeId> },
      Loaded { window: p::WindowId, tree: p::TreeSnapshot, selected: Option<p::NodeId> },
  }
  ```

  `Inspected` の遷移は `state.rs` の純粋なメソッド（runtime なしで試験できる）に閉じる: `window()`、`selected()`、`select(Option<NodeId>)`、`finish_load(TreeSnapshot)`（今が同じ window の `Loading` のときだけ `Loaded { selected: select（新しい tree に無ければ None） }` にし、それ以外の答えは捨てる）、`apply_diff(&TreeChanged)`（`Loaded` で同じ window のときだけ、`upserted` で同じ id を置き換え（無ければ足し）、`removed` を消し、`revision` を更新し、選んだ node が消えたら `None`）、`select_in(window, node) -> bool`（`Loaded` で同じ window かつ tree に node があれば `selected` を変えて false、それ以外は `Loading { window, select: Some(node) }` にして true = 呼んだ側が `get_tree` を呼ぶ）、`keep_window(&[WindowInfo]) -> bool`（今の window が一覧に無ければ `Loading { window: MAIN, select: None }` にして true）。選んだ node の詳細は、`selected` を source にした `kivact::task::resource`（`get_node` を呼ぶ。選択が無ければ `None`）。`overrides`・`emulation`・`recording`・`overlays` も agent が持つ事実なので、アプリは写しを保存せず resource（`list_style_overrides`・`get_emulation`・`get_recording`）で読み、変更の呼び出しの後に `refetch()` する。resource は method の形ではなく `Session` の field として一度だけ作る（method で作ると呼ぶたびに新しい resource ができ、setup 以外では作れない）。
- 通信: data の出どころは `Source { Live(Arc<client::Connection>), Fixture(Rc<Fixture>) }`（`source.rs`）。`Source::call::<M>(params)` は async で、`Live` は `kivact::task::blocking` の中で `Connection::call::<M>` を呼ぶ。panel は `Session::send::<M>(params, done)`（task を起こして `call` を待ち、成功なら UI thread で `done(result)`、失敗なら `status` に `"<method>: <message>"` を書く）を使う。失敗の扱いを panel ごとに書かせないための共通の入口である。**`Lost` にするのは通知の loop だけ**: `Source::listen` が `blocking(|| connection.next_notification(500ms))` を繰り返し、`Err(_)`（client の接続が閉じた）で `Connection::Lost { path, app_name, reason }` にして終わる。call の失敗は error として `status` に出すだけにする（切断なら loop が 500ms 以内に気づく）。loop は `Session::new` が起こすので、`Session` が消えると止まる（「Reconnect」が 2 本目を起こしても 1 本のまま）。`Lost` の画面は「Reconnect」（`list_targets()` の中に同じ path の file があればそれ、無ければ `app_name` が同じものが 1 つだけならそれ、それ以外は `Choosing`）と「Choose another target」を出す。
- 版: `DiscoveryFile.protocol` が `PROTOCOL_VERSION` と違う target は、接続せずに `Lost`（reason は「版が違う (target v<N>, DevTools v<M>)」）にする。`initialize` が `VERSION_MISMATCH`（-32003）を返したときも同じ文にする。
- 購読: `Session::new` は接続した直後に、`subscribe` で全 topic（`Tree`・`Turns`・`Frames`・`Pick`・`Errors`・`Devtools`・`Scripts`・`Windows`）を購読する（agent は購読された topic にしか通知しないので、これが無いと tree も pick も届かない）。
- 通知の扱い（`Session::apply(notification)`。`session.rs`）:

  | 通知 | 扱い |
  |---|---|
  | `tree_changed` | `inspected` の `apply_diff(&d)` を呼ぶ（`Loaded` で同じ window のときだけ効く。差分であり snapshot ではない） |
  | `turn_recorded`・`frame_recorded`・`error_reported`・`script_table` | それぞれ `turns`・`frames`・`errors`・`tables` の末尾に足す（上限を超えたら古い方から捨てる） |
  | `pick` | `Cancel` は何もしない。`Hover` と `Select` は `panel` を Elements にする。`Select { window, node: Some(node) }` は `inspected` の `select_in(window, node)` が true を返したら `load_window(window, Some(node))` |
  | `windows_changed` | `windows` を置き換え、`inspected` の `keep_window(..)` が true なら `load_window(MAIN, None)`（調べている window が閉じたら main に戻る） |
  | `focus_requested` | `kivact::platform::focus_window()` を呼ぶ（1 章の D-16。自分の window を前面に出す） |

  `load_window(window, select)` は `inspected` を `Loading { window, select }` にして `get_tree` を呼び、答えを `finish_load` に渡す。別の window へ切り替えたあとに古い window の答えが着いても、`finish_load` が捨てる。
- 外観: `kivact_ui_basic::theme`（Light/Dark は OS の `theme_hint` に従う）、文字は Inter と Geist Mono（`examples/shared-assets/fonts` から `include_bytes!`）。
- 型の名前: protocol の型は `use kivact_devtools_protocol as p;` で `p::Role` のように書き、client は `use kivact_devtools_client as client;` で `client::Connection` と書く（prelude の `Role`・`Rect`・`Size` とぶつけない。アプリの `Connection` と client の `Connection` も別物）。

#### 4.5.2 画面の構成

```
┌ toolbar ─ [target 名] [window の select] [pick] [記録] [review (件数)] … [Pop out] ┐
├ panel の tab: Elements | Updates | Performance | Accessibility | Input | Animations | Async | Theme | Console | Emulation | Review | Scripts ┤
│ （選んだ panel の中身）                                                         │
└ status bar: 接続先・記録中か・emulation 中か・override の数・最後の失敗         ┘
```

- 今の panel は toolbar の「Pop out」で別の window に出せる（3 章の `window()` を使う。dogfooding。出した panel は `Session.popped` に入り、その window を閉じると外れる）。
- module の構成（`crates/kivact-devtools/src/`。型は `state.rs` に集め、依存は一方向）: `lib.rs`、`main.rs`、`args.rs`、`fonts.rs`、`source.rs`、`state.rs`（`Connection`・`Target`・`Session`・`Inspected`・`Panel`）、`session.rs`（`impl Session`）、`connection.rs`（target の選択・接続・`Lost` の画面）、`editor.rs`、`app.rs`、`toolbar.rs`、`status_bar.rs`。panel ごとの module: `panels/elements.rs`（と `elements/` の leaf）、`panels/updates.rs`、`panels/performance.rs`、`panels/accessibility.rs`、`panels/input.rs`、`panels/animations.rs`、`panels/tasks.rs`、`panels/theme.rs`、`panels/console.rs`、`panels/emulation.rs`、`panels/review.rs`、`panels/scripts.rs`。共通部品は `widgets/`（仮想化した tree、property の行、color の入力、source の link、timeline の canvas、graph の canvas、flame chart の canvas、箱の図）。

#### 4.5.3 各 panel の仕様

| panel | 中身 | 使う部品 |
|---|---|---|
| Elements | 左: 仮想化した element tree（`use_tree` + `virtual_list`。行に role・name・作成位置・更新回数の badge。Contents と退場中は薄く）。「Owners」の toggle で、同じ仮想化 tree が `get_owners` の owner tree（行は label と作成位置。行を選ぶとその owner の最初の node を選ぶ）に変わる。右: tab（Styles／Computed／Layout／Semantics／Why／Handlers）。行を選ぶと `highlight`。pick ボタンと Ctrl+Shift+C（調べている window の `start_pick`）。作成位置をクリックすると editor で開く（設定の `editor` の command を起動する。4.2.7）。「Copy locator」（protocol の `p::locator(&ReviewItem)` が返す Rust の文を clipboard へ。review の Markdown の `Query:` 行と同じ関数で、test id があればそれ、無ければ role + name、副 window は `screen.window("<title>").` を前に付ける） | tree（自作の仮想化）、`split`、`tabs`、`toolbar`、`badge`、`tooltip` |
| Styles（Elements の tab） | 層ごとの property（4.4.5）。各行に checkbox（override の有効・無効）と値の text input（Enter で `set_style_override`）。color は color の入力（hex の text input・HSL の slider・見本）。forced states の toggle。override の範囲の select（このノード／同じ呼び出し位置／token）。「変更を review に記録」 | `text_input`、`checkbox`、`toggle_group`、`select`、`slider`、`dialog`（popover） |
| Computed・Layout | 箱の図（margin・border・padding・content を入れ子の枠で描く canvas）、確定値の一覧、flex/grid の重ね表示の toggle（`highlight` の `layout: true`。子の箱の外形と隙間を target の画面に重ねる） | canvas、`description_list` |
| Semantics・Why・Handlers | 意味情報の一覧、why の依存グラフ（Updates と同じ graph の部品）、handler の一覧 | graph canvas、`description_list` |
| Updates | 上: 時間軸のレーン（入力・更新・フレーム。きっかけで色分け、予算線と超過の赤）。範囲選択で拡大。下: 選んだ turn の原因グラフ（左から右へ、きっかけ → 状態の変化 → 再計算 → 変わった要素 → layout/paint/frame の帯）。カードのクリックで target に強調、ダブルクリックで editor。まとめた node は「×N」で展開できる。時間の内訳の帯と件数（変わった要素・再計算・同じ値の書き込み）。「画面で点滅」「選んだ要素だけ」の切り替え。画面上のヒートマップ（`set_overlays`） | canvas + region、`toggle_button` |
| Performance | フレーム時間の棒（予算線、window の select）、turn の Stage と frame の段階の flame chart、GPU/CPU と fallback の表示、Counters の折れ線（`basic::chart` の Line）、snapshot の取得と差分の表、「Perfetto 形式で書き出す」、描き直した領域の点滅（paint flashing）の toggle | canvas、`basic::chart`、`data_table` |
| Accessibility | a11y tree、監査の結果の表（行を選ぶと強調）、Tab 順の番号表示の toggle、focus の履歴 | 仮想化した tree、`data_table` |
| Input | 入力の log の表（種類・target・経路・handler の結果）、IME の状態 | 仮想化した表（`virtual_list` の行） |
| Animations | 動いている animation の一覧、時間の倍率（停止・0.1・0.25・0.5・1）の toggle group | `data_table`、`toggle_group` |
| Async | task・resource・action・timer の一覧、blocking の数 | `data_table` |
| Theme | token の一覧（色は見本付き）、値の編集（token の override） | property の行、color の入力 |
| Console | error と notes の表（種類・message・位置・turn・handled）、種類の filter、位置の link | 仮想化した表、`select` |
| Emulation | theme（なし/Light/Dark）、reduced motion、locale（text input）、pseudo-locale、DPI 倍率、touch | `select`、`switch`、`text_input` |
| Review | コメントの一覧（item ごと。node へ飛ぶ・強調。今の tree に無い node の item には「(removed)」を出し、強調を無効にする）、コメントの追加・編集・削除、「Open folder」（`review.json` と `review.md` は変更のたびに自動で保存されるので書き出しの操作は無い。ボタンは `export_review` の結果の path の directory を開く） | `item_row`、`text_area`、`button` |
| Scripts | `.kivact/devtools/scripts/` の `.py` の一覧、実行ボタン、標準出力と標準エラーの表示、`show_table` の表、interpreter の表示 | `list`、`code_block`、`data_table` |

#### 4.5.4 足りない部品の扱い

- 仮想化した tree: DevTools アプリの `widgets/virtual_tree.rs` に作る（`use_tree` + `virtual_list`）。選択は呼び出し側の `Binding<Option<K>>` を受け取る（`use_tree` はそのまま受け付ける）ので、`Inspected` の `selected` を `binding(..)` で渡せる。選択の写しを作らない。そのために `kivact-interaction` の `TreeApi` に `scroll_with(f)` と `page_size(n)` を足す（内部の `ListApi` へ渡すだけの最小の公開 API の追加。同じ変更で E2E を足す）。
- color の入力、property の行、箱の図、timeline、graph、flame chart は DevTools アプリの中に作る（kit には入れない。6 章の取り込みの規則 1）。

### 4.6 MCP（`kivact-devtools-mcp`）

- MCP の stdio transport（1 行 1 JSON-RPC）を手で実装する（依存を増やさない）。対応する request: `initialize`、`notifications/initialized`、`ping`、`tools/list`、`tools/call`。
- `initialize` の `protocolVersion` は、client が送った版をそのまま返す（`2025-06-18` 以降を想定）。`capabilities` は `{ "tools": {} }`。
- tool の一覧: 4.2.5 の method から `initialize`・`subscribe`・`show_table` の 3 つだけを除いたすべて（同じ名前。`get_config`・`get_owners`・`get_overlays` も tool）に、local の 2 つを足す。2 つの `inputSchema` は `tools.rs` に手で書く:
  - `list_targets`（params なし）: 起動中の target の一覧。結果は `{ "targets": [ { "target": "<discovery file の path>", "app_name", "pid", "port", "kivact_version", "started_unix_ms", "workspace_root" } ] }`（`token` は出さない。AI の context に token を入れないため）。
  - `select_target`（`{ "target": "<discovery file の path>" }`）: 以後の tool の対象を選ぶ。pid ではなく path で選ぶのは、同じ process に複数の App があり得て、path が Python の `connect(target=)`・`KIVACT_DEVTOOLS_TARGET`・DevTools アプリの `--target` と同じ指し方だから。
- 対象の決め方: `select_target` で選んだもの → 環境変数 `KIVACT_DEVTOOLS_TARGET` → 起動中の target が 1 つならそれ。どれでも決まらないとき（0 個か 2 個以上）は error（`list_targets` を呼ぶよう message に書く）。選んだ target の接続が閉じたら（`ClientError::Closed`。client の読み込み loop は EOF でも I/O の失敗でも `Closed` で終わるので、`Closed` だけを見ればよい）選択を捨て、`"target closed; call list_targets"` の error を返す。
- `inputSchema` は生成した schema（protocol crate の `SCHEMA_JSON`）から params の型を取り出し、`$ref` を展開したもの（params に再帰型は無い）。
- 結果: `structuredContent` は**常に object**（MCP の仕様。配列は返さない）で、result の JSON。`content` に同じ JSON の文字列（`type: "text"`）。`screenshot` は `content` に `type: "image"`（`data` は base64、`mimeType: "image/png"`）と、`structuredContent` に `{ "width", "height" }`。protocol の error は `isError: true` と `"<code>: <message>"`。
- `inject_input` の後の turn は、AI が `list_turns` に `wait_ms` を付けて待つ（subscribe が無いので、通知を待つ代わり）。
- stdout への書き込みは `std::io::stdout().lock()` に書く（clippy の `print_stdout` を避ける）。log は stderr。
- 設定例（利用者ガイドに載せる）: 先に `cargo install --git https://github.com/Kashiwade-music/kivact kivact-devtools kivact-devtools-mcp` で PATH に入れ、`claude mcp add kivact-devtools -- kivact-devtools-mcp`。

### 4.7 Python（`python/kivact-devtools`）

#### 4.7.1 構成

```
python/kivact-devtools/
  pyproject.toml            （setuptools、name = "kivact-devtools"、requires-python = ">=3.10"、依存なし）
  README.md
  src/kivact_devtools/
    __init__.py             （公開する名前の再公開）
    py.typed
    _generated.py           （生成。手で編集しない）
    _rpc.py                 （socket・JSON-RPC・通知の受信）
    _discovery.py           （discovery の一覧）
    client.py               （connect、App、Window、Element、Recording）
    errors.py               （DevtoolsError、NotFound、MultipleFound、Timeout）
  tests/test_client.py      （unittest。標準ライブラリの socketserver で作った偽の agent）
  examples/                 （scripts の例: list_buttons.py、record_click.py）
```

#### 4.7.2 生成する型（`_generated.py`）

- 名前付き field の struct → `@dataclass(frozen=True, slots=True)`。`Option<T>` → `T | None = None`、`Vec<T>` → `list[T] = field(default_factory=list)`（既定のある field は後ろに並べる）。各 class に `from_json(cls, data: dict[str, Any])` と `to_json(self) -> dict[str, Any]`。
- unit enum → `class X(str, Enum)`（値は snake_case の文字列。member 名は UPPER_SNAKE_CASE）。
- tag 付き enum → variant ごとの `@dataclass`（class 名は `<Union の名前><Variant の名前>`、例 `OverrideScopeNode`。最後の field に `kind: Literal["node"] = "node"`）と、その union の別名 `X = A | B | ...`、`x_from_json(data)`（関数名は union の名前の snake_case + `_from_json`）。
- dataclass の field は「既定の無いもの → 既定のあるもの」の順に並べ替える（Python の規則）。`from_json`・`to_json` は field の名前で読み書きするので、並べ替えは JSON に影響しない。
- Rust の field 名が Python の予約語（`keyword.kwlist`）と同じなら、Python の属性名の末尾に `_` を付け、JSON の key は元の名前のままにする（protocol の型は予約語を避けて名付けてあるが、生成器はこの規則を持つ）。
- 別名 → `NodeId = int` など。
- method → 公開の `class Methods`（生成物。手書きの `App` とは継承の関係を持たない）。`def __init__(self, call: Callable[[str, dict[str, Any]], Any]) -> None` と、protocol の method ごとに wire の名前の関数を 1 つ: `def get_tree(self, params: GetTreeParams) -> TreeSnapshot: return TreeSnapshot.from_json(self._call("get_tree", params.to_json()))`。params が `Empty` の method は引数を取らず（`def list_windows(self) -> WindowList`）、result が `Empty` の method は `None` を返す。
- docstring は Rust の `///` から。

#### 4.7.3 手で書く API（`client.py`）

手書きの層は `Methods` を継承しない（生成した低水準の method と、同じ名前で形の違う手書きの method がぶつかり、`pyright --strict` の `reportIncompatibleMethodOverride` で落ちるため）。低水準は `app.rpc`、手書きは `App`・`Window`・`Element`・`Recording` の 4 つで、名前が重ならない。名詞の読み取りは Rust の API と同じく property ではなく method（`app.windows()`）。見つけた node を包む class は `kivact::test` と同じ `Element`（`kivact::view::Node` と混ざらない）で、包まれる protocol の型は `ElementInfo`。

```python
def connect(*, target: str | os.PathLike[str] | None = None, name: str | None = None) -> App
    # target: discovery file の path。name: AppInfo.name（App の title）の完全一致。2 つ以上なら MultipleFound
    # どちらも無ければ KIVACT_DEVTOOLS_TARGET → 起動中の target が 1 つならそれ → それ以外は error
class App:                     # with 文で使える。close() もある
    rpc: Methods               # 生成した低水準の method のすべて（wire の名前。app.rpc.get_tree(GetTreeParams(...))）
    info: AppInfo
    def windows(self) -> list[Window]
    def window(self, title: str) -> Window                       # NotFound / MultipleFound
    def record(self, *, updates: bool = True, frames: bool = True, input: bool = False) -> Recording
    def clear_highlight(self) -> None
    def export_review(self) -> list[Path]                         # [review.json, review.md] の path
    def show_table(self, title: str, columns: Sequence[str], rows: Sequence[Sequence[str]]) -> None
    def close(self) -> None
class Window:
    info: WindowInfo
    # 12 の query: get_by_role(role: Role, name: str | None = None) -> Element、query_* -> Element | None、
    #              find_*(…, timeout: float = 5.0) -> Element、all_* -> list[Element]
    #              （*_by_text(text)、*_by_test_id(id) も同じ形。name=None は name を比べない）
    def tree(self) -> TreeSnapshot
    def type_text(self, text: str) -> None
    def press_key(self, key: str) -> None                         # 4.2.7 の chord の書き方（"Enter"、"Ctrl+Enter"）
    def screenshot(self) -> bytes                                 # PNG
    def add_comment(self, text: str, *, screenshot: bool = True) -> ReviewItem
    def lint_layout(self) -> LayoutLint
    def audit_a11y(self) -> AuditReport
class Element:
    info: ElementInfo
    source: SourceLocation | None                                  # info.source。呼び出しは要らない
    def detail(self) -> NodeDetail
    def highlight(self) -> None
    def click(self) -> None
    # 12 の query（この element の中だけ）
    def set_style_override(self, property: str, value: str, *, scope: Literal["node", "call_site"] = "node") -> StyleOverride
    def add_comment(self, text: str, *, screenshot: bool = True) -> ReviewItem
class Recording:               # with 文。出た後に turns: list[TurnGraph]、frames: list[FrameInfo]
```

例:

```python
import kivact_devtools as kd

app = kd.connect(name="Counter")
win = app.window("Main")                                   # title で選ぶ（無ければ NotFound）
button = win.get_by_role(kd.Role.BUTTON, "Increment")      # Element
button.detail().bounds                                     # NodeDetail（get_node）
button.highlight(); app.clear_highlight()
button.click(); win.type_text("abc"); win.press_key("Ctrl+Enter")
with app.record(updates=True, frames=True) as rec:
    button.click()
for turn in rec.turns: print(turn.turn, turn.duration_ns, len(turn.changed_nodes))
button.set_style_override("padding", "12")                 # scope は既定でこのノード
button.add_comment("ラベルを「1 増やす」にしたい")
app.export_review()
app.show_table("Buttons", ["name"], [[e.info.name or ""] for e in win.all_by_role(kd.Role.BUTTON)])
```

- `Recording` は `__enter__` で、今ある最新の turn の番号を `since` に覚えてから（`list_turns(limit=1)`）`start_recording` を呼び、`__exit__` で `stop_recording` を呼んでから `list_turns(since=since)` を読んで `turns` に入れる（`stop_recording` が待っている graph を先に完成させるので、`with` の中で起こした操作の turn が含まれる）。`frames` は `turns` の各 `TurnGraph.frames` を並べたもの。
- 例外: `NotFound`（`NO_MATCH`・`NODE_NOT_FOUND`・`WINDOW_NOT_FOUND`）、`MultipleFound`（`MULTIPLE_MATCHES`）、`Timeout`（`TIMED_OUT`）、それ以外は `DevtoolsError`（`code` と `message`）。code の数は手で写さず、生成した `ErrorCode`（`_generated.py`）を使う。`initialize` の結果の `protocol` が `PROTOCOL_VERSION` と違えば `VERSION_MISMATCH` の `DevtoolsError`。
- 接続が閉じたら、待っている call も以後の call も `DevtoolsError(INTERNAL_ERROR, "connection closed")` で終わる。call は既定で最大 30 秒待ち、超えたら `Timeout`。

#### 4.7.4 導入と実行

- 導入（Kivact の repo の外でも使える）: `pip install "kivact-devtools @ git+https://github.com/Kashiwade-music/kivact#subdirectory=python/kivact-devtools"`。Kivact の repo を編集する人だけ `pip install -e python/kivact-devtools`。
- VS Code: 普通に実行・debug する。型は `py.typed` と型ヒントで Pylance が読む。
- Scripts パネル: interpreter は (1) 設定の `python`、(2) workspace root の `.venv` の python（`.venv/Scripts/python.exe`、`.venv/bin/python` の順に存在を確かめる。`cfg` は使わない）、(3) PATH の `python3`、(4) `python`。環境変数 `KIVACT_DEVTOOLS_TARGET` と `PYTHONIOENCODING=utf-8` を付けて起動し、標準出力と標準エラーを行ごとに表示する。
- 型の品質: `pyright --strict python/kivact-devtools/src`（pyright がある環境だけ。作業指示書の検査の段階で実行する）。`python -m unittest discover python/kivact-devtools/tests`。

### 4.8 review の Markdown

protocol crate の `p::to_markdown(&ReviewDocument) -> String`（純粋関数。`lib.rs` から直接出す。`p::locator` も同じ）が次の形で書く。agent の `ReviewStore` は変更のたびに `review.json` と同じ内容の `review.md` を書く（4.3）:

````markdown
# DevTools review: <app.name>

- Session: <session> / Updated: <updated>
- Build: <build> / Kivact <kivact_version>
- Emulation: <emulation の要約。上書きが無ければ none>

## C1 <Role の Rust の variant 名> "<name>"  (window "<window_title>")

- Source: <file>:<line>:<column>
- Call chain: <app_code の frame を先に、`file:line` を ` ← ` でつなぐ>
- Query: <locator>
- State: <states をカンマ区切り。無ければ none> / Bounds: x <x>, y <y>, <width>×<height>
- Image: images/C1.png
- Style changes:
  - <property>: <before または (unset)> → <after>  (<scope>)
- Comments:
  1. <text>  (<created>)
  2. <text>  (<created>)
````

- `<locator>` は protocol の `p::locator(&ReviewItem) -> Option<String>` の結果で、DevTools アプリの「Copy locator」と同じ関数である。test id があれば `screen.get_by_test_id("<id>")`、無ければ `screen.get_by_role(Role::<Rust の variant 名>, "<name>")`。main でない window の node には `screen.window("<window_title>").` を前に付ける。role の名前は serde の `check_box` ではなく Rust の `CheckBox` で書く。
- window へのコメントは見出しを `## C2 window "<title>"` にし、Source・Call chain・Query・State・Bounds の行を書かない。
### 4.9 型の追補（作業指示書が使う、まだ定義していない内部の型）

```rust
// kivact-types: `TimeScale { anchor_host_ns, anchor_motion_ns, scale }` と `motion_ns`・`rescaled` は T2.4 が motion.rs に作る（runtime と engine が使う一つの型。ここでは定義しない）

// kivact-runtime（4.4.3。reactive/detail.rs と各 file。TurnDetail 系は #[doc(hidden)]）
pub struct OwnerInfo { pub parent: Option<OwnerId>, pub created_at: &'static Location<'static> }
pub struct LiveOwner { pub id: OwnerId, pub parent: Option<OwnerId>, pub created_at: &'static Location<'static> }
pub struct LiveObject { pub id: u64, pub owner: OwnerId, pub name: &'static str, pub created_at: &'static Location<'static> }
pub struct LiveSnapshot { pub owners: Vec<LiveOwner>, pub objects: Vec<LiveObject> }
pub enum AnimKind { Animated, Timeline, FrameLoop }
pub struct AnimInfo { pub kind: AnimKind, pub created_at: Option<&'static Location<'static>>, pub owner: OwnerId,
    pub running: bool, pub paused: bool, pub decorative: bool }
pub struct TaskEntry { pub created_at: &'static Location<'static>, pub owner: Option<OwnerId>, pub deadline_ns: u64 }
pub struct Reported { pub seq: u64, pub handled: bool, pub error: UiError }
pub enum CauseTag { Input, Timer, Task, Mount, Devtools }
pub struct CauseEntry { pub tag: CauseTag, pub label: String }
pub struct StageSpan { pub stage: Stage, pub start_ns: u64, pub duration_ns: u64 }   // start_ns は loop の時計の絶対値
pub struct DetailWrite { pub node: NodeId, pub cause: Option<u32>, pub ty: &'static str,
    pub created_at: &'static Location<'static>, pub at: &'static Location<'static>, pub changed: bool }
pub struct DetailRun { pub node: NodeId, pub cause: Option<u32>, pub kind: &'static str,
    pub at: &'static Location<'static>, pub changed: bool }
pub struct DetailEdge { pub source: NodeId, pub target: NodeId }
pub struct TurnDetail { pub turn: u64, pub duration_ns: u64, pub causes: Vec<CauseEntry>, pub writes: Vec<DetailWrite>,
    pub runs: Vec<DetailRun>, pub edges: Vec<DetailEdge>, pub stages: Vec<StageSpan>, pub dropped: u32 }
// OwnerId::raw() -> u64、NodeId::raw() -> u64（protocol の id にする。identity.rs）
// &'static Location の公開 field には、TraceWrite と同じ理由付きの
// `// xtask-allow(api-shape): diagnostic source positions retain Rust caller locations.` を付ける。
// TurnDetail 系の型と関数には `#[doc(hidden)]` と
// `// xtask-allow(internal-contract): the core's DevTools reads the detailed trace.` を付ける。

// kivact-engine（4.4.2。contract/devtools.rs。`kivact_engine::contract` は state-fields の value_modules）
pub struct BoxModelRects { pub margin: Rect, pub border: Rect, pub padding: Rect, pub content: Rect }
pub struct ImeStateInfo { pub node: Option<NodeId>, pub composing: bool, pub preedit: Option<String>,
    pub cursor_area: Option<Rect>, pub purpose: InputPurpose }
pub struct DevtoolsKeys { pub open: Vec<KeyChord>, pub pick: Vec<KeyChord> }
pub(crate) enum DevtoolsChange { Overlay(WindowId, DevtoolsOverlay), Pick(Option<WindowId>), Keys(DevtoolsKeys) }
/// 記録の有無と記録は、この一つの型にだけある（全 window の Model と Host が clone を共有する）。
pub(crate) struct EngineLog(Rc<RefCell<EngineLogState>>);
pub(crate) struct EngineLogState { pub(crate) syncs: Log<SyncRecord>, pub(crate) frames: Log<FrameRecord> }
pub(crate) enum Log<T> { Off, Recording { ring: VecDeque<T>, dropped: u32 } }

// kivact-core の devtools（4.4.1。view/devtools_state.rs）
pub(crate) struct PropWrite { pub node: NodeId, pub prop: Prop, pub effect: Option<EffectId>, pub changed: bool }
/// A call answered after a later turn: `find_*`, `list_turns` with `wait_ms`, and `inject_input` wait.
/// Each entry carries its window, so closing the window answers it.
pub(crate) enum Pending {
    Find { by: QueryBy, window: WindowId, deadline_ns: u64, reply: Responder },
    Inject { window: WindowId, request: RequestId, reply: Responder },
    Turns { params: ListTurnsParams, deadline_ns: u64, reply: Responder },
}
pub(crate) enum QueryBy { Role(RoleQuery), Text(TextQuery), TestId(TestIdQuery) }
/// A turn graph waiting for the layout and frame of its revision (completed by
/// the rule of 4.2.4 step 4: its frame, or any record of a newer revision, or
/// `stop_recording`).
pub(crate) struct PendingGraph { pub revision: Revision, pub graph: TurnGraph }
/// An engine event for the DevTools, queued by the hook the client dispatch calls
/// and handled by `devtools::run_events` (the client does not reference `devtools`).
pub(crate) enum DevtoolsEvent {
    Key { window: WindowId, pick: bool },
    Pick { window: WindowId, node: Option<NodeId>, phase: PickPhase },
    InjectDelivered { request: RequestId },
}
```

### 4.10 DevTools を使わないときの費用

- feature `devtools` が無いビルドには agent も socket も devtools module も `DevtoolsState` も入らない。core に残る `#[cfg(feature = "devtools")]` の分岐は、`Extensions` への追加と agent の起動、`Launch::turn` の 4 か所の呼び出し（`take_incoming`・`run_incoming`・`run_events`・`after_turn`）、`bind_style` の層の保存、`CoreNode.style_layers`、`devtools_in_tests` の設定だけ。`view/devtools_hooks.rs` は常にコンパイルし、hook は同じ signature で本体が何もしない（入力をそのまま返す）。
- engine・runtime の口は常にコンパイルするが、設定されていなければ（記録・重ね表示・pick・key が無効なら）何もしない。runtime の詳細な記録は `set_detailed_trace(true)` のときだけ。

---

## 5. 暗黙の状態マシンの解消

### 5.1 定義

`architecture.md` の S-1〜S-9 に反する次の形を「暗黙の状態マシン」とし、すべて直す。

| 記号 | 形 |
|---|---|
| a | 互いの有効な組合せが依存する 2 つ以上の bool・Option・counter・Cell・RefCell |
| b | State/Phase の enum の隣にあり、ある phase でだけ有効な Option |
| c | 「もう起きた」を記録して後の振る舞いを変える flag・Option（S-9 の名前付きの型 `ReportOnce`・`Sent`・`Generation`・`Mailbox`・純粋な cache・統計を除く） |
| d | 古い仕事を捨てる・同期するための counter・epoch・generation（S-5） |
| e | 状態を補修・複写する effect（S-5） |
| f | 構築の後で埋める slot（S-7） |
| g | 型・状態・guard の無い、呼び出し順だけで表した phase（S-6） |
| h | 同じ事実の二重保存（S-4） |

残してよいもの: builder・option・設定の field、受動的な record、Options/Event/Phase/Reason/Input/Report/Tokens/Recipes/Api/Policy の値、caller が持つ Binding/Value、独立した一つの bool、測定値、統計、純粋な cache、S-9 の名前付きの型。

### 5.2 監査の結果

基準 commit の `// xtask-allow(state-fields)` の 674 か所（`crates/` 651、`examples/` 19、`dev/` 4）を全件見直し、印の無い箇所も crate ごとに調べた。修正の内容（置き換えのコード、読み書きのすべての箇所、消す印、試験）は [付録](WORK-ORDER-appendix-state-machines.md) にある。

| 範囲 | 見直した印 | 残す | 直す（印あり） | 直す（印なし） | 複数ウィンドウの書き直しに含める |
|---|---|---|---|---|---|
| kivact-ui・basic・pop・game | 170 | 166 | SM-UI-01〜03 | SM-UI-04〜07 | — |
| kivact-ui の設計判断の案件 | — | — | SM-UI-08（Variants）、SM-UI-09（SelectionField）、SM-UI-10（不要な印） | — | — |
| kivact-ui-soft | 204 | 192 | SM-SOFT-01〜04 | SM-SOFT-05〜10 | — |
| interaction・creative・style | 138 | 133 | SM-INT-01〜06 | SM-INT-07、08 | — |
| interaction・creative の設計判断の案件 | — | — | SM-INT-09〜15（D1〜D7） | — | — |
| core・engine・その他・examples・dev | 162 | 153 | SM-CORE-01〜04 | SM-CORE-08〜13、17 | SM-CORE-05、06、07、14、15、16 |
| style（見落としの追加） | — | — | — | SM-INT-17（`Transform.stage`） | — |
| 5.3 の規則で初めて検出される既存の箇所 | — | 印を書くだけ | — | — | — |

最後の行は付録の 7 章（struct 24、enum の variant 16、static 1 組）。T15.4 が表の印の文をそのまま写す。分類の見直しは済んでいて、コードを直す案件は SM-CORE-17 と SM-INT-17 の 2 つだけである。

設計判断が必要だった案件は次のとおり決めた（付録の SM-INT-09〜15、SM-UI-08〜09 がこれに従った修正内容）。

| 案件 | 決定 |
|---|---|
| menu の `ready` flag（on_mount で立てる） | core に `kivact::view::is_mounted() -> Value<bool>` を足す（今の owner の view が commit され on_mount が走った後 true。事実の持ち主は core）。prelude には入れない |
| submenu の項目が持つ `hovered` の複写 | 削除し `item_ref.hovered()`（engine が持ち主）を読む。押したまま submenu の項目に乗ると開く（native の menu と同じ） |
| hover card の双方向の effect | delayed machine に `Shown`・`Hidden` の effect を足し、caller の binding はその effect だけが書く。binding の変化は Machine の `feed` で送る |
| list の active の位置の複写 | 位置は items と active の key からの memo（投影）にする。位置を machine に送る effect を消す |
| dock の drag の閾値の latch | `DragApi::slop(threshold)` を使う（capture は閾値を越えてから） |
| outliner の paint と EditSession | paint を EditSession の phase の payload に移す（S-3） |
| palette の Open/Closed の複写 | 機械の状態を消し、caller の `open` binding を唯一の事実にする |
| `Variants` の共有 cell | 4 つの部品が recipe の style を `into_view` で入れるようにし、`Variants` を所有する値にする |
| SelectionField の surface の snapshot | open の source が true になったとき（caller の binding で開いたときを含む）に snapshot を作り直す |

### 5.3 xtask の検査の強化（`dev/xtask/src/state_fields.rs`）

今の検査は、構造体の前に置いた印で構造体全体を黙らせてしまい（671 か所のうち 267 か所がこの形）、関数の中の局所変数・static・tuple・atomic を見ていない。次のとおり強める。

1. 印は field（static は static）の直前だけで効く。「直前」は、その field の `///` と `#[..]` の行の上である（印と field の間に doc を挟まない）。構造体の前の印は無効にし、「state-fields: struct-level marker」として error にする（`architecture.md` の「型全体の曖昧なコメントは使わない」をそのまま検査する）。
2. 使われていない印を error にする（「state-fields: unused marker」）。印が使われた = その印の直後の field が、下の 4 で「数える」ものである。数えない field の前の印、検査しない型（`NodeBuilder` を実装する struct と `value_modules` の module）の中の印は、使われていない。（「その印を消すと error になる」とは定めない。印を全部の field に書いた型で、全部の印が使われていないことになるため。）
3. tuple の field は要素ごとに数える。tuple 型の field（例 `(Cell<bool>, Cell<u32>)`）の直前の印は、その要素すべてに効く。
4. field の型を次のとおり分類する。「一番外側」は型の path の最後の segment の名前。
   - 数えない: 一番外側（`Cell`・`RefCell`・`Rc`・`Arc` を剥がした後）の名前が `named_helpers`（`Generation`・`Sent`・`ReportOnce`・`ReportOnceSet`・`Mailbox`・`LocalMailbox`・`Registry`）にあるもの。一番外側が `Value` か `Binding` のもの。
   - phase: `Cell`・`RefCell`・`Signal`・`Var`・`Rc` を剥がした型が `MachineHandle<_>` であるか、名前が `State` か `Phase` で終わる enum であるもの。
   - stored（保存する状態）: 上のどちらでもなく、次のどれかであるもの。
     - (i) 一番外側が `Option`。ただし `Option<Callback<..>>`・`Option<Handler<..>>`・`Option<ViewFn>`・`Option<Rc<dyn Fn..>>`・`Option<Box<dyn Fn..>>` は設定なので数えない。
     - (ii) `bool`。
     - (iii) `Rc`・`Arc` を剥がした型の名前が `Atomic` で始まる（`AtomicBool`・`AtomicU64`・`AtomicU8` など、すべて）。
     - (iv) `Rc`・`Arc` を剥がし、`Cell`・`RefCell`・`Signal`・`Var`・`Mutex`・`RwLock` のどれかを 1 つ以上剥がして（剥がした後の `Rc`・`Arc` も剥がして）残った型が、`bool`・整数（`u8`〜`u64`・`usize`・`i8`〜`i64`・`isize`）・一番外側が `Option` のどれか。`Signal<Vec<bool>>` や `RefCell<BTreeMap<u64, X>>` のように残った型が `Vec` や `BTreeMap` などなら数えない。
     - (v) field の名前が `generation`・`epoch`・`serial`・`seq`・`token`・`next`・`next_*`・`*_count`・`pending_*` で、型が整数そのもの。
5. error にする組: 一つの struct（tuple struct は要素ごと）か enum の名前付き field の variant（variant ごと。tuple variant は検査しない）で、印の無い stored が 2 つ以上、または phase と印の無い stored が 1 つ以上ある。phase だけ、phase と数えない field だけの組は error にしない。module の `static` と `thread_local!` の中の `static` は、型を 4 の stored の基準で見て（名前の基準 (v) には static の名前を小文字にしたものを使う）、印の無い stored が一つの module に 2 つ以上あれば error にする。
6. 免除: `NodeBuilder` を実装する struct と `value_modules` の module は検査しない。名前が `*Api`・role suffix・`record_types` の struct と enum は、stored のうち (iii)(iv) だけを数える（phase と (i)(ii)(v) は数えない）。免除は enum の名前にも当てる。
7. 関数の中の局所変数（`let x = signal(false|true|None|整数)`、`Cell::new(..)`、`Rc::new(Cell::new(..))`、`RefCell::new(None)`）は error にせず、`--report` のときだけ「state-locals:」の一覧として出す（目で確かめるための手がかり。T15.4 が読む）。

印の理由は、その field が状態でない正確な理由を書く。形は `<分類> - <その field が何の事実か>`、分類は付録 7.4 の一覧（`builder option`・`passive record`・`independent fact`・`S-8 ID` など）。付録の「残す」表と 7 章の表の分類をそのまま使ってよい。

### 5.4 新しいコードの規則

- この変更で足すすべての状態は、明示的な enum（S-1）か Machine で持つ。3.11 節と 4.3 節・4.5.1 節の表がその一覧。
- 新しい counter は ID の採番か統計だけ（S-8）。
- protocol crate は値だけの crate なので `[state_fields] value_modules` に入れる。agent・client・アプリ・MCP は通常どおり検査する。

---

## 6. 文書・policy・道具の更新

同じ変更（このブランチ）の中で更新する。作業指示書の担当に割り振ってある。

| 対象 | 変更 |
|---|---|
| `Cargo.toml` | members に 5 crate。`[workspace.dependencies]` の serde・serde_json・png をそのまま使う（新しい外部 crate は足さない） |
| `Cargo.lock` | 新しい crate の分を更新（serde・serde_json はすでに lock にあるので network は要らない見込み。要るなら一度だけ取得） |
| `dev/xtask/policy/workspace.toml` | `[visible_to]` に新しい crate（4.1 節）、`[analysis] trusted_attributes = ["serde"]`、`[state_fields] value_modules` に `kivact-devtools-protocol` と `kivact-types::window`、`[structural_options]` に新しい公開 builder の method（`Window::content` などは引数があるので不要。引数なしの `-> Self` が増えたら足す）、`[module_rules.kivact-core.uses]` に `devtools` の行（`devtools` は `launch` を参照しない。`launch` の配列に `devtools` と `geometry`、`app` の配列に `geometry` を足す（`check_window_sizes` を `geometry` へ移すため。3.1 の 2.）。`client` は `devtools` を参照しない（`view/devtools_hooks.rs` の hook を呼ぶ）。T0.1）と `view` 内の新 module の向き、2 つの binary（`kivact-devtools`・`kivact-devtools-mcp`）の `[visible_to]` は `[]`、glob import の例外（T0.1 の指定どおり） |
| `.gitattributes` | 生成物（schema の JSON と `_generated.py`）を `text eol=lf` にする（Windows の checkout でも生成の検査が byte 単位で一致する。T0.1） |
| `dev/tools/check-features.sh` | `kivact/devtools`、`kivact-core/devtools`、`kivact-test/devtools` |
| `docs/for-kivact-developer/architecture.md` | crate の表と数（42 → 47）、DEP の表（`devtools` feature）、事実の持ち主の表（window・focus の window ごと、DevTools の override）、module の向き、追加の checklist に「window に向けた service」と「DevTools の method」の行。加えて次の行を直す（T15.3）: engine の行「window ごとの tree/model（`Model`）と App 全体の `AppModel`」、package の数、shell の行に `input` と `window_input`、Role の行「types の Role、native の role mapping、protocol の `Role` と core devtools の `convert`（`cargo xtask devtools-codegen`）」、入力の正規化の行「window ごとの `WindowInput`（native の winit・virtual・注入が共有。embed の port は別）」。「DevTools の method」の行: 「protocol の leaf の型と `lib.rs` の再公開、`protocol_methods!` の 1 行、`cargo xtask devtools-codegen`、core の `devtools/` の振り分けと leaf、必要なら Python の `client.py`、MCP の除外の一覧、ガイドの tool の一覧。版の規則（wire の型の field の削除・改名・型の変更、method・通知の削除・改名は `PROTOCOL_VERSION` を上げる。追加は上げない）」 |
| `docs/for-kivact-developer/manual-verification.md` | native の複数 window（開く・閉じる・modal・`focus_window`）と DevTools（F12・Ctrl+Shift+C・接続・pick）の手動確認の行（T15.3）。C2 では未確認として報告する |
| `docs/for-kivact-developer/decisions.md` | 新しい行: 複数ウィンドウの SPI（ShellLoop と EventTarget）、per-window の engine model、window の API（Portal 型・dialog と同じ形）、DevTools の別プロセスと protocol の単一の正本、state-fields の印は field だけ |
| `docs/for-kivact-developer/strategy.md` | 既存の 2 行をその場で書き換える（行は増やさない。T1.6）: 5 章の「window の起動設定と OS サービス」を「サイズ・最小サイズ・複数の window（位置は親に対する相対の意図だけ）・clipboard・…」に、「試験と診断」の行の末尾に「DevTools（別プロセスの agent と MCP・Python の client）で更新理由・性能・意味情報を調べる」を足す |
| `crates/kivact-core/src/platform/*.rs` の rustdoc | window を問う関数（size・scale・visible など）に「the window of the calling owner」と書く（T7.1） |
| `docs/for-kivact-user/{en,jp}/how-to-guide/windows.md` | 新規（3.1 の API、Portal との対応、dismiss、環境の問い合わせ、試験での `screen.window`） |
| `docs/for-kivact-user/{en,jp}/how-to-guide/devtools.md` | 新規（有効化、キー、各 panel、review の書き出し、MCP の設定、Python の導入と例、Scripts） |
| `docs/for-kivact-user/{en,jp}/how-to-guide/testing.md` | `screen.window(title)` と `screen.windows()` の節（handle は先に `let` に入れる）。`Screen::has_window()` を `is_mirrored()` に改名した（公開の変更。意味は「本物の window に写しているか」のまま）ことと、`RenderOptions::single_window()` |
| `docs/for-kivact-user/{en,jp}/how-to-guide/coming-from-react-or-solid.md` | 対応表に Portal の行（同じ window の中は `overlay`、OS の window は `window(title, open)`）、prelude の名前を隠す例の一覧に `window`（T1.6） |
| `docs/for-kivact-user/{en,jp}/how-to-guide/{platform-services,diagnostics,embed}.md` | window ごとの問い合わせと `focus_window`、更新理由は DevTools へ誘導、embed では追加の window と入力の注入が使えないこと（T15.3） |
| `README.md`・`README_JP.md` | crate の表、package 数（今の「40」は古い。47 にする）、feature の表に `devtools` |
| `dev/README.md` | feature の一文、check-features の一覧 |
| `THIRD-PARTY-NOTICES.md` | serde・serde_json を直接の依存にした行 |
| `.gitignore` | `__pycache__/`、`*.pyc`、`.venv/`、`*.egg-info/`、`python/kivact-devtools/build/`、`.kivact/review/`、`.kivact/traces/` |
| `CONTRIBUTING.md` | 試験の表に `python/kivact-devtools/tests`、merge 前の検査に `cargo xtask devtools-codegen --check` は `cargo xtask check` に含まれる旨と、Python の試験（任意） |
| `architecture/` | `cargo xtask check` で再生成（Node.js が要る） |
| 削除 | merge 前に、この設計書・作業指示書・付録をリポジトリから消す |

---

## 7. 非目標と制限

| 項目 | この変更での扱い |
|---|---|
| main window の閉じる操作の取り消し（未保存の確認） | 作らない。今どおり閉じる操作で終了する |
| window の座標・display の公開 | 作らない（D-11） |
| menu・popover を OS window にして親の外へ出す | 作らない（overlay は今どおり window の中） |
| embed からの追加の window | `Unsupported` |
| embed での入力の注入 | `inject_input` は `UNSUPPORTED`（-32015）。embed の port（win32・X11）は今どおり自分の入力の状態を持つ（`WindowInput` へ移すのは別の変更。D-17） |
| layer borders（compositor の layer の境界の表示） | 作らない。再描画の調査は paint flashing（`SyncRecord.redrawn` の箱）で行う（D-18） |
| `focus_window` が効かない環境 | OS が前面化を許さない場合（Wayland など）は何もしない。error にしない（D-16） |
| GPU device の window 間の共有 | 作らない（window ごとに presenter と device） |
| signal の値の表示 | 作らない（D-14） |
| theme の Light/Dark の emulation | `platform::theme_hint()` に従うアプリにだけ効く。DevTools の画面に表示する |
| time travel、hot reload、ソースへの自動書き戻し、記録と再生、network panel | 作らない（strategy の N-05、N-07） |
| macOS・mobile | 今どおり対象外（multi-window の capability は native の Windows・Linux と virtual で true） |
