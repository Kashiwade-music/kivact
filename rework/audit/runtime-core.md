# Runtime / core / macros / i18n audit (RC)

Scope: `crates/kivact-runtime` (13.7k lines), `crates/kivact-core` (32.1k), `crates/kivact-macros` (0.6k), `crates/kivact-i18n` (1.2k).
Read-only audit at 2b8df5ac. No cargo was run, so byte sizes are estimates from field layouts.

## 0. Summary

1. **Always-on diagnostics in release builds.** Every write, rerun and event records into a 256-turn trace ring. Each event formats a `String` cause twice. Every graph node carries about 48 B of diagnostic data. Every node with a dynamic prop keeps a provenance `Vec`. The DevTools hooks do a thread-local runtime lookup on every prop write. There are three trace systems (TurnRecord, DetailLog, debug `client::trace`). Affects K2, K3 and K5.
2. **O(all) work per turn or per change, against PERF-1:**
   - motion plans scan every animation on every commit;
   - DevTools rebuilds and diffs the whole tree every turn;
   - `is_visible`/live-region use App-global notifiers with ancestor walks;
   - a measured `virtual_list` is O(N) per scroll;
   - style recompiles fully on any input change;
   - selector prune, timer cancel and poll are O(N);
   - ICU formatters and collators are rebuilt on every call.
3. **The turn pipeline is ad hoc.** "Is there pending work" is defined four different ways. Core keeps an activation queue the runtime does not know about. The idle shortcut ignores `WakeSent`. Settle loops are nested (async 1000 rounds inside commit 100 rounds) and stabilize is called redundantly. The runtime stores its own `TaskState`/`MotionState` in a TypeId-scanned `Extensions` list, and core must install `MotionEnv` there or runtime motion panics.
4. **Core has many implicit state machines and duplicated facts:**
   - a window declaration stored 3×, plus a `ForeignWindows` copy of the window table;
   - `PointerObservation::Plain` phase;
   - focus requests kept in sync through `order` + map + stage;
   - a text-edit suppression counter;
   - `layout_seeds`;
   - asset-failure cleanup through a `removed` side list;
   - three waiter maps;
   - AsyncCell with two signals; Action acceptance held in two places;
   - Machine transitions not run as pure.
5. **Public API: one concept, several entrances.**
   - `App` and the public `launch::AppOptions` (plus `Launch`/`ClientMode`/`Inspect` reachable on kivact-core);
   - two `Owner` types and two context stores;
   - six conversion traits for "a prop";
   - two `FrameLoop`s;
   - `ActionOutcome`/`AsyncState`/`Result`;
   - `on_close_request` (main window) vs `on_dismiss` (other windows), with different setup rules;
   - `use_direction()` leaks one constant per call.

**Target shape.**
- **kivact-runtime** owns the graph, the turn state, `TaskState`/`MotionState` as plain fields, one deferred-handler queue and one `has_work()`. Diagnostics are off unless a recorder (DevTools/test/debug) turns them on; node metadata goes to a side table.
- **kivact-core** keeps one `AppState` struct (one downcast) instead of a type map.
- The turn is one linear function: `begin → per segment {mounts, events → settle} → loop {commit, mounts, user effects, settle} → finish`.
- Every per-frame and per-change index is maintained by its writer (running animations, observed nodes, engine change journal).
- DevTools hangs off a `DevtoolsHooks` trait object, so `view` carries no cfg stubs.
- `draw::Offscreen` leaves core as a leaf crate.
- The public API has one way per concept: `App` only, one `Owner`, `IntoValue`/`IntoBinding`, one `FrameLoop`, `Result` for async outcomes, and one window close-request API.

## 1. Current structure map

| crate / module | responsibility | lines | used by | boundary verdict |
|---|---|---|---|---|
| kivact-runtime | reactive graph, owners, turn state, task, motion, Machine | 13,671 | core, style, i18n, test | **Real**: style and i18n need it without views. Its public "driver" API (`Runtime`, `TurnGuard`, `Stage`, `Pending`, `Extensions`, `Host`, `TurnReport`, `CauseTag`…) is a leak; see RC-30 |
| runtime/reactive | arena, graph, frames, owners, handles, props, diag, detail | 7,676 | all of runtime | core of the crate |
| runtime/task | executor, timers, spawn, resource, action, blocking pool, drain | 2,730 | core, ui | ok. Drain is over-generalised (RC-37) |
| runtime/motion | Animated, Timeline, FrameLoop, MotionState | 2,791 | core, ui, kits | ok. Indexing is wrong (RC-08) |
| runtime/machine | Machine reducer | 260 | interaction, ui | ok |
| kivact-i18n | Locale, Messages, ICU formatting | 1,187 | core, test | **Real**: ICU is enclosed. Depends on runtime only for `constant_value!` |
| kivact-macros | `#[component]` | 634 | facade | required (proc-macro) |
| kivact-core | App, view, mount, client dispatch, platform, devtools, embed, offscreen, gpu_interop | 32,128 | facade, interaction, ui, test | Large, but view, client and platform share internals. Splittable: only `draw::Offscreen` (RC-29) |
| core/view | node table, mount, flows, windows, overlays, requests, hooks | 18,532 | client, devtools, custom, gpu_interop | Cannot be split from client: dispatch reads `CoreNode.handlers`, `Requests` and focus internals directly |
| core/devtools (+ view/devtools_*) | DevTools session over the agent | 5,167 + ~1,500 | launch | Must stay in core (it reads ViewState internals, so an SPI would be fake). Fix the direction instead (RC-27) |
| core/client | host event dispatch | 1,614 | launch | Part of view's boundary |
| core/gpu_interop + sync.rs | import/export of GPU textures | 1,370 + 111 | facade (feature) | Reads `ViewState.requests`/tree, so not clean. Keep |
| core/custom | custom-element facade | 1,457 | kits | Stays with view |
| core/launch + app.rs + embed | App builder, turn loop, test launch, embed entry | 1,036 + 272 + 424 | facade, test | **Missing boundary**: the turn loop is mixed with test plumbing and three `ClientMode` special cases (RC-12) |
| core/draw (Offscreen) | offscreen PNG/SVG | 411 | facade | **Clear leaf**. Its only internal deps are `Draw`/`Size` |
| core/platform | env reads, services, AppHandle, lifecycle, close request | 1,149 | view, client | ok |

## 2. Findings

### RC-01 The idle-turn shortcut ignores `WakeSent` and queued work
- kind: BUG (latency) / STATE
- where: crates/kivact-core/src/launch/runtime.rs:250-257; crates/kivact-runtime/src/host.rs:68-99; crates/kivact-runtime/src/reactive/clock.rs:14-19
- evidence:
  - `idle = events.is_empty() && boundaries.last().is_some_and(|b| !rt.mounts_due(b.revision))`, then `return output(rt.report_now())`.
  - `report_now` never leaves `TurnState::WakeSent`. A boundary-only turn after `finish()` asked for another turn (pending user effects, `later` reports, ready tasks) does no work and leaves the state at `WakeSent`.
  - Meanwhile every `request_turn` is swallowed, because it acts only on `Quiet`. Recovery relies on the separately posted `ShellEvent::Wake` reaching `PumpHost::wake` with an empty queue (engine host/events.rs:341).
- fix:
  - `idle` also requires `!rt.has_work()` (RC-02).
  - Every early return goes through `Runtime::end_idle(&self) -> TurnReport`, which sets `Quiet` and re-requests a turn if work exists.
  - `report_now` becomes private to `finish`/`end_idle`. It must not write `TurnState` (today it sets `Running(Output)` from a query).
- breaking: no. kpi: K5. size: S. confidence: medium (I traced the native pump; I did not trace the test host)

### RC-02 "Pending work" is defined four times, and activations live outside the runtime
- kind: DUP / STATE (S-4)
- where:
  - runtime host.rs:141-157 (`finish`: user|render|ready|later|client_pending);
  - host.rs:159-166 (`finish_setup`: user|render|ready);
  - core launch/inspect.rs:99-105 (`Inspect::pending`: turn_requested|user|render|ready);
  - launch/runtime.rs:366-385 (`settle_user_effects`: mounts|user|render);
  - view/requests.rs:236-282 (`pending_activations`), drained at launch/runtime.rs:297 and 375.
- evidence: each caller decides which queues count. `NodeRef::activate` keeps a core-only queue, which the runtime knows about only through the `client_pending: bool` argument of `finish`.
- fix:
  - Add `Runtime::defer(owner: Owner, f: Box<dyn FnOnce()>)`: one deferred-handler queue that runs in a handler frame of `owner`. `NodeRef::activate` pushes into it.
  - `Pending` becomes `{ effects_render, effects_user, deferred, later, tasks_ready, mounts_due }`, and `Runtime::has_work() -> bool` is the only predicate.
  - Delete `finish(.., client_pending)`, `requests::{take,has}_activations` and `run_requested_activations`.
  - Delete `Pending.turn_requested`; `TurnState` is the owner of that fact.
- breaking: no (hidden driver API). kpi: -. size: M. confidence: high

### RC-03 The turn loop repeats settle sequences and nests bounded loops
- kind: DUP / INEFF
- where: launch/runtime.rs:271-313 and 366-417; runtime host.rs:104-138
- evidence:
  - After each event: `if run_requested_activations { turn.stabilize() } turn.stabilize()`.
  - `settle_user_effects` mixes `turn.stabilize()` and `rt.stabilize()` (the latter records no stage).
  - `settle_async` (up to 1000 rounds, each with stabilize) is called inside the commit loop (up to 100 rounds).
  - `check_draining` runs every turn.
  - Stage timing is recorded on every `stage()` call even with no recorder.
- fix: one module `launch/turn.rs` with
  ```rust
  fn settle(turn: &TurnGuard) {           // the only settle
      loop { turn.stabilize(); let d = turn.run_deferred(); let a = turn.poll_async_round();
             if !d && !a { break } }       // one shared 1000-round budget per turn
  }
  fn turn(..) { begin; for seg in segments { turn.run_mounts(seg.rev); for e in seg.events { dispatch; settle } }
                settle; loop { commit; if !rt.has_work() {break}; turn.run_mounts(done); turn.run_user_effects(); settle } finish }
  ```
  - Delete `TurnGuard::stabilize` vs `Runtime::stabilize` duality: only `TurnGuard` methods run work during a turn.
  - `check_draining` runs only while `ExitState::Draining` or when a queue's owner is disposed.
  - `record_stage` is a no-op unless the detail recorder is on.
- breaking: no. kpi: K3, K5. size: M. confidence: high

### RC-04 `Extensions` is a TypeId list used as glue, including for runtime-owned state
- kind: STRUCT / INEFF
- where:
  - runtime reactive/data.rs:551-579 (Vec scan), reactive/ext.rs:16-27, host.rs:44-54 (runtime installs its own `TaskState`, `MotionState`);
  - motion/state.rs:151-202 (`rt.ext::<MotionEnv>()`, installed by core at launch/runtime.rs:193);
  - core launch/runtime.rs:190-214 (10 extensions).
- evidence:
  - `ViewState::of(rt)` and `HostState::of(rt)` do an OnceCell read plus a linear 128-bit TypeId scan. They are called on every dispatch, hook and request.
  - Runtime motion panics ("extension was not installed") if core forgets `MotionEnv`, which is an implicit cross-crate contract (S-7).
- fix:
  - `Runtime` gets plain fields `tasks: TaskState`, `motion: MotionState`, `motion_env: OnceCell<MotionEnv>`; `RuntimeConfig` carries a builder for the env memos. It also gets `app: OnceCell<Box<dyn Any>>`.
  - Core defines `pub(crate) struct AppState { view, host, handle, foreign?, gpu, lifecycle, close, devtools: Option<..>, trace }` and `AppState::of(rt)` does one downcast (cache the `&AppState`).
  - Delete the `Extensions` public type, `Runtime::ext` and `missing_extension`.
- breaking: no (hidden). kpi: K3. size: M. confidence: high

### RC-05 Turn trace and causes are always recorded, and three trace systems overlap
- kind: INEFF / DUP
- where:
  - runtime reactive/diag.rs:1-30, 216-221 (`label.clone()`);
  - graph.rs `write_node` (record_write + `written`) and `recompute` (`record_run`);
  - core launch/runtime.rs:295 (`describe(event)`, a `format!` per event including pointer moves);
  - client/trace.rs:154 (`rt.turns().last()` clones 256 records per turn in debug).
- evidence: 256 `TurnRecord`s × (64 causes as `String`, 64 writes, 64 reruns) live in every release App. DetailLog (DevTools) and `client::trace` (debug) record the same facts again.
- fix:
  - One recorder: `DetailLog`, on only when DevTools, `kivact::test` `Screen::trace` or the debug trace sink enables it.
  - Release keeps counters (`turns`, `runs`, `errors`) only.
  - Causes are `Cause { tag: CauseTag, node: Option<NodeId>, event: &'static str }` (Copy), formatted on read.
  - Delete `TurnRecord.{causes,writes,reruns}`, the ring, `describe()` and `SourceNode.written` (moved to RC-06's side table).
- breaking: yes (`diagnostics::TurnRecord`/`Runtime::turns`, `Screen::trace`). kpi: K2, K3, K5. size: M. confidence: high

### RC-06 Graph nodes carry diagnostics and a redundant fat pointer
- kind: INEFF
- where: runtime reactive/data.rs (RNode, SourceNode, EffectNode, Computation); graph.rs:128-199 (`cell` and `compute` both point at one Rc)
- evidence:
  - RNode is about 160 B (arena slot about 168 B). 48 B of it are diagnostics: `at` 8, `ty: &str` 16, `written` 24.
  - Memos and effects store `cell: Rc<dyn Any>` plus `compute: Rc<dyn Compute>` for the same allocation.
  - Effects keep `cleanups: Vec` (24 B) inline even though almost none has cleanups.
  - There is no size assert, although MEM-6 requires one for per-node types.
- fix:
  - `RNode { owner: Key, owner_index: u32, data: NodeData }` with `NodeData::Source { cell: Rc<dyn Any>, subs: Links, changed_at }`, `Memo { comp: Rc<dyn Compute>, subs, deps, state }` and `Effect { comp, deps, state, kind, seq, cleanups: Option<Box<Vec<..>>> }`.
  - Add `Compute::as_any()` so one pointer serves both uses.
  - Diagnostics go to `Diag.node_meta: HashMap<Key, NodeMeta{at, ty, written}>`, filled only when recording. The `at` used by `Disposed` tombs stays (Tomb).
  - Add `const _: () = assert!(size_of::<RNode>() <= 96);`.
- breaking: no. kpi: K2. size: M. confidence: medium (sizes not measured)

### RC-07 Allocation and bookkeeping on every recompute and stabilize
- kind: INEFF
- where:
  - graph.rs:409-468 (`subs.iter()…collect::<Vec<_>>()` per changed memo);
  - frame.rs:39-71 (new `Vec` collector per run, plus a graph borrow to compute `writer`);
  - turn.rs:136-163 (`HashMap own_runs` and `limit = 100*(graph.len()+1)` per `stabilize`);
  - turn.rs:46-70 (`take_user`/`take_render` clone the BTreeSet);
  - registry `runtime()` clones an `Rc<Runtime>` per handle read (RC-T20).
- fix:
  - Reuse one `Runtime.dep_buf: RefCell<Vec<Key>>` pool (stack of Vecs) for collectors, and iterate `subs` by index under the borrow instead of collecting.
  - `tracking_frame` gets `writer` from its caller, which knows `kind`.
  - `stabilize` counts per-effect runs in a field on `EffectNode` (`runs_this_pass: u8`, reset when popped from the queue), not a HashMap.
  - `take_*` uses `std::mem::take` on the set.
  - Add `registry::with_runtime(app, |&Runtime| ..)` for handle reads, cloning only the cell Rc.
- breaking: no. kpi: K3, K5. size: S. confidence: high

### RC-08 MotionState scans every animation on each commit and tick; frame loops write a signal every frame
- kind: INEFF (PERF-1/PERF-4)
- where:
  - runtime motion/state.rs `plan_frame`/`wanted_frame`/`on_frame_tick`/`live_loops`/`live_anims`;
  - kivact-types cell.rs:239 (`Registry::live()` allocates a `Vec<Rc>`);
  - motion/frame_loop.rs (`set_run(Some(next))` every tick, the `rate` memo, and `frames` duplicating `Cadence.index`);
  - motion/animated.rs (`current.borrow().clone()` and `to_vec()` per step, `phase.get_untracked()` clones a Vec, a `running` memo per Animated).
- evidence:
  - Every commit round (several per turn) walks all animations, idle ones included, with two Vec allocations.
  - Each running loop writes a graph signal per frame. Each running Animated allocates at least 2 Vecs per frame. Each Animated has 4 graph nodes.
- fix:
  - `MotionState { running_anims: RefCell<Vec<Weak<dyn Anim>>>, running_loops: RefCell<Vec<Weak<LoopCell>>> }`, written by phase transitions only. `plan_frame` returns `None` when both are empty.
  - `LoopCell { rate: Signal<Option<FrameRate>>, cadence: Cell<Cadence> }`, so only start, stop and rate changes write the signal. Delete the `rate` memo and `frames`.
  - Animated components become `Comps { n: u8, v: [f64; 4] }` (Copy), stepped in place. Read the phase with `with_untracked`. Delete the `running` memo: `is_running()` is a `phase.with(..)` projection.
  - Keep `Registry<dyn Anim>` only for DevTools `animations()`.
- breaking: no. kpi: K3, K4, K2. size: M. confidence: high

### RC-09 DevTools does O(tree) work every turn and keeps per-node maps forever
- kind: INEFF / STATE
- where:
  - core devtools/tree.rs:155-206 (`notify_tree_changes` builds `ElementInfo` for all nodes and diffs `sent_tree`);
  - devtools/console.rs:43-58 (clones and sorts the whole error ring every turn);
  - devtools/turn.rs:73-113 (`sweep_closed_windows` every turn; its `sent_tree` retain is dead);
  - devtools/query.rs:254-287 (each waiting `find_*` re-searches the tree);
  - view/devtools_state.rs:51,62 (`forced`, `update_counts` keep removed nodes).
- fix:
  - The engine keeps a change journal while recording (`Reader::take_tree_changes() -> TreeChanges { upserted, removed }`). The tree notification, `find_*` re-evaluation and pruning of `forced`/`update_counts` use only that journal. A subscription starts with one full `get_tree`.
  - Add `Runtime::reported_since(seq)` (O(new)); `errors_sent` becomes `Cell<u64>`.
  - The closed-window sweep runs from `notify_windows_changed`'s closed set only. Delete the `sent_tree` retain.
- breaking: no. kpi: K3, K4 (DevTools on), K2. size: M. confidence: high

### RC-10 DevTools costs every App even when it is off: hooks, memos, agent start
- kind: INEFF / STRUCT
- where:
  - view/devtools_hooks.rs:30-47 (`prop_written`: `Runtime::current()` + ext scan on every prop write, also when `DevtoolsState::Off`), 375-395 (`StyleHook`: 2 memos + owner-chain walk per styled node while the agent runs), 338-356 (`wrap_capabilities` makes a copy memo even with the feature off);
  - launch/runtime.rs:174-180 → devtools/start.rs:12-56 (synchronous filesystem walk, TCP bind, discovery file and thread spawn before the root mounts; PERF-2);
  - client/dispatch/delivery.rs:28-44 (`record_input` on every event).
- fix:
  - `ViewNodes` caches `hooks: Option<Rc<dyn DevtoolsHooks>>` at construction; each hook is one `if let`.
  - Per-node style hooks become one App-level `Value<StyleOverrides>` (parsed once in `set_override`, keyed by NodeId/SourceLocation), read only when an override exists.
  - Delete `wrap_capabilities`: the engine applies touch emulation to the env it reports (it already rewrites `pointer.primary`).
  - The agent starts on a background thread: `AgentSlot::Starting(Receiver<Agent>)`, promoted on the first turn after it is ready.
- breaking: no. kpi: K1, K2, K3. size: M. confidence: high

### RC-11 Two exit paths, and two close-request APIs with different rules
- kind: DUP / API / CONTRA
- where:
  - core client/dispatch/env.rs:154-176 and launch/close.rs:24-28 (identical `request_exit(); ForeignWindows::close_all_for_exit(rt)`);
  - platform/close_request.rs:56-70 (`on_close_request(Fn() -> R)` uses `Owner::ambient`, App-wide);
  - platform/lifecycle.rs:33 (`on_lifecycle` uses `Owner::current`, setup-only);
  - secondary windows close through `WindowRecord.dismiss` (`on_dismiss(DismissReason::WindowClose)`); the main window record has `dismiss: None` (launch/runtime.rs:340).
- evidence:
  - The "user asks to close a window" event has two APIs: the main window uses a global registry, other windows use the window's `on_dismiss`.
  - `on_close_request` can be registered from a handler, while `on_lifecycle` cannot.
  - The payload-less handler takes `Fn() -> R` instead of `Into<Callback<()>>` (API-3).
- fix:
  - One `fn begin_exit(rt)` in `launch/close.rs`, called by both paths.
  - One API: `platform::on_close_request(f: impl Into<Callback<()>>)`, setup-only (`Owner::current`), scoped to the **calling owner's window**: the main window, or a `window()`/foreign window. `close_requested(window)` runs the handlers of that window's record, and for a non-main window then falls back to `dismiss`.
  - `WindowRecord.close_handlers: Registry<Registration>` replaces `CloseRequestState`.
- breaking: yes (signature, setup-only). kpi: -. size: S. confidence: high

### RC-12 `ClientMode` special cases are scattered through the turn and launch
- kind: STRUCT / DUP
- where:
  - launch/runtime.rs:179 (devtools multi-window flag), 243-244 (close acceptance), 288-290 (`closing.is_set()` break inside the event loop), 453-459 (`retain_module` in `Drop`);
  - embed/embedded.rs:150-158 (`Retention` calls `retain_module` again);
  - launch/inspect.rs:147-162 (`timing` pulled out of `ClientMode::Test` and passed separately);
  - settings.rs:39-41 (`devtools_in_tests` duplicates the mode).
- evidence: each site matches on the mode to derive one policy bit. After the embedded `closing` break, `acknowledge(events.len())` still runs mounts for undelivered segments.
- fix:
  - `ClientMode` → `struct LaunchPolicy { close: ClosePolicy /*Accept|ReportUnsupported*/, devtools: DevtoolsPolicy /*Off|On{multi_window}*/, timing: Option<Rc<dyn Fn(u64)>>, trace_name: Option<String>, closing: Option<CloseSignal> }`, built once by `App::run`, `App::embed` and kivact-test.
  - The `closing` check moves before the turn: a closing embedded App runs no turn at all.
  - Module retention lives only in the engine's `EmbeddedHost` (drop and failed open). Delete both core call sites.
  - Delete `AppSettings.devtools_in_tests` and its hidden setter.
- breaking: no (hidden). kpi: -. size: S. confidence: high

### RC-13 `App` duplicates a public `launch::AppOptions`; launch internals are public on kivact-core
- kind: API / DUP
- where: core app.rs:11-130 (every `App` method forwards to `AppOptions`); launch.rs:7-9 (`pub use inspect::{Inspect, LaunchHandle}`, `runtime::{ClientMode, Launch, TestTiming}`, `settings::AppOptions`); kivact-test/src/options.rs:41-63, screen/driver.rs:17-48.
- evidence:
  - Two builders with identical methods exist for one fact.
  - `Launch`, `ClientMode`, `Inspect` and `LaunchHandle` are `pub` without `doc(hidden)`/internal-contract markers, and are reached only by kivact-test.
  - `App: Clone` exists, and a second `run` of a clone panics ("one AppHandle drives one run").
- fix:
  - Delete `AppOptions`: `App` is the options value. kivact-test builds an `App` and calls a hidden `App::launch_test(policy) -> (Client, LaunchHandle)`.
  - Mark `Inspect`/`LaunchHandle` `#[doc(hidden)]` with an internal-contract reason and move them to `kivact_core::__test`.
  - Remove `Clone` from `App`, so `run(self)` makes a second run impossible at compile time. Delete the runtime panic path in `AppLink::begin` for `Ended`.
- breaking: yes (App: !Clone). kpi: -. size: S. confidence: high

### RC-14 i18n rebuilds ICU objects on every call
- kind: INEFF
- where: crates/kivact-i18n/src/regional.rs:22-195 (`format_date`, `parse_date`, `format_time`, `hour_cycle`, `month_name`, `weekday_name`, `plural_category`, `compare` each `try_new` per call); numbers.rs:42-62; locale.rs:85-90 (`icu()` re-parses the tag with `replace` + `parse` each time); callers kivact-ui calendar/caption.rs:23-56, plot/ticks.rs:127, sparkline.rs:162, column_values.rs:118 (`compare` per sort comparison).
- evidence:
  - Sorting a table builds a `Collator` per comparison, so O(N log N) collator constructions.
  - Plot ticks and sparkline labels build a `DecimalFormatter` per label per render.
  - A calendar builds name tables per cell.
- fix:
  - `Locale { tag: Arc<str>, icu: Arc<LocaleCache> }` with `LocaleCache { id: icu_locale_core::Locale, collator: OnceLock<Collator>, plurals: OnceLock<PluralRules>, decimal: OnceLock<DecimalFormatter>, names: OnceLock<..> }`. Equality uses the tag. Formatters with options are cached by option in a small `Mutex<Vec<(Options, F)>>`.
  - If an ICU type is not `Send + Sync`, use a `thread_local!` cache keyed by `(Arc<str>, kind)` instead.
  - Reads stay `&self` and keep PERF-2 laziness.
- breaking: no. kpi: K3, K5. size: S. confidence: high

### RC-15 Async state stores its value twice and clones it per read
- kind: STATE (S-4) / INEFF
- where: runtime task/async_state.rs (`AsyncCell { state: Signal<Rc<AsyncState<T>>>, latest: Signal<Option<T>> }`); resource.rs/action.rs `state()` (`.get().as_ref().clone()`); hidden `AsyncSnapshot` (lib.rs:45-47, core task.rs:18-20).
- fix:
  - One `Signal<AsyncCellState<T>>` with `{ phase: AsyncPhase /*Idle|Pending|Ready|Failed(UiError)*/, latest: Option<Rc<T>>, run: Option<RunInfo<I>>, waiting: usize }`.
  - Public `state()` builds `AsyncState<T>`; add `with_state(|s| ..)` to read without a clone.
  - Delete `AsyncSnapshot` and the sealed `AsyncSource::snapshot`. Async views listen to the signal.
- breaking: yes (hidden item, ui async views). kpi: K2, K3. size: M. confidence: high

### RC-16 Action keeps "accepting" in two places and side-channels through the state signal
- kind: STATE / DUP
- where: task/action.rs (`Runner::Latest(Rc<LatestRun>, Cell<Acceptance>)` vs `Queue.phase: ActionPhase::{Open,Closed,Detached(QueueDrain)}`, all with the same payload plus `parts()`/`parts_mut()`; `last: RefCell<Option<Submitted>>` "projection"; `async_state.state.with(|_| ())` used only to subscribe; `queue_status()` re-publishes `Pending` on every enqueue).
- fix:
  - `Queue<I> { mode, worker, waiting: VecDeque<Submitted<I>> }` plus one `ActionCell.lifecycle: Cell<Lifecycle { Open, Closed, Detached }>` for both runners.
  - `last`, `waiting.len()` and the attempt number go into RC-15's single reactive record.
  - Delete `Acceptance`, `ActionPhase`, `QueueDrain`, the dummy `with(|_| ())` subscriptions and the re-publish.
- breaking: no. kpi: K3. size: M. confidence: high

### RC-17 Machine transitions are not run as pure; fired timers stay registered
- kind: CONTRA / BUG
- where: runtime machine.rs:202 (`untrack(|| (cell.input)().transition(..))`, no `pure`); machine.rs:112, 207-221 (`timers: HashMap<Discriminant, Task>` keeps a finished task after it fires).
- evidence: architecture.md says rule 2, "transition は純粋に Step を返す", but a signal write inside `transition` is accepted silently.
- fix:
  - `let step = pure(|| untrack(|| (cell.input)().transition(..)))`.
  - `timers: HashMap<Discriminant, (Generation, Task)>`; the fire closure removes the entry if the generation still matches.
- breaking: no (rejects code that already violated the documented contract). kpi: K2 (minor). size: S. confidence: high

### RC-18 `use_direction()` leaks one constant per call outside a provider
- kind: BUG
- where: core view/boundary.rs:230-235 (`unwrap_or_else(|| owner.constant(ReadingDirection::Ltr, ..))`), called from the overlay Render effect view/overlays.rs:428 and from interaction/ui handlers (`data_table/rows.rs:241`, `list/input.rs:66`).
- evidence: `Owner::constant` inserts a ConstEntry into the owner's arena on every call, so it grows on every anchor move or scroll until the owner is disposed.
- fix:
  - `install_app_contexts` provides `Ltr` at the root, and the fallback is deleted.
  - Direction is a plain value (`provide_direction` takes a value), so the signature becomes `use_direction() -> ReadingDirection` (Copy context).
  - Add a runtime debug assertion: `Owner::constant` outside a Setup frame is a misuse.
- breaking: yes (return type). kpi: K2. size: S. confidence: high

### RC-19 One window declaration is stored three times, plus a duplicate window table
- kind: STATE (S-4) / DUP
- where:
  - view/foreign.rs:39-56 (ForeignHooks repeats title, open, size, min_size, mode, placement, resizable, dismiss) and 91-130 (`ForeignWindows` Weak registry, only for `close_all_for_exit`);
  - window_view.rs:220-234, 378-391;
  - state.rs:238-266 (`title`, `mode` `dead_code`);
  - launch/runtime.rs:322-361 vs window_view.rs:186-236 (two record builders);
  - tree.rs:422-438 (`ViewNodes::new` hard-creates the main root; `mount_window_root` branches on `existing`).
- evidence: this contradicts the architecture fact table: "開いている window | core の WindowRecord の表".
- fix:
  - `WindowRecord { decl: Rc<WindowSlot>, env, focus, hover: Signal<Option<FocusTarget>>, mouse: Signal<Option<Point>>, close_handlers }`.
  - ForeignHooks shrinks to `{ resizable, ready, resize, destroy, phase }` with plain `Option` callbacks.
  - One `open_window_record(..)` for every window, the main one included. Exit iterates `windows` where `foreign.is_some()`.
  - Delete `ForeignWindows`, `PointerObservation` (`Plain` exists only between insert and root mount), `hover::install`, `foreign_window_view` and the pre-created root.
- breaking: no. kpi: K2, K1. size: M. confidence: high

### RC-20 Core caches engine facts and keeps them in sync with flags and counters
- kind: STATE (S-5)
- where:
  - view/text_edits.rs + tree.rs:405, 693-702 + launch/runtime.rs:275 (`TextEditsAhead` counter: `InputValue` writes are dropped while undelivered `TextChanged` exist);
  - tree.rs:254-266, 525-548 + client/dispatch/env.rs:111-130 (`layout_seeds`);
  - state.rs:278, 572-576 + launch/runtime.rs:534-539 + client/state.rs:21 (`removed` list + O(F·R) `asset_failures.retain`);
  - tree.rs:399-402, 567-611, 680-702 (App-global visibility/live-region notifiers and ancestor walks, PERF-1);
  - launch/runtime.rs:132-133 + client/dispatch/env.rs:20-26 (system font enumeration at startup and on every env change, PERF-2).
- fix:
  - The engine owns each fact. `Writer::set(InputValue)` ignores a value while the model has an undelivered `TextChanged` for that node. `observe_node` seeds from `Reader::layout_report(id)`. Per-observed-node `visible`/`in_live_region` signals are written by an engine report.
  - Report-once of asset failures moves to a per-node `CoreRare` flag that dies with the node.
  - `font_families()` creates its value lazily; only a font-generation change writes it.
  - Delete `TextEditsAhead`, `TextEditsInFlight`, `layout_seeds`, `note_layout`, `ViewState.removed`, `HostState.asset_failures`, both global revision notifiers and the eager font calls.
- breaking: no. kpi: K1, K3, K5. size: M. confidence: medium (text-edit move needs engine work)

### RC-21 Request bookkeeping: an order Vec, a map and a stage kept in sync; three waiter maps
- kind: STATE / DUP
- where: view/state.rs:39-58, 203-215; view/requests.rs:27-52, 99-101, 125-200.
- evidence: `flush_focus` removes and re-inserts a request to flip `Queued→Sent`. Every host answer first goes through `resolve_foreign_size(id, result.clone())`.
- fix:
  - `FocusRequests { queued: Vec<FocusRequest>, sent: HashMap<RequestId, FocusReply> }`; delete `RequestStage` and `order`.
  - `waiters: HashMap<u64, Waiter>` with `Waiter::{Report, Future, ForeignSize{window,..}, Snapshot{..}}`; delete the clone.
- breaking: no. kpi: K3. size: S. confidence: high

### RC-22 Mounting cost: style recompiles everything, boundaries add extra fragments, provenance is kept always
- kind: INEFF / CONTRA (MEM-7)
- where:
  - view/mount/style.rs:107-158 (each run calls every layer closure, then `merge_layers`, `compile_style` and 4 cloned `update()`s; every focus stop gets the effect even with zero layers; a fixed theme is wrapped in a Signal, root.rs:63-66);
  - boundary.rs:50-193 (five copies; `scope(|| (a,b))` builds fragment→fragment);
  - tree.rs:311, 340, 684-691 (`origins`, `semantic_origins` on every node);
  - node_modifiers.rs:795-834 (`later_callers` O(M²·P)).
- evidence: one `node().style(s).role(..).focusable(true).name(move||..).on_press(..)` creates the following, i.e. 8–10 heap allocations before any text:
  - an engine node and a CoreNode;
  - `Box<Handlers>` (~220 B) and `CoreRare` + `Vec<Location>`;
  - an `origins` Vec;
  - 2 Render effects (an RNode, an Rc and a Box each), the style one also holding a dependency edge on the root theme.
- fix:
  - A memo `Rc<CompiledStyle>` over (layers, theme), plus a cheap state effect. Disabled/selected go to the engine's host-evaluated `States`, and values move into `PropValue` instead of being cloned.
  - A fixed theme is `owner.constant`; it becomes reactive only via a DevTools wrap.
  - Boundaries return `mount_in(..)` directly through one `fn boundary(cx, at, install, build)`.
  - Provenance goes to `ViewNodes.provenance: IdMap<Provenance>`, filled only while recording or under `cfg(debug_assertions)`.
  - Modifiers become one `Vec<Modifier>` resolved with a reverse suffix accumulator.
- breaking: no. kpi: K1, K2, K3, K5. size: M. confidence: medium

### RC-23 Measured `virtual_list` is O(N) per scroll; a writable list row edit clones the list
- kind: INEFF (PERF-1)
- where: view/flow/virtual_list.rs:624-680; view/flow/list.rs:383-405.
- fix:
  - Keep a Fenwick prefix of measured heights, updated when the list or a measurement changes, and binary-search `lo/hi` on scroll. Duplicate-key detection moves to the list-change pass.
  - A Signal source is edited in place with `signal.update(|l| l[idx] = v)`, located by the row's current index.
- breaking: no. kpi: K3, K4, K5. size: M. confidence: high

### RC-24 Selector creates nodes from untracked calls and repairs its key map by pruning
- kind: INEFF / STATE (repair)
- where: runtime reactive/selector.rs (`is` calls `create_source` with no tracking frame, `nodes.retain(node_alive)` on every call, `PruneSchedule`/`prune` scan all collectors and nodes).
- fix:
  - Without a tracking frame `is` returns `current == Some(k)`.
  - On node creation, register an owner cleanup that removes `(k, node)`.
  - Delete `PruneSchedule`, `prune`, `SELECTOR_PRUNE_MIN` and the `retain`.
- breaking: no. kpi: K3, K2. size: S. confidence: high

### RC-25 Timers: O(N) cancel and O(N) poll lookup
- kind: INEFF
- where: runtime task/state.rs `Scheduler::drain_cancel` (`retain` per id); task/time.rs `SleepState::Registered{id}` (`timers.keys().find`).
- fix: `SleepState::Registered { key: (u64 deadline, TimerId) }`, `CancelQueue(Vec<(u64, TimerId)>)` and `timers.remove(&key)`.
- breaking: no. kpi: K3, K5. size: S. confidence: high

### RC-26 Owner disposal boxes every released member
- kind: INEFF
- where: runtime reactive/owner.rs `dispose_members` (`graveyard.push(Box::new(node))`, `Box::new(obj.data)`; `sort_by_key(rank)` borrows the graph per comparison).
- fix: `struct Graveyard { nodes: Vec<RNode>, data: Vec<Rc<dyn Any>> }` reused via `Runtime.graveyard: RefCell<Graveyard>`. Compute ranks once into `Vec<(u8, Key)>`.
- breaking: no. kpi: K3, K5. size: S. confidence: high

### RC-27 DevTools state lives in `view`, with about 20 cfg stub pairs and a second style parser
- kind: STRUCT / DUP
- where:
  - view.rs:17-34; view/devtools_state.rs, devtools_apply.rs, inspect_mode.rs, devtools_hooks.rs;
  - devtools_apply.rs:12-299 (its own property table, plus parsing kivact-style `Debug` output back);
  - review.rs:209-224 (scope matching duplicated);
  - view/boundary.rs:142-157 (`provide_locale` is not wrapped by `wrap_locale`, so locale emulation misses provider subtrees, while `provide_theme`, boundary.rs:111, is wrapped).
- evidence: architecture.md:67 says devtools sits above view, but `view` imports the agent and protocol crates.
- fix:
  - Move `devtools_state`, `devtools_apply` and `inspect_mode` to `devtools/`. `view/devtools_hooks.rs` keeps only `pub(crate) trait DevtoolsHooks { prop_written, input_event, handler_ran, focus_changed, focus_requested, host_event, style_overrides, wrap_theme, wrap_locale, pseudo }` over core and types values, stored as `AppState.devtools: Option<Rc<dyn DevtoolsHooks>>`.
  - kivact-style owns one `internal::StyleProperty { name, describe, parse }` table.
  - `install_locale` and `install_theme` apply the wraps in one place.
- breaking: no. kpi: K6, maintainability. size: M. confidence: high

### RC-28 The input source is guessed from pending injections
- kind: BUG
- where: view/devtools_hooks.rs:146-153; devtools/input.rs:186-194, 263-276.
- evidence: real input that arrives while an `inject_input` call is pending is logged as `Injected`.
- fix: routed `HostEvent`s carry `origin: InputOrigin { Platform, Injected(RequestId) }` from the engine injection path. Delete the scan.
- breaking: no. kpi: -. size: S. confidence: medium

### RC-29 `draw::Offscreen` is a clear leaf inside core
- kind: STRUCT
- where: core draw/offscreen.rs (374 lines; uses only `crate::draw::Draw`, `crate::geometry::Size`, kivact-style, kivact-engine `OffscreenRenderer`/`TextMeasurer`, kivact-svg); core Cargo.toml (`kivact-svg` exists only for it).
- evidence:
  - Inputs and outputs are explainable by few types: a `Theme`, fonts and a paint closure go in; `Pixels` or `String` comes out. It does not peek into view or runtime.
  - Its API is inconsistent: `render(u32, u32)` vs `render_svg(f64, f64)`.
- fix:
  - New crate `kivact-offscreen` (deps: kivact-style, kivact-engine api, kivact-svg, kivact-types), re-exported as `kivact::draw::Offscreen`. kivact-core drops the kivact-svg dependency.
  - Both methods take `(width: f64, height: f64)` logical size plus `.scale(f64)` for pixel density.
- breaking: yes (render arguments). kpi: K6 (core), maintainability. size: S. confidence: high

### RC-30 The runtime's host-driver API is public, unmarked and wide
- kind: API / STRUCT
- where: runtime lib.rs:22-31 (public `Runtime`, `TurnGuard`, `Stage`, `Pending`, `ExitState`, `TurnReport`, `Host`, `ThreadWaker`, `Extensions`, `RuntimeConfig`, `LiveSnapshot/LiveOwner/LiveObject/OwnerInfo`, `CauseTag/CauseEntry/Detail*`, `Kind`, `EffectKind`, `NodeInfo`, `Turn*`).
- evidence: none of these are used by apps. They are the core↔runtime turn contract, yet they appear in kivact-runtime docs next to `signal`. `TurnGuard::cause_with` is the only one marked internal-contract.
- fix:
  - Move them under `kivact_runtime::driver` (doc(hidden), one internal-contract marker on the module).
  - The facade exposes only the user names. `diagnostics` re-exports only the new typed `Cause`/counter types (RC-05).
  - `MotionState`/`TaskState` stay `pub(crate)` with `Runtime::{tasks(), motion()}` accessors in `driver`.
- breaking: no for apps (hidden), yes for kivact-test internals. kpi: -. size: S. confidence: high

### RC-31 Two `Owner` types and two context stores
- kind: API / DUP
- where: runtime reactive/owner_handle.rs (`Owner::provide<T>`/`context<T>` store raw `T`); reactive/setup.rs:191 (`use_context::<T>()` looks up `Value<T>`); core view/owner.rs (second public `Owner`: store, store_with, provide, context, report, on_cleanup), state.rs:3.
- evidence: a value provided one way is invisible to the other (different TypeId keys).
- fix:
  - One user API: `provide(value, || view)` + `use_context::<T>() -> T` (`T: Clone`; users provide a `Signal`/`Value` when they want reactivity).
  - The core wrapper keeps `current`, `root`, `on_cleanup` and `report`; `store`/`store_with` become doc(hidden) custom-component internals.
  - Internal crates use the runtime `Owner::provide/context` with the same key scheme.
- breaking: yes. kpi: -. size: M. confidence: medium

### RC-32 Six conversion traits for one idea ("a prop")
- kind: API
- where: runtime reactive/props.rs (`IntoValue`, `IntoText`, `IntoReactive` marker, `Reactive<T>` stored prop, hidden `Prop`, `PropValue`, `constant_value!`, `IntoValue<Rc<str>>`); handles.rs `IntoBinding`; macros/src/expand.rs:226-262 (generated code names `::kivact::custom::Reactive`).
- evidence:
  - A JS developer meets `IntoValue<T>`, `IntoText`, `IntoReactive<T>`, `IntoBinding<T>` and `Reactive<T>` for what React calls a prop.
  - Every literal `into_value()` allocates a ConstEntry + Rc and forces the owner's `OwnerExtra` box (RC-T14).
- fix:
  - Public input traits are `IntoValue<T>` and `IntoBinding<T>` only. `IntoText` becomes `IntoValue<String>` (closures returning `impl Into<String>` via the `Fn` impl).
  - Delete `IntoReactive` (flows take `IntoValue<bool>`).
  - Delete `Reactive<T>`: store `Prop<T>`, made public with `get`/`get_untracked`. The macro stores `Prop<T>` and converts in `scope`.
  - Core mount and builders accept `Prop<T>` so a Static stays inline, with no const arena slot.
  - Delete the `IntoValue<Rc<str>>` impls.
- breaking: yes. kpi: K2, K5. size: M. confidence: medium

### RC-33 `#[component]` callback props: a box, then a callback object, per prop
- kind: INEFF / API
- where: macros/src/expand.rs:229-235 (`Box<dyn FnOnce() -> Callback<E>>`), 260-262 (`callback(#input)` inside the box), 172-178 (optional props as `Option<..>` with `.map(|f| f())`).
- evidence:
  - Each handler prop costs a Box, an `Rc<CallbackFn>` and an object arena slot. The body sees `Callback<E>` (Copy), which is the useful part.
  - Required handlers are positional by API-3, so `card("t", |_| ..)` reads poorly for JS users.
- fix:
  - Store `Handler<E>` (runtime's one internal handler type, `Rc<dyn Fn>`) and bind the owner lazily: `Callback::from_handler(owner, h)` creates the object only when the body copies it into a child. The body's parameter type stays `Callback<E>`.
  - Keep required handlers positional (rule).
- breaking: no. kpi: K2, K1. size: S. confidence: medium

### RC-34 FrameLoop exists twice; the presence-gate effect does nothing
- kind: DUP / DEAD
- where: core motion/frame_loop.rs (wrapper adds `while_present(NodeRef)`); runtime motion/frame_loop.rs `with_presence_gate` (doc(hidden), creates a user `effect` that calls `rt.request_turn()`, a no-op inside a turn).
- fix:
  - Delete the core wrapper. Runtime `FrameLoop::while_present(self, present: impl IntoValue<bool>)` becomes public; users write `.while_present(move || node.is_present())`.
  - Delete the effect.
- breaking: yes. kpi: K2. size: S. confidence: high

### RC-35 Three shapes for one async result
- kind: API / DUP
- where: runtime task/action.rs (`ActionOutcome<T>{Succeeded,Failed}`), `AsyncState<T>`, `Result<T,UiError>`; task/async_publication.rs (`Retry`); `ActionFn<T,M>`/`BlockingWork<R,M>` marker generics re-exported at core task.rs:9-12; `spawn_in(owner, fut, at: &Location)` public (API-8); `task.rs` doc says "four async entries" while seven are public.
- fix:
  - `on_complete(Fn(Result<T, UiError>))`; delete `ActionOutcome`.
  - `AsyncSource` keeps `state()` and `retry()`; `Retry` folds into `Callback<()>`.
  - `ActionFn`/`BlockingWork` become doc(hidden) and leave the facade.
  - `spawn_in` becomes `pub(crate)` plus a hidden bridge for core. Fix the doc.
- breaking: yes. kpi: -. size: S. confidence: high

### RC-36 Constant-like signals written every turn and phase flags with contradicting docs
- kind: CONTRA / STATE
- where: launch/runtime.rs:271 (`initial_render.set(false)` every turn); state.rs:278 ("until the first frame is drawn"); platform/env.rs:42-50 ("after the first host revision"); state.rs:283-287 (`current_window` silently falls back to MAIN, while architecture forbids implicit main fallbacks, and root provides `WindowScope(MAIN)`).
- fix:
  - `initial_render` is written once by `start_mount` after `finish_setup`, and both docs say "until the first turn after setup".
  - `current_window` uses `expect("invariant: every owner is under a WindowScope")`.
- breaking: no. kpi: -. size: S. confidence: high

### RC-37 The exit drain has three deadline mechanisms for one need
- kind: STRUCT (over-generalisation)
- where: runtime task/drain.rs; task/state.rs (`queues: Registry<dyn Drain>`, `draining`, `check_draining`, `queues_busy`, `abandon_all_queues`); host.rs:59-99 (`ExitState::Draining`); action.rs (`close()`, `CloseFuture`, `drained` wakers), `DEFAULT_DRAIN_NS`.
- fix:
  - One mechanism: a queued action whose owner is disposed moves to `TaskState.draining`, and App exit waits for `draining.is_empty()` with one deadline.
  - Delete `Action::close(deadline)`, `CloseFuture`, the `drained` wakers and the `Closed` lifecycle (see RC-16). Users call `cancel()` or await `done()`.
- breaking: yes. kpi: K3 (per-turn check). size: M. confidence: medium

### RC-38 Thread-retention lists exist three times, and two types are named `Completion`
- kind: DUP
- where: task/state.rs `fallback_threads`/`retain_fallback`; worker_pool.rs `Pool.fallbacks`; blocking.rs `start` (spawns a thread when there is no pool); public `task/completion.rs::Completion` vs private `blocking.rs::Completion` (VOC-3).
- fix:
  - `blocking()` always uses `TaskState::blocking_pool()`. Outside an App it raises a located `WrongApp`, as `sleep` does.
  - Delete `fallback_threads`/`retain_fallback`. Rename the private type `JobGuard`.
- breaking: yes (blocking outside an App errors). kpi: -. size: S. confidence: high

### RC-39 `Reported.handled` mislabels diagnostics
- kind: CONTRA
- where: runtime reactive/diag.rs `reported()` (sets `handled: true` for every diagnostic, `false` for default-sink errors, and never records errors caught by `on_error`).
- fix: replace the field with `kind: ReportKind { Error, Diagnostic }`, and map it in devtools/console.rs.
- breaking: yes (hidden DevTools mapping). kpi: -. size: S. confidence: high

### RC-40 `state-fields` markers are mostly noise, and some excuses are wrong
- kind: PROCESS
- where:
  - 147 `xtask-allow(state-fields)` in core and runtime (111 core, 36 runtime) plus 7 in i18n;
  - "option:" used on bool fields: view/mount/create.rs:31, element_data.rs:554/559, editing_data.rs:106/110, style.rs:73, facts.rs:13;
  - a check dodge: view/flow/virtual_list.rs:596-623 (`Held::{Nothing, Holding}` = `Option<K>` renamed);
  - runtime data.rs `OwnerEntry.parent` ("independent"), whose None-ness is the "detached disposal root" phase together with `OwnerPhase::Disposing`;
  - host_types.rs:3-19 (`Pending` 4 bools: a snapshot, but `turn_requested` mirrors `TurnState`);
  - `ext: OnceCell` ("never observed empty", an S-7 dodge deleted by RC-04).
- fix:
  - Narrow the rule to types stored across turns (Cell/RefCell fields, extensions, arena payloads). By-value builder and mount-input structs (`Parent`, `AppSettings`, `element_data`, `editing_data`) are exempt structurally, so delete their markers.
  - Replace `Held` with `Option<K>`.
  - `OwnerEntry` keeps `parent`, and detachment is `OwnerPhase::Disposing { round, detached: bool }`.
  - Delete `Pending.turn_requested` (RC-02).
- breaking: no. kpi: -. size: S (code) / M (xtask). confidence: high

### RC-41 Smaller items (all S, no breaking unless noted)
- **DEAD** runtime reactive/data.rs:82 + mounts.rs:12-15: `mount_revision` is read only by an assert. Assert against `awaiting.back()` and delete the field.
- **DEAD** core view/devtools_hooks.rs:264-271: the `is_running` stub (only caller is cfg-gated).
- **DEAD** view/mount/style.rs:33-45: `bind_states` is a no-op. Also delete the unused `_kind` (resolve.rs:239) and `_at` (node_ref.rs:107) parameters, `Leaving.on_exit: Option` that is always Some (tree.rs:978-980), and the `dead_code` fields `host_capabilities`, `title` and `mode` (state.rs:238-285). builders.rs:263-267 sets `n.at = Some` and then `None`: pass `Option<&Location>` instead.
- **STATE** view/mount/resolve.rs:208-212: a no-op selection handler is installed only to make `report_session` true. Use `report_session = selection.is_some() || draw_with_layout.is_some()`.
- **STATE** view/state.rs:69-136: `EnvState::new` eagerly makes 10 field memos per window behind a string-keyed `Box<dyn Any>` map (its doc says "made on first use"). Use `EnvFields { fullscreen: OnceCell<Value<bool>>, … }` with typed lazy getters (K1, K2).
- **DUP** view/flow/branch.rs:197-660: `show`, `show_some` and `dynamic` are three copies of one slot machine. Use one `BranchSlot<Key>` with `Arm::{Pending, Mounted(key, Option<Branch>), Leaving{next}}` (K6, M).
- **INEFF** core gpu_interop/import/registry.rs:16-30: every App installs `gpu_interop::State` with a signal even without the feature. Create it lazily on first `adapter()`/`import`.
- **INEFF** platform/app_handle.rs: `take_request` locks a Mutex every turn. Add `AtomicBool requested` as a fast path and lock only when it is set.
- **CONTRA** architecture.md:290 says the turn order is owned by "core の AppClient". No such type exists; it is `launch::Launch::turn`. Rename the doc (or the type `Launch` → `AppClient`, with `turn.rs` holding the loop, RC-03).
- **API** view/devtools_state + launch/settings.rs:39-41: `devtools_in_tests` duplicates `ClientMode::Test` (folded into RC-12).
- **CONTRA** runtime host.rs:68-71: `report_now` mutates `TurnState` (sets `Running(Output)`). Fold into `finish`/`end_idle` (RC-01).
- **INEFF** `Notifier::notify_from_render` (reactive/trigger.rs:70) exists only for core's revision notifiers (tree.rs:621/709/725, mount/style.rs:211). Deleted with RC-20; then delete the method.
- **API** `Owner::ambient` vs `Owner::current`: the two names do not say "setup only" vs "any owner". Rename to `Owner::setup()` and `Owner::running()` (hidden API, internal crates).
- **i18n**: the only runtime dependency is `constant_value!`. When RC-32 lands, constants become `Prop::Static`, which needs no trait impl. If `constant_value!` stays, keep the dependency (cycle-free); no crate move is needed.

## 3. Target structure for this area

```
kivact-runtime        (no core knowledge; style/i18n/core/test depend on it)
  reactive/           arena, graph (RNode ≤96 B), frames, owners, handles, props (Prop/IntoValue/IntoBinding),
                      turn (TurnState, one deferred queue, has_work), diag (counters only; DetailLog recorder opt-in)
  task/               executor, timers (keyed by (deadline,id)), spawn, resource/action over one AsyncCellState,
                      blocking pool (no fallback threads), drain (one mechanism)
  motion/             MotionState with running-sets written by transitions; Animated (Copy comps); FrameLoop (+while_present); Timeline
  machine.rs          pure transitions, generation-checked timers
  driver/ (doc hidden) Runtime, TurnGuard, Stage, Pending, ExitState, TurnReport, Host, ThreadWaker, RuntimeConfig{motion_env builder}
kivact-i18n           Locale{Arc<str>, Arc<LocaleCache>} with lazily built ICU objects; Messages; pure date math
kivact-macros         #[component] storing Prop<T>/Handler<E>
kivact-offscreen  NEW Offscreen (PNG/SVG); deps: style, engine api, svg, types
kivact-core
  app.rs              App (the only builder; !Clone); App::run / App::embed / hidden App::launch_test
  launch/             turn.rs (the single ordered loop: RC-03), policy.rs (LaunchPolicy: RC-12), close.rs (begin_exit), inspect.rs (hidden __test)
  state/  AppState    one struct, one downcast (replaces Extensions)
  view/               node table, mount, flows (one BranchSlot), windows (one WindowRecord), overlays, requests (typed queues),
                      devtools_hooks.rs = trait only
  client/             dispatch (no activation queue; uses runtime defer)
  platform/           env (typed lazy fields), services, AppHandle, lifecycle, on_close_request per window
  devtools/           all DevTools state (session, apply, inspect mode), journal-driven
  embed/, gpu_interop/ (lazy State), custom/, diagnostics/ (why + Stats + typed causes)
```
- Boundary justifications:
  - **runtime**: input is closures and keys, output is values and handles. It knows neither views nor host events, and style/i18n need it without core.
  - **offscreen**: in = `Theme` + fonts + paint closure, out = `Pixels`/`String`. It never reads view or runtime state.
  - **core stays one crate**: view, client, platform and devtools share the node table internals. Every split tried (view|client, core|devtools) needs about 20 private items made public. That is a fake boundary per B-1.
- Deleted:
  - runtime: `Extensions`, `Runtime::ext`, the TurnRecord ring fields, `describe()`, `AsyncSnapshot`, `ActionOutcome`, `Acceptance`/`ActionPhase`/`QueueDrain`, `CloseFuture`, `PruneSchedule`, `fallback_threads`, `IntoReactive`, `Reactive`, `mount_revision`, `notify_from_render`;
  - core: `AppOptions` (public), `ForeignWindows`, `PointerObservation`, `TextEditsAhead`, `layout_seeds`, `ViewState.removed`, `HostState.asset_failures`, `RequestStage`/`order`, the core `FrameLoop` wrapper, `wrap_capabilities`, ~20 devtools cfg stubs, `devtools_apply`'s parser, `Held`, and about 100 `state-fields` markers.

## 4. Public API

Worst friction, before → after:

```rust
// Prop inputs (RC-32)
fn badge(label: impl IntoText, on: impl IntoReactive<bool>) -> ...      // before
fn badge(label: impl IntoValue<String>, on: impl IntoValue<bool>) -> ... // after

// Context (RC-31): two stores today
Owner::current(..).provide(Theme(..)); use_context::<Value<Theme>>()      // before (two schemes)
provide(theme_signal, || view);  let t: Signal<Theme> = use_context();    // after (one)

// Close request (RC-11)
on_close_request(move || if dirty.get() { Handled::Yes } else { Handled::No });   // main window only, any phase
window(..).on_dismiss(|reason| ..)                                                 // other windows
// after: one, setup-only, scoped to the caller's window
on_close_request(move |_| if dirty.get() { Handled::Yes } else { Handled::No });

// Async outcome (RC-35)
action(..).on_complete(|o: ActionOutcome<T>| match o { Succeeded(v) => .., Failed(e) => .. })
action(..).on_complete(|r: Result<T, UiError>| ..)

// Frame loop gated by presence (RC-34)
kivact::motion::frame_loop(..).while_present(node_ref)            // core wrapper type
frame_loop(..).while_present(move || node_ref.is_present())        // one FrameLoop

// Direction (RC-18)
let d = use_direction().get();   →   let d = use_direction();
```

- **Delete:** `launch::AppOptions` (public), `IntoReactive`, `custom::Reactive`, `IntoText` (becomes `IntoValue<String>`), `ActionOutcome`, `Retry`, `AsyncSnapshot` (hidden), `Action::close`, core `motion::FrameLoop` wrapper, `IntoValue<Rc<str>>` impls, and `ActionFn`/`BlockingWork` from the facade. The diagnostics `TraceRun`/`TraceWrite` are replaced by typed causes.
- **Make hidden or move:**
  - to `kivact_runtime::driver`: Runtime, TurnGuard, Stage, Pending, ExitState, TurnReport, Host, ThreadWaker, RuntimeConfig, Live*, Cause*/Detail*;
  - to `kivact_core::__test`: Launch, ClientMode→LaunchPolicy, TestTiming, Inspect, LaunchHandle;
  - `spawn_in`.
- **Rename:** `Owner::ambient/current` → `Owner::running/setup` (internal); `ClientMode` → `LaunchPolicy`.
- **Signature changes:** `App: !Clone`; `on_close_request(impl Into<Callback<()>>)` is setup-only; `use_direction() -> ReadingDirection`; `Offscreen::render(f64, f64)`; `blocking()` outside an App returns `WrongApp`.

## 5. Open questions for the owner

1. **Diagnostics in release (RC-05/06).** Should release builds keep any structured turn trace? Proposal: counters only, with the recorder enabled by DevTools, test or debug. This changes `Screen::trace` and `Runtime::turns`.
2. **DEP-4 exception for DevTools.** Should the runtime `detail.rs`, the engine DevTools requests and events, and kivact-style `describe` compile only under a `devtools` feature (K6), contrary to "intermediate layers always compile"?
3. **Engine change journal (RC-09/RC-20).** Do you accept a small always-compiled cost in the engine mutation path, gated at runtime by recording? It is needed for O(changed) DevTools and core observation.
4. **`Action::close(deadline)` (RC-37).** Delete it in favour of the one exit-drain mechanism, or keep it as a user feature?
5. **Context API (RC-31).** Should `use_context::<T>() -> T` (`T: Clone`) replace `use_context::<Value<T>>`? This changes every kit's provide/use pair.
6. **Close request scope (RC-11).** Should `on_close_request` become per-window and replace `on_dismiss(DismissReason::WindowClose)` for windows, or stay main-window-only with the two APIs kept distinct and documented?
