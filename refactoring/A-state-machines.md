# 付録 A: 暗黙の状態機械の一覧と解消後の形

2026-11-05 改訂: 担当の unit を新しい ID に直し、設計書 19 章による解消後の形の変更を反映した。

[設計書](../REFACTORING_DESIGN.md) 13 章の付録。`file:line` は `main` @ `a3536ef`（作業が進むとずれるので名前で検索し直す）。path は `crates/` からの相対。

各項目の書式:

- **場所**: 現在のコード
- **現在**: 状態を表している field
- **不正**: 表せてしまう不正な組合せ、または守り方の問題
- **解消後**: 決定した形。型名と variant は決定。variant の field の名前・event・effect・遷移は、設計書 0.3 節の M-1〜M-3（現在の code からの機械的な導出）で作る
- **unit**: 作業手順書の担当 unit

**`Machine` の形は設計書 18.13 節が正**（`Input`・`Timer` の関連型と `MachineCx::after` はない。timer は遷移が `Step::after(delay, event)` で出す）。この付録の項目の「timer は `after`」「`MachineCx::after(..)`」「`type Timer = ..`」は、`Step::after` の意味に読む。

**この付録の項目は調査で見つけたもので、unit P-1（設計書 17.8 節の全数調査）が足す項目を含めて全部になる。** 足した項目は接頭辞のあとに 9 から始まる番号を付ける（例: SM-E901）。

共通の部品（`kivact-types::cell` に置く。std だけの小さな型で、engine・Arvane・中核・kit が使う。unit V-4 が作る）:

```rust
/// One-shot diagnostic latch (S-9).
pub struct ReportOnce(Cell<Reported>);      enum Reported { Not, Done }
/// Set of keys reported once (replaces `noted_missing: RefCell<HashSet<..>>`).
pub struct ReportOnceSet<K>(RefCell<HashSet<K>>);
/// Last value handed to an outside party, for deduplication (S-9).
pub struct Sent<T>(Option<T>);  // impl: fn update(&mut self, v: T) -> Option<&T> (Some = changed, send it)
/// Monotonic generation for discarding stale results (S-9).
pub struct Generation(u64);     // next(), is_current(g)
/// Take-once cross-thread mailbox (S-9).
pub struct Mailbox<T>(Mutex<Option<T>>); // put(v) (latest wins), take()
/// Take-once UI-thread mailbox (S-9).
pub struct LocalMailbox<T>(Cell<Option<T>>); // put(v) (latest wins), take()
/// Registry of weakly held entries with one prune rule (replaces three copies).
pub struct Registry<T: ?Sized> { entries: Vec<Weak<T>> }
```

`kivact-types::process` に置くもの: `DetachedThreads`（SM-R20）。`Machine` は `kivact-runtime`、`OptionsCell<S>` は `kivact-interaction`（crate private）に置く。

---

## SM-R: `kivact-runtime`（旧 `kivact-reactive`・`kivact-task`・`kivact-motion`）

### SM-R01 effect の予定の状態
- **場所**: `kivact-reactive/src/data.rs:185-192`（`Computation { state: Color, running: bool }`）、`turn.rs:24-30`（`Sched.push` が bool で重複を除く）、`turn.rs:109-118`（`abandon_render_effects`）、`graph.rs:288-332,413-427,495-518`
- **現在**: 色（Clean/Check/Dirty）× `running` × 「`Sched.render`/`Sched.user` の集合に入っているか」
- **不正**: Clean なのに queue にある、削除済みの effect が `running = true`。queue への所属が node に記録されていない
- **解消後**:
  ```rust
  enum CompState { Clean, Stale(Staleness), Queued(Staleness), Running }
  enum Staleness { Check, Dirty }
  ```
  queue に key があること ⇔ `Queued` を不変条件にし、push と pop は `CompState` の遷移メソッドだけが行う。`abandon_render_effects` は `Queued → Clean` の遷移として書く
- **unit**: R-3

### SM-R02 effect の一 pass あたりの実行回数
- **場所**: `data.rs:180-181`（`stab_epoch`、`stab_runs`）、`turn.rs:21,78-89`（`CYCLE_LIMIT`）
- **現在**: epoch の不一致を「この pass でまだ実行していない」と読む
- **不正**: field を直接比べる箇所が散る
- **解消後**: `struct PassRuns { epoch: Epoch, runs: u16 }` と `fn note_run(&mut self, epoch) -> Result<(), CycleDetected>`。field は `impl` の外から触らない（S-9 の counter）
- **unit**: R-3

### SM-R03 owner の lifecycle
- **場所**: `data.rs:274-291`（`OwnerEntry.disposing: bool`）、`owner.rs:46,50-57,80-95,105-111,265-296`（`RELEASE_ROUNDS = 16`）
- **現在**: `disposing` と arena にあるかで Live/Disposing/Gone を計算する。破棄中の親の下に作ると即座に tombstone
- **不正**: `disposing` を見ずに子を足せる経路がありうる
- **解消後**: `enum OwnerPhase { Live, Disposing { round: u8 } }` を `OwnerEntry.phase` に保存。Gone は arena にないこと。破棄中の親の下での作成は `Owner::dead()`（一度も Live にならない handle）を返す
- **unit**: R-3

### SM-R04 turn の要求
- **場所**: `data.rs:74`（`turn_requested: Cell<bool>`）、`driver.rs:35-57`、`kivact-core/src/client/link.rs:31-33,46-50,73-82`（`in_turn: Cell<bool>`）、`client/app_client.rs:259-263,290,355`
- **現在**: 二つの bool と、client の 3 か所での clear の呼び出し順
- **不正**: turn 中に要求が来たのに clear が先に走ると取りこぼす。二つの bool の 4 通りの組合せの意味が書かれていない
- **解消後**: `TurnState::{Quiet, WakeSent, Running(Stage)}`（非公開。設計書 19 章「turn の順序は core の一つの関数が持つ」）。`RunningRequested` はなく、turn の中の要求は wake しない。turn の後に wake するかは `TurnGuard::finish` の pending の判定だけで決まる。`Host::wake` は `Quiet → WakeSent` の遷移だけが呼ぶ
- **unit**: R-2

### SM-R05 `on_mount` の待ち
- **場所**: `data.rs:424-430`（`PendingMount.token: Option<u64>`）、`mounts.rs:34-67`、`kivact-core/src/client/phases.rs:86`、`app_client.rs:193-199,294,338`
- **現在**: `None` = まだ commit の revision が付いていない、`Some(t)` = revision t の確認を待つ
- **不正**: client が commit の後に `defer_until` を呼ばないと永遠に `None`
- **解消後**: 二つの型付きの queue。`unstamped: Vec<Mount>` → `awaiting: VecDeque<(Revision, Mount)>`（値を移すので同時に二つに入らない）。core の `AppClient::turn` が commit の中で `TurnGuard::stamp_mounts(revision)`（`Stage::Commit` の外は misuse）を、boundary と commit の後で `run_mounts(acked)` を呼ぶ（設計書 19 章「turn の順序は core の一つの関数が持つ」）
- **unit**: R-2

### SM-R06 後で届ける報告
- **場所**: `data.rs:73`（`later: Vec<(Option<Key>, UiError)>`）、`kivact-core/src/client/phases.rs:58`
- **現在**: turn の始めに `deliver_later` を呼ぶという呼び出し順の規則
- **解消後**: `TurnGuard::deliver_reports`（`Stage::Reports`。turn の始めの snapshot だけ）が届ける（設計書 19 章「turn の順序は core の一つの関数が持つ」）
- **unit**: R-2

### SM-R07 実行中の writer
- **場所**: `data.rs:77`（`writer: Cell<Option<Key>>`）、`graph.rs:436-454`（保存と復元）
- **現在**: save/restore で表した再入の stack
- **解消後**: frame の stack の各 `Frame` が `writer: Option<Key>` を持つ（SM-R08）。別の `Cell` は消す
- **unit**: R-3

### SM-R08 実行の文脈（frame の stack）
- **場所**: `data.rs:310-317`（`Ctx { owner, setup, pure, effect, tracking }`）、`frame.rs:32-95,183-195`
- **現在**: 5 つの独立した field の組合せで setup・tracking・pure・untracked・callback・handler を表す
- **不正**: `setup: Some` かつ `pure: true`。`tracking: Some(slot)` が `collectors.len()` を越える（`debug_assert` だけ）
- **解消後**:
  ```rust
  struct Frame { owner: Option<Key>, kind: FrameKind, tracking: Option<CollectorSlot>, pure: bool, writer: Option<Key> }
  enum FrameKind { Handler, Setup(Key), Computation { node: Key, effect: bool } }
  ```
  `pure_section`・`untracked_frame`・`callback_frame` は親の frame の `kind` を写し、`pure`・`tracking`・`owner` だけを変える（現在の `..self.context()` と同じ意味。設計書 19 章「reactive の frame と Watcher」）。`setup_frame` は `Setup(owner)`、`tracking_frame` は `Computation`、`handler_frame` は `Handler`。`CollectorSlot` は push の時に作る型で、範囲外を表せない
- **unit**: R-3

### SM-R09 driver の後付け
- **場所**: `driver.rs:22-26`（`driver: OnceCell<Rc<dyn Driver>>`）
- **解消後**: `Runtime::new(host, waker, config, build)` の引数（設計書 19 章「host との境界と構築」）。`Driver` trait は消える
- **unit**: R-2

### SM-R10 root owner の後付け
- **場所**: `data.rs:60`（`root_owner: Cell<Option<Key>>`）、`host.rs:83-87`（`create_root`）
- **解消後**: `Runtime::new` が root owner を作る
- **unit**: R-2

### SM-R11 `effect_on` の初回の skip
- **場所**: `setup.rs:171-185`（`first: Cell<bool>`）
- **解消後**: graph の node の種類 `Watcher`。自分の queue（`effect_on` は User、`Animated` は Internal）の最初の実行で source を pure・tracking で評価して依存を取るだけで `f` を呼ばず、以後の再実行のたびに source を評価して `f` を untracked で呼ぶ。前の値との比較はしない（設計書 19 章「reactive の frame と Watcher」）
- **unit**: R-3

### SM-R12 `Selector` の cache
- **場所**: `selector.rs:20-31`（`current: RefCell<Option<K>>`、遅延の `nodes`、倍々の `prune_at`）
- **解消後**: `struct SelectorCache<K> { current: Option<K>, nodes: HashMap<K, Key>, prune: PruneSchedule }`。`PruneSchedule` は名前の付いた cache の規則（S-9）
- **unit**: R-3

### SM-R13 `Action`
- **場所**: `kivact-task/src/action.rs:114-125`（`Worker`）、`:142-148,495-509`（`worker_ids` の世代）、`:179-187`（`Lifecycle`）、`:189-205`（`ActionCell` 13 field）、`:345-361`（`on_dispose`）、`:477-488`（`queue_status`）
- **現在**: `Lifecycle` と `Worker` の enum の外に `state`・`latest`・`last`・`waiting`・`worker_ids`・`drained`・`next_handler`
- **不正**: `Lifecycle::Detached` と `Runner::Latest` の組（早い return だけが防ぐ）。`Pending` なのに `Worker::Stopped` で `waiting` が空。`Ready` なのに `last == None`
- **解消後**:
  ```rust
  enum ActionPhase<I> { Open { worker: Worker, waiting: VecDeque<Submitted<I>> }, Closed, Detached(Drain) }
  // Drain は Runner::Queue | Runner::All からだけ作れる（Runner::Latest は Detached にならない）
  struct ActionCell<T, I> { phase: ActionPhase<I>, async_state: AsyncCell<T>, last_input: Option<I>, generation: Generation }
  ```
  `AsyncState::Pending` ⇔ `Worker::Running|Picking` または `waiting` が空でない、を遷移関数の中で保つ。状態は `AsyncCell` のメソッドを通してだけ書き、`last` は `AsyncCell` の `latest`（SM-R15）。`worker_ids` は `Generation`（設計書 19 章「非同期の状態」）
- **unit**: R-5

### SM-R14 `LatestRun`
- **場所**: `kivact-task/src/latest.rs:12-23,83-89`
- **現在**: `current: Cell<Option<Inflight { task, id }>>` と `next_id`
- **解消後**: `enum Latest { Idle, Running { task: Owned<TaskCell>, generation: Generation } }`（設計書 19 章「非同期の状態」）
- **unit**: R-5

### SM-R15 `Resource` と `Action` の状態の重複
- **場所**: `kivact-task/src/resource.rs:16-21`、`action.rs:196-197`
- **現在**: どちらも `state: Signal<AsyncState<T>>` と `latest: Signal<Option<T>>` を別々に持つ
- **解消後**: `struct AsyncCell<T> { state: Signal<AsyncState<T>>, latest: Signal<Option<T>> }`。書くのは `start`・`finish`（Ok は Ready と latest、Err は Failed で latest を保つ）・`reset` だけ。`state()`・`latest()` はそれぞれの signal を読み、memo にしない（`T` は `Clone` だけで `PartialEq` を求めない。設計書 19 章「非同期の状態」）
- **unit**: R-5

### SM-R16 `Sleep` の timer の登録
- **場所**: `kivact-task/src/time.rs:16-66`（`timer: Option<(deadline, id)>`、`Drop` の `try_borrow_mut` が失敗すると黙って消し損ねる）
- **解消後**: `enum SleepState { Unregistered { deadline }, Registered { id: TimerId }, Fired }`。`Drop` は `Scheduler` の `CancelQueue`（`RefCell` を借りない `LocalMailbox`）に id を積み、`take_due` と `next_deadline` は heap を読む前に必ず `CancelQueue` を空にする（設計書 19 章「frame と animation の時間」）。消し損ねを表せないようにする
- **unit**: R-4

### SM-R17 `TaskCell` の後付け
- **場所**: `kivact-task/src/state.rs:48-51`、`spawn.rs:49-63`（`task: OnceCell<RawTask>`）
- **解消後**: 解消しない（`TaskCell.task: OnceCell` を残す。task の future は cell を owner に保存した後の key を持つ必要があり、cell は同じ関数の中で最初の poll の前に埋まるので、空の状態は外から見えない。設計書 19 章「非同期の状態」）。`finished: Vec<Owned<TaskCell>>` は型付きの queue（13.1 節の 7 の例外）としてそのまま
- **unit**: なし（解消しない）

### SM-R18 blocking の pool
- **場所**: `kivact-task/src/state.rs:140-148,222-230`（`pool: RefCell<Option<Arc<Pool>>>`、`shutdown` で `None` に戻り、後の job で作り直す）
- **解消後**: `enum PoolState { NotStarted, Running(Arc<Pool>), ShutDown }`。`ShutDown` の後の `blocking` は invariant の panic（黙って作り直さない。`blocking` は `UiError` を返せず、shutdown の後に App の code は走らないので、`ErrorKind::Shutdown` は足さない。設計書 19 章「非同期の状態」）
- **unit**: R-5

### SM-R19 executor の waker
- **場所**: `kivact-task/src/executor.rs:32-35,53-55`（`Arc<Mutex<Option<ReadyWaker>>>`）、`kivact-core/src/client/app_client.rs:230`
- **解消後**: `Runtime::new` の引数の `ThreadWaker` を executor の構築に渡す（`TaskState::set_waker` は消える。設計書 19 章「host との境界と構築」）
- **unit**: R-2

### SM-R20 process 全体の「切り離したスレッド」
- **場所**: `kivact-task/src/detached.rs:4`（`static DETACHED: AtomicBool`）、`arvane/src/shutdown.rs:9`、`arvane-native/src/detached.rs:3`、`arvane-asset/src/detached.rs:4`（計四つ）、`kivact-engine/src/embed.rs`（`detached_threads`、`retain_module`）
- **解消後**: `kivact_types::process::DetachedThreads`（`static` の `AtomicUsize` 一つ、`note()`、`any()`）。四つの static を消して全部がこれを使う。`retain_module` は arvane-native の embed の `Session` が handler を drop した直後に `DetachedThreads::any()` なら呼ぶ（`RetainOnDrop` を移す。設計書 19 章「`Backend` は既存の `ShellEventLoop` の改名にする」）
- **unit**: V-4（`retain_module` の移動は A-1a）

### SM-R21 終了時の drain の所属
- **場所**: `kivact-task/src/state.rs:68-71`（`queues: Vec<Weak<dyn Drain>>`、`draining`）、`action.rs:345-361`
- **現在**: 「どちらの list に入っているか」が状態
- **解消後**: `ActionPhase::Detached(Drain)` が drain の状態を持ち、`TaskState.draining: Vec<Rc<DrainEntry>>` は `Detached` への遷移で移される型付きの queue。`queues` の Weak の表は `Registry<dyn Drain>`
- **unit**: R-5

### SM-R22 motion の環境の後付け
- **場所**: `kivact-motion/src/state.rs:60-67,78-90`（`env: Cell<Option<MotionEnv>>`）、`kivact-core/src/view/root.rs:594-598`（`attach_env`）
- **不正**: `attach_env` の前に frame を計画すると panic
- **解消後**: `MotionEnv { reduced_motion, surface_active, frame_period_ns }`（`Value` の束）を core が `Runtime::new` の `build` の中で `Extensions` に入れ、motion は `ext::<MotionEnv>()` で読む。`attach_env`・`env: Cell<Option<MotionEnv>>` を消し、`RuntimeConfig.reduced_motion`・`Runtime::set_reduced_motion` は作らない（設計書 19 章「frame と animation の時間」）
- **unit**: R-4

### SM-R23 frame の時計
- **場所**: `kivact-motion/src/state.rs:18-25`（`requested: Option<u64>`、`last_tick: Option<u64>`、`ticks`）、`:63-66,155-174`（`applied_offset_ns`）、`:186,235-266`
- **解消後**:
  ```rust
  struct FrameClock { phase: FramePhase, offset: ClockOffset, ticks: u64 }
  enum FramePhase { Idle, Requested { at: u64, last_tick: Option<u64> }, Ticked { last: u64 } }
  ```
  要求中も前の tick の時刻を読むので、`FramePhase` は要求と最後の tick を一緒に持つ（設計書 19 章「frame と animation の時間」）。`ClockOffset` は engine の時計の offset のうち適用済みの量を持つ名前の付いた型
- **unit**: R-4

### SM-R24 `FrameLoop`
- **場所**: `kivact-motion/src/state.rs:294-334`（`LoopCell { run: Cell<Option<LoopRun>>, rate: Signal<Option<FrameRate>>, frames: Cell<u64> }`、`set_run` だけが `rate` を合わせる）
- **解消後**: `LoopCell { owner, f: Handler<FrameInfo>, state: Signal<LoopState>, frames: Cell<u64> }`、`enum LoopState { Stopped, Running(LoopRun) }`。`rate()` は `state` の memo。frame loop は今までどおり `FrameTick` の event の中で同期に走る（`next_frame()` と task の frame loop は作らない。設計書 19 章「frame と animation の時間」）
- **unit**: R-4

### SM-R25 `Animated`
- **場所**: `kivact-motion/src/animated.rs:24-46`（`Phase` と `running: Signal<bool>`）、`:26-30`（from/to/current の `RefCell<Vec<f32>>`、`SpringRun`）、`:33,96,379-387`（`completed: Signal<u64>` と読む側の `seen: Cell`）、`:262-270`（`first_cell: Cell<bool>`）
- **解消後**:
  ```rust
  pub(crate) enum AnimPhase { Idle, Running { start_ns: u64 }, Paused { start_ns: u64, at_ns: u64 }, Completed }
  ```
  `Animated` は `phase: Signal<AnimPhase>` 一つ（`running`・`phase: Cell`・`completed` の counter を消す）。自然に終わった run は `Completed`、`is_running()` は memo。`on_complete` は `phase` を source にした Internal の Watcher が `Completed` を見たときに handler の frame で呼ぶ（遷移の中では呼ばない）。初回の skip は Watcher（SM-R11）（設計書 19 章「frame と animation の時間」）
- **unit**: R-4

---

## SM-C: `kivact-core`

### SM-C01 起動（`Launch`）
- **場所**: `client/app_client.rs:92-97,224-227`
- **現在**: 既に enum（`Waiting(Box<AppConfig>)`、`Started`）。ただし runtime と extension を `Waiting` の前に作っている
- **解消後**: `Launch::{Waiting { options: AppOptions, mode: ClientMode, root }, Started(Started)}`（`AppOptions` の field は private。設計書 19 章「`AppOptions` と起動の経路」）。`Started` を作るときに runtime・`ViewState`・`HostState` を全部作る（S-7）
- **unit**: C-1

### SM-C02 終了
- **場所**: `client/state.rs:19-30,62-68`（`ExitPhase`）、`app_client.rs:365-397`、`TaskState.queues_busy()`
- **現在**: 遷移が `request_close` と `output()` に分かれ、条件が別の crate にある
- **解消後**: runtime の `ExitState::{Running, Draining { deadline }, Exited}`。遷移は `Runtime::request_exit` と `TurnGuard::finish`・`Runtime::report_now` だけ（設計書 19 章「turn の順序は core の一つの関数が持つ」）
- **unit**: R-2

### SM-C03 embed の閉じる要求
- **場所**: `app_client.rs:109-110,146-160,205-207`（`embed_closing: Option<Arc<AtomicBool>>`）
- **解消後**: `ClientMode::Embedded`。閉じる flag は engine の `CloseSignal`（SM-E02）で、立っていれば engine は `Closed` へ進み exit の turn を走らせない。`HostEvent::CloseRequested` と、そのための `Runtime::request_exit` の呼び出しは作らない。core の `AppClient.embed_closing` は `Option<CloseSignal>` にし、event ごとの中断を保つ（設計書 19 章「embed の閉じる要求は exit の turn にしない」）
- **unit**: C-1、E-2（core の `Option<CloseSignal>` への型の差し替え）

### SM-C04 `Embedded.host` の drop の順
- **場所**: `embed.rs:108-117,154-166`（`host: Option<EmbeddedHost>`、drop の順のためだけの `Option`）
- **解消後**: field の宣言の順で drop の順を表し、`Option` を消す（順序を doc comment に書く）
- **unit**: C-1

### SM-C05 `HostLink`
- **場所**: `client/link.rs:26-33,46-50,72-82`（`now_ns: Cell<u64>`、`waker: OnceCell<HostWaker>`、`in_turn: Cell<bool>`）
- **解消後**: 消える。wake は `kivact_runtime::Host` の実装（core の `ClientHost(HostWaker)`）、時刻は runtime の時計一つ（`RuntimeConfig.start_ns` で始まり `Runtime::advance_clock` で進む。設計書 19 章「host との境界と構築」）
- **unit**: R-2

### SM-C06 turn の中の段階の順
- **場所**: `client/phases.rs:9-21,32-43,46-59,64-88`、`app_client.rs:188-219,271-360`、`client/settle.rs:13-47`
- **解消後**: core の `AppClient::turn` の一つの関数が上から下への並びで順序を持ち、runtime は `TurnGuard` の部品（`stage`、`stabilize`、`deliver_reports`、`run_mounts`、`settle_async`、`finish` …）を出す。`client/phases.rs`・`client/settle.rs` は消える（設計書 19 章「turn の順序は core の一つの関数が持つ」）
- **unit**: R-2

### SM-C07 削除の dirty flag と asset の失敗の掃除
- **場所**: `view/tree.rs:289,644-646,840`（`removes: bool`、`take_removes`）、`client/phases.rs:69-77`（`asset_failures` の掃除）
- **解消後**: `Tree::remove` が `Removed { ids, records }`（`#[must_use]`。`ids` は消した部分木の全 node の前順）を返し、commit の段階で `ids` を `asset_failures` から消す。flag は消える（設計書 19 章「削除の結果の型」）
- **unit**: C-3a

### SM-C08 `ViewState` の後付け
- **場所**: `view/state.rs:223-238,261-272`（`started: OnceCell<Started>`、読む前に始まっていないと panic）
- **解消後**: `ViewState` を `Launch::Started` の構築時に完成した形で作る（`Started` の中身を `ViewState` の field にする）
- **unit**: C-1

### SM-C09 入力の modality
- **場所**: `view/state.rs:233`（`pointer_modality: Cell<bool>`）、`client/dispatch/common.rs:13-24`
- **解消後**: `enum Modality { Pointer, Keyboard }`
- **unit**: C-3a

### SM-C10 focus の要求
- **場所**: `view/state.rs:152-199`（`queued_focus: Vec`、`sent_focus: HashMap<id, FocusWaiter>`、`FocusWaiter::{Report, Restore { report: Option }}`、`effects: HashMap<id, EffectWaiter>`、`pending_activations`、`next_request`）、`view/requests.rs:149-190`
- **現在**: 要求の段階を collection への所属で表し、queue の restore は flush のときに黙って消える
- **解消後**:
  ```rust
  struct FocusRequests { requests: HashMap<RequestId, FocusRequest>, order: Vec<RequestId> }
  struct FocusRequest { target: FocusTarget, reply: FocusReply, stage: RequestStage, visible: bool }
  enum RequestStage { Queued, Sent }
  struct FocusCompletion { owner: Owner, at: &'static Location<'static> } // private
  enum FocusReply { Report(FocusCompletion), Restore, RestoreAndReport(FocusCompletion) }
  ```
  `RequestId` の表は `HashMap`。flush の restore 取消条件は既存のまま `Queued → 消える` とする。取消は内部の `Canceled` で、既存どおり callback・UiError を発生させない。host の拒否だけを保存した owner/location に報告する。visible は独立入力、FocusCompletion は runtime の future 用 Completion とは別。
- **unit**: C-3b

### SM-C11 先に届いた文字の編集
- **場所**: `view/tree.rs:287,597-643`（`future_text_edits: IdMap<usize>`）、`view/text_edits.rs:13-39`（guard）
- **現在**: 「N 件が engine で先に適用済み」の counter。「一度に一 batch」は `debug_assert`
- **解消後**: `TextEditsInFlight` は単一の共有 counter store（`Rc<RefCell<IdMap<u32>>>`）、`TextEditsAhead` は独立した所有 guard（begin/drop）。dispatch 中に Tree の可変借用を保持せず、store の二重 begin を拒否する。
- **unit**: C-3a

### SM-C12 node の退場
- **場所**: `view/tree.rs:248-249,839-857`（`TNode.exit_layout`、`exit_complete: Option<Callback<()>>`、`exit_callbacks`）、`client/dispatch.rs:85-90`
- **解消後**: `leaving: IdMap<Leaving>`、`struct Leaving { on_exit: Option<Handler<()>> }`（設計書 19 章「削除の結果の型」）。node の記録を消すときに `Leaving` へ移す（同時に二つに入らない）
- **unit**: C-3a

### SM-C13 遅延で作る revision の trigger
- **場所**: `view/tree.rs:291-292,373-391`（`visibility_revision`、`live_region_revision: Option<Trigger>`）
- **解消後**: `Tree::new` で作る
- **unit**: C-3a

### SM-C14 `NodeRef`
- **場所**: `view/state.rs:61-70`（`bound: Signal<Option<Bound>>`、`observes_layout: Cell<bool>`）、`view/node_ref.rs:136-178,465-471`
- **現在**: node が消えても owner の cleanup まで `Some` のまま（stale）。`rect()` と bind の順の依存
- **解消後**:
  ```rust
  enum RefState { Unbound, Bound(Bound), Detached }
  enum LayoutObservation { NotRequested, Requested }
  ```
  `Tree::remove` の `Removed.ids`（SM-C07）で、同じ turn の中で `Bound → Detached` にする（設計書 19 章「削除の結果の型」）。`LayoutObservation::Requested` は bind 済みならその場で engine に伝え、未 bind なら bind のときに伝える（順序に依らない）
- **unit**: C-3a

### SM-C15 `Part` の入力の状態
- **場所**: `view/part.rs:111-133,141-240`（`pointer_pressed`、`key_pressed`、`hovered`、`disabled`、`hidden` の 5 つの `Signal<bool>`、`previously_focused: Cell<bool>`、二つの hover の情報源 `:145,160` と `:222-223,228`）
- **不正**: disabled なのに pressed（memo が隠し、effect が後で消す）
- **解消後**:
  ```rust
  struct PartInput { press: Press, hover: Hover }   // Signal<PartInput> 一つ
  enum Press { Idle, Pointer(PointerId), Key(Key) }
  enum Hover { Outside, Inside }
  ```
  disabled と hidden は状態ではなく入力（`Value<bool>`）。disabled か hidden になる遷移が `Press::Idle` と `Hover::Outside` にする。focus の in/out は engine の event で受け、edge の検出を消す。hover は engine の `HoverChanged { target: Option<FocusTarget> }` だけで、canvas の region の part は `target == FocusTarget { node, region: Some(region) }` のとき `Inside`（pointer の Enter/Leave の observer を消す。設計書 19 章「part の hover は engine の hover の target から」）
- **unit**: C-4

### SM-C16 canvas の region の登録
- **場所**: `view/common.rs:170-217`（`Entry.adopted: RefCell<Option<Rc<Adopted>>>`、`Adopted.bound: Cell<bool>`、`retired: Vec<Owner>`、`duplicates`、`duplicate_keys`、`next` の世代、`revision` と `changed: Signal<u64>` の counter）、`view/mount/regions.rs:33,73,118-121,143,167-170,214`
- **解消後**:
  ```rust
  enum RegionEntry { Pending, Adopted(Adopted), Shadowed { by: RegionKey } }
  struct Adopted { owner: Owner, generation: Generation, binding: RegionBinding }
  enum RegionBinding { Unbound, Bound }
  ```
  変化の通知は `Trigger`。重複の報告は `ReportOnceSet`。`RegistryPhase` は既存のまま
- **unit**: C-4

### SM-C17 part の pointer の capture
- **場所**: `view/common.rs:225-253`（`Capture { target: Option<Rc<Target>>, buttons: Vec<MouseButton>, fallback: Option<MouseButton>, last }`、`Gestures.last: Option<PointerEvent>`）、`view/mount/part_pointer.rs:88-342`、`client/dispatch/pointer.rs:178-184`（`take_adopted_activation`）
- **現在**: engine にも同じ capture がある（`adopt_pointer`/`forget_adopted_pointer`）
- **解消後**:
  ```rust
  enum PartCapture { None, Captured { target: CaptureTarget, buttons: PointerButtons } }
  enum CaptureTarget { Part(Rc<Target>), Region(RegionId), Fallback { button: MouseButton, target: Rc<Target> } }
  ```
  node 単位の capture は engine だけが持つ。engine は activation を `Routed<PointerSample>` の field として渡し、`take_adopted_activation`（`Reader` を通した変更の呼び出し）を消す（設計書 7.5 節）
- **unit**: C-4

### SM-C18 一度だけの報告の guard
- **場所**: `view/mount/style.rs:109-113`、`control/text_binding.rs:27,40`、`control/node.rs:394,399`、`view/mount/draw.rs:49,88`、`view/flow/virtual_list.rs:307,399`（`Cell<bool>`）、`view/state.rs:227,318-320`（`noted_missing`）
- **解消後**: `ReportOnce`、`ReportOnceSet<K>`
- **unit**: C-3a

### SM-C19 `dynamic` の分岐
- **場所**: `view/flow/branch.rs:332`（`Option<(T, Branch)>`）
- **解消後**: `enum DynamicArm<T> { Pending, Mounted { value: T, branch: Branch } }`（`show` の `ShowArm` と同じ形）
- **unit**: C-3a

### SM-C20 `virtual_list` の保持
- **場所**: `view/flow/virtual_list.rs:196-203,281-295`（`VState.held: Option<K>` と `held_version` の trigger）
- **解消後**: `enum Held<K> { Nothing, Holding { key: K } }` と、その変化を通知する `Trigger` を一つの struct にまとめ、遷移メソッドだけが両方を更新する
- **unit**: C-3a

### SM-C21 sync hook の提供元
- **場所**: `view/sync_hook.rs:35`（`Rc<RefCell<Option<Rc<Hooked>>>>`、cleanup の後は `None`）
- **解消後**: `enum HookSlot { Live(Rc<Hooked>), Disposed }`
- **unit**: C-3a

### SM-C22 overlay の modal の復元
- **場所**: `view/overlays.rs:476-498`（mount で捕まえ、`on_cleanup` の closure で戻す）
- **解消後**: `struct ModalScope { .. }` を owner に `Owned` として置き、`Drop` で戻す（RAII の型が状態を表す）
- **unit**: C-3a

### SM-C23 lifecycle の購読と Weak の表
- **場所**: `platform/lifecycle.rs:15-49`（`LifecycleState { next: Cell<u64>, registrations: BTreeMap }`）。同じ形が `kivact-motion/src/state.rs:19-20,119-138` と `kivact-task/src/state.rs:69` にある
- **解消後**: `Registry<T>`（共通の部品）。登録の id は `Generation`
- **unit**: C-3b（motion の分は R-4、task の分は R-5 の SM-R21）

### SM-C24 初回の描画
- **場所**: `platform/env.rs:37-43`（`is_initial_render()` を `engine().stats().revision == 0` から推定）
- **解消後**: runtime の `Turn::{Setup, Number(n)}`（既存）から `is_initial_render() = matches!(turn, Turn::Setup)` を読む
- **unit**: C-3a

### SM-C25 host の統計の counter
- **場所**: `client/state.rs:39-41`（`turns`、`stale_events`）
- **解消後**: `struct HostStats { turns: u64, stale_events: u64 }`（統計の counter、S-9）。状態の判定に使わない
- **unit**: C-3a

### SM-C26 共有画像の発行
- **場所**: `gpu_interop/share/app_side.rs:91-102`（`issued: RefCell<(mode, size)>` と、違えば作り直す effect）
- **解消後**: `enum Issue { NotIssued, Issued { mode: ShareMode, size: Size, generation: Generation } }`。作り直しは import が `Lost` になる遷移か、`mode`・`size` が変わる遷移の中で行う
- **unit**: C-3b

### SM-C27 共有の link の画素
- **場所**: `gpu_interop/share/link.rs:110-145`（`Phase` の enum の外の `Open.pixels: Option<Pixels>`）
- **解消後**: `Open.pixels` は `Mailbox<Pixels>`（S-9、V-4 の latest-wins/take-once）として `Phase` と独立して保持する。`publish_pixels` は GPU の request 中にも CPU picture を受け取る現在の契約を保存し、`Phase::Pixels` の variant に閉じ込めない（P-1 で code/doc を照合して主担当が訂正、DECISIONS）
- **unit**: C-3b

### SM-C28 embed の panic
- **場所**: `embed.rs:120-152`（`PanicSlot` を毎回の呼び出しで resume）
- **解消後**: `enum EmbeddedState { Running, Panicked(PanicPayload), Closed }`。`Panicked` の後の呼び出しは resume して以後 `Closed`
- **unit**: C-1

---

## SM-I: `kivact-interaction`

全項目で、状態は `Machine` の `State`、遷移は `transition`、timer は `MachineCx::after`、option は behavior ごとの `XOptions` を入れる `OptionsCell<XOptions>`（crate private。unit I-0 の形で、設計書 11.1 節の `Settings<S>` を読み替える）にする。以下は各 behavior の `State` の決定。

### SM-I01 disclosure
- **場所**: `disclosure.rs:51-65,97-102,170-175`（`Signal<DisclosureSettings { disabled: Option<Value<bool>>, leaving: Option<Value<bool>> }>`）
- **解消後**: open は caller の `Binding<bool>`（状態ではなく入力）。閉じたら内容の node を unmount し、退場の動きは engine の presence（設計書 17.10 節）が持つ。部品が退場の間の見た目を変えるときは `NodeRef::presence() -> Value<Presence>` を読む。`DisclosureSettings.leaving`（外からの leaving の入力）と `use_content_reveal` を消す。`OptionsCell<DisclosureOptions>`
- **unit**: I-1（option）、I-7（presence）

### SM-I02 popup
- **場所**: `popup.rs:106-131`（`opening: Var<Option<Opening { opener, policy, closed_by: Option<PopupCloseReason>, returned: bool }>>`）、`:203-279,365-427`（`entered`、`anchor_seen`、`modal: Var<bool>`）
- **不正**: `returned = true` で `closed_by = None`。`Some(Opening)` なのに shown が false
- **解消後**:
  ```rust
  enum PopupState {
      Closed,
      Open { opener: Option<NodeToken>, policy: FocusPolicy, focus: FocusRange },
      Closing { reason: PopupCloseReason, opener: Option<NodeToken> },
      ReturningFocus { opener: NodeToken },
  }
  enum FocusRange { NotEntered, Entered, AnchorSeen, AnchorRemoved }
  ```
  `modal` は option（`OptionsCell`）。`watch` の「既に `Some` なら raise」は `Open` での `Open` event を misuse にする
- **unit**: I-1

### SM-I03 delayed
- **場所**: `delayed.rs:40-46,168-220`（`State::{Hidden, Opening(Task), Shown, Closing(Task), Dismissed}`、`spawn(sleep)`）
- **解消後**: 同じ状態から task の handle を除き、`MachineCx::after(DelayedTimer::Open|Close, ..)`（`type Timer = DelayedTimer`）にする
- **unit**: I-1

### SM-I04 tooltip の hub
- **場所**: `tooltip.rs:73-117`（`wanting: Signal<Vec<u64>>`、`hidden: Signal<Option<(id, at_ns)>>`、`next: Signal<u64>`、`warm_for`）、`:202-226`（`TooltipConfig` を文脈で渡す）
- **不正**: skip-delay の窓が時刻の比較だけで、名前がない
- **解消後**:
  ```rust
  struct TooltipHub { stack: Vec<TooltipId>, warmth: Warmth }
  enum Warmth { Cold, Warm { until_ns: u64 } }
  ```
  id は `Generation`。時間は `InteractionPolicy.tooltip: TooltipTiming`（旧 `TooltipConfig`。文脈で渡す経路は消す。設計書 19 章「`InteractionPolicy` の形と置き場所」）。tooltip は doc に「popup」と書きながら `Popup` を使っていない（`tooltip.rs:3`、`:202-207`）ので、doc を事実（`Delayed` と `PopupState` で作る）に直す
- **unit**: I-1

### SM-I05 dialog
- **場所**: `dialog.rs:152-170`（`settings: Var<DialogSettings>`、`anchored: Var<bool>`）
- **解消後**: `anchored` は `DialogKind::Popover { anchor }` の variant に入れる（構造の option）。状態は `PopupState`
- **unit**: I-1

### SM-I06 combobox
- **場所**: `combobox.rs:93-109`（`popup: Signal<Option<Binding<bool>>>`、None = 常に表示）、`:156-178`（`active` を二つの effect で有効に保つ）、`:215`（`popup_with_focus_transfer`）、`:234-237`
- **解消後**: `ComboboxMode::{Inline, Popup(Binding<bool>)}`（構造の option）。`State { query: String, highlight: Option<K> }` で、表示する highlight は「rows にある key だけ」を memo で派生させる（effect で書き戻さない）
- **unit**: I-2a

### SM-I07 select
- **場所**: `select.rs:110-166`（二つの `List`、共有の `typed: Var<Typeahead>`、`hovered: Signal<Option<K>>` を閉じたら effect で戻す、`arrow_edge`、`arrow_open`・`retain_trigger_focus` の bool、`preview: Var<Option<Callback>>`）、`:583-605`（`reported: Var<Option<K>>`）
- **解消後**:
  ```rust
  enum SelectState<K> {
      Closed { typeahead: Typeahead },
      Open { highlight: Option<K>, hovered: Option<K>, opened_by: OpenedBy, typeahead: Typeahead },
  }
  enum OpenedBy { Pointer, Key(Key), Typeahead }
  ```
  `arrow_open`・`retain_trigger_focus` は option（`OptionsCell`）。preview の重複除去は `Sent<Option<K>>`
- **unit**: I-2a

### SM-I08 menu の session
- **場所**: `menu/state.rs:594-637`（`opened: Signal<Option<Opened>>`、`path: Signal<Vec<K>>`、`focus_key: Signal<Option<(depth, K)>>`、`want: Signal<Option<Want<K>>>`、`surface_rects`、`typed`）、`menu/session.rs:15-106`（三つの effect: 閉じたら reset、開いていない level への要求を捨てる、level の切り詰め）
- **解消後**:
  ```rust
  enum MenuSession<K> { Closed, Open { opened_by: OpenedBy, levels: Vec<Level<K>>, want: Option<Want<K>>, typeahead: Typeahead } }
  struct Level<K> { parent: Option<K>, highlight: Option<K>, submenu: Submenu<K> }
  enum Submenu<K> { None, Opening { key: K }, Open { key: K }, Closing { key: K } }
  ```
  三つの effect は遷移関数の中の規則になる。submenu の開閉の時間は `Step::after` と `InteractionPolicy.menu.submenu`（設計書 19 章「`InteractionPolicy` の形と置き場所」）
- **unit**: I-2b

### SM-I09 menu の押したままの gesture
- **場所**: `menu/press_state.rs:26-52`（`gesture: Var<Option<Gesture { phase: Phase::{Pending { timer: Task, from }, Open { at, entered: bool }}, .. }>>`、`highlight: Signal<Option<Option<NodeRef>>>` の三値）、`menu/press.rs:15-140`（`spawn(sleep(HOLD))`、10.0 / 13.0 px の literal）
- **解消後**:
  ```rust
  enum PressGesture { Idle, Pending { from: Point, source: PointerId }, Holding { at: Point }, Dragging { at: Point } }
  enum Highlight { Unset, Nothing, Item(NodeRef) }
  ```
  hold の timer は `after`、drift の閾値は `InteractionPolicy.menu.drift`（`PressDrift { context: 10.0, other: 13.0 }`、今と同じ値。設計書 19 章「`InteractionPolicy` の形と置き場所」）
- **unit**: I-2b

### SM-I10 menu の level の submenu の時間
- **場所**: `menu/level.rs:22-48,74-75,107,337,360`（theme の拡張 `SubmenuTiming`、`DisabledMenuFocus` を読み、`use_delayed` で開閉）
- **解消後**: SM-I08 の `Submenu` の遷移。theme の拡張の型は `InteractionPolicy.menu` の field（`submenu: SubmenuTiming`、`disabled_focus: DisabledMenuFocus`。設計書 19 章「`InteractionPolicy` の形と置き場所」）
- **unit**: I-2b

### SM-I11 編集の session（`EditPhase`）
- **場所**: `edit.rs:76-83,130-160,231-246`（`initial: Signal<Option<T>>`、Some = 編集中）
- **解消後**: `enum EditSessionState<T> { Idle, Editing { initial: T, origin: EditOrigin } }`。`EditPhase` 自体は `kivact::view`（設計書 10.2 節）
- **unit**: I-3

### SM-I12 drag
- **場所**: `drag.rs:108-143`（`DragSession::{Idle, Active, Releasing { owns_edit }}`、`Stop`）、`:181-187`（`settings` の `edit: Option<(BeginAt, Rc<dyn DragEdit>)>`）、`:252-260`（`disabled: Signal<Option<Value<bool>>>`）、`sources: Var<u64>`
- **解消後**: `DragSession::{Idle, Pending { origin: Point, pointer: PointerId }, Active(Active), Releasing { edit: EditOwnership }}`。`Pending` は slop を越えるまで（設計書 11.1 節）。`EditOwnership::{Owned, Borrowed}`。`sources` の id は `Generation`。`DragApi` に `slop`・`axis_lock`・`track_velocity` を足し、`DragUpdate.velocity: Point`（`track_velocity` がなければ `Point::ZERO`）を足す（設計書 19 章「`DragApi` の追加と touch だけの gesture」）
- **unit**: I-3

### SM-I13 文字編集の control
- **場所**: `text_edit/handle.rs:257-259`（`was_composing: Cell<bool>` の edge の検出）、`:291-331`（`mounted: Cell<Option<u64>>`、`next_control`、7 つの `Signal<Option<Value<..>>>`、`submit`・`composition` の `RefCell<Option<Rc<dyn Fn>>>`）
- **解消後**:
  ```rust
  enum ControlState { Unmounted, Mounted { id: ControlId, composition: Composition } }
  enum Composition { Idle, Composing }
  ```
  composition の変化は IME の event で遷移する（edge の検出を消す）。二つ目の同時の mount は `Mounted` での `Mount` event を報告する。7 つの settings と callback は `OptionsCell<TextEditOptions>`
- **unit**: I-3

### SM-I14 list
- **場所**: `list/state.rs:21-64`（`retained_focus: Cell<bool>`、`page: Cell<Value<usize>>`、`typed: Cell<Var<Typeahead>>`、`leave`・`scroll` の `RefCell<Option<..>>`、`pending: RefCell<Option<Pending { key, origin, origin_ref }>>`）、`:95,109`（`active` の memo の中で `position.set`）、`list/handle.rs:193`、`list/parts.rs:84`
- **不正**: builder の `retained_focus` を item の作成のときに読むので、呼び出し順に依存する。memo の中の副作用
- **解消後**: `retain_focus()` は構造の option（設計書 10.6 節）で、part の bind（mount）のときに読む。`position` は別の memo。`enum ListPending<K> { Nothing, Activating { key: K, origin: Origin } }`。`OptionsCell<ListOptions>`
- **unit**: I-4a

### SM-I15 typeahead
- **場所**: `list/typeahead.rs:9-58`（`{ text, at_ns }`、500 ms の窓）
- **解消後**: `enum Typeahead { Idle, Typing { text: String, expires_ns: u64 } }`
- **unit**: I-0

### SM-I16 roving
- **場所**: `roving.rs:61-73`（`members: Rc<RefCell<HashMap>>`、`changed: Signal<()>`、`requested: Signal<Option<Id>>`、`observed`・`restore_from: Rc<Cell<Option<NodeToken>>>`）
- **解消後**: `changed` は `Notifier`（公開）。`enum RovingFocus { Unset, Requested(Id), Observed(NodeToken) }` と、復元の `enum Restore { Nothing, From(NodeToken) }`
- **unit**: I-4a

### SM-I17 scalar
- **場所**: `scalar/edit_state.rs:16-69`（`touch: Var<Option<Touch>>`、`disabled: Signal<Option<Value<bool>>>`、`key_step: Var<Option<f64>>`、`thumb_focus: Signal<Vec<Value<bool>>>`、focus を失ったら drag を止める effect）
- **解消後**: touch は SM-I18。`OptionsCell<ScalarOptions>`。「どの thumb にも focus がない」は thumb の blur の event で遷移する
- **unit**: I-5a

### SM-I18 touch の slider
- **場所**: `scalar/touch.rs:17,25-55,143-226`（`contact: Var<Option<Contact { anchor: Option<f32>, moved: bool, .. }>>`、`lifted: Signal<Option<usize>>`、`coast: Signal<Option<Coast>>`、`clock: FrameLoop`）
- **不正**: contact と coast が両方 `Some`。coast が `Some` なのに FrameLoop が止まっている
- **解消後**: `use_drag` の上に作り直さず、今の raw pointer の handler を machine の event にする（touch だけ・primary だけ・`suppress_activation` の順は今のまま。設計書 19 章「`DragApi` の追加と touch だけの gesture」）。`enum TouchSlider { Idle, Pressed { contact }, Tracking { contact, anchor: f32 }, Coasting { coast: Coast } }`。slop の判定は `DragAxis::past_slop`、速度は `DragVelocity`。coast の frame は `Coasting` の間だけ動く
- **unit**: I-5a

### SM-I19 zoom・pan
- **場所**: `zoom_pan.rs:106,124-156`（`Gesture::{Idle, Pan, Pinch}` と別の `touches: Var<Vec<(PointerId, Point)>>`、`EditSession`、`Var<Settings>`）
- **不正**: 「編集中」と「`Gesture != Idle`」が同じ事実を二つで持つ
- **解消後**: `Gesture::Pan`・`Pinch` の variant が touch の点と edit の session を持つ。`Idle` に戻る遷移が edit を commit する
- **unit**: I-5b

### SM-I20 space での pan
- **場所**: `space_pan.rs:22-24`（`held: Signal<bool>`）
- **解消後**: `enum SpaceKey { Up, Held }`
- **unit**: I-5b

### SM-I21 hover の通り道
- **場所**: `hover_corridor.rs:71-122`（`path: Rc<Cell<Option<(origin, previous)>>>`、`timer: Rc<Cell<Option<Task>>>`、`holding: Signal<bool>`、`progress: Signal<u64>` の counter）
- **不正**: `holding = true` で `timer = None`
- **解消後**: `enum Corridor { Idle, Tracking { origin: Point, previous: Point }, Holding { origin: Point } }`、timer は `after`、`progress` は `Notifier`
- **unit**: I-6

### SM-I22 toaster
- **場所**: `toaster.rs:52-63`（`QueuedToast.focus: Rc<Cell<Option<ToastFocus>>>`）、`:285-329`（`queue`、`next: Signal<u64>`、`limit`、`holds: Signal<usize>` の counter）、`:582-606`（toast ごとの `engaged: Signal<bool>`、effect の cleanup と `left: Rc<Cell<u64>>` による一時停止）
- **解消後**:
  ```rust
  enum ToastTimer { Running { deadline_ns: u64 }, Paused { left_ns: u64 }, Expired }
  struct HoldSet(BTreeSet<HoldId>);      // 「何件の hold」ではなく「どの hold」
  enum ToastFocus { None, Inside(NodeToken) }
  ```
  toast の id は `Generation`。toast ごとの期限は toast ごとの machine が `Step::after` で持つ（設計書 18.13 節）
- **unit**: I-6

### SM-I23 末尾への追従
- **場所**: `follow_end.rs:92-99`（`FollowState { locked: bool, geometry: Option, offset, expected: Option<f32>, revision: Option<u64>, history_pending: bool }`）
- **解消後**: `enum Follow { Following { expected: Option<f32> }, Detached { offset: f32 }, AwaitingHistory { anchor: f32, revision: Revision } }`。field の組合せから variant を一つずつ作り、表せない組合せをなくす
- **unit**: I-6

### SM-I24 toggle
- **場所**: `toggle.rs:75-96`（`Kind::Checkbox { indeterminate: Signal<Option<Value<bool>>> }`、`Core.disabled: Signal<Option<Value<bool>>>`）
- **解消後**: `OptionsCell<ToggleOptions>`（この unit）。`on_edit` は N-G3 が足す（設計書 11.2 節）
- **unit**: I-3

### SM-I25 announcer
- **場所**: `announcer.rs:9-42`（`Notice { revision: u64, message }`、counter で再読み上げ）
- **解消後**: `Notice { id: NoticeId, message }`、id は `Generation`（識別。S-8 の例外）
- **unit**: I-6

### SM-I26 生存の token
- **場所**: `list/state.rs:85`、`calendar/state.rs:39,57`、`table/state.rs:25`（`alive: Owned<()>`）
- **解消後**: 既存の `Owner::is_alive()` を使い、`alive: Owned<()>` の token を消す（足す API はない）
- **unit**: I-4a（list）、I-4b（calendar、table）

### SM-I27 table
- **場所**: `table/state.rs:19-41`（17 field、個別の `Signal<Option<Value<..>>>` の settings）
- **解消後**: `OptionsCell<TableOptions>`。列の幅の変更は `use_splitter` の状態、並べ替えは `enum SortState { Unsorted, Sorted(Vec<(ColumnKey, SortDirection)>) }`
- **unit**: I-4b

### SM-I28 option の保存の形の不揃い
- **場所**: `Signal<Settings>`（`disclosure`、`splitter.rs:122-128`、`tabs.rs:32-35,90`）、`Var<Settings>`（`popup`、`dialog`、`zoom_pan.rs:106,145`、`drag`）、個別の `Signal<Option<Value>>`（`text_edit`、`table`）、`Var<Option<..>>`（scalar の `key_step`）
- **解消後**: 全部、behavior ごとの `XOptions` を入れる `OptionsCell<XOptions>`（crate private。unit I-0 の形で、設計書 11.1 節の `Settings<S>` を読み替える）
- **unit**: I-0（`OptionsCell<S>` を作り、各 behavior の unit I-1〜I-6 がそれに移す）

---

## SM-U: `kivact-ui`

### SM-U01 switch の drag
- **場所**: `switch_drag.rs:25-36`（`Contact { started_on, visually_on, armed, flipped, horizontal, vertical, touch: bool ×7, started, origin, id }`）、`:92-123`（`visual: Signal<Option<SwitchDragState>>`、disable で増やす `generation: Signal<u64>`）、`:130`（`pointer`、142 行）
- **解消後**: `use_drag` の上に作り直さず、今の raw pointer の handler を machine の event にする（設計書 19 章「`DragApi` の追加と touch だけの gesture」）。`enum SwitchGesture { Idle, Pressed { origin: Point, started_on: bool }, Dragging { axis: Axis, visually_on: bool } }`（`SwitchDrag` は policy の型、`SwitchDragState` は recipe の状態の型として既にあるので別の名前にする）。slop は `DragAxis::past_slop`（10 px は名前の付いた private の定数）、速度は `DragVelocity`。disable は `Idle` への遷移で、`generation` を消す
- **unit**: U-1

### SM-U02 toast の swipe
- **場所**: `toast_swipe.rs:11`（`SLOP = 8.0`）、`:51-98`（`Contact { promoted: bool, .. }`、`blocked: Var<Option<PointerId>>`、`held: Signal<bool>`、`visual: Signal<Visual::{Rest, Drag, Dismiss}>`、effect で start/stop する `frame_loop`）、`:226`
- **解消後**: `use_drag` の上に作り直さず、今の raw pointer の handler を machine の event にする（slop は `InteractionPolicy.toast.swipe_slop`（8.0）と `DragAxis::past_slop`、速度は `DragVelocity`。設計書 19 章「`DragApi` の追加と touch だけの gesture」）。`enum Swipe { Rest, Pressed { pointer: PointerId, origin: Point }, Dragging { pointer: PointerId, offset: f32 }, Blocked { pointer: PointerId } }`。閉じる動きは、drag の位置からの toast の node の退場（engine の presence、設計書 17.10 節）で描き、`Visual::Dismiss` と `frame_loop` を消す
- **unit**: U-1（gesture の状態）、I-7（閉じる動きを node の退場にする部分）

### SM-U03 select の touch の sheet
- **場所**: `select.rs:226-275`（`contact: Var<Option<(PointerId, Point, bool)>>`、`released: Var<Option<bool>>`、`sheet_open: Signal<bool>`、10 px）
- **解消後**: `use_drag` の上に作り直さず、今の raw pointer の handler を machine の event にする（slop は `InteractionPolicy.select.sheet_slop`（10.0）と `DragAxis::past_slop`。設計書 19 章「`DragApi` の追加と touch だけの gesture」）。`enum SheetGesture { Idle, Pressed { pointer, origin }, Moved { pointer }, Released(Outcome) }`。sheet の開閉は I-2a の `SelectState`（SM-I07）の `Open { surface: Surface::{Popover, Sheet} }`
- **unit**: U-1

### SM-U04 tree view の高さの hysteresis
- **場所**: `tree.rs:163-178`（memo の中で `previous_open: Cell<bool>` を書く、`last_height: Signal<f32>`）
- **解消後**: `RowReveal` は作らない。子の行の node の presence（`enter_size`・`exit_size`、設計書 17.10 節）で描き、memo の中の `previous_open` の書き込みと `last_height` を消す（設計書 19 章「toast の行と tree の行の出現と退場も presence で描く」）
- **unit**: I-7

### SM-U05 inline の編集
- **場所**: `inline_edit.rs:34,~96`（`editing: Binding<Option<K>>`、cleanup で閉じる）、`text_draft.rs:24-40`（field ごとの draft の signal と `EditSession`）
- **解消後**: `enum InlineEdit<K> { Idle, Editing { key: K, draft: String, session: EditSession<String> } }`。caller の `Binding<Option<K>>` は `Editing` の key の写し（`Sent` で書く）
- **unit**: U-2

### SM-U06 disclosure の動き
- **場所**: `kivact-ui/src/disclosure.rs:102,143`（`animated(bool)`、200 ms 固定、`ContentReveal` から leaving を渡す）
- **解消後**: leaving は engine の presence（SM-I01、設計書 17.10 節）。動きの有無と時間は `UiTokens.disclosure.motion`（`DisclosureMotion { animate: bool, duration: Duration }`、既定 `animate: false`、200 ms）。`ui::disclosure(..).animated(bool)` はその部品の `motion.animate` の上書きとして残す（設計書 19 章「`DisclosureMotion` の既定と `animated`」）。高さの動きは recipe の `exit_size(Orientation::Vertical)`・`enter_size(Orientation::Vertical)`
- **unit**: I-7

---

## SM-E: `kivact-engine`・`kivact-lower`・`kivact-layout`・`kivact-editing`

### SM-E01 host の状態
- **場所**: `kivact-engine/src/host/pump.rs:19-81`（`Phase`、`suspended: bool`、`embedded_closing`、`last_frame`、`next_wake`、`soft_keyboard: bool`、`accessibility_noted: bool`、`reduced_motion_override`、`environment`、`adapter`）、`host/events.rs:365-376`（`drawable` の三つの情報源）
- **解消後**: `Host` は `phase: HostPhase { Starting, Running, Exiting, Closed }`、`presence: AppPresence { Foreground, Suspended }`、`surface: Surface { Drawable { generation }, Unavailable(Unavailable { NoSurface, ZeroSize }) }`、`last_present: LastPresent` を並べて持ち、`drawable()` はこれらから計算する唯一の関数（設計書 19 章「embed の閉じる要求は exit の turn にしない」）。`environment`・`adapter` は `Sent<T>`、`accessibility_noted` は `ReportOnce`、`reduced_motion_override: Option<bool>` は設定の値。`soft_keyboard` は SM-E22（E-4）だけが扱う
- **unit**: E-2（`soft_keyboard` は E-4）

### SM-E02 embed の閉じる flag の問い合わせ
- **場所**: `pump.rs`（`self.closing()` 10 か所）、`events.rs`（4 か所）、`kivact-engine/src/embed.rs:33-43`
- **解消後**: `CloseSignal(Arc<AtomicBool>)`（`kivact_engine::contract`）。`Host::observe_close()` を host の入口・client の呼び出しの直後・効果を一つ処理した直後に呼び、立っていれば `HostPhase::Closed` へ進んで turn も効果も行わない（exit の turn は走らせない。設計書 19 章「embed の閉じる要求は exit の turn にしない」）
- **unit**: E-2

### SM-E03 surface の喪失の往復
- **場所**: `events.rs:182-195`（`set_surface(false)` → `HostFailure` → `set_surface(true)` → `invalidate_paint`）
- **解消後**: `Presented::SurfaceLost` は状態を持たず、`HostFailure` を一度積んで paint を無効にするだけ（`set_surface(false)` → `(true)` の往復を消す。`Unavailable::Lost` は作らない。設計書 19 章「embed の閉じる要求は exit の turn にしない」）
- **unit**: E-2

### SM-E04 model の frame の時計
- **場所**: `model/frame/state.rs:16-37`（`Clock::{Running, Stopped { since }}` と `env.surface_active` の重複）、`model/frame/ticks.rs:82-86`（`debug_assert_eq!`）
- **解消後**: `Clock` は `HostPhase` から派生する（host が turn ごとに `drawable` を渡す）。`env.surface_active` の field は消す（設計書 5.5 節）
- **unit**: E-2

### SM-E05 frame の要求の列
- **場所**: `model/frame/state.rs:32`（`frames: Vec<u64>`、重複を除かない）、`ticks.rs:32,102,125,148`
- **解消後**: `struct FrameRequests(BTreeSet<u64>)`
- **unit**: E-2

### SM-E06 smooth scroll の開始
- **場所**: `model/frame/state.rs:8-12`（`SmoothScroll.began: Option<u64>`、None = 予定済みで未刻印）、`ticks.rs:131`
- **解消後**: `enum SmoothScroll { Scheduled { target }, Running { target, began_ns: u64 } }`
- **unit**: E-2

### SM-E07 model の最後の frame
- **場所**: `model/frame/state.rs:34`（`last_frame: Option<u64>`、None = 一度も tick していない。`Host.last_frame` とは別物で名前が同じ）
- **解消後**: `enum TickHistory { NeverTicked, Ticked { last_ns: u64 } }`。`Host.last_frame` は `LastPresent` と改名（unit E-2）
- **unit**: E-2

### SM-E08 再描画の要求
- **場所**: `paint/state.rs:9-11,30,45`（`needs_redraw: bool`、初期値 true）、`events.rs:166-178`、`pump.rs:392-394,412-414,437-440`
- **解消後**: `enum Redraw { Clean, Needed }`。`paint_due()` は `Redraw::Needed && drawable()`
- **unit**: E-7

### SM-E09 paint list の状態
- **場所**: `paint/state.rs:160-180`（`stale: bool`、`built_for: Option<(Size, f32)>`、`pass`、`ink_gen`）、`paint/list/walk.rs:260,299-300`、`invalidation.rs:58,82`
- **解消後**: `enum ListState { Stale, Built { size: Size, scale: f64 } }`（初期値 `Stale`）。`pass` と `ink_gen` は `Generation`（unit E-7 の形）
- **unit**: E-7

### SM-E10 pointer の経路
- **場所**: `model/pointer/state.rs:114-122`（`Route { capture: Option<NodeId>, press: Option<Press>, touch: Option<TouchCandidate> }`）
- **解消後**: `struct Route { capture: Option<NodeId>, press: Option<Press> }` は struct のまま（capture と press は独立の事実で、四つの組合せが全部ある）。touch の候補だけを `TouchGesture::Pending { route, candidate }`（SM-E11）へ移す（設計書 19 章「pointer の経路と modal の受け入れ」）
- **unit**: E-3

### SM-E11 track ごとの gesture の制約
- **場所**: `model/pointer/state.rs:93-104`（`Gesture::{Route, Scrollbar, Pan}`、doc で「Scrollbar は mouse だけ」「Pan は touch だけ」）
- **解消後**:
  ```rust
  enum Track { Mouse(MouseTrack), Touch(TouchTrack), Pen(PenTrack) }
  struct MouseTrack { source: PointerSource, at: Option<Point>, over: Option<NodeId>, hover: Hover, gesture: MouseGesture }
  struct TouchTrack { source: PointerSource, at: Point, over: Option<NodeId>, hover: Option<Hover>, gesture: TouchGesture }
  struct PenTrack { source: PointerSource, at: Point, over: Option<NodeId>, hover: Option<Hover>, route: Route }
  enum MouseGesture { Route(Route), Scrollbar(ScrollDrag), SelectText { editor: NodeId, route: Route } }
  enum TouchGesture { Route(Route), Pending { route: Route, candidate: TouchCandidate }, Pan(TouchPan) }
  ```
  （設計書 19 章「pointer の経路と modal の受け入れ」）
- **unit**: E-3

### SM-E12 primary でない pointer の hover
- **場所**: `model/pointer/state.rs:82-84`（`Track.hover` は primary でなければ常に空、の不変条件）
- **解消後**: hover は `MouseTrack` と、primary の `TouchTrack`・`PenTrack`（`hover: Option<Hover>` が `Some`）だけが持つ（SM-E11 の型で表す）
- **unit**: E-3

### SM-E13 マウスの位置の重複
- **場所**: `model/pointer/state.rs:62`（`PointerState.mouse: Option<Point>`）と `:81`（`Track.at`）
- **解消後**: `PointerState.mouse` を消し、`MouseTrack.at` を読む
- **unit**: E-3

### SM-E14 文字の選択の drag
- **場所**: `model/pointer/machine.rs:459-478`（「focus のある editor ∧ primary を押している ∧ 押下の chain が focus を含む」から推定）、`:444-447`（button のない move で `Cancel` を合成）
- **解消後**: `MouseGesture::SelectText { editor, route }`（SM-E11）。press の時点で今の推定（focus のある editor、primary の押下、押下の chain が focus を含む）が成り立つときに入る（設計書 19 章「pointer の経路と modal の受け入れ」）
- **unit**: E-3

### SM-E15 modal の受け入れの表
- **場所**: `model/modal/state.rs:27-30`（`Admission { drags: RefCell<BTreeMap<u64, NodeId>>, adoptions: RefCell<..> }`）
- **解消後**: 解消しない（`Admission` のまま（dispatch の順の表）。`Admission` は client の dispatch の順で問われる表で、`Track` は入力の時点の状態なので、`Track` へ入れると振る舞いが変わる。設計書 19 章「pointer の経路と modal の受け入れ」）
- **unit**: なし（E-3 で行わない）

### SM-E16 key の離しの宛先
- **場所**: `model/keys.rs:150-161`（`key_up` は今 focus がある node へ）
- **解消後**: `struct KeyDownTargets(HashMap<KeyCode, FocusTarget>)`。`key_up` は `key_down` を受けた target へ送る。記録がない・`KeyCode::Unidentified`・node がもうないときは今と同じく focus のある node へ送り、window の blur で表を空にする（設計書 19 章「pointer の経路と modal の受け入れ」）。**振る舞いの変更**で、E2E `key_up_goes_to_the_node_that_received_key_down` を足す
- **unit**: E-3

### SM-E17 modifier の重複除去
- **場所**: `model/aggregate.rs:27`（`Model.modifiers`）、`host/events.rs:298-304`
- **解消後**: shell の `ModifierState` を正とし、engine は `Sent<Modifiers>` で core への event を重複除去する
- **unit**: E-3

### SM-E18 effect の queue に入る到達しない request
- **場所**: `host/pump.rs:175-177`（`PlatformRequest::RegisterFont` が `unreachable!`）
- **解消後**: `PlatformRequest` から `RegisterFont`・`ReducedMotion` を外し、engine の契約 `HostRequest { Platform(PlatformRequest), RegisterFont(Asset), ReducedMotion(Option<bool>) }`（`Writer::effect`）で core から渡す（表せないようにする。設計書 19 章「OS のサービスは窓の同期のメソッド一つ」）
- **unit**: A-2

### SM-E19 変換中に保留した controlled の値
- **場所**: `kivact-engine/src/measure/editor.rs:16,24,44-58`（`pending_controlled: Option<Rc<str>>`、`controlled_at: u64`）、`model/edit/inputs.rs:52-53,92-99`
- **解消後**: `enum ControlledValue { InSync { at: Revision }, HeldDuringComposition { value: Rc<str> } }`。`TextSession` の composition が終わる遷移で `InSync` に戻す
- **unit**: E-4

### SM-E20 IME に渡した値
- **場所**: `model/edit/state.rs:36-50`（`ImeGiven::{Unset, Nothing, Input { source, start: Option<usize> }, Session(..)}`）、`model/edit/ime.rs:65-71`（`start: None` を黙って無視）
- **解消後**: `Input { source, start: InputStart }`、`enum InputStart { At(usize), NoText }`。`ime_select` は `NoText` を明示の分岐で扱う
- **unit**: E-4

### SM-E21 IME の二つの経路
- **場所**: `model/edit/session.rs:30`（`focused_session`）、`model/edit/ime.rs:11-49,101-141`
- **解消後**: `enum ImeTarget { None, Native(EditorSlot), Session(NodeId) }` を focus が変わったときに一度だけ決め、全部の IME の入口がこれを読む
- **unit**: E-4

### SM-E22 IME の許可と soft keyboard
- **場所**: `host/pump.rs:364-389`（`set_ime_allowed` を毎 pump、`soft_keyboard` の写し）
- **解消後**: `Sent<ImeAllowed>`、`Sent<SoftKeyboard>`（`enum SoftKeyboard { Hidden, Shown }`）
- **unit**: E-4

### SM-E23 focus の重複と reveal
- **場所**: `model/focus/state.rs:9-30`（`Focused { target, visible: bool, reveal_at_commit: bool }`、`reveal: Vec<NodeId>`）、`a11y/state.rs:50`（`A11yState.focus`）
- **解消後**: `Focused { target, visibility: FocusVisibility::{Hidden, Visible}, reveal: Reveal::{No, AtCommit} }`。`A11yState.focus` は消し、publish のときに `Focus` から読む
- **unit**: E-6

### SM-E24 支援技術の変更の重複
- **場所**: `model/changes.rs:63-65`（`Dirty::A11Y_LOCAL_TEXT`、`A11Y_METADATA`）、`:122-125`（`a11y_structure: bool`、`a11y_subtrees`、`a11y_removed`、`a11y_focus: bool`）、`model/interact/sync.rs:154-178`（`Dirt` へ畳む）
- **解消後**: 変更は `A11yState.dirt: Dirt` に直接積む。`Changes` から a11y の field を消す
- **unit**: E-6

### SM-E25 支援技術の公開の状態
- **場所**: `a11y/state.rs:42-58`（`root: Option<NodeId>`、`empty_root: Option<u64>`、`hashes`、`next_semantics_id` が `u64::MAX` から減る）、`:165-166`、`a11y/publish.rs:22-26`
- **不正**: `root == None` が「未受理」と「支援技術が動いていない」の二つを意味する
- **解消後**: `enum A11yPublish { Inactive, AwaitingFirstTree, Published { root: SemanticsId } }`。id の割り当ては `SemanticsIdAllocator`（名前の付いた型）。hash は `Semantics<R>` の `derive(Hash)`（設計書 5.3 節）
- **unit**: E-6

### SM-E26 支援技術の有効の問い合わせ
- **場所**: `host/pump.rs:356`（毎 pump `window.accessibility_active()`）
- **解消後**: backend の `ShellEvent::AccessibilityActivated` / `Deactivated` で `A11yPublish` を遷移させる
- **unit**: E-6

### SM-E27 layout の dirty の状態
- **場所**: `model/changes.rs:112-129`（`Dirty` の `MEASURE`・`LAYOUT_STYLE`・`LAYOUT_CHAIN`・`SCROLLED`・`STUCK`、`needs_layout: bool`、`layout_changed: bool`、`viewport_chain: bool`、`geometry: Geometry`）、`:212-232`（`MEASURE` が dispatch を越えて残る）、`:285-287`（`settled()` が `needs_layout` だけを消す）
- **解消後**: engine の `Changes` の `needs_layout`・`layout_changed`・`viewport_chain` は `LayoutReasons`（bit の集合: `PASS`・`STYLE`・`VIEWPORT`）。`Computed(GeometryChanges)` の状態は作らない（geometry はその場で drain する。設計書 19 章「layout は一つの持ち主、型状態の pass は作らない」）。node ごとの `Dirty::MEASURE` の bit は残す（cache の無効化で、S-9 の cache）
- **unit**: E-5a

### SM-E28 layout の pass の呼び出し順
- **場所**: `kivact-layout/src/lib.rs:11-23`、`pass_state.rs:26-43`（`changed: bool` 初期値 true、`epoch`、`fixed_cursor: Cell<Option<..>>`、`relayout: Vec<NodeId>`）、`columns.rs:30-37`（`ChildCursor(Cell<Option<Counted>>)`、`Record.incomplete: bool`）
- **解消後**: `kivact_layout::Layout` 一つの持ち主（`NodeLayoutColumns` と `LayoutState` は crate の中だけ）。`take_changed` は `compute` の中へ入れ、型状態の pass（`Edit`・`Pending`・`Computed`）は作らない（設計書 19 章「layout は一つの持ち主、型状態の pass は作らない」）。`Record.incomplete` は `enum RecordState { Complete, Incomplete }`
- **unit**: E-5a

### SM-E29 relayout の呼び出しの散らばり
- **場所**: `model/interact/sync.rs:88,91`（`pack_child_orders` を二回）、`:204-218`（`relayout = sync → layout → lint → sync`）、`host/events.rs:141,156,163,233,296`（`layout()` を 5 か所）
- **解消後**: `Model::settle(&mut self) -> Settled` が今の `relayout` の順（sync → layout → lint → sync、`pack_child_orders` は一度）を持つ唯一の関数。入力の時点の幾何が要る所（hit test の前、file drop、wheel、redraw）はそれぞれ `settle()` を呼ぶ（何もなければすぐ返る。設計書 19 章「layout は一つの持ち主、型状態の pass は作らない」）
- **unit**: E-5a

### SM-E30 最後に報告した layout と scroll
- **場所**: `tree/node.rs:28-29`（`sent_layout: Option<(Rect, Size, bool)>`、`sent_scroll`）、`:67`（`Place.stacks: bool`「一度立てたら残る」）
- **解消後**: `Sent<LayoutReport>`、`Sent<ScrollReport>`（`LayoutReport` の bool は `Clipping::{No, Yes}`）。`enum Stacking { Flat, Stacks }`
- **unit**: E-5a

### SM-E31 presence と状態の動き
- **場所**: `tree/node.rs:37,43-49`（`Presence::{Mounted, Leaving}`）、`model/presence/state.rs:11-13`（`Presences { arriving, leaving, notify_exit }`）、`model/state_motion/state.rs:37-60`（`AnimPhase::{Captured, Running}` と `colors_text`・`colors_base_text` の bool）
- **解消後**: `enum NodePresence { Arriving, Mounted, Leaving(LeavingRole) }`、`enum LeavingRole { Root { notify: Notify }, Descendant }`、`enum Notify { Silent, OnComplete }`。`Presences` の集合は遷移関数だけが保つ索引。`colors_text`・`colors_base_text` は `ColorChannels`（bit の集合の型）。state の動きの `AnimPhase` は runtime の `AnimPhase` と名前を分けて `StateAnimPhase` と改名する
- **unit**: E-5b

### SM-E32 編集の状態の重複と内部の可変
- **場所**: `kivact-editing/src/edit_state.rs:47-57,466`（`damage: Cell<Option<Damage>>` を `&self` で取る）、`editor.rs:54,67`（`max_length` を副作用で変える）、`kivact-engine/src/model/aggregate.rs:17` と `model/edit/state.rs:54-57`（`editors[slot].is_some()` と `Edits.inputs: BTreeSet` の二重の索引）、`kivact-engine/src/measure/input.rs:73-87`（`try_borrow_mut` の失敗で黙って `Ok(None)`）
- **解消後**: `take_damage(&mut self)`。`max_length` は `EditState::set_max_length` の明示の呼び出しだけで変える。索引は `editors` だけ。`EditLines` の借用の失敗は不変条件の違反として `expect("invariant: ..")`
- **unit**: E-4

### SM-E33 lowering の latch と asset の store の差し替え
- **場所**: `kivact-lower/src/context.rs:44-57`（`error: RefCell<Option<String>>`、`dropped: Cell<u32>`、`failures: Cell<u32>`、「non-reentrant」）、`kivact-lower/src/assets.rs:48-61`（`revision`、`reported_images`、`reported_families`、`pending: Vec<HostEvent>`）、`kivact-engine/src/host/launch.rs:62-67`、`events.rs:110`（`AssetStore::inline()` で始めて start で本物に替える）
- **解消後**: `kivact-lower` は engine の `lower` module になり、`Context::finish(self) -> LowerReport`（`LowerReport { dropped: u32, failures: Vec<Failure> }`）で一回の lowering の報告を返す（設計書 19 章「`kivact-lower` を engine の module にする」）。`Model` は `enum Launch { Pending(Box<LaunchArgs>), Starting, Running(Box<Host>) }` の `start` で本物の store を受け取って作る（差し替えない。設計書 19 章「embed の閉じる要求は exit の turn にしない」）
- **unit**: E-1（latch）、E-2（store の差し替え）

---

## SM-A: Arvane と `kivact-gpu-share`

### SM-A01 embed の子 window の停止の表現
- **場所**: `arvane-native/src/embed/child.rs:34-65,206-288`（`ChildState::{Running, Stopped { cause, window }}`、`Stop`、`NativeWindow::{Live, Gone}`、`generation`）、`embed/session.rs:13-101`（`HandlerState::{Unstarted, Running, Closed}`、panic の後の `mem::forget`）、`embed/windows/state.rs:16-25`（`Phase::{Opening, Live, Disposed, Leaked}`）、`embed/x11/window.rs:21`（`opening: bool`）、`embed/x11/conn.rs:33`（`exists: AtomicBool`）、`kivact-engine/src/embed.rs:33-43`（`closing`）
- **不正**: 4 つの停止の表現が独立に変わる
- **解消後**: thread をまたぐ `enum ChildStatus { Running, Stopped { cause: Stop, window: NativeWindow } }`（`Shared` の `Mutex` の中）と、UI スレッドだけの `enum ChildLifecycle { Opening(HandlerSlot), Live(HandlerSlot), Disposed, Leaked }`（embed の `Session`。Win32 と X11 が同じ enum で遷移する）の二つ（`Send` でない handler を `Mutex` に入れない。設計書 19 章「embed の子 window の lifecycle は thread ごとに二つ」）。`HandlerSlot::{Unstarted, Running { handler: Box<dyn ShellHandler>, control: ShellControl }}`、panic の後の `mem::forget` は `Leaked` への遷移。X11 の `exists: AtomicBool` は `ConnectionAlive(AtomicBool)`。engine の `closing` は SM-E02
- **unit**: A-4

### SM-A02 Win32 の再入の guard
- **場所**: `embed/windows/state.rs:46-69`（`in_callback: Cell<bool>`、`draining: Cell<bool>`、guard の `Hold`）、`window.rs:341,383,424,486`
- **解消後**: `struct ReentryGuard(Cell<Held>)`・`struct DrainGuard(Cell<Held>)`、`enum Held { Free, Held }`（drain の中で callback が走り二つは重なるので一つの enum にしない。設計書 19 章「embed の子 window の lifecycle は thread ごとに二つ」）。`Hold` は各 guard の `enter(&self) -> Option<Hold<'_>>`
- **unit**: A-4

### SM-A03 Win32 の子 window の付帯の状態
- **場所**: `embed/windows/state.rs:64-85`（`pending_surrogate: Cell<Option<u16>>`、`frame: RefCell<Option<CpuFrame>>`、`uia: RefCell<Option<Adapter>>`、`events: RefCell<VecDeque<ShellEvent>>`）
- **解消後**: `Utf16Decoder`（名前の付いた型）、`enum Uia { Inactive, Active(Adapter) }`、`frame` は `CpuSurface`（native と同じ enum、SM-A07 の前の共有）
- **unit**: A-4

### SM-A04 Win32 の wake の合体と作成の guard
- **場所**: `embed/windows/native.rs:17`（`wake_pending: AtomicBool`）、`window.rs:331-335`、`create.rs:202-230`（`Opening.armed: bool`）
- **解消後**: `WakeCoalescer`（`Mailbox<()>` と同じ意味の名前の付いた型）。`enum OpeningGuard { Armed, Disarmed }`
- **unit**: A-4

### SM-A05 X11 の子 window の状態
- **場所**: `embed/x11/window.rs`（`redraw: Cell<bool>` 初期値 true）、`embed/x11/event.rs:16`（`Input.focused: bool`）
- **解消後**: `enum Redraw { Clean, Needed }`（SM-E08 と同じ名前で別の型でよい: crate が違う。ただし VOC-3 に従い `ChildRedraw` と名付ける）、`enum WindowFocus { Blurred, Focused }`
- **unit**: A-4

### SM-A06 XIM の接続
- **場所**: `embed/x11/ime.rs:50-115`（`App.allowed: bool`、`App.focused: bool`、`Connection.protocol_window`・`server_window: Option<u32>` は handshake の間だけ有効、`Context`、`Flight`、`Session`）
- **解消後**: `enum ImeAllowance { Disallowed, Allowed }`、focus は SM-A05 の `WindowFocus`。`Connection` を `enum Connection { Handshake { protocol_window: u32, server_window: Option<u32> }, Established { .. } }` にする（`Context` の既存の enum と合わせる）
- **unit**: A-4

### SM-A07 winit の runner の重複の状態
- **場所**: `arvane-native/src/runtime.rs:513-531`（`cursor: Option<(f64, f64)>`、`shift: bool`、`:912`）
- **解消後**: 両方を消し、shell の `Mouse` と `ModifierState` を使う（設計書 7.5 節）
- **unit**: A-3

### SM-A08 Windows の subclass との mailbox
- **場所**: `arvane-native/src/native_input/state.rs:33-66`（`reconvert: Cell<Option<(usize, usize)>>`、`pointer: Cell<Option<(f64, f64)>>`、`file_pointer`、`env_changed: Cell<bool>`、`subclass_installed: Cell<bool>`、`text: RefCell<Option<ShellImeText>>`）
- **解消後**: 各 mailbox は `Mailbox<T>` の UI スレッド版 `LocalMailbox<T>`（`Cell<Option<T>>` を包み、`put`/`take` だけ）。`env_changed` は `LocalMailbox<()>`。`enum Subclass { NotInstalled, Installed }`
- **unit**: A-3

### SM-A09 clipboard の遅延の初期化
- **場所**: `arvane-native/src/services.rs:11-23`（`clipboard: Option<arboard::Clipboard>`）
- **解消後**: `enum Clipboard { Unopened, Open(arboard::Clipboard), Failed(String) }`。`Failed` の後は `PlatformResult::Failed` を返し、毎回の再試行をしない
- **unit**: A-2

### SM-A10 wake の多重の意味
- **場所**: `arvane-native/src/runtime.rs:715-749`（`user_event` が file-over、`env_changed`、支援技術の action を流してから `Wake`）、`arvane/src/window.rs:451-474`（`Driver` が全部の `Wake` で GPU の準備を進める）
- **解消後**: `ShellEvent::Wake(WakeReasons)`、`WakeReasons` は `Client`・`Assets`・`A11yActions`・`Environment`・`GpuStep` の bit の集合。`Driver` は `GpuStep` のときだけ GPU を進める
- **unit**: A-3

### SM-A11 GPU の presenter
- **場所**: `arvane/src/presenter/gpu.rs:56,63`（`RESTARTS`、`SURFACE_FAILURES`）、`:218-236`（`Step::Configuring { worker, init, stale: bool }`、`:560-566`）、`:84-88`（`Worker.run: Option<Running<T>>`）、`:169-185`（drop で detach）、`:188-196`（`Device.surface: Option<Box<GpuSurface>>`、`failures: u8`）、`:209-256`（`Background.restarts: u8`、`State::Foreground` は試験だけ）、`:389-616`、`arvane/src/presenter/health.rs`（`FrameHealth`、`LOSSES = 3`）
- **不正**: surface の失敗を `Device.failures`（2 で CPU へ）と `FrameHealth`（3 で error）の二つの counter が別の reset の規則で数える
- **解消後**: 失敗の数え方は `FrameHealth { Healthy, Skipping { frames }, Losing { losses } }` 一つと `fn lost(&mut self, renderer) -> LossAction`（`LossAction { Recover, FallbackToCpu, Fail }`。続けての失敗が GPU で 2 回なら CPU へ、どの renderer でも 3 回なら error、数え直しは `shown` と `skipped`。設計書 19 章「surface の失敗の数え方」）。`enum DeviceSurface { Pending, Live(Box<GpuSurface>) }`、`enum WorkerState<T> { Running(Running<T>), Joined }`。`Step::Configuring { stale: bool }` は `Step::Configuring { .., surface: ConfigureTarget::{Current, Destroyed} }`。`State::Foreground` は `arvane-virtual` が本番の経路を通るので消える（設計書 9.2 節）
- **unit**: A-5

### SM-A12 arvane-render の GPU の状態
- **場所**: `arvane-render/src/gpu/state.rs:87`（`failure: Arc<Mutex<Option<String>>>`、device lost の callback が立てる latch）、`:90`（`offscreen: Option<wgpu::Texture>`）、`:147-171`（`GpuSurface { swapchain: Swapchain, raster: Option<Texture>, keep: bool }`、`Configured.skipped: bool`）
- **解消後**: `DeviceLost`（`Mailbox<String>`、S-9）。`enum Offscreen { None, Allocated { texture, size } }`。`keep: bool` は `enum Retain { Drop, Keep }`、`skipped: bool` は `enum LastConfigure { Applied, Skipped }`
- **unit**: A-5

### SM-A13 shell の支援技術の木
- **場所**: `arvane-shell/src/a11y/shared.rs:11-44`（`Reader::{Inactive, Active { unsent }}`、`Unsent::{Nothing, Changed { nodes, runs, merged: bool }, Tree}`、`Shared.tree: Option<AcceptedTree>`、`actions: Vec<ShellEvent>`）、`arvane-shell/src/a11y/state.rs:28`（`AcceptedTree.focus`）
- **不正**: engine の fingerprint の差分と shell の受理済みの木の差分が直列に二つある
- **解消後**: engine が `Semantics` の差分の更新を作る。shell は `AcceptedTree` を残し（支援技術の起動時の木と action の検査に要る）、受理済みの木との差分の計算だけをやめて、update が運ぶ node の列をそのまま適用・検証する（設計書 19 章「付録 A の直し」）。`AcceptedTree.focus` は保存せず update の値を使う。`merged: bool` は `enum Merge { Separate, Merged }`
- **unit**: A-3

### SM-A14 GPU 共有の slot と pass
- **場所**: `kivact-gpu-share/src/pool.rs:113-169`（`drawn: Vec<Option<NonZeroU64>>`、None = 一度も描いていない、`last`、`next`、`finish` の `assert`）、`kivact-gpu-share/src/wgpu/pool.rs:208`（`failed: bool`）、`arvane-render/src/gpu/external.rs:70-131`（`frame: Option<Frame>` は `with_external` の中だけ、`Frame.submitted: bool` を `expect` で守る、`highest`・`released`・`missed: Option<u64>`）
- **解消後**: `enum SlotState { Fresh, Drawn(NonZeroU64) }`。`enum PoolHealth { Ok, Broken }`。`with_external` は型状態の `ExternalPass<'_>`（`submit(self) -> Submitted`）にし、`frame: Option` と `submitted: bool` を消す
- **unit**: A-5

### SM-A16 shutdown の module の保持
- **場所**: `arvane/src/shutdown.rs:21`（`RetainOnDrop(bool)`）、`:18,25`（`detached_threads`、`retain_module`）
- **解消後**: `RetainOnDrop(bool)` を消し、arvane-native の embed の `Session` が handler を drop した直後に `DetachedThreads::any()` なら `retain_module()` を呼ぶ。`Backend` は `shutdown` を持たない（設計書 19 章「`Backend` は既存の `ShellEventLoop` の改名にする」）
- **unit**: A-1a

### SM-A15 試験の窓の状態
- **場所**: `arvane-native/src/test_display.rs:112-122`（`Pending { frame, accessibility: bool, title, resize, gpu: bool, scheduled: bool }`）、`arvane/src/test_display/shell.rs:38,68`（`redraw: Cell<bool>`、`cpu_frame: Cell<bool>`）、`virtual_window.rs:135-171`（`VirtualShell` 約 35 field）、`kivact-engine/src/host/testing/test_host.rs:51,227-230`（`woken: AtomicBool`）
- **解消後**: `arvane-virtual` で作り直す（設計書 9.3 節、設計書 19 章「`arvane-virtual` の台本・読み出し・段」）。mirror の送り待ちは `arvane-virtual` の feature `mirror` の adapter の `enum MirrorPending { Idle, Scheduled(MirrorBatch) }`、`MirrorBatch` は送る項目の `Option` を持つ option の束（S-2 の対象外: 状態ではなく次に送る内容）。試験 host の `woken: AtomicBool` は消し、`StepOutcome::Idle { woken, .. }` の値を読む
- **unit**: A-1a（virtual と mirror）、A-1b（`woken`）

---

## SM-K: 公式 kit

kit の状態機械の多くは、中核へ上がる部品（付録 C.2）の中で消える。その場合の解消後は「上がる先」を書く。kit に残るものは `Machine` で書く。

### SM-K01 soft の `use_carousel`
- **場所**: `kivact-ui-soft/src/carousel.rs:31-48,268-270`（mount の後に設定した option が builder と食い違う）、`:308-323`（`sleep(120 ms)` の debounce で snap）、`:327-344`（Shift+wheel を手で横 scroll に）
- **解消後**: `kivact::interaction::use_carousel`（付録 C.2）。`enum CarouselState { Idle { index }, Scrolling { target_index, settle_at_ns } }`、snap の待ちは `after`
- **unit**: K-1b

### SM-K02 basic の carousel
- **場所**: `kivact-ui-basic/src/carousel.rs:124-140,245-271`（`Binding<usize>` の index、独自の矢印 key）
- **解消後**: SM-K01 と同じ `use_carousel` の上の `ui::carousel`
- **unit**: K-1b

### SM-K03 soft の `use_collapsible`
- **場所**: `kivact-ui-soft/src/collapsible.rs:35-81,179`（`CollapsibleConfig { enabled, default_open, open: Option, value: Option }`、`is_open = !enabled || open`、別の `disabled`）
- **不正**: `open` が `Some` のとき `default_open` は無視される。disable の flag が二つ
- **解消後**: `ui::disclosure`（`use_disclosure`）。「無効なら常に開く」は soft の recipe でなく、`disclosure(..).always_open()`（構造の option）で表す
- **unit**: K-2a

### SM-K04 soft の accordion
- **場所**: `kivact-ui-soft/src/collapsible.rs:298-327`（`GroupCx { open: Binding<Vec<String>>, mode: Single|Multiple }`、Single で key が二つの Vec を表せる）
- **解消後**: `kivact::interaction::use_disclosure_group(selection: impl Into<Selection<K>>)`（Single は `Binding<Option<K>>`、Multiple は `Binding<Vec<K>>`）と `ui::disclosure_group`
- **unit**: K-2a

### SM-K05 soft の `use_lightbox`
- **場所**: `kivact-ui-soft/src/lightbox.rs:63-75`（`open: bool`、`index: usize` を使うときに clamp）、`:106,119-171`（`zoomed: bool`、`pan`、`drag_start`、「pan ≠ 0 ⇒ zoomed」を手で保つ）、`:138`（画像の失敗の `failed` signal。SM-K32 の lightbox の分もこの項目で扱う）、`:265-293`（独自の key）
- **解消後**: soft に残す（Tier 3 ではなく soft 固有）。`use_dialog` と `use_zoom_pan` の上に作り直し、`enum LightboxState { Closed, Open { index: usize, view: LightboxView } }`、`enum LightboxView { Fit, Zoomed { pan: Point } }`（pan は Zoomed にだけある）
- **unit**: K-5

### SM-K06 soft の `use_resizable`
- **場所**: `kivact-ui-soft/src/resizable.rs:69-198,323-357`（`expanded_size`、`collapsed: Binding<bool>`、`is_collapsed = collapsible && collapsed`、`expand()` が `collapsible` を見ない、独自の key）
- **解消後**: `use_splitter` に collapse と snap を足す（`SplitterApi::collapsible(threshold)`、`snaps(..)`）。`enum PanelSize { Expanded(f64), Collapsed { restore: f64 } }`
- **unit**: I-5b（splitter の collapse・snap と `PanelSize`）、K-5（soft の `use_resizable` を消す）

### SM-K07 soft の scroll の状態と overflow
- **場所**: `kivact-ui-soft/src/scrollable_area.rs:159-306`（`rect()` と `scroll_offset()` の memo、scroll しないとき `at_start` と `at_end` が両方 true、key の scroll の再実装）、`soft_overflow.rs:97-99,205-256`（仮の 80 px、二つ目の fit の algorithm）、`use_overflow`（未使用）
- **解消後**: `NodeRef::scroll_state() -> Value<ScrollState>`（core、`enum ScrollState { NotScrollable, AtStart, Middle, AtEnd }`）。key の scroll は core の scroll の既定の key 操作を使う。`use_overflow` は消す。overflow の fit は soft に残し、`enum OverflowFit { Measuring, Fitted { visible: usize } }`
- **unit**: K-5

### SM-K08 soft の streaming text
- **場所**: `kivact-ui-soft/src/soft_chat_tools.rs:497-571`（`visible: Signal<String>`、`task: Rc<Cell<Option<Task>>>`、`streaming` の flag、`unreachable!()`）
- **解消後**: soft に残す。`enum Stream { Idle, Revealing { shown: usize }, Done }`、進みは `after`
- **unit**: KL-soft

### SM-K09 soft の mobile の app shell
- **場所**: `kivact-ui-soft/src/mobile_nav.rs:17-28`（`AppShellMobile` の 5 つの `Option`）
- **解消後**: private `enum AppShellMobileState { Desktop, Mobile { nav: MobileNavState } }`、`enum MobileNavState { Closed, Open }`（既存の公開 AppShellMobile/MobileNav と名前を分ける）
- **unit**: KL-soft

### SM-K10 手で作った roving focus
- **場所**: `kivact-ui-soft/src/button_group.rs:126-197`、`tab_list.rs:260-336`（`FocusTrack { roving: Signal<Option<usize>> }`）、`outline.rs:300,372-385`、`avatar_group.rs:14-67`（`active` を書くが読まない。全部が focusable）
- **解消後**: 場所ごとに決める: `button_group.rs` は `ui::button_group`（中に roving を持つ、付録 C.2）。`tab_list.rs` は `use_tabs` と、tab の列の中の menu の trigger を含めるために core の `NodeBuilder::roving_focus`。`outline.rs` は端で止まる（clamp）振る舞いが core の `roving_focus` と違うので、soft に残して `roving: Signal<Option<String>>` を `enum OutlineFocus { Unset, At(String) }` にする（振る舞いを変えない）。`avatar_group.rs` は `ui::avatar_group`（unit K-6a）
- **unit**: K-3a（`button_group.rs`、`tab_list.rs`）、KL-soft（`outline.rs`）、K-6a（`avatar_group.rs`）

### SM-K11 creative の outliner の key
- **場所**: `kivact-ui-creative/src/outliner/view.rs:550,602`
- **解消後**: `ui::tree`（`use_tree`）の restyle
- **unit**: K-4

### SM-K12 basic の navigation menu
- **場所**: `kivact-ui-basic/src/navigation_menu.rs:77-81`（`NavOpen::{Closed, Hover(i), Pinned(i)}`）、`:226-298`（hover で閉じる effect、pointer Enter、独自の Escape/ArrowDown）
- **解消後**: `kivact::interaction::use_navigation_menu`（付録 C.2）。状態は既存の `NavOpen` を `Machine` にしたもの。閉じる effect は遷移へ
- **unit**: K-3b

### SM-K13 soft の top nav と side nav
- **場所**: `kivact-ui-soft/src/top_nav.rs:516-531`、`side_nav.rs:562-581`（`pinned: Signal<bool>` と `use_hover_surface().shown()` の組合せ。同じ code が二か所）
- **解消後**: SM-K12 と同じ `use_navigation_menu`
- **unit**: K-3b

### SM-K14 soft の hover card
- **場所**: `kivact-ui-soft/src/hover_card.rs:113-161`（`dismissed`、`initially_open`、`visited`、controlled の `Binding`、二つの `use_delayed`、`open = ..` の式、`previous` の var で `on_open_change`）
- **解消後**: `ui::hover_card`（`use_hover_surface` の上）。`enum HoverCardState { Closed, Opening, Open { by: OpenedBy }, Closing, Dismissed }`
- **unit**: K-2a

### SM-K15 basic の hover card
- **場所**: `kivact-ui-basic/src/hover_card.rs:104-123`
- **解消後**: SM-K14 と同じ `ui::hover_card`
- **unit**: K-2a

### SM-K16 combobox の mode
- **場所**: `kivact-ui-basic/src/combobox.rs:31-37,347-357`（`ComboMode::{Closed, Browsing, Filtering}`）、`kivact-ui-soft/src/typeahead.rs:157-160`（`open`、`sheet_open`、`editing`、`typed` の 4 つの signal）
- **解消後**: `ui::combobox`（`use_combobox`）。状態は `ComboboxState { mode: ComboMode, query, highlight }`、sheet は SM-U03 と同じ `Surface::{Popover, Sheet}`
- **unit**: K-2b

### SM-K17 soft の selector の sheet
- **場所**: `kivact-ui-soft/src/selector.rs:143-184`（`open`、`sheet_open`、`released`。`sheet_open && !open` を `open.get() &&` で隠す）
- **解消後**: `ui::select` の touch の sheet（SM-U03）
- **unit**: K-2b

### SM-K18 soft の date input
- **場所**: `kivact-ui-soft/src/date_input.rs:116-148`（`open`、`invalid`、`opened_from_input`、`draft`、`announcement`。閉じても `opened_from_input` が残る、draft が空なら `invalid` を隠す）
- **解消後**: `ui::date_input`（付録 C.2）。`enum DateInputState { Closed { draft: Draft }, Open { opened_by: OpenedBy, draft: Draft } }`、`enum Draft { Empty, Valid(CalendarDate), Invalid(String) }`
- **unit**: K-1a

### SM-K19 soft の date time input
- **場所**: `kivact-ui-soft/src/date_time_input.rs:94-96`（`open`、`time_panel`、`draft_time`）
- **解消後**: `ui::date_time_input`。`Open { panel: Panel::{Date, Time}, .. }`
- **unit**: K-1a

### SM-K20 pop の date input と range input
- **場所**: `kivact-ui-pop/src/date/input.rs:112`、`date/range_input.rs:125-200`（`cursor`、`open`、開くたびの `RangePick`、start > end を表せて error として見せる、`valid: Option<Binding<bool>>` を effect で書く）
- **解消後**: `ui::date_range_input`（`CalendarSelection::Range` を使う）。start > end は遷移で作らない
- **unit**: K-1a

### SM-K21 soft の power search
- **場所**: `kivact-ui-soft/src/power_search.rs:60-64`（`open`、`editing: Option<(usize, filter)>`、`field`、`operator`、`value`）
- **解消後**: soft に残す。`enum PowerSearchState<K> { Closed, Composing { draft: FilterDraft }, Editing { index: usize, original: SearchFilter<K>, draft: FilterDraft } }`、`FilterDraft { field, operator, value }`。original は外部で変わった filter を上書きしないための編集 baseline。
- **unit**: KL-soft

### SM-K22 soft の banner の focus の復元
- **場所**: `kivact-ui-soft/src/banner.rs:137-155`（`dismissed` と、`Owner::current`・`focused_node` で手で持つ前の focus）
- **解消後**: core の `restore_focus`（`FocusReturn`）を使い、`enum BannerState { Shown, Dismissed }`
- **unit**: K-6b（banner は `ui::alert` の variant になる）

### SM-K23 soft の empty state の読み上げ
- **場所**: `kivact-ui-soft/src/empty_state.rs:88-117`（`mounted: Option<bool>`、`first`、`spoken`、`has_notice`）
- **解消後**: `ui::empty_state`（付録 C.2）は `announcer` で一度だけ読み上げる。`enum Announcement { Pending, Spoken }`
- **unit**: K-6b

### SM-K24 copy の feedback
- **場所**: `kivact-ui-soft/src/code_block.rs:339,520-533`（`copied` と世代の counter と 2 s の `sleep`）、`kivact-ui-basic/src/code_block/view.rs:297-321`（`use_delayed(..).pulse()`）
- **解消後**: `kivact::interaction::use_copy_feedback(duration)`（二つの kit が同じ振る舞いを持つので中核へ、戦略 6 章の 5）。`enum Copied { Idle, Copied }`、戻りは `after`
- **unit**: K-2a

### SM-K25 soft の toolbar の key の hint
- **場所**: `kivact-ui-soft/src/toolbar.rs:266-286`（`anchor`、`seen`、3 s の sleep）、`segmented_control.rs:109-121`
- **解消後**: soft に残す。`enum KeyHint { Hidden, Showing { anchor: NodeToken }, Seen }`、消すのは `after`
- **unit**: K-3a

### SM-K26 soft の outline の scroll の完了
- **場所**: `kivact-ui-soft/src/outline.rs:205`（scroll の完了の代わりの 250 ms の `sleep`）
- **解消後**: soft に残す。`enum OutlineScroll { Idle, Scrolling { target: String } }`。`Scrolling` を抜けるのは、今と同じく 250 ms の `after`（振る舞いを変えない）
- **unit**: KL-soft

### SM-K27 soft の segmented control の disabled
- **場所**: `kivact-ui-soft/src/segmented_control.rs:49,87-107`（builder で作った `Signal<bool>` を `disabled(bool)` で書き、`into_view` で `get_untracked`。reactive でない）
- **解消後**: `NodeBuilder::disabled(impl IntoValue<bool>)` を使う（固有の `disabled` を消す）。segmented は `ui::radio_group` の variant（付録 C.2）
- **unit**: K-3a

### SM-K28 kit の drag と gesture
- **場所**: `kivact-ui-basic/src/drawer.rs:122-157`（`pulled`、`last: (f32, u64)`、`speed` で手で速度）、`kivact-ui-soft/src/bottom_sheet.rs:163-235`（`dragged: Option<f32>`、毎回 clamp する `selected`、`velocity`）、`month_swipe.rs:10-151`（`Axis::{Pending, Inline, Block}`、contact、`shift`、`entering`、`suppress_choice`、6 px、`sleep(1 ms)`）、`selector_surface.rs:62-91`（10 px の tap の検出）
- **解消後**: `DragApi` の `slop`・`axis_lock`・`track_velocity`（`DragUpdate.velocity`）と、snap の既存の関数（`nearest_snap`、`pull_to_snap`。設計書 19 章「`DragApi` の追加と touch だけの gesture」）。drawer は `ui::dialog` の `Drawer` の swipe で dismiss（`InteractionPolicy.drawer.swipe`、設計書 19 章「`InteractionPolicy` の形と置き場所」）、soft の bottom sheet の snap は soft に残す（`enum Sheet { Resting { snap: usize }, Dragging { from: usize, offset: f32 }, Settling { to: usize } }`）。month swipe は `use_drag` の上に作り直さず、raw pointer の event を受ける machine にし（slop は `DragAxis::past_slop`、6 px は名前の付いた private の定数、速度は `DragVelocity`）、`sleep(1 ms)` を消す（activation の順は engine の `Routed` の activation の field で決まる、SM-C17）
- **unit**: K-1a（month swipe）、K-2a（drawer）、K-5（bottom sheet）

### SM-K29 soft の date の wheel
- **場所**: `kivact-ui-soft/src/date_touch.rs:330-400`（scalar の再実装と `release: Signal<u64>` の counter）
- **解消後**: `use_scalar` の上に作り直す。`release` は `Notifier`
- **unit**: K-1a

### SM-K30 creative の drag の開始時の写し
- **場所**: `kivact-ui-creative/src/feedback.rs:520-548`、`bezier/view.rs:99`、`gradient/view.rs:121`、`numeric.rs:334-335`、`panel_group.rs:60`、`range.rs:213`（開始時の値を var に写す）、`editors/shared/gesture.rs:12-60`（`CanvasGesture`、`Signal<Option<Pressed>>`）
- **解消後**: `use_drag::<T>` の `on_start` が返す `T` を `DragUpdate<T>.origin` として後の update に渡す（既存の `Drag<T>` の payload を使う）。var の写しを消す。`CanvasGesture` は `use_drag` の `Pending` を使う
- **unit**: KL-creative

### SM-K31 dropzone
- **場所**: `kivact-ui-pop/src/form.rs:128`（`hovered: bool`）、`kivact-ui-game/src/input.rs:85,163`（`over: bool`、`PickStatus::{Ready, Picking, Failed}`）、`kivact-ui-soft/src/file_input.rs:102-104`（`pending`、`dragging`、`status: Option<String>`、`std::fs::metadata` の検証）
- **解消後**: `kivact::interaction::use_dropzone` と `ui::dropzone`（付録 C.2）。`enum DropzoneState { Idle, DragOver { valid: Validity }, Picking, Failed(UiError) }`。file の検証は caller の callback（`validate(impl Fn(&FileInfo) -> Option<String>)`）にし、kit が `std::fs` を呼ばない
- **unit**: K-1b

### SM-K32 画像の読み込みの失敗の fallback
- **場所**: `kivact-ui-soft/src/avatar.rs:283`、`thumbnail.rs:294`（`failed` の signal を繰り返す。lightbox の分は SM-K05）
- **解消後**: core の `NodeRef::image_status() -> Value<ImageStatus>`（`enum ImageStatus { Loading, Ready, Failed }`）を足し、`on_image_error` で signal を作る写しを消す。avatar は `ui::avatar`（付録 C.2）
- **unit**: K-6a

### SM-K33 game の frame の loop と focus の復元
- **場所**: `kivact-ui-game/src/feedback.rs:274-285`、`surface.rs:120-133`（見えているときだけの `frame_loop` の同じ block）、`navigation.rs:131-133`（作り直しのときの `restore_focus` の var）、`display.rs:461`（`reveal_once`）
- **解消後**: 見えているときだけ動かす loop は `FrameLoop::while_present(node_ref)`（K-0。二か所の写しを消す）。focus の復元は `ui::pagination`（付録 C.2）の中の `use_list` の roving が持つ。`reveal_once` は `enum Reveal { Pending, Done }`
- **unit**: K-4（frame の loop、focus の復元）、KL-game（`reveal_once`）

---

## P-1 全数調査の追加（2026-11-04）

以下の場所は P-1 の調査時点のもの。既存項目の同じ状態は再登録せず、別の状態の持ち主・同期規則だけを追加する。検索の除外理由と検査結果は [DECISIONS](DECISIONS.md) の P-1 に記録する。timer の解消後はすべて 18.13 節の `Step::after`。

### SM-I901 scalar の track の取り付け
- **場所**: `kivact-interaction/src/scalar/track.rs:130-153,203-252`（`Pointer.geometry`、`attached`、`grab`）
- **現在**: `attach_track` が `geometry = Some` と `attached = true` を別々に書く。`grab` は geometry があるときだけ有効。
- **不正**: 取り付け済みなのに geometry がない、未取り付けなのに grab がある。SM-I17 の settings と SM-I18 の touch の coast とは別の track の状態。
- **解消後**: `Signal<TrackPhase>`、`enum TrackPhase { Detached, Attached { geometry: Geometry, gesture: TrackGesture } }`、`enum TrackGesture { Idle, Grabbing(Grab) }`。`attached` は memo、取り付けと grab の遷移だけが phase を書く。
- **unit**: I-5a

### SM-I902 range の変更に追従する scalar の値
- **場所**: `kivact-interaction/src/scalar/range.rs:83-92`、`range_scalar.rs:102-113`
- **現在**: range が変わった後、effect が binding を範囲内へ戻す。二つの end は書く順まで effect が決める。
- **不正**: 新しい range と古い範囲外の値が別々の段階で見える（S-5）。
- **解消後**: scalar の machine が `RangeChanged(ScalarRange)` を受け、正規化と値の effect を一つの遷移で決める。range scalar は両端を同じ遷移で計算し、現在の書く順を effect の列に保存する。
- **unit**: I-5a

### SM-I903 calendar の range の選び直し
- **場所**: `kivact-interaction/src/calendar/state.rs:13-34,64-80`
- **現在**: `Choice::Range { value, pick }` の `RangePick::End` と、binding が anchor の単日 range であることを effect で一致させる。
- **不正**: 外部から value が変わっても effect が走るまで古い anchor の続きを選べる。SM-I26 の生存 token とは別。
- **解消後**: `enum CalendarRangePhase { Start, End { anchor: CalendarDate, before: Option<DateRange> } }` を machine が持ち、`ExternalValueChanged` と日付の選択を同じ遷移入口で処理する。binding の更新と phase の reset は一度に決める。
- **unit**: I-4b

### SM-U901 text input の callback の後付け
- **場所**: `kivact-ui/src/text_input.rs:32-47,81-97,350-379`
- **現在**: `invalid` と `edited` の `Rc<OnceCell<Rc<dyn Fn..>>>` を field の作成後、`into_node` で埋める。
- **不正**: style と binding の callback が構築途中の空の slot を読む。builder の option の束とは別の S-7。
- **解消後**: `into_node` で draft と validity を解決してから、callback を引数に取る完成した input を構築する。`OnceCell` を消し、draft の有無は `enum InputDraft { Immediate, Drafting(Installed) }` で分ける。公開 builder のメソッドはそのまま。
- **unit**: U-2

### SM-U902 number input の文字と値
- **場所**: `kivact-ui/src/number_input.rs:191-239`
- **現在**: `NumberText` と scalar の binding を effect で同期し、focus を失ったときも effect が文字を戻す。
- **不正**: 外部値と表示の文字が effect まで食い違う。既存の `NumberText` enum があっても遷移の入口が分散する。
- **解消後**: `Signal<NumberText>` を machine が持ち、`ValueChanged`・`Typed`・focus の out event を受ける。数値と文字・`produced` は一つの遷移で決める（既存の文字の保持条件は保存）。
- **unit**: U-2

### SM-U903 toast の退場する行
- **場所**: `kivact-ui/src/toast_rows.rs:21-47,58-87`
- **現在**: `slots` は退場中を含む全行、`live_slots` はその live な部分集合。正常な live 行は両方に入る。この包含関係と `mounted`・`height`・animation の値から行の段階を推定し、複数の effect が collection の追加・更新・削除を分担する。
- **不正**: 同じ live 行を二つの collection に保持する重複と、行の phase が複数の signal/effect に分散するため、包含関係と phase の一致を更新順で保つ必要がある。同じ id の再表示の区別も生の世代 counter に依存する。両方への所属自体は正常。SM-U02 の swipe とは別。
- **解消後**: `ToastRowPhase` は作らない。行の node の `exit_size(Orientation::Vertical)`（engine の presence）で縮め、退場中を含む `slots` と `live_slots` の二重の列、世代の counter、`animated` の高さを消す（設計書 19 章「toast の行と tree の行の出現と退場も presence で描く」）
- **unit**: I-7

### SM-U904 table の filter の文字と validation
- **場所**: `kivact-ui/src/data_table/filters.rs:239-269`
- **現在**: `raw`・`valid`・`sent` を別々に更新し、外部の binding が変わったら effect が三つを戻す。
- **不正**: 新しい文字と前の validation が共存する。SM-I27 の table settings とは別の ui の入力。
- **解消後**: `enum FilterDraft { Valid { text: String, value: Option<FilterValue> }, Invalid { text: String } }` を一つの signal に置き、valid は memo。最後に外へ書いた値は `Sent<Option<FilterValue>>`、外部値と文字の編集は一つの遷移で決める。
- **unit**: U-2

### SM-U905 progress と button の motion の段階
- **場所**: `kivact-ui/src/progress.rs:166-198`、`button.rs:240-329`
- **現在**: progress は frame loop の start/stop と `phase` を effect で同期する。button は loading の表示と `reveal_pending`、sleep、fade の開始を別々に持つ。
- **不正**: 止まった loop に前の phase が残る、loading の内容と spinner の表示の開始が別の状態になる。
- **解消後**: progress は `enum ProgressMotionPhase { Stopped, Running }`、button は `enum BusyPhase { Idle, Waiting, Revealing, Shown }` を machine が持つ。progress の位相は既存と同じ clock の絶対時刻から計算し、再開時や period 変更時にも開始時刻からの相対計算へ変えない。button の待ちは `Step::after`、start/stop は遷移 effect。
- **unit**: U-2

### SM-K901 pie menu の公開状態
- **場所**: `kivact-ui-creative/src/pie.rs:36-62,84-105`
- **現在**: 公開 `PieState { open, held, center, hot }`。close は `open` だけを書き換える。
- **不正**: closed でも held/hot が残り、caller は任意の組合せを作れる。
- **解消後**: 公開 `enum PieState { Closed, Open { mode: PieMode, center: Point, hot: Option<usize> } }`、`enum PieMode { Click, Held }`。閉じる遷移は `Closed`。公開型の変更は P-1 主担当がユーザーの自己判断の指示に従って確定（DECISIONS）。
- **unit**: KL-creative

### SM-K902 color wheel と gradient の入力の複製
- **場所**: `kivact-ui-creative/src/color_wheel/state.rs:20-28`、`color_wheel/view.rs:187-212`、`gradient/state.rs:22-31`、`gradient/view.rs:154-164`
- **現在**: `blocked: Signal<bool>` を part の disabled/hidden から effect で複製し、別の edit/drag を cancel。drag も `Var<Option<Drag>>` に後付けする。
- **不正**: part が無効でも effect まで編集が有効。SM-K30 の drag 開始値の写しとは別の S-4/S-7。
- **解消後**: blocked は part の値からの memo、drag は完成した controller の構築引数。無効化と focus の out event を編集 machine が受け、同じ遷移で cancel を決める。
- **unit**: KL-creative

### SM-K903 color wheel の値と active channel
- **場所**: `kivact-ui-creative/src/color_wheel/view.rs:134-142,200-205`、`color_wheel/fields.rs:172-184`
- **現在**: hue を外部色から effect で同期し、alpha がなくなると別の effect が active channel を Hue へ、palette の縮小で選択を clamp する。
- **不正**: 使えない Alpha や消えた swatch を effect まで選んだ状態になる。
- **解消後**: `Signal<ColorControls>`、`struct ColorControls { channel: Channel, swatch: SwatchSelection, hue: f64 }`、`enum SwatchSelection { None, Selected(usize) }`。channel と swatch は独立して共存する現在の意味を保存し、`ColorChanged`・`AlphaChanged`・`PaletteChanged` の遷移内でそれぞれの有効性を決める。無彩色で hue を保持する現在の規則も保存。
- **unit**: KL-creative

### SM-K904 creative range の active end
- **場所**: `kivact-ui-creative/src/range.rs:205-225`
- **現在**: pressed を effect で active に写し、前回の両端 `moved` と比較して動いた end を推定する。
- **不正**: press と active end、値と active end の変更が別の段階（S-4/S-5）。SM-K30 の drag origin とは別。
- **解消後**: press の event と `ValueChanged { before, now }` を range の machine が受け、`RangePart` の変更を遷移内で決める。前回値は machine の単一の snapshot に閉じる。
- **unit**: KL-creative

### SM-K905 picker の column の追従
- **場所**: `kivact-ui-creative/src/picker.rs:474-480,532-562`
- **現在**: query/list が変わると effect が `active_column` を次の候補へ直す。
- **不正**: 候補のない column が effect まで active。既存 `PickerPhase` は報告の一度だけの制御で、この column とは別。
- **解消後**: `enum PickerColumn { Empty, Active(usize) }` を machine に置き、`QueryChanged`・`ItemsChanged`・column 移動の遷移内で選ぶ。preview と pick は選んだ column と同じ遷移の effect。
- **unit**: KL-creative

### SM-K906 creative editor の focus と data の取消し
- **場所**: `kivact-ui-creative/src/editors/node/view.rs:196-225`、`editors/arrangement/view.rs:288-310`、`editors/time/view.rs:217-235`、`editors/graph/view.rs:243-260`
- **現在**: node editor は focus/disabled の effect が接続の `mode` と edit を終了する。arrangement/time/graph も focus/visibility/data の effect が現在の drag を cancel する。
- **不正**: focus を失っても接続/drag が残り、消えた item の drag は effect まで有効。SM-K11 の key と SM-K30 の gesture origin とは別。
- **解消後**: 各 editor の gesture/接続の machine が focus の out・visibility・無効化・data の変更を受け、取消しと rollback を同じ遷移で決める。
- **unit**: KL-creative

### SM-K907 editor の region id の純粋でない計算
- **場所**: `kivact-ui-creative/src/editors/shared/targets.rs:78-90`
- **現在**: memo の中で `Var<TargetTable>::update` が region id の割当て表と counter を更新する。
- **不正**: pure な計算が状態を変更する。単なる geometry cache と違い、割当てた id が外へ出る。
- **解消後**: target source の変更を controller の遷移で受け、id の割当てと `Scene` を同時に決める。割当ては `Generation` を包む名前の付いた allocator、返す `Value` は完成した scene の memo にする。
- **unit**: KL-creative

### SM-K908 meter の clip hold
- **場所**: `kivact-ui-creative/src/audio/meter.rs:88-103`
- **現在**: level の effect が clipped を立て、press の handler が下ろす。
- **不正**: level の変更と hold の開始が別の段階（S-5）。単独 bool を理由に検出するのでなく effect との同期を対象にする。
- **解消後**: `enum MeterClip { Clear, Held }` の machine が `LevelChanged` と `Reset` を受ける。level > 1 の遷移が Held にする。
- **unit**: KL-creative

### SM-K909 soft rich blocks の編集状態
- **場所**: `kivact-ui-soft/src/rich_blocks.rs:20-52,103-117,176-190,265-293`
- **現在**: source/document の両方を保存し effect で一致させ、target/composing/whole/focus_after_mount を別々に変更する。
- **不正**: 外部 source と古い document、全体選択と composition、削除済み target の組合せが表せる。
- **解消後**: `enum BlockEditPhase { Idle, Focused { target: NodeRef, selection: BlockSelection, composition: BlockComposition }, AwaitingMount { block: usize } }`、`BlockSelection::{Caret, Whole}`、`BlockComposition::{Idle, Composing}`。source の変更と block 編集を一つの遷移で parse/write し、document はその snapshot。focus の event で遷移する。
- **unit**: KL-soft

### SM-K910 soft timer と timestamp の表示の更新
- **場所**: `kivact-ui-soft/src/timer.rs:93-116`、`timestamp.rs:289-329`
- **現在**: timer は effect ごとに `Option<Task>` を cancel/restart し、display を書く。timestamp は now/value と display を effect で同期する。
- **不正**: start や now の変更と表示が別の段階。timer の登録/実行中を task の有無で表す。
- **解消後**: timer は `enum ElapsedTimer { Running { start: SystemTime, next: Duration } }` の machine が start の変更と tick を受け、`Step::after` を返す。文字は snapshot からの memo。timestamp の文字も now/value からの memo、tick の時刻だけを machine が更新する。
- **unit**: KL-soft

### SM-K911 soft time input の draft
- **場所**: `kivact-ui-soft/src/time_input.rs:112-153`
- **現在**: draft と invalid を別々に持ち、外部 value を effect で draft に写して invalid を消す。
- **不正**: 無効な文字が新しい value と共存し、memo が空文字の invalid を隠す。SM-K18/19 の日付・日時の field とは別の time field。
- **解消後**: `enum TimeDraft { Empty, Valid { text: String, value: TimeOfDay }, Invalid(String) }` を signal に持ち、外部値・入力・commit の遷移で一度に決める。invalid は memo。
- **unit**: K-1a

### SM-K912 calendar kit の範囲選択と月の通知
- **場所**: `kivact-ui-soft/src/calendar.rs:303-319,348-380`、`kivact-ui-pop/src/date/calendar.rs:193-199`
- **現在**: soft は local range と外部 value、pending anchor、hovered を effect で同期する。月の変更は previous_month との比較で announcement を作る。pop は selected の月を effect で visible に写す。
- **不正**: 外部 range が変わっても古い pending が残る。月の表示と通知が別の段階。SM-K28/29 の swipe/wheel とは別。
- **解消後**: range は `enum KitRangePick { Idle, Picking { anchor: CalendarDate, preview: Option<CalendarDate> } }`、range と month の変更を calendar の遷移内で決め、announcement はその effect。pop の visible month も選択変更を同じ controller の event にする。
- **unit**: K-1a

### SM-K913 pop の typed date の追従と validation
- **場所**: `kivact-ui-pop/src/form.rs:36-70`、`date/text.rs:67-94`
- **現在**: `Draft::new` の effect が外部 value を text に写す。DateText の checked は focus を見た effect で立てる。
- **不正**: text/value と validation の段階が分散する。SM-K20 の range の選択順とは別。
- **解消後**: `enum TypedDate { Following(CalendarDate), Editing { text: String, validation: DateValidation } }`、`DateValidation::{Unchecked, Checked}` を machine が持つ。外部値と入力と focus の out event を遷移入口にし、parsed/shown は memo。
- **unit**: K-1a

### SM-K914 questionnaire の提出の失効
- **場所**: `kivact-ui-basic/src/questionnaire.rs:448-472`
- **現在**: 提出済みの `Phase` に step/answers の写しを保存し、effect が変更を見て Asking に戻す。
- **不正**: 新しい回答なのに effect まで Submitted と見える。
- **解消後**: 既存 `Phase::{Asking, Submitted { at, answers }}` を machine に置き、`StepChanged`・`AnswersChanged`・`Submit` を遷移入口にする。提出と失効を一つの場所で決める。
- **unit**: KL-basic

### SM-K915 skip link の focus の要求
- **場所**: `kivact-ui-soft/src/app_shell.rs:238-260`
- **現在**: `skip_requested` を effect が見て focus を要求し、別の effect が focus を失ったら消す。
- **不正**: focus の要求と focus の状態を effect 二つの順で表す。SM-K09 の mobile nav の状態とは別。
- **解消後**: `enum SkipFocus { Idle, Requested }` の machine が activate と focus の in/out event を受ける。focus の要求は activate の遷移 effect。
- **unit**: KL-soft

### SM-K916 avatar の status label の複製
- **場所**: `kivact-ui-soft/src/avatar_status_dot.rs:113-121`、`avatar.rs` の `AvatarStatusContext`
- **現在**: status dot の label を context の signal へ effect で写し、cleanup で None に戻す。
- **不正**: status の label と avatar の accessible name が effect まで食い違う。SM-K32 の画像 fallback とは別。
- **解消後**: context は label の `Value` を持つ登録を mount 時に受け、名前は登録された source からの memo。登録の所属は `enum AvatarStatusSource { None, Mounted { label: Value<Option<String>> } }` とし、cleanup はその遷移で外す。
- **unit**: K-6a

### SM-K917 outliner の active item の reveal
- **場所**: `kivact-ui-creative/src/outliner/view.rs:235-275`
- **現在**: active の変更を effect が見て ancestors を expanded に足し、未 mount の行は pending に保存する。
- **不正**: active、展開済みの ancestors、pending の reveal が別々の段階になる。SM-K11 の key と SM-K30 の drag origin とは別。
- **解消後**: `enum OutlinerReveal { Idle, AwaitingRow { target: u64 } }` を machine が持ち、active の変更と ancestors の展開と reveal を一つの遷移で決める。row の token は `Generation`。
- **unit**: K-4

### SM-K918 viewport の最後に報告した size
- **場所**: `kivact-ui-creative/src/viewport_surface.rs:245-263`
- **現在**: `previous: Var<Option<SurfaceSize>>` を effect が読んで更新し、resize callback の重複を除く。
- **不正**: 最後に渡した値の規則を生の Option と呼び出し順で表している（13.2 の cache の行）。
- **解消後**: `Sent<SurfaceSize>`。resize callback を呼ぶ一つの入口だけが `update` する。size 自体は rect と scale の memo。
- **unit**: K-8

### SM-K919 typeahead の選択済み label の cache
- **場所**: `kivact-ui-soft/src/typeahead.rs:184-224`
- **現在**: `cached` は選択済み item の label を保持するための履歴。検索完了後の effect が新しい結果を基に、旧 cache にある選択済み item の不足分を残す。`labels` は source と cache からの memo、`options` は resource の `state` の Ready を直接読む memo（211〜224行）で、cache を候補の状態として使わない。
- **不正**: label の保持規則を生の signal と effect の更新手順で表しており、cache の責務と保持する条件が明示の型に閉じていない。options の遅延や検索状態の二重保持を問題としない。SM-K16 の open/filtering mode とは別の S-9 の cache。
- **解消後**: S-9 の明示の `LabelCache<K>` に保持規則を閉じる。検索完了 event で、新しい結果を基に現在と同じ選択済み label の保持規則を更新する。`labels` は source と cache からの memo、`options` は resource 状態を直接読む memo を維持し、現在の検索結果を別 controller にも保存しない。debounce/cancellation は既存 resource が所有し、callback と表示の契約を変えない。
- **unit**: K-2b

### SM-I904 command palette の閉じると query の reset
- **場所**: `kivact-interaction/src/palette.rs:326-331`
- **現在**: caller の open と combobox の query/highlight を別々に持ち、閉じた後の effect が `combobox.reset()` を呼ぶ。
- **不正**: 閉じた palette に前回の query/highlight が effect まで残る（helper 経由の S-5）。
- **解消後**: palette の machine の `Closed/Open` の遷移が combobox の query/highlight の reset を同時に決める。combobox の snapshot を再利用し、別の query の写しを作らない。
- **unit**: I-2a

### SM-K920 bezier の無効化と drag の取消し
- **場所**: `kivact-ui-creative/src/bezier/view.rs:178-185`
- **現在**: part の disabled/hidden を effect が見て `gesture.cancel()` を呼ぶ。
- **不正**: part が無効になっても effect まで drag が active。SM-K30 の drag origin とは別の入力との同期。
- **解消後**: gesture の machine が part の入力変化を受け、無効化と cancel を同じ遷移で決める。disabled/hidden の写しは保存しない。
- **unit**: KL-creative

### SM-K921 color popup の開閉と edit session
- **場所**: `kivact-ui-creative/src/color_popup.rs:22-28,54-69`
- **現在**: open は signal、snapshot は別の EditSession。閉じた後の effect が commit し、Escape は dismiss callback が cancel する。
- **不正**: 閉じているのに session が active、Escape の cancel と close の commit が二つの入口に分かれる。
- **解消後**: `enum ColorPopupPhase { Closed, Editing { initial: Color } }` を machine が持つ。開く・Escape・その他の close の遷移が snapshot/rollback/commit と popup の effect を一度に決める。
- **unit**: KL-creative

### SM-K922 pagination の page の draft
- **場所**: `kivact-ui-soft/src/pagination.rs:626-642`
- **現在**: page と draft を effect で同期し、focus を失った effect が commit する。
- **不正**: page と前の文字が別の段階で見え、focus の event でなく派生値の観測順で commit する。
- **解消後**: `enum PageDraft { Following(u32), Editing(String) }` の machine が page の変更・入力・Enter・focus の out を受け、clamp と書く値と表示文字を一つの遷移で決める。
- **unit**: K-3b

### SM-K923 nav heading の二つの開閉状態
- **場所**: `kivact-ui-soft/src/nav_heading.rs:119-132`
- **現在**: use_hover_surface の shown と use_menu の shown を別々に持ち、effect が menu の open/close を呼ぶ。
- **不正**: hover surface が開いて menu が閉じた組合せを effect で直す（S-4/S-5）。SM-K13 の top/side nav とは別の heading の adapter。
- **解消後**: `use_navigation_menu` の一つの開閉 machine を使い、hover の timing と menu の session を同じ遷移で決める。表示の shown はその snapshot からの memo。
- **unit**: K-3b

### SM-R901 generational arena の slot
- **場所**: `kivact-reactive/src/arena.rs:33-37,57-68,96-122`
- **現在**: `Entry { generation, value: Option<T>, tomb: Option<Tomb> }`。free list への所属と generation の飽和から reusable/retired を推定する。
- **不正**: value と tomb が同時に Some、occupied slot が free list にある組合せを表せる。tomb 自体が None の削除も現在の API が認める。
- **解消後**: `enum SlotState<T> { Occupied(T), Vacant { tomb: Option<Tomb> }, Retired { tomb: Option<Tomb> } }`。slot の generation は `SlotGeneration(u32)` に閉じ、remove/insert の遷移だけが SlotState と free list を更新する。飽和時の再利用禁止と tomb の診断の現在の規則を保存。
- **unit**: R-2

### SM-A901 native input の wake と環境の後付け
- **場所**: `arvane-native/src/native_input/state.rs:33-60`、`runtime.rs:653-664`、`embed/windows/window.rs:295`
- **現在**: NativeInput::default の後に scale/reduced_motion/mouse/locale と waker を埋める。native runner は Some(waker)、embed は既定の None のままで直接 event を届ける。
- **不正**: native input が未設定の環境と None の waker で存在でき、subclass の設定通知は waker がないと wake しない。SM-A08 の take mailbox と異なり、waker は一度だけの構築情報。
- **解消後**: `NativeInput::new(environment, wake)` の構築引数にし、`enum InputWake { Direct, EventLoop(ShellWaker) }` を保存する。embed の Direct と native の EventLoop を明示し、後付けの RefCell<Option<ShellWaker>> を消す。環境のその後の変更は現在と同じ event で更新する。
- **unit**: A-3
