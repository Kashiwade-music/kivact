# P6 設計：GPU OnDemand の E2E（AC-4 U56）、`Windows.gpu`/`Driver.gpu` の閉じ方、公開 API のレビュー

> **V2 付属書 D6**（設計担当 P6 の成果物）。正本は [SLIM_FAST_V2_DESIGN.md](../SLIM_FAST_V2_DESIGN.md)、手順は [SLIM_FAST_V2_WORK_ORDER.md](../SLIM_FAST_V2_WORK_ORDER.md)。
> - 本書の「8 章 設計者（人間）の判断が要る点」は V2 設計書 4 章（DEC-01〜DEC-23）で決定済み。実装者は 8 章を読んで選ばない。V2 設計書 5 章（付属書間の調整）が本書を上書きする箇所がある。
> - 本書の unit 名と V2 の unit ID の対応は V2 設計書 0.2 節。unit の実施順・lane・wave は V2 作業指示書だけが決める（本書 6 章の「依存」「並列可」は参考）。
> - 文中の P1〜P8 は付属書 D1〜D8（`design-P<n>.md` は `D<n>-*.md`）を指す。`W:\w\kv-prof`・`W:\w\kv-mem`・`W:\w\kv-size`・scratchpad の path は設計時の試作・計測の場所（設計した機械にだけある参考）で、実装に必要ではない。行番号は HEAD `73c36c7c` のもの。

対象 HEAD 73c36c7c（= db8d679b と引継ぎ文書）。リポジトリは読取りのみで調べた。実 window の試験は実行していない。結論は静的な読解による。推測の箇所には「推測」と明記する。

---

## 1 事実

### 1.1 GPU OnDemand の実装（U24 6f657823 と、その後の U45 87bd78e9・U32 2e04aa26 の追従）

| 旧設計 4.3 の項目 | 実装 | 位置 |
|---|---|---|
| `GpuStart { OnDemand(default), AtLaunch }`、`GpuPreference.start` | 設計どおり | `crates/kivact-types/src/gpu.rs:48-60, 80` |
| 再公開 | `kivact::gpu_interop::GpuStart`。ただし `gpu_interop` は feature `gpu-interop` の時だけ公開される。`App::gpu_preference` も同じ feature の時だけある | `crates/kivact-core/src/gpu_interop.rs`（末尾）、`lib.rs:65-74`、`app.rs:104`、`launch/settings.rs:135` |
| `ARVANE_RENDERER` の優先順位 | `cpu`→None、`gpu`→AtLaunch、`auto`→OnDemand、未設定→`prefer_gpu.then_some(start)`。表どおり | `crates/arvane/src/env.rs:32-43` |
| `State::Deferred { slow, waker }` | 足されている | `crates/arvane/src/presenter/gpu.rs:250-260` |
| 起動の入口 `Gpu::start` | **`State::Off` からも始める**（`matches!(Off \| Deferred)`）。Off は「CPU だけ」の window の状態でもある | `gpu.rs:358-371` |
| `demand_gpu` | `Presenter::demand_gpu` → `Gpu::start`（Off からも始まる） | `frame_policy.rs:223-225`、`window.rs:241-245` |
| `note_cpu_frame`（8 回） | 状態の判定は Deferred のときだけ。ただし呼出し側は引数 `composition.needs_gpu()` を**状態に関係なく CPU の Shown frame ごとに評価する** | `gpu.rs:384-408`、`frame_policy.rs:328-341` |
| `Composition::needs_gpu` | 全 layer の filter、backdrop、全 item を走査する | `crates/kivact-types/src/render/composition.rs:28-52` |
| `HostRequest::DemandGpu`（4 箇所） | import（`import/image.rs:192`）、shared image（`share/app_side.rs:80`）、`Viewport::gpu_frame`（`viewport.rs:83-93`、builder の時に `Owner::current`）、drawable の GPU export（`drawable/requests.rs:53-56`）。設計の「import registry を作る時」は「import のたび」に置き換わっている | 同左。共通関数は `gpu_interop/demand.rs:12` |
| engine の配送 | writer は window の `effects` に積み（`host/write/writer.rs` の `DemandGpu` の腕）、pump が `windows.demand_gpu(id)` を呼ぶ（`host/pump.rs:197-200`）。結果は返さない（`continue`）。`apply_window_changes`（435 行）の後に `effects`（437 行）が走るので、新しい window の presenter は先にある | `pump.rs:144-210, 410-437` |
| frame log | Deferred から起動した時の note は無い（作業指示書の手順 11 のとおり）。`Start{gpu}` は AtLaunch の時だけ true | `window.rs:226, 443` |
| 試験の host | `prefer_gpu = (gpu == Required)`。Required の時は `start` を AtLaunch に**上書き**する（`fn gpu_preference`）。Off の時は GPU を使わない | `crates/kivact-engine/src/host/testing/test_host.rs:181-228, 288-315` |
| 公開の試験 API | `RenderOptions::gpu()`（bool）だけ。`Screen` に renderer を読む method は無い。`lib.rs:31-32` には「no API says which one drew」とある。`gpu_tests.rs:7-9` の doc は存在しない `Screen::renderer` を参照している | `crates/kivact-test/src/options.rs:41, 96, 164, 222`、`lib.rs:31` |
| 試験の settle | GPU の worker は待たない。待つのは blocking job だけ | `crates/kivact-test/src/screen/driver.rs:211-225` |
| skip の流儀 | `if let Some(reason) = gpu_unavailable() { eprintln!("SKIPPED {name}: {reason}"); return; }`（`shaders.rs:35-43` は `gpu_lane(name)` という helper）。options は `RenderOptions::from_env().gpu().mirror(MirrorMode::Required)` | `dev/tests/kivact-public-api/tests/e2e/shaders.rs:35-47`、`gpu_interop.rs:136-160` |
| GPU の adapter と import の順序 | `gpu_interop::adapter()` は device が Ready になるまで None（`registry.rs:146-153`）。`DrawableShare::request_gpu(adapter)` も `import(textures)` も、その adapter が要る（`drawable_share.rs:373-376`、how-to `gpu-texture-sharing.md:335-338`） | — |
| `std::env::set_var` | 禁止はされていない。ただし edition 2024 では unsafe で、試験の process は共有される | `clippy.toml` |

### 1.2 `Windows.gpu`（`window.rs:193`）と `Driver.gpu`（`window.rs:407`）

- 型はどちらも `Option<GpuStart>`。
- 書き手は一つ。`driver()` の `crate::env::gpu_start(env.renderer, prefer_gpu, gpu_preference.start)` だけが `Driver.gpu` を作る（`window.rs:386-400`）。
- `Windows.gpu` は callback のたびに `Driver.gpu` を写す（`window.rs:423, 456, 492`）。
- 読み手は `Windows::open`（`window.rs:226-233`）と `Driver::start`（`window.rs:443-450`）。どちらも同じ 8 行の分岐（note と start/defer）を重ねて持つ。
- 値は loop の間変わらない。`Env.renderer: Option<RendererChoice>`（`env.rs:50`）は `gpu_start` の入力として使われるだけである。

### 1.3 公開 API（別の subagent による全数調査の結果を含む。file:line は確認済み）

- `#[component]`（`crates/kivact-macros/src/{parse,expand}.rs`）
  - `mount()`（`expand.rs:125-137`）は Text と Value の optional を区別しない。
  - `parse::component` は doc 以外の属性を捨てる（`parse.rs:74-79`）。
  - `check_signature` は async、const、unsafe、extern の fn を拒まない（`parse.rs:107-130`）。
- `Binding::update`（`crates/kivact-runtime/src/reactive/handles.rs:573-584`）は設計どおり。Signal の binding は in-place で変え、pair の binding は clone に f を適用して set を一度呼ぶ。
- CPU 合成の高速経路（`crates/arvane-compose/src/cpu/draw.rs:272-363`）
  - 条件は設計 4.2 の表どおり：整数の平行移動、画素に揃った 4 角、opacity 1、cover なし、Normal、SourceOver、Premul。
  - 範囲外の texel は読み飛ばす。透明の src-over と同じ結果になる。
  - scissor の判定は画素ごと。
  - 汎用 loop は画素中心 `(x+0.5, y+0.5)` を写すので、整数の平行移動では texel の中心に一致する。nearest と bilinear で同じ texel を読むので、正しい。

---

## 2 原因

1. **U56 の STOP の直接の原因**：試験の API が「GPU を使える window」と「いつ起動するか」を一つの bool（`RenderOptions.gpu`）にまとめている。そのうえ engine の test host が `start` を AtLaunch に上書きする。このため OnDemand の window を作れない。renderer を観測する公開の口も無い。GPU の起動は real time の worker で進むが、仮想時間の settle はそれを待たない。
2. **設計 4.3 そのものの欠陥**
   - (a) `State::Off` を「CPU だけ（起動しない）」と定義しながら、「`Off` と `Deferred` で始める」と書いている。この矛盾がそのまま実装された（7 章 D1）。
   - (b) 需要の地点を「registry を作る時」としている。registry は App の作成時に入るので、gpu-interop の App は常に起動時に GPU を始めることになる。実装はこれを import ごとに置き換えたが、import と GPU export は adapter（Ready 後に得られる）を前提とする。このため最初の需要にならない（D3）。
   - (c) `needs_gpu` を composition の走査で判定するので、PERF-1 に反して毎 frame 全体を走査する（D2）。
3. **`Windows.gpu`/`Driver.gpu` が OPEN の理由**：環境変数とアプリの希望から一度だけ導く方針を、`Option` の field として二つの struct に保持している。そのうえ導出元の `Env.renderer` も並べて保持している。

---

## 3 設計（確定版）

### 3.1 方針（一つの表）

起動の規則は `RendererStart`（arvane 内部の enum）一つで表す。window ごとの状態は `presenter/gpu.rs` の `State` だけが持つ。

| 入力（上が強い） | `RendererStart` |
|---|---|
| `ARVANE_RENDERER=cpu` | `Cpu` |
| `ARVANE_RENDERER=gpu` | `GpuAtLaunch` |
| `ARVANE_RENDERER=auto` | `GpuOnDemand` |
| `Shell::prefer_gpu(false)`、`ShellSetup.prefer_gpu = false`、試験の既定 | `Cpu` |
| `GpuPreference.start == AtLaunch`、試験の `RenderOptions::gpu()` | `GpuAtLaunch` |
| `GpuPreference.start == OnDemand`（既定）、試験の `RenderOptions::gpu_on_demand()` | `GpuOnDemand` |

試験の `gpu()` と `gpu_on_demand()` は、`ARVANE_RENDERER` が running App に対して行うのと同じく、`GpuPreference.start` を決める。`RenderOptions::gpu_preference` の `start` は使わず、その旨を doc に書く。

起動の理由は三つだけである。

| 理由 | 検出する所 |
|---|---|
| 描かれた user shader | CPU の compositor が frame の report に `shader_fallback = true` を立てる。走査は増やさない |
| App の需要 | `gpu_interop::adapter()` を読んだこと（window ごとに一度）→ `HostRequest::DemandGpu` |
| 遅い CPU frame | `raster + present > period/2` が 8 回続く |

`Cpu` の window は、どの理由でも起動しない。

### 3.2 arvane（閉じ方：B の確定も含む）

**`crates/arvane/src/env.rs`**：`RendererChoice` と `gpu_start` を削除し、次で置き換える。

```rust
use kivact_types::gpu::GpuStart;

/// How a native window's renderer starts, decided once per loop
/// ([`Env::read`]): `ARVANE_RENDERER` (`cpu`, `gpu`, `auto`), else the host's
/// `prefer_gpu` and `GpuPreference::start` (the table of the module doc).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RendererStart {
    /// Frames are drawn on the CPU only; no device ever starts.
    Cpu,
    /// The device starts on a worker with the window.
    GpuAtLaunch,
    /// The device starts when the window needs it.
    GpuOnDemand,
}

impl RendererStart {
    fn resolve(forced: Option<&str>, prefer_gpu: bool, start: GpuStart) -> Self {
        match forced {
            Some("cpu") => Self::Cpu,
            Some("gpu") => Self::GpuAtLaunch,
            Some("auto") => Self::GpuOnDemand,
            _ if !prefer_gpu => Self::Cpu,
            _ => match start {
                GpuStart::AtLaunch => Self::GpuAtLaunch,
                GpuStart::OnDemand => Self::GpuOnDemand,
            },
        }
    }
}

/// What the environment and the host ask of Arvane's renderers.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Env {
    /// How every window's renderer starts.
    pub(crate) start: RendererStart,
    /// How GPU devices are made.
    pub(crate) gpu: arvane_compose::GpuEnv,
}

impl Env {
    /// Read the environment now, for windows the host wants drawn on the GPU
    /// (`prefer_gpu`) with devices made and started as `preference` wishes.
    pub(crate) fn read(prefer_gpu: bool, preference: kivact_types::GpuPreference) -> Self {
        let forced = std::env::var("ARVANE_RENDERER").ok();
        Self {
            start: RendererStart::resolve(forced.as_deref(), prefer_gpu, preference.start),
            gpu: arvane_compose::GpuEnv::from_env(preference),
        }
    }
}
```

**`crates/arvane/src/window.rs`**

- `Windows` と `Driver` から `gpu` field を削除する。B の閉じ方は「導出して保持しない」と「型で表す」である。方針は `Env.start` の 3 値の enum に一度だけ導出し、`Env` は Copy の設定として渡す。
- `Windows::open` の 226-233 行を `presenter.begin(self.env.start, self.shell.waker());` の一行にする。
- `Driver::start` の 443-450 行を `presenter.begin(self.env.start, waker.clone());` にする。
- `Windows::demand_gpu`（241 行）の本体は `presenter.demand_gpu();` にする（waker は `Deferred` が持つ）。doc は「Start `id`'s GPU device when it waits for a reason (`GpuStart::OnDemand`); nothing for a window on the CPU only or whose device started.」
- `driver()`（386 行）は `let env = crate::env::Env::read(prefer_gpu, gpu_preference);`。`Driver { handler, presenters, env, observer }`。
- `Windows { shell, presenters, env, observer }` の 3 箇所（423、456、492 行）から `gpu: self.gpu` を消す。
- `use kivact_types::gpu::GpuStart;` を消す。

**`crates/arvane/src/presentation.rs`**：variant と enum を足し、`lib.rs:67` の `pub use presentation::{Renderer, RendererChange}` に `GpuStartCause` を加える。

```rust
/// Why a window's GPU device starts after the window opened.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GpuStartCause {
    /// A frame drew a user shader by its fallback on the CPU.
    Shader,
    /// The App asked for the device: it reads the window's adapter to share
    /// GPU textures with it (`HostRequest::DemandGpu`).
    App,
    /// CPU frames kept missing half the display period.
    SlowFrames,
    /// The device failed and a new one starts (once per session).
    Restart,
}

// enum RendererChange に足す（Start の直後）:
    /// A device the window waited for (`GpuStart::OnDemand`), or a new one
    /// after a device failure, starts on a worker; frames stay on the CPU
    /// until it is ready (`renderer=cpu gpu=starting cause=…`).
    GpuStarting {
        /// Why it starts.
        cause: GpuStartCause,
    },

// Display に足す:
            Self::GpuStarting { cause } => {
                let cause = match cause {
                    GpuStartCause::Shader => "shader",
                    GpuStartCause::App => "app",
                    GpuStartCause::SlowFrames => "slow_frames",
                    GpuStartCause::Restart => "restart",
                };
                write!(f, "renderer=cpu gpu=starting cause={cause}")
            }
```

bench は `renderer=gpu` だけを grep する（`dev/bench/run.sh:107`）ので、影響は無い。`kivact-engine/src/host/launch.rs:42` と `embed.rs:70` の `_ => log.note(..)` が、この行をそのまま frame log に書く。

**`crates/arvane/src/presenter/gpu.rs`**

- `State::Off` の doc を「No device, and none starts: the window draws on the CPU only (the host or `ARVANE_RENDERER` chose it), or its presenter shut down. A new presenter is here until [`Gpu::begin`].」にする。
- `Deferred` の doc を「No device yet (`GpuStart::OnDemand`): one starts on the App's demand ([`Gpu::demand`]), for a drawn user shader, or after [`SLOW_FRAMES`] CPU frames in a row over half the display period ([`Gpu::note_cpu_frame`]); `waker` wakes the window's thread for its start steps.」にする。
- `pub(crate) fn start` と `defer` を削除し、次に置き換える。

```rust
    /// The renderer the window starts with, as `start` says (once, when the
    /// window opens): the CPU only, a device started on a worker now, or a
    /// device that waits until the window needs it. Notes
    /// [`RendererChange::Start`].
    pub(crate) fn begin(&mut self, start: RendererStart, waker: ShellWaker, notes: &mut Notes) {
        debug_assert!(matches!(self.state, State::Off), "invariant: begin runs once, on a new presenter");
        notes.note(RendererChange::Start { gpu: start == RendererStart::GpuAtLaunch });
        match start {
            RendererStart::Cpu => {}
            RendererStart::GpuAtLaunch => self.spawn(waker, notes),
            RendererStart::GpuOnDemand => self.state = State::Deferred { slow: 0, waker },
        }
    }

    /// Whether the device waits for a reason to start (`GpuStart::OnDemand`).
    pub(crate) fn waits(&self) -> bool {
        matches!(self.state, State::Deferred { .. })
    }

    /// Start the device the window waited for, for `cause`. Nothing in any
    /// other state: a window on the CPU only never starts one.
    pub(crate) fn demand(&mut self, cause: GpuStartCause, notes: &mut Notes) {
        let State::Deferred { waker, .. } = &self.state else {
            return;
        };
        let waker = waker.clone();
        notes.note(RendererChange::GpuStarting { cause });
        self.spawn(waker, notes);
    }

    /// A CPU frame took `spent` of the display's `period`: a device that
    /// waits starts after [`SLOW_FRAMES`] frames in a row over half of it.
    pub(crate) fn note_cpu_frame(&mut self, spent: Duration, period: Duration, notes: &mut Notes) {
        let State::Deferred { slow, .. } = &mut self.state else {
            return;
        };
        if spent <= period / 2 {
            *slow = 0;
            return;
        }
        *slow += 1; // < SLOW_FRAMES here: it starts at SLOW_FRAMES
        if *slow >= SLOW_FRAMES {
            self.demand(GpuStartCause::SlowFrames, notes);
        }
    }

    /// Create the device on a worker (from `Off` at launch, or `Deferred`).
    fn spawn(&mut self, waker: ShellWaker, notes: &mut Notes) {
        let wake: Wake = Arc::new(move || waker.wake(crate::WakeReasons::GPU_STEP));
        self.state = match start_device(self.env, &wake) {
            Some(worker) => State::Background(Background { wake, restarts: RESTARTS, step: Step::Starting(worker) }),
            None => cpu_session(notes, "the GPU worker thread cannot start"),
        };
    }
```

- `device_failed`（543 行）で restart の worker を作れたら、`notes.note(cpu(..restarting..))` の後に `notes.note(RendererChange::GpuStarting { cause: GpuStartCause::Restart });` を足す。こうすると試験の host が「失敗、再起動中」を正しく GpuStarting と読む。
- `use crate::env::RendererStart; use crate::presentation::GpuStartCause;` を足す。
- 不変条件
  - 起動の入口は `begin`（Off から AtLaunch の時だけ）と `demand`（Deferred の時だけ）の二つである。
  - `Off` からの起動は `begin` 以外に無い。
  - `shutdown` は `Off` に戻す。window は閉じられ、presenter は捨てられる。

**`crates/arvane/src/presenter/frame_policy.rs`**

- `start_gpu` と `defer_gpu` を削除し、次を足す。

```rust
    /// The renderer the window starts with (see [`Gpu::begin`]); call once,
    /// when the window opened.
    pub(crate) fn begin(&mut self, start: RendererStart, waker: ShellWaker) {
        self.gpu.begin(start, waker, &mut self.notes);
    }
```

- `demand_gpu` を `pub(crate) fn demand_gpu(&mut self) { self.gpu.demand(GpuStartCause::App, &mut self.notes); }` にする。
- `present_native` の 328-341 行を次に置き換える。`needs_gpu` の走査は消える。

```rust
        if self.gpu.waits()
            && let Ok(drawn) = &drawn
        {
            if drawn.report().shader_fallback {
                self.gpu.demand(GpuStartCause::Shader, &mut self.notes);
            } else if let Drawn::Shown { renderer: Renderer::Cpu, raster, present, .. } = drawn {
                self.gpu.note_cpu_frame(*raster + *present, Duration::from_nanos(period), &mut self.notes);
            }
        }
```

**`crates/kivact-types/src/render/composition.rs`**

- `impl Composition { needs_gpu }` と `fn is_shader`（28-52 行）を削除する。呼出し元は frame_policy だけである。
- `FrameReport` に field を足す。

```rust
    /// A user shader (paint or filter) was drawn by its fallback on the CPU
    /// this frame; the GPU draws it.
    pub shader_fallback: bool,
```

**`crates/arvane-compose/src/cpu/compositor.rs:239-258`**：flag を集める（O(ops)。新しい走査は無い）。

```rust
    let mut missed: Vec<ExternalFrame> = Vec::new();
    let mut shader_fallback = false;
    for op in &plan.ops {
        shader_fallback |= matches!(
            op,
            Op::Shader(_) | Op::Filter { filter: LayerFilter::Shader(_), .. }
        );
        exec(op, &mut targets, &stores.chunks, &mut missed)?;
    }
    // FrameReport { .., shader_fallback, .. }
```

`crates/arvane-compose/src/gpu/exec.rs:129` の `FrameReport {` には `shader_fallback: false` を足す。他に `FrameReport { .. }` を literal で作る所は無い（grep 済み）。

### 3.3 engine

**`crates/kivact-engine/src/contract/host.rs`** の `DemandGpu` の doc を次にする。

```rust
    /// Start `window`'s GPU device when it waits for a reason
    /// (`GpuStart::OnDemand`): the App reads the window's adapter to share
    /// GPU textures with it. Nothing for a window on the CPU only or whose
    /// device started; nothing is answered.
```

`pump.rs:197` と `writer.rs` はそのままでよい。

**`crates/kivact-engine/src/host/testing/test_host.rs`**

- `use arvane::{.., Renderer, ..}` から `Renderer` を消す。
- `fn gpu_preference`（181-191 行）を削除する。`start_shell` は `gpu_preference: options.gpu_preference` をそのまま渡す。
- 公開の enum と、内部の観測状態を次のように置く。

```rust
/// What draws a test window's frames, and where its GPU device stands
/// (`kivact::test::Screen::renderer`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Renderer {
    /// The CPU, and no GPU device is starting: the window draws on the CPU
    /// only, or waits until it needs the GPU (`GpuStart::OnDemand`).
    Cpu,
    /// The CPU, while the window's GPU device starts on a worker thread.
    GpuStarting,
    /// The GPU.
    Gpu,
    /// The CPU for the rest of the session: the GPU device failed or could
    /// not start.
    GpuFailed,
}

/// A window's renderer as its notes said, with the reason of a failure.
enum Observed {
    Cpu,
    GpuStarting,
    Gpu,
    GpuFailed(String),
}

impl Observed {
    fn renderer(&self) -> Renderer {
        match self {
            Self::Cpu => Renderer::Cpu,
            Self::GpuStarting => Renderer::GpuStarting,
            Self::Gpu => Renderer::Gpu,
            Self::GpuFailed(_) => Renderer::GpuFailed,
        }
    }
}
```

- `renderer: Rc<RefCell<BTreeMap<WindowId, Observed>>>` にし、旧 `enum RendererState` を削除する。
- `renderer_observer` の対応は次のとおり。
  - `Start { gpu: false }` → `Cpu`
  - `Start { gpu: true } | GpuStarting { .. }` → `GpuStarting`
  - `Gpu { .. }` → `Gpu`
  - `Cpu { reason }` → `GpuFailed(reason.clone())`
  - それ以外 → `return`
- `pub fn renderer(&self, window) -> Option<Renderer>` は `self.renderer.borrow().get(&window).map(Observed::renderer)` とする。
- `wait_for_gpu` を公開にし、window を引数に取る（旧 288-315 行を置換）。

```rust
    /// Run the host (real time, at most 5 s) until window `window`'s frames
    /// are drawn on the GPU, while its device starts on a worker thread.
    pub fn wait_for_gpu(&mut self, window: WindowId) -> Result<(), HostError> {
        let started = std::time::Instant::now();
        loop {
            if let Some(failure) = self.failure() {
                return Err(HostError::new(failure.to_string()));
            }
            match self.renderer.borrow().get(&window) {
                Some(Observed::Gpu) => return Ok(()),
                Some(Observed::GpuFailed(reason)) => {
                    return Err(HostError::new(format!("the GPU device failed: {reason}")));
                }
                Some(Observed::Cpu) | None => {
                    return Err(HostError::new(
                        "the window's GPU device is not starting (the window draws on the CPU only, or nothing needed the GPU yet)",
                    ));
                }
                Some(Observed::GpuStarting) => {}
            }
            if started.elapsed() >= std::time::Duration::from_secs(5) {
                return Err(HostError::new("the GPU device did not start within five seconds"));
            }
            std::thread::park_timeout(std::time::Duration::from_millis(5));
            self.step(Step::Wake);
        }
    }
```

- `new`（281 行）の待ちは、`if options.gpu == GpuMode::Required && options.gpu_preference.start == kivact_types::gpu::GpuStart::AtLaunch { driver.wait_for_gpu(WindowId::MAIN)?; }` にする。
- **`testing.rs:11`**：`pub use arvane::{ShellEvent, ShellImeText};` にし、`pub use test_host::{ArvaneTestHost, Renderer};` とする。
- **`test_options.rs:59-62`** の `gpu` の doc を次にする。「The window can draw on the GPU, through a GPU surface of the real window (requires a native mirror). The device starts as `gpu_preference.start` says: with the window (the host then waits until it is ready, and fails to start when it cannot) or when the window needs it. `Off`: the CPU only.」

### 3.4 kivact-test（公開の試験 API）

**`options.rs`**

```rust
pub use kivact_engine::testing::{MirrorMode, Renderer};   // 既存の MirrorMode の行に Renderer を足す

/// How a test window uses the GPU (`RenderOptions::gpu`, `gpu_on_demand`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum TestGpu {
    /// The CPU only (`ARVANE_RENDERER=cpu`).
    #[default]
    Off,
    /// The device starts with the window (`ARVANE_RENDERER=gpu`).
    AtLaunch,
    /// The device starts when the window needs it (`ARVANE_RENDERER=auto`).
    OnDemand,
}
```

- field は `gpu: TestGpu`。`Default` では `TestGpu::Off`。
- `from_env` は `KIVACT_TEST_GPU=1` のとき `TestGpu::AtLaunch`。
- `fn uses_gpu` を削除する。

```rust
    /// Draw on the GPU from the first frame: the window's device starts with
    /// the window and the test waits until it is ready, as
    /// `ARVANE_RENDERER=gpu` does for a running App (`GpuStart::AtLaunch`).
    /// Needs a real window (`MirrorMode::Required`) and a GPU
    /// ([`gpu_unavailable`](crate::gpu_unavailable)).
    pub fn gpu(mut self) -> Self {
        self.gpu = TestGpu::AtLaunch;
        self
    }

    /// Draw on the CPU until the window needs the GPU, then start its device,
    /// as a running App does by default (`GpuStart::OnDemand`,
    /// `ARVANE_RENDERER=auto`): a user shader drawn, or the App reading
    /// `gpu_interop::adapter` (a `shared_image` does), starts it.
    /// [`Screen::renderer`](crate::Screen::renderer) says what draws, and
    /// [`Screen::wait_for_gpu`](crate::Screen::wait_for_gpu) waits for the
    /// device. Needs a real window and a GPU, as [`Self::gpu`] does.
    pub fn gpu_on_demand(mut self) -> Self {
        self.gpu = TestGpu::OnDemand;
        self
    }
```

- `gpu_preference`（129 行）の doc に「Its `start` is the test's: [`Self::gpu`] or [`Self::gpu_on_demand`] decide it, as `ARVANE_RENDERER` does for a running App.」を足す。
- `into_host(self, fonts)` から `gpu: bool` 引数を消し、host の 2 行を次にする。

```rust
            gpu: match self.gpu {
                TestGpu::Off => kivact_engine::testing::GpuMode::Off,
                TestGpu::AtLaunch | TestGpu::OnDemand => kivact_engine::testing::GpuMode::Required,
            },
            gpu_preference: kivact_types::GpuPreference {
                start: match self.gpu {
                    TestGpu::OnDemand => kivact_types::gpu::GpuStart::OnDemand,
                    TestGpu::Off | TestGpu::AtLaunch => kivact_types::gpu::GpuStart::AtLaunch,
                },
                ..initial.gpu_preference
            },
```

その他の変更：

- **`render.rs:36-43`**：`Screen::start(options, Vec::new(), ..)`。
- **`screen/driver.rs:24-38`**：`start` から `gpu` 引数を消し、`options.into_host(fonts)` とする。doc の「drawing on the GPU when `gpu`」を消す。`impl Screen` に次を足す（`wait_for` の直後）。

```rust
    /// Wait until the window's frames are drawn on the GPU, while its device
    /// starts on a worker thread ([`Renderer::GpuStarting`]): the host keeps
    /// running in real time, at most 5 s (the virtual clock does not move),
    /// then settles. Fails the test, with the failure report, when the device
    /// failed, did not start in time, or is not starting (the window draws
    /// on the CPU only, or nothing needed the GPU yet).
    #[track_caller]
    pub fn wait_for_gpu(&self) {
        self.log("wait for gpu".into());
        let waited = self.host_mut().wait_for_gpu(self.window);
        self.run_until_idle();
        if let Err(error) = waited {
            let report = self.report(&format!("wait_for_gpu: {error}"), None, &[]);
            self.fail(&report);
        }
    }
```

- **`screen/state.rs`**：`frame_stats` の前に次を足す。

```rust
    /// What draws the window's frames now, and where its GPU device stands:
    /// [`Renderer::Cpu`] for a window on the CPU only or one waiting until it
    /// needs the GPU ([`RenderOptions::gpu_on_demand`](crate::RenderOptions::gpu_on_demand)),
    /// [`Renderer::GpuStarting`] while its device starts (frames are still
    /// drawn on the CPU), [`Renderer::Gpu`], or [`Renderer::GpuFailed`].
    pub fn renderer(&self) -> Renderer {
        self.check_open();
        self.host()
            .renderer(self.window)
            .expect("invariant: an open window noted the renderer it started with")
    }
```

- **`lib.rs`**
  - `pub use options::{MirrorMode, RenderOptions, Renderer, TestFonts};` とする。
  - 31-32 行の doc を次にする。「a test picks how it draws with `gpu` (the GPU from the first frame) or `gpu_on_demand` (the CPU until the window needs the GPU), and reads what draws with `Screen::renderer`.」
  - facade は `pub use kivact_test as test` なので、`kivact::test::Renderer`、`Screen::renderer`、`Screen::wait_for_gpu`、`RenderOptions::gpu_on_demand` が自動で公開される。
- **`screen/gpu_tests.rs`**：3 箇所の `Screen::start(options, gpu, ..)` を、`let options = if gpu { options.gpu() } else { options };` と `Screen::start(options, Vec::new(), ..)` に変える。`use kivact_engine::testing::{Renderer, gpu_unavailable};` の `Renderer` は新しい enum になる。`Renderer::Cpu` と `Renderer::Gpu` はそのまま compile できる。

命名の理由：

- `gpu()` と `gpu_on_demand()` は `GpuStart::OnDemand` と同じ語で、`ARVANE_RENDERER=gpu/auto` に 1:1 で対応する。
- `Screen::renderer()` は `screen.renderer() == Renderer::Gpu` と読める（JS の `expect(...).toBe(...)` に近い）。
- 待ちは Playwright の `waitFor` の流儀で `wait_for_gpu()` とし、`wait_for` と同じ「失敗で report」の契約にした。
- 互換のための別名は作らない。`gpu()` の意味は変えない。

### 3.5 core：需要は `adapter()` の読取りだけ（D3、D4、D5 の修正）

**`crates/kivact-core/src/gpu_interop/import/registry.rs`**

```rust
pub(crate) struct State {
    imports: RefCell<Imports>,
    adapter: Signal<BTreeMap<WindowId, GpuAdapter>>,
    // xtask-allow(state-fields): independent: whether the App asked for a window's device, whatever its adapter: a window started with its device has an adapter unasked, an asked one has none until its device is ready (or ever, on the CPU)
    /// The windows whose GPU device the App asked for, once each.
    asked: RefCell<BTreeSet<WindowId>>,
}
// State::new: asked: RefCell::default(),

impl State {
    /// Ask `window`'s host, once per window, to start its GPU device: the App
    /// reads its adapter to share textures with it. A device waiting for a
    /// reason (`GpuStart::OnDemand`) starts; the host ignores it otherwise.
    fn demand(&self, rt: &Runtime, window: WindowId) {
        if !self.asked.borrow_mut().insert(window) {
            return;
        }
        let id = ViewState::of(rt).requests.borrow_mut().request_id();
        ViewState::tree(rt)
            .engine()
            .effect(RequestId(id), HostRequest::DemandGpu { window });
        rt.request_turn();
    }
}

/// … 既存 doc に一文足す: Reading it asks the window to start its GPU device
/// when the device waits until it is needed (`GpuStart::OnDemand`, the
/// default): it is `None` until the device is ready.
#[track_caller]
pub fn adapter() -> Option<GpuAdapter> {
    let owner = Owner::ambient("adapter", Location::caller());
    let window = current_window(owner);
    let rt = owner.runtime();
    let signal = State::of(&rt).adapter;
    drop(rt);
    let adapter = signal.with(|adapters| adapters.get(&window).cloned());
    if adapter.is_none() {
        let rt = owner.runtime();
        State::of(&rt).demand(&rt, window);
    }
    adapter
}
```

（import に `use crate::view::state::ViewState; use kivact_engine::contract::HostRequest; use kivact_types::RequestId; use std::collections::BTreeSet;`）

需要を出す所は `adapter()` の一箇所だけになる。他の需要の所は次のとおり削除する。

| 削除する所 | 置き換え |
|---|---|
| `gpu_interop/demand.rs` | file を削除。`gpu_interop.rs` の `mod demand;` と `pub(crate) use demand::demand_gpu;` も削除 |
| `import/image.rs:191-192` | 削除。import の前に App は `adapter()` を読んでいる |
| `share/app_side.rs:77-80` | `let rt = owner.runtime();` とコメントと demand を削除（`rt` が後で使われないこと：U24 の差分で足された行だけである）。`mode_of(adapter())`（82 行）が需要になるので、doc のコメントを「Reading the adapter (below) asks for the window's device」に変える |
| `view/elements/viewport.rs:83-93` | `gpu_frame` を U24 前の形（`#[track_caller]` と `Owner::current` なし）に戻す。GpuFrame は import か shared image からしか作れず、どちらも先に adapter を読んでいる |
| `view/drawable/requests.rs:31-56` と `handle.rs:69-90` | `window` 引数と capture、`DemandGpu` の 3 行を削除。`request_gpu(adapter)` の adapter は `adapter()` から来る |

文書を次のとおり変える。

- `docs/for-kivact-user/en/how-to-guide/gpu-texture-sharing.md:367-377`：起動の理由を「a user shader drawn, the App reading `gpu_interop::adapter()` (a `shared_image` does, and an engine that imports textures or receives a drawable's export reads it first), or CPU frames that keep missing half the display period」に変える。jp は `:296` を同じ趣旨に変える。
- `kivact-types/src/gpu.rs:49-51` の `OnDemand` の doc を「…until the window needs the GPU: a user shader drawn, the App reading the window's adapter to share GPU textures, or CPU frames that keep missing half the display period (the default).」にする。
- `README.md:179`、`README_JP.md:163`、`docs/for-kivact-user/{en,jp}/README.md` の該当文も同じ語にする。

### 3.6 E2E：`dev/tests/kivact-public-api/tests/e2e/gpu_on_demand.rs`（完全版）

`main.rs` には `mod focus;` の次に `mod gpu_on_demand;` を足す（cfg は付けない。CPU の lane の判定は file の中で行う）。

```rust
//! Role: The GPU device a window starts when it needs it (`GpuStart::OnDemand`).
//!
//! A window that may draw on the GPU starts on the CPU and starts its device
//! only when something needs it: a user shader drawn (the CPU draws its
//! fallback meanwhile), or the App reading the window's adapter, as a
//! `gpu_interop::shared_image` does. A window on the CPU only (the test
//! default, `ARVANE_RENDERER=cpu`) never starts one. The tests run where the
//! window can have a GPU device and are skipped, saying why, elsewhere
//! (`gpu_unavailable`). Under `ARVANE_RENDERER=cpu` the on-demand tests check
//! that no device starts instead.
#![allow(
    clippy::print_stderr,
    reason = "report unavailable optional hardware explicitly"
)]
#![allow(
    unsafe_code,
    reason = "providing shared textures is an unsafe fn: the test engine keeps its contract"
)]

use std::time::Duration;

use super::paint::{BLUE, GREEN, RED, at, expect};
use super::support::mount_with;
use kivact::draw::{Paint, PaintShader};
use kivact::prelude::*;
use kivact::style::rgb;
use kivact::test::{MirrorMode, RenderOptions, Renderer, gpu_unavailable};

/// A paint shader that returns blue (its fallback in the tests is red).
const BLUE_PAINT: &str = "fn paint(position: vec2<f32>, context: Context) -> vec4<f32> {\n    return vec4<f32>(0.0, 0.0, 1.0, 1.0);\n}\n";

/// The real time a device that must not start is given to start.
const GRACE: Duration = Duration::from_millis(300);

/// Whether the GPU lane can run here; says why when it cannot.
fn gpu_lane(name: &str) -> bool {
    match gpu_unavailable() {
        Some(reason) => {
            eprintln!("SKIPPED {name}: {reason}");
            false
        }
        None => true,
    }
}

/// `ARVANE_RENDERER=cpu`: every window draws on the CPU only.
fn cpu_forced() -> bool {
    std::env::var("ARVANE_RENDERER").as_deref() == Ok("cpu")
}

/// A window that may draw on the GPU, its device started when needed.
fn on_demand() -> RenderOptions {
    RenderOptions::from_env()
        .window_size(200.0, 100.0)
        .gpu_on_demand()
        .mirror(MirrorMode::Required)
}

/// A real window on the CPU only (the test default).
fn cpu_only() -> RenderOptions {
    RenderOptions::default()
        .window_size(200.0, 100.0)
        .mirror(MirrorMode::Required)
}

/// A green box, and beside it, while `shaded`, a box painted by `shader`
/// (blue on the GPU, its red fallback on the CPU).
fn scene(shaded: Signal<bool>, shader: PaintShader) -> impl IntoView {
    node()
        .child((
            node().style(at(0, 0, 100, 100).background(rgb(GREEN))),
            show(shaded, move || {
                node().style(
                    at(100, 0, 100, 100)
                        .background(Paint::from(shader.paint().fallback(rgb(RED)))),
                )
            }),
        ))
        .style(style().size(200, 100).background(rgb(0xffffff)))
}

/// A shader drawn in a window started with `options` is drawn by its
/// fallback, and no device starts, given the time to.
#[track_caller]
fn shader_stays_on_cpu(options: RenderOptions) {
    let shader = PaintShader::wgsl(BLUE_PAINT).expect("a valid paint shader");
    let (screen, shaded) = mount_with(options, move || {
        let shaded = signal(false);
        (shaded, scene(shaded, shader))
    });
    shaded.set(true);
    screen.run_until_idle();
    std::thread::sleep(GRACE);
    screen.run_until_idle();
    assert_eq!(screen.renderer(), Renderer::Cpu, "a window on the CPU only starts no device");
    expect(&screen.screenshot(), 150, 50, RED, 2, "the CPU draws the fallback");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A window waits on the CPU until it draws a user shader; the frame that
/// draws it first starts the device (drawn by the fallback meanwhile), and
/// the GPU then draws the shader.
#[test]
fn a_shader_starts_the_gpu_on_demand() {
    if !gpu_lane("a_shader_starts_the_gpu_on_demand") {
        return;
    }
    if cpu_forced() {
        return shader_stays_on_cpu(on_demand());
    }
    let shader = PaintShader::wgsl(BLUE_PAINT).expect("a valid paint shader");
    let (screen, shaded) = mount_with(on_demand(), move || {
        let shaded = signal(false);
        (shaded, scene(shaded, shader))
    });
    // Nothing needs the GPU, and an idle window's cheap CPU frames do not
    // start the device.
    screen.advance(Duration::from_millis(500));
    assert_eq!(screen.renderer(), Renderer::Cpu, "no device before the window needs one");
    expect(&screen.screenshot(), 50, 50, GREEN, 2, "the CPU's frame");
    shaded.set(true);
    screen.run_until_idle();
    match screen.renderer() {
        Renderer::GpuStarting => expect(
            &screen.screenshot(),
            150,
            50,
            RED,
            2,
            "the CPU draws the fallback while the device starts",
        ),
        Renderer::Gpu => {}
        other => panic!("a drawn shader starts the device, not {other:?}"),
    }
    screen.wait_for_gpu();
    assert_eq!(screen.renderer(), Renderer::Gpu);
    let shot = screen.screenshot();
    expect(&shot, 150, 50, BLUE, 2, "the GPU draws the shader");
    expect(&shot, 50, 50, GREEN, 2, "and the rest of the window");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A window on the CPU only (as with `ARVANE_RENDERER=cpu`) draws a user
/// shader by its fallback and never starts a device.
#[test]
fn a_cpu_window_never_starts_the_gpu_for_a_shader() {
    if !gpu_lane("a_cpu_window_never_starts_the_gpu_for_a_shader") {
        return;
    }
    shader_stays_on_cpu(cpu_only());
}

#[cfg(all(feature = "gpu-interop", any(windows, target_os = "linux")))]
mod shared {
    //! `shared_image` reads the window's adapter: a window that waits starts
    //! its device, and the engine's GPU frames arrive; a window on the CPU
    //! only asks for pixels and starts none.

    use std::cell::RefCell;
    use std::rc::Rc;
    use std::time::{Duration, Instant};

    use gpu_interop_producer::{Producer, pattern};
    use kivact::draw::ImageOptions;
    use kivact::geometry::Rect;
    use kivact::gpu_interop::{self, ShareMode, ShareStatus, SharedImage};
    use kivact::prelude::*;
    use kivact::test::{RenderOptions, Renderer, Screen};
    use kivact::view::canvas;

    use super::super::support::mount_with;
    use super::{GRACE, cpu_forced, cpu_only, gpu_lane, on_demand};

    /// The image's size.
    const W: u32 = 128;
    const H: u32 = 64;
    /// The longest a step of the exchange waits (real time).
    const WAIT: Duration = Duration::from_secs(10);

    /// A window that makes a `shared_image` once `go` is set and draws its
    /// GPU frame at its own size.
    #[track_caller]
    fn mount(options: RenderOptions) -> (Screen, Signal<bool>, Rc<RefCell<Option<SharedImage>>>) {
        let image = Rc::new(RefCell::new(None::<SharedImage>));
        let handed = image.clone();
        let (screen, go) = mount_with(options, move || {
            let go = signal(false);
            let view = show(go, move || {
                let shared = gpu_interop::shared_image(W, H);
                *handed.borrow_mut() = Some(shared.clone());
                canvas(move |d, _| {
                    if let Some(f) = shared.frame() {
                        let rect = Rect::new(0.0, 0.0, f64::from(W), f64::from(H));
                        let options = ImageOptions { pixelated: true, ..ImageOptions::default() };
                        d.gpu_frame(&f, rect, options);
                    }
                })
                .style(style().size(W, H))
            });
            (go, view)
        });
        (screen, go, image)
    }

    /// A shared image in a window started with `options` asks for pixels,
    /// and no device starts, given the time to.
    #[track_caller]
    fn shares_pixels_on_cpu(options: RenderOptions) {
        let (screen, go, image) = mount(options);
        go.set(true);
        screen.run_until_idle();
        std::thread::sleep(GRACE);
        screen.run_until_idle();
        let image = image.borrow().clone().expect("the window made its image");
        assert_eq!(image.status(), ShareStatus::Pixels, "a window on the CPU asks for pixels");
        assert_eq!(screen.renderer(), Renderer::Cpu, "and starts no device");
        let request = image.link().poll_request().expect("the image asks");
        assert!(matches!(request.mode, ShareMode::Pixels), "{:?}", request.mode);
    }

    /// Making a shared image starts the waiting device; the image then asks
    /// for textures on the window's adapter, and the engine's frame arrives
    /// and is drawn by the GPU.
    #[test]
    fn a_shared_image_starts_the_gpu_on_demand() {
        if !gpu_lane("a_shared_image_starts_the_gpu_on_demand") {
            return;
        }
        if cpu_forced() {
            return shares_pixels_on_cpu(on_demand());
        }
        let (screen, go, image) = mount(on_demand());
        assert_eq!(screen.renderer(), Renderer::Cpu, "nothing needs the GPU yet");
        go.set(true);
        screen.run_until_idle();
        assert!(
            matches!(screen.renderer(), Renderer::GpuStarting | Renderer::Gpu),
            "making a shared image starts the device, not {:?}",
            screen.renderer()
        );
        screen.wait_for_gpu();
        let image = image.borrow().clone().expect("the window made its image");
        let link = image.link();
        // A request of pixels may come first, while the device started.
        let start = Instant::now();
        let (request, adapter) = loop {
            screen.run_until_idle();
            if let Some(request) = link.poll_request()
                && let ShareMode::Gpu(adapter) = &request.mode
            {
                let adapter = adapter.clone();
                break (request, adapter);
            }
            assert!(start.elapsed() < WAIT, "the image asks for textures on the ready device");
            std::thread::sleep(Duration::from_millis(5));
        };
        assert_eq!((request.width, request.height), (W, H));
        let mut producer =
            Producer::new(&adapter, Default::default(), W, H).expect("the engine starts on the adapter");
        let shared = producer.shared_textures().expect("the engine shares its pool");
        // SAFETY: the test engine keeps the contract of `SharedTextures`.
        unsafe { link.provide(&request, shared) }.expect("the newest request takes the pool");
        let start = Instant::now();
        while image.status() != ShareStatus::Ready {
            assert!(start.elapsed() < WAIT, "the pool is imported: {:?}", image.status());
            std::thread::sleep(Duration::from_millis(5));
            screen.run_until_idle();
        }
        let frame = producer.draw(0).expect("the engine draws");
        link.publish(&request, frame.slot, frame.value);
        screen.run_until_idle();
        assert!(image.frame().is_some(), "the GPU frame arrived");
        let shot = screen.screenshot();
        assert_eq!(
            pattern::check(&shot.pixels, shot.width as usize, (0, 0), (W, H), 0, 0),
            pattern::Verdict::Ok,
            "the window draws the engine's frame"
        );
        assert_eq!(screen.renderer(), Renderer::Gpu);
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
        go.set(false);
        screen.run_until_idle();
        assert!(
            producer.wait_release(frame.value, Duration::from_secs(5)),
            "the dropped image releases its frame (release {})",
            producer.release_value()
        );
    }

    /// A shared image in a window on the CPU only asks for pixels and starts
    /// no device (the App's demand is ignored on the CPU).
    #[test]
    fn a_cpu_window_shares_pixels_without_a_device() {
        if !gpu_lane("a_cpu_window_shares_pixels_without_a_device") {
            return;
        }
        shares_pixels_on_cpu(cpu_only());
    }
}
```

GPU の無い機械の扱い：既存の GPU の lane と同じく、まず `gpu_unavailable()` で判定し、`SKIPPED <name>: <reason>` を出して return する。headless（`KIVACT_TEST_MIRROR=off`）でも同じ判定で skip される（`render.rs:9-13`）。CPU だけの window の試験も GPU の lane に置く。GPU が起動し得る機械でだけ「起動しない」ことに意味があるからである。

`ARVANE_RENDERER=cpu` の行の扱い：試験の中で `set_var` はしない（process を共有し、edition 2024 で unsafe であるため）。CPU だけの window の試験は、`prefer_gpu=false` の経路として、env の `cpu` と同じ `RendererStart::Cpu` を確かめる（`RendererStart::resolve` の一つの関数）。加えて、env が `cpu` の時は on-demand の 2 試験が「起動しない」側を確かめる。checkpoint で次を一度実行する。

```bash
ARVANE_RENDERER=cpu KIVACT_TEST_MIRROR=required dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --features gpu-interop --test e2e gpu_on_demand
```

---

## 4 見積り（目標との対比）

- **K 系**
  - 旧実装は、Deferred（既定）、CPU だけ、Failed のどの window でも、CPU の Shown frame ごとに `Composition::needs_gpu()` で全 layer と全 item を走査していた。3000 行（B1 で tree_nodes 36023）では推測で数万 item × 1 frame になる。分岐の予測が効く走査で、frame あたり 20〜60 µs 程度と見積もる（推測）。K4 の必達 3.67 ms の 1〜2 % に当たる。
  - 新設計は、plan の ops を実行する loop の中の `matches!` 一つ（O(ops)。ops は damage の範囲の分だけ）に置き換える。走査は 0 になり、PERF-1 に合う。
- **K2（idle WS）**：既定の OnDemand では、shader も `adapter()` も無い画面で device を起動しない。B1 の counter default 34912 と cpu 34968 がほぼ同じであることから、現状でも起動していないと読める。この設計はそれを変えない。
  - 起動条件の欠陥 D1（CPU だけの window でも App の需要で起動する）を直すので、`ARVANE_RENDERER=cpu` の計測（K2 cpu、kv_heap）に GPU のドライバのメモリが混ざる経路も閉じる。gpu-interop を使わない計測 App は該当しない。
- 本設計は KPI の大きな改善要因ではなく、AC-4 と AC-3 の 2 行を閉じるものである。

---

## 5 正しさ・リスク・E2E

| 境界 | 振る舞い |
|---|---|
| window の close | `Gpu::shutdown` が `Off` に戻す。Deferred が持つ waker は捨てられる。close 後の `DemandGpu` は `Windows::demand_gpu` が presenter を見つけられず、何もしない |
| 二つ目以降の window | `Windows::open` が同じ `begin` を呼ぶ。需要は window ごと（`current_window(owner)`）。registry の `asked` も window ごと |
| 新しい window への需要の到着 | pump は `apply_window_changes`（435 行）の後に `effects`（437 行）を処理する。window が開く前に需要は届かない |
| device の喪失 | `Cpu{restarting}` の後に `GpuStarting{Restart}` → `Gpu`。試験の host は最後の値が GpuStarting なので、`wait_for_gpu` で待てる。2 回目の失敗は `GpuFailed` |
| CPU だけ（`Off`） | `demand` と `note_cpu_frame` は何もしない（D1 の修正）。`adapter()` は需要を一度送るが、host が無視する |
| reduced motion、DevTools | 関係しない。DevTools の Performance panel は `FrameRecord.gpu` を使い、変わらない |
| frame log の互換 | `renderer=gpu` の行は変わらない。新しい行 `renderer=cpu gpu=starting cause=…` が増えるだけである |
| `adapter()` を memo や draw の中で読む | 需要は window ごとに一度だけ（`asked`）。turn の要求は一度だけで、loop にはならない |
| `#[component]` の builder を setup の外で作る | `Viewport::gpu_frame` から `Owner::current` を除いたので、owner の外でも panic しない（D5 の修正） |
| AtLaunch の試験 | `gpu()` は今までどおり AtLaunch で、起動時に待つ。既存の GPU の E2E の期待値は変えない |

既存の E2E での担保：

- `shaders.rs` の gpu_* 4 本、`gpu_interop.rs`、`viewport_gpu_interop.rs`、`drawable_export_gpu.rs`（`gpu()` で AtLaunch）。
- `crates/kivact-test/src/screen/gpu_tests.rs` の 3 本（`gpu()` に置換）。
- `drawable_export_gpu::gpu_export_recovers_after_device_loss`（Restart の経路）。

追加する E2E は 3.6 節の 4 本（gpu-interop の無い build では 2 本）。

リスク：

- `FrameReport` に field が増えるので、他の担当が `FrameReport { .. }` を literal で足すと compile error になる。気付けるので害は無い。
- `arvane-compose/src/cpu/compositor.rs` は CPU の性能の担当と重なる可能性がある。変更は 3 行に留める。
- `adapter()` が engine の effect を出すのは「読み取り関数の副作用」である。doc に明記する。gpu_interop の目的（window と GPU texture を共有する）に限られる関数なので、許容する。

---

## 6 unit 分割案

| unit | 所有 file | 依存 | 完了時の確認 |
|---|---|---|---|
| P6-1 arvane の起動規則と OPEN 2 件 | `crates/arvane/src/{env.rs,window.rs,presentation.rs,lib.rs}`、`crates/arvane/src/presenter/{gpu.rs,frame_policy.rs}`、`crates/kivact-types/src/render/composition.rs`、`crates/arvane-compose/src/cpu/compositor.rs`、`crates/arvane-compose/src/gpu/exec.rs`、`crates/kivact-types/src/gpu.rs`（doc） | — | `cargo check -p arvane --all-targets --features gpu`、`cargo clippy -p arvane --all-targets --all-features -- -D warnings`、`cargo check -p arvane-compose --all-targets --features gpu`、`cargo xtask check --no-write --only state-fields --crate arvane` |
| P6-2 engine の試験 host と kivact-test の API | `crates/kivact-engine/src/host/testing/{test_host.rs,test_options.rs}`、`crates/kivact-engine/src/host/testing.rs`、`crates/kivact-engine/src/contract/host.rs`（doc）、`crates/kivact-test/src/{options.rs,render.rs,lib.rs}`、`crates/kivact-test/src/screen/{driver.rs,state.rs,gpu_tests.rs}` | P6-1 | `cargo check -p kivact-engine --all-targets --features testing`、`cargo check -p kivact-test --all-targets`、`cargo clippy -p kivact-test --all-targets -- -D warnings` |
| P6-3 core の需要を `adapter()` に集約 | `crates/kivact-core/src/gpu_interop.rs`、`gpu_interop/{demand.rs（削除）,import/registry.rs,import/image.rs,share/app_side.rs}`、`view/elements/viewport.rs`、`view/drawable/{handle.rs,requests.rs}`、`docs/for-kivact-user/{en,jp}/how-to-guide/gpu-texture-sharing.md`、`README.md`、`README_JP.md`、`docs/for-kivact-user/{en,jp}/README.md` | なし（P6-1 と並行できる） | `cargo check -p kivact-core --all-targets --all-features`、`cargo xtask check --no-write --only state-fields --crate kivact-core` |
| P6-4 E2E | `dev/tests/kivact-public-api/tests/e2e/gpu_on_demand.rs`（新規）、`e2e/main.rs`（1 行） | P6-2、P6-3 | `cargo check -p kivact-public-api-tests --tests --features gpu-interop`。実行は checkpoint で `KIVACT_TEST_MIRROR=required dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --features gpu-interop --test e2e gpu_on_demand` |
| P6-5 macro の修正（C-1〜C-4） | `crates/kivact-macros/src/{parse.rs,expand.rs}`、`dev/tests/kivact-public-api/tests/e2e/component_macro.rs`、`docs/for-kivact-user/{en,jp}/how-to-guide/custom-components.md` | — | `cargo check -p kivact-macros`、`cargo check -p kivact-public-api-tests --tests` |
| P6-6 文書の誤り（C-6〜C-11） | 9 章の C-6〜C-11 の file（docs、examples の catalog の文字列、各 crate の rustdoc）。kit の公開 API の改名は含まない | P6-5 と並行 | `cargo doc -p kivact-core --no-deps`（警告 0）、`cargo check -p kivact-ui-soft --all-targets` |
| P6-7 ARG-1 の残りと api-shape | `crates/kivact-ui/src/calendar/view.rs`、`crates/kivact-interaction/src/menu/handle.rs`、`dev/xtask/src/api_shape.rs` | — | `cargo check -p kivact-ui -p kivact-interaction`、`cargo test -p xtask api_shape`、`cargo xtask check --no-write --only api-shape` |
| P6-8 GpuPreference の移動（C-5、8 章で承認後） | `crates/kivact-core/src/{launch/settings.rs,app.rs,gpu_interop.rs,lib.rs}`、`crates/kivact-test/src/options.rs`（cfg を外す）、docs と E2E の import | P6-2、P6-3 | `cargo check -p kivact --no-default-features`、`cargo check -p kivact --features gpu-interop`、`cargo check -p kivact-public-api-tests --tests --features gpu-interop` |
| P6-9 ARG-2/3/6 の改名（8 章で承認後） | kivact-ui、kivact-interaction、各 kit、docs | — | 各 crate の `cargo check` |

---

## 7 既存欠陥（79bbd409..db8d679b）

### D1【バグ・重大】CPU だけの window でも App の需要で GPU が起動する

- 位置：`crates/arvane/src/presenter/gpu.rs:358-371`（`start` が `State::Off` から始める）、`frame_policy.rs:223-225`（`demand_gpu` → `start`）。
- 再現条件（静的な読解から推測）：`ARVANE_RENDERER=cpu`、`Shell::prefer_gpu(false)`、`ShellSetup.prefer_gpu=false`（embed の既定 false の経路、`crates/arvane/tests/e2e/embed.rs:114`）、または CPU の試験の host のいずれかで、App が `shared_image`、`Viewport::gpu_frame`、`import`、GPU の export を使う。すると worker が device を作る。試験の host が mirror の実 window を持つ場合は GPU の surface も作れるので、renderer は Gpu になる。
- これは設計の表の 1 行目と `Windows::demand_gpu` の doc に反する。
- 修正：3.2 節（`demand` は `Deferred` の時だけ、`begin` が `Off` から起動する唯一の入口）。E2E は `a_cpu_window_shares_pixels_without_a_device`。

### D2【性能・PERF-1 違反】毎 CPU frame で composition 全体を走査する

- 位置：`frame_policy.rs:328-341`。`composition.needs_gpu()` は引数として先に評価される。状態が Off、Failed、Background の時も走査する。
- 修正：3.2 節（`FrameReport.shader_fallback`、`waits()` の時だけ判定する）。`Composition::needs_gpu` は削除する。

### D3【機能】既定の OnDemand では低水準の import と drawable の GPU export が始まらない

- 位置：`import/image.rs:191-192`、`drawable/requests.rs:53-56`。
- どちらも adapter（Ready 後に得られる）を前提とするので、最初の需要になり得ない。how-to の手順（`adapter()` → engine の device → `import`）は、shared image を併用しない限り、ずっと `None` を待つ。
- 試験は `gpu()`（AtLaunch）なので発覚していない。
- 修正：3.5 節（需要を `adapter()` に置く）。

### D4【軽】「同じ object から一度」が守られていない

- drawable の export は、`(mode, generation, scale)` が変わるたびに需要を再送する（`requests.rs:50-56`）。
- 修正：3.5 節で削除し、`asked` で window ごとに一度にする。

### D5【バグ】`Viewport::gpu_frame` が builder の時に `Owner::current` を呼ぶ

- 位置：`viewport.rs:83-93`。
- owner の外で作った builder（5.2 節で許した使い方。`#[component]` の struct を setup の外で作る）では、located の SetupOnly misuse で panic する（推測：`Owner::current` の契約から）。
- builder を作った時点の window に需要が出るので、mount しない viewport も GPU を起動する。
- 修正：3.5 節で削除する。

### D6【文書】存在しない API や古い記述を参照している

- `crates/kivact-test/src/screen/gpu_tests.rs:7-9` は存在しない `Screen::renderer` を参照し、`lib.rs:31-32` は「no API says which one drew」と書く。どちらも 3.4 節で整合させる。
- `HostRequest::DemandGpu` の doc（`contract/host.rs`）は「a composition with a user shader」を理由に挙げるが、shader は host request を通らない。3.3 節で直す。

### D7【設計の不整合】試験の host が `GpuPreference.start` を黙って上書きする

- 位置：`test_host.rs:181-191`。
- 試験が `gpu_preference(GpuPreference { start: OnDemand, .. })` を与えても、`gpu()` があると無視される。U56 の STOP の直接の原因である。
- 修正：3.3 節と 3.4 節（kivact-test が `start` を決める）。

---

## 8 設計者（人間）の判断が要る点

1. **`GpuPreference`、`GpuPower`、`GpuBackend`、`GpuStart`（と `AdapterMatch`）の所在**（C-5）
   - 現状、これらと `App::gpu_preference` は feature `gpu-interop` の時だけ公開される（`kivact-core/src/app.rs:104`、`lib.rs:65`）。既定の build の App は AtLaunch も HighPerformance も選べず、`ARVANE_RENDERER` に頼るしかない。
   - 推奨：`kivact::launch` に移して feature なしで公開し、`App::gpu_preference` と `RenderOptions::gpu_preference` の cfg を外す。feature `gpu` が無い build では無視されることを doc に書く。`gpu_interop` からの再公開は削除する（別名を作らない規則）。
   - 公開の path が変わるので承認が要る。不承認なら、現状のまま「start を選べるのは gpu-interop の build だけ」と README に書く。
2. **ARG-6 の命名の衝突**（C-13）
   - `MenuApi::open(self)`/`close` と `ComboboxApi::open(self)`/`close` を `show`/`hide` にすると、既存の `MenuApi::show(factory)` と `CommandPaletteApi::show(factory)`（content を mount する method）と名前が重なる。
   - 推奨：content の方を `content(factory)` に改名する。kit の公開 API の改名なので、範囲の承認が要る。
3. **API-3「handler を位置引数にしない」と `#[component]` の必須 callback の位置引数**（C-14）
   - architecture.md:136 の規則と、custom-components.md:43、257、305、soft の `more_menu(items, choose)` が食い違う。
   - 推奨：`#[component]` では「必須の props は位置引数」（5.1 の表）を優先する。architecture.md の API-3 を「kit の builder の任意 handler は setter」と限定して書き直す。

---

## 9 公開 API のレビュー（React/Solid/JS/Python 利用者の観点、重要度順）

| ID | 種別 | 内容と位置 | 修正 |
|---|---|---|---|
| C-1 | バグ | `#[component]` の `#[prop(optional)] x: Option<Value<T>>` で、mount が `IntoValue::into_value(x)` を `Option<Reactive<T>>` に適用する（`expand.rs:128-130`）。`impl IntoValue<Option<T>> for Option<T>` が選ばれて `Value<Option<Reactive<T>>>` になり、本体の型と合わず compile error になる（推測：impl の解決から）。E2E は optional の Value を試していない（`component_macro.rs:29-30` は default と optional の callback だけ） | `mount` の先頭に `PropKind::Text if prop.is_optional() => Some(quote!{ let #name = #name.map(|v| ::kivact::state::IntoValue::<::std::string::String>::into_value(v)); })` と、`PropKind::Value(ty) if prop.is_optional()` で `IntoValue::<#ty>` を使う腕を足す。E2E の `component_macro.rs` に `#[prop(optional)] subtitle: Option<Value<String>>` を足し、省略時と指定時の表示を確かめる |
| C-2 | バグ | doc 以外の属性（`#[cfg]`、`#[cfg_attr]`、`#[allow]`、`#[expect]`、`#[deprecated]`）を捨てる（`parse.rs:74-79`）。`#[cfg(feature="x")] #[component] fn ..` が無条件に compile される | `Model` に `attrs: Vec<Attribute>` を足す。`cfg`、`cfg_attr`、`allow`、`expect`、`warn`、`deny`、`deprecated` だけを受け取り、struct、constructor、`impl`（setter）、`impl IntoView` のすべてに付ける。それ以外は `compile_error!("#[component] keeps only cfg, lint and deprecated attributes")` |
| C-3 | 診断 | `async fn`、`const fn`、`unsafe fn`、`extern "C" fn` を拒まず、展開後に分かりにくい error になる（`parse.rs:107-130`） | `check_signature` で `sig.asyncness`、`constness`、`unsafety`、`abi`、`variadic` を拒む。message は「a #[component] function is a plain `fn`」 |
| C-4 | 意味 | callback の `#[prop(default = expr)]` は、`store` が `Box::new(move \|\| Into::into(expr))` を作るので、`expr` を mount の時に評価する。規則 5「呼ばれた時」と異なる | `initializer` の `Default` の腕を `{ let __default = #expr; <store(prop, quote!{__default})> }` にする |
| C-5 | API の欠落 | `GpuStart`、`GpuPreference`、`App::gpu_preference` が gpu-interop の build だけにある | 8 章の 1 |
| C-6 | 文書のバグ | custom-components en:82-84、jp:68 は「本体が `on_close.map(\|f\| f())` で変換する」と書く。実際は展開が変換済みの `Option<Callback<()>>` を渡す（`expand.rs:131-133`）ので、書いたとおりにすると compile error になる | 文を「the body receives `Option<Callback<()>>`, already converted in the component's setup」にする（jp も同様） |
| C-7 | 文書のリンク切れ | `provide_theme`、`provide_locale`、`provide_messages` の intra-doc link が `crate::view::…` を指す。`kivact-core/src/app.rs:23,33,42`、`launch/settings.rs:73,83`、`embed.rs:213`。利用者向けの error の文字列 `launch/settings.rs:62,188` は「use view::provide_theme」。ほかに `view/boundary.rs:264,297,320`、`kivact-style/src/theme/value.rs:198,495`、`crates/kivact-ui-creative/README.md:24`、`docs/.../coming-from-react-or-solid.md` en:375-376、jp:326 | `crate::style::provide_theme`、`crate::i18n::provide_locale`、`crate::i18n::provide_messages` に直す。error の文字列は `style::provide_theme` にする |
| C-8 | 改名漏れ | 試験の guide が `Element` のまま。en `testing.md:104,179,336`、`windows.md:325`、jp `testing.md:88,153,287`、`windows.md:268` | `Locator` |
| C-9 | catalog の誤り | `examples/kivact-ui-soft/src/docs/overlays.rs:203,219,270` の表示用の文字列が旧 `menu` の arity と旧名のまま | `menu("Actions", items).on_activate(on_activate).attach(button("Actions"))`、`…attach_context(button("Document"))`、`more_menu(items, on_activate)` |
| C-10 | 規約文の誤り | `crates/kivact-ui/src/lib.rs:108` が旧 ARG-1（`impl Fn() -> R` を許す）を書く。`:114` の `use kivact::ui::ButtonVariant;` は prelude と重複する | ARG-1 の文に直し、import を消す |
| C-11 | 5.4 節の文書修正の未実施 | `examples/kivact/component/src/main.rs:22` の `.to_owned()`、`examples/kivact/form/src/main.rs:23` の `kivact::view::InputPurpose::Password`。入門 guide の `kivact::ui::ButtonVariant::X`（en 06:41、144、07:43、08:54、overlays.md:36、39、theming.md:227-231 と jp の対応箇所）。docs の code block の prelude と重複する import が en の for-kivact-user だけで 82 箇所（canvas.md、animation.md 各 23、custom-controls.md 17 など） | 短い名前に直す。重複 import は削除し、空になった `use` 行と一要素の `{X}` も消す。数が多いので一括の置換でよい。対象の file は再調査の上で P6-6 が持つ |
| C-12 | ARG-1 違反 | `kivact-ui/src/calendar/view.rs:280` の `on_commit(f: impl Fn() -> R)`。`kivact-interaction/src/menu/handle.rs:190` の `trigger_action(action: impl Fn() + 'static)`。xtask の api-shape は `Fn() -> R` を受け入れ（`dev/xtask/src/api_shape.rs:139-151`、試験 `:331`）、`on_` で始まらない名前を見ない | 両方を `impl Into<Callback<()>>` にする。api-shape の `fn_bound_handler` で入力の無い `Fn()` を拒み、試験の期待を 3 件に変える。`trigger_action` は名前に依らず、公開の builder method で `impl Fn()` を受けるものを拒む規則を足す |
| C-13 | ARG-2/3/6 の未適用 | `combobox(label, options, selection)`、`tabs(label, options, binding)`、`toggle_group(.., options, selection)`、`disclosure_group(selection)`、`tree(..).selection(..)`。interaction の `use_*` hook、soft の `tab_list(binding)`、creative の `gradient_editor(selection)`。`ComboboxApi::open(self)`、`MenuApi::open(self)`、`InlineEditApi::open(self, key)` | `items`、`selected`、`value` に改名する。`tree(label, items, selected)` にし、setter を削除する。命令の method は 8 章の 2 に従う |
| C-14 | 規約の矛盾 | API-3 と、`#[component]` の必須 callback の位置引数 | 8 章の 3 |
| C-15 | 小 | `kivact-test/src/lib.rs:64`、`render.rs:55` の doc の例が `Element` を import しながら使わない。prelude に `provide_theme` はあるが `provide_locale` は無い。prelude の `use` が同じ module ごとに複数行に分かれている | 例から `Element` を消す。prelude は `provide_locale` と `provide_messages` を足す（`provide_theme` と対にする）か、`provide_theme` を外して揃える。推奨は外す（provide は App の root で一度書くだけで、明示の import が分かりやすい） |
| C-16 | 良好 | `Binding::update` は設計どおりで、Signal と pair の違いが rustdoc にある。改名（NodeBuilder→Element、component()→scope()、test の Element→Locator、Component の削除）は code の上では完了している（prelude も要件どおり） | — |
| C-17 | 高速経路 | `cpu/draw.rs:309-350` は正しい（1.3）。性能面では、scissor の矩形の判定が画素ごとに `rects.iter().any` を行う | 行ごとに x の区間を前計算すると、さらに速くなる。任意で、CPU の性能の担当に渡す |
