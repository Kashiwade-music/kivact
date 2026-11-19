# 付属書 L：状態の台帳の閉じ方（`legacy:` の印）

版 1.0（2026-11-19）。基になる台帳は `rework/state-ledger.md`（CP1、194 件）。根拠は設計書 6.1 節・12 章 ST-01・DEC-17。調べたコードは統合 branch の `5d46cd5f`（W5a の E1・E2 と C04 の修正の後）。本書の行番号は書かない。場所は型名・関数名と file で示す。

## 1. 範囲と前提

### 1.1 目的

W1 の印の変換が機械的に作った `// xtask-allow(state): legacy: ...` の印を、すべて閉じる。閉じ方は次の 4 つのどれか（設計書 6.1 節）。

| 処置 | 意味 |
|---|---|
| **書く** | 組合せがすべて有効なことを確かめた。印の文を、閉じ方を表す 1 行に書き直す |
| **変える** | 無効な組合せか、同じ事実の二重の保存がある。型を直す（本書の手順）。その後で印を書くか消す |
| **他 unit** | 後の unit（付属書 A・C・E・R・S・U・K）が型を直す、消す、または自分の印を書く。L の unit は、その unit の後に `legacy:` が残っていないことだけを確かめる |
| **外す** | 印ではない（xtask 自身が `legacy:` を探すための文字列）。最後の unit（L12）が xtask の仕組みと一緒に消す |

印の文の形：`// xtask-allow(state): ` に、本書の表の「印の文」を続けた 1 行。文は次の接頭辞のどれかで始まる：

- `independent:`：組合せがすべて有効。
- `record:`：互いに関係の無い記録・表の集まり。
- `cache:`：cache。
- `identity:`：id・世代を配るだけ。
- `statistic:`：統計。
- `projection:`：別の field の投影で、書き手は一つ。

### 1.2 前提

- W8 の全 unit が merge されている（L1〜L6）。W10 の各 kit の lane の最後の unit が終わっている（L7〜L11）。L1〜L11 がすべて merge されている（L12）。
- 台帳 `rework/state-ledger.md` は CP1 の時点のもの。その後の unit が型を消す、移す、改名する、field を足す・消す、印の文に節を足すことがある。2 章の共通手順は、これを機械的に扱う。

### 1.3 調べて分かったこと（設計者の判断の記録）

- 194 件の内訳：書く 176 件。変える 5 件（行 31・32・37 の型の別名、行 54 の cache、行 71 の重複）。他 unit 8 件（そのうち行 7・39 は、その unit の後に L が印を書く）。外す 5 件。
- kivact-ui の builder の struct（台帳の 131〜165 の大半）が検査に当たるのは、`Element::node_mut(&mut self)` があるから。field はすべて値渡しの setter だけが書く。W8 の U の unit（U23・U40・U44〜U54）が、これらの struct を `#[component]` にして消す。L6 の時点では、ほぼ残らない見込み。
- kit の builder（96〜130）も同じ理由で検査に当たる。W9〜W10 の K の unit が `#[component]` にして消す見込み。
- 台帳の 167〜171 は印ではない：xtask の `LEGACY` 定数、件数の note、xtask の試験の fixture。
- 型の別名（`Build`、`SecondaryContextHandler`、`SessionHandler`）は、xtask の `is_setting` に setting（`ViewFn`・`Handler`）だと分からない。そのため、constructor だけが書く field が状態に数えられる。L1 は別名を使わずに型を書き、印を消す。
- `EditorNode`（engine）：font の登録で `lines` は捨てるが、`measure_lines` は捨てない。幅の測定が古い shaping を使いうる不具合で、L3 で直す。

---

## 2. 共通手順 L（どの L の unit も、自分の表の行ごとに行う）

1. **残っている印を集める**：`rg -n "xtask-allow\(state\): legacy:" <unit の所有>` を実行する。
2. **印と台帳の行を対応させる**：
   - 印の `legacy:` の後の field の並び（最初の `;` まで）で、本書の unit の表の行を探す。台帳の「印の文面」と同じ並びの行が、その印の行。
   - 型・関数が移動・改名されていても、field の並びで対応させる。
   - 対応する行が無い印は**差戻し**（CP1 の後に足された `legacy:` の印。足してはいけない）。
3. **表に行があるのに印が無い**：型が消えたか、後の unit が閉じた。何もしない。unit の報告に「GONE」と書く。
4. **処置が「変える」**：unit の手順に従って型を直す。その後で 5 を行う。
5. **処置が「書く」**：印の行を、`// xtask-allow(state): ` と表の「印の文」の 1 行に置き換える。そのとき、機械的に次を直す：
   - **台帳の後に足された節を残す**：今の印の文に、台帳の「印の文面」に無い節（後の unit が `; ...` で足した節）があれば、新しい文の最後に `; ` で続ける。
   - **無くなった field を消す**：新しい文が名指す field が型に無ければ、その名前とその field について述べた語句を消す（文の意味が残るようにする）。
   - **新しい field は差戻し**：型に、状態に数える field（`bool`・`Option`・名前が counter のもの・`Cell`・`RefCell`・`Mutex`・`RwLock`・`Atomic*`・`Signal`・`Var`）が CP1 の後に増えた場合を扱う。新しい文にも、残した節にも、その field の説明が無ければ**差戻し**。
6. **処置が「他 unit」**：印が残っていれば**差戻し**（その unit が型を直していない）。
7. **検査**：`cargo xtask check --no-write --crate <crate>` を実行する。
   - その型の印が「未使用の印」と報告されたら、印を消す（作業指示書 0.5 節の表の行）。
   - 状態の検査が、本書に無い型を報告したら**差戻し**。
8. **報告**：表の行ごとに、次のどれかを書く。
   - 書いた（節を残した・field を消した場合はその旨）
   - 消した（未使用の印）
   - 変えた
   - GONE
   - 差戻し

## 3. unit の一覧

| ID | wave | lane | 前提 | 所有 | 台帳の行 | 規模 |
|---|---|---|---|---|---|---|
| L1 | W8L | L1 | W8 | `crates/kivact-core/` | 21〜39 | S |
| L2 | W8L | L2 | W8 | `crates/kivact-runtime/`、`crates/kivact-interaction/` | 63〜69、71〜82 | M |
| L3 | W8L | L3 | W8 | `crates/kivact-engine/`、`crates/kivact-layout/`、`crates/kivact-editing/` | 45〜61、70 | S |
| L4 | W8L | L4 | W8 | `crates/kivact-style/`、`crates/kivact-i18n/`、`crates/kivact-test/`、`crates/kivact-devtools/`、`crates/kivact-devtools-protocol/` | 40〜44、62、83〜95 | S |
| L5 | W8L | L5 | W8 | `crates/arvane-*/`、`dev/tests/gpu-interop-producer/` | 1〜20、166 | S |
| L6 | W8L | L6 | W8 | `crates/kivact-ui/`、`examples/kivact/` | 131〜165、194 | S |
| L7 | W10 | K1（K35 の後） | K35 | `crates/kivact-ui-basic/`、`examples/kivact-ui-basic/` | 96〜102、172〜177 | S |
| L8 | W10 | K2（K37 の後） | K37 | `crates/kivact-ui-pop/`、`examples/kivact-ui-pop/` | 108、188〜193 | S |
| L9 | W10 | K3（K38 の後） | K38 | `crates/kivact-ui-creative/`、`examples/kivact-ui-creative/` | 103〜106、178〜179 | S |
| L10 | W10 | K4（K40 の後） | K40 | `crates/kivact-ui-game/`、`examples/kivact-ui-game/` | 107、180〜187 | S |
| L11 | W10 | K5（K43 の後） | K43 | `crates/kivact-ui-soft/`、`examples/kivact-ui-soft/` | 109〜130 | S |
| L12 | W11a | L | L1〜L11 | `dev/xtask/src/{local_checks.rs,state.rs,state/check.rs,state/tests.rs}` | 167〜171 | S |

kit の行（L7〜L11）を W8L ではなく W10 に置いた理由：kit は W6a〜W10 の間 compile の対象外で、W9〜W10 の K の unit が書き直す。W8L で閉じても、K の unit がその型を作り直す。

---

## 4. unit の手順

### 各 unit の共通の確認（tier a）

各 unit は、自分の所有の crate ごとに次を実行する（L12 は `dev/xtask` だけ）。

```sh
cargo fmt -p <crate>
cargo clippy -p <crate> --all-targets -- -D warnings
cargo xtask check --no-write --crate <crate>
```

完了条件（L1〜L11）：`rg -n "xtask-allow\(state\): legacy:" <unit の所有>` が 0 件。上のコマンドが通る。差戻しの行は残してよい（差戻しとして書く）。

## L1：kivact-core の印を閉じる

- **wave / lane / 前提**：W8L / L1 / W8 の全 unit
- **所有**：`crates/kivact-core/`
- **手順**：
  1. 型の別名に隠れた setting（行 31・32・37）：
     - `view/flow/branch.rs` の `ShowSlot` と `SomeSlot` の field `fallback` の型を `Option<crate::view::ViewFn<()>>` と書く（別名 `Build` を field に使わない。別名を他で使っていれば別名は残す）。
     - `view/tree.rs` の `Handlers` の `secondary_context` を `Option<Handler<PointerEvent>>`、`selection` を `Option<Handler<TextSession>>` と書く（`SecondaryContextHandler` と `SessionHandler` を field に使わない。使う所が無くなった別名は消す）。
     - 3 つの型の印を消す。xtask が「状態の field が 2 つ以上」と報告したら差戻し。
  2. 他の行は共通手順 L。
- **確認**：共通の確認に加えて、次の 2 つ。
  ```sh
  cargo clippy -p kivact-core --all-targets --features devtools -- -D warnings
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- flows view
  ```

| # | 項目 | file | 処置 | 印の文 |
|---|---|---|---|---|
| 21 | `HostState` | `client/state.rs` | 書く | `independent: stats is the statistic of turns and stale events (added by the turn and dispatch); asset_failures is the report-once set of asset failures, inserted by asset_failed and pruned of removed nodes by the turn's commit`（C07 が `asset_failures` を消していれば、印は未使用になり消える） |
| 22 | `Sink` | `client/trace.rs` | 書く | `independent: stage, semantics and host_events are the KIVACT_TRACE streams chosen once by Trace::new; flush and line write only file and previous, so any selection of streams stays valid` |
| 23 | `Inspector` | `devtools/state.rs` | 書く | `independent: agent is the agent's start phase (Starting, Ready, Failed), written only by Inspector::promote; can_inject is the launch's permission, set by DevtoolsState::from_slot and never written again` |
| 24 | `HookData` | `devtools/state.rs` | 書く | `record: DevTools hook tables with separate writers: recording (updates start and stop), prop_writes (the prop_written hook, drained by updates), overrides and forced (styles calls; forced is pruned by the turn), emulation (emulation calls), focus_history (the focus_changed hook, swept per closed window), input_log with its sequence allocator next_input (input_event); node_overrides is the projection of overrides written only by rebuild_node_overrides` |
| 25 | `SessionData` | `devtools/state.rs` | 書く | `record: DevTools session tables with separate writers: events (the host_event hook, taken by the turn), pending (find, inject and list_turns calls), pending_graphs and turns (the graph pipeline of updates), frames and update_counts (records), inspector (pick mode), overlays and sent_overlays (settings and what was last sent to the engine), snapshots (perf), sent_windows (tree) and errors_sent (console); next_override and next_snapshot only allocate ids` |
| 26 | `State`（gpu_interop） | `gpu_interop/import/registry.rs` | 書く | `independent: imports is the import table (register gives ids, forget drops them); adapter is each window's device adapter, written only by adapter_changed; whether an import fits the adapter is kept in that import's own status signal` |
| 27 | `SharedImage` | `gpu_interop/share/app_side.rs` | 書く | `independent: current is the import of the newest request, written only by replace_current; pixels is the newest CPU picture whatever the request, written only by receive; size is the size the App asks for, written only by set_size` |
| 28 | `ExportOpen` | `view/drawable_share.rs` | 書く | `independent: scale is the app's export scale, written only by DrawableShare::set_scale (None means the window's own); next_number numbers the frames accepted over the drawable's life, written only by on_frame; the request phase is the ExportRequest enum` |
| 29 | `Common` | `view/element_data.rs` | 書く | `independent: each field is one builder modifier, written only by its own Element or Node modifier and moved out once at mount; drop_misused clears the ones a node kind cannot show, so every combination is a valid builder value` |
| 30 | `Registry` | `view/element_data.rs` | 書く | `independent: state is the region tables (adopt, retire and cleanup in mount/regions); phase is the mount phase, written only by Registry::bind; gestures is the pointer gesture record of mount/part_pointer; each adopted region keeps its own binding, so phase never contradicts state` |
| 31 | `ShowSlot` | `view/flow/branch.rs` | 変える | （手順 1。印を消す） |
| 32 | `SomeSlot` | `view/flow/branch.rs` | 変える | （手順 1。印を消す） |
| 33 | `Seeker` | `view/flow/virtual_list/seeker.rs` | 書く | `independent: state is the list's shared row state (seek, rows, index and heights, each with its own writer); viewport is the host's last reported viewport, written only by the scroll handler of virtual_list/mount and only read here` |
| 34 | `Overlay` | `view/overlays.rs` | 書く | `independent: viewport_margin, keep_anchor_clear and modal are separate builder settings, each written only by its own by-value builder; node_mut, the only &mut self method, lends root and writes none of them` |
| 35 | `NodeRefState` | `view/ref_data.rs` | 書く | `cache: rect_value and scroll_value are the memos NodeRef::rect and NodeRef::scroll_offset create once, after raising observation; bound is the binding phase, written only by bind_target and its cleanup; observation only rises, in require_observation` |
| 36 | `ViewState` | `view/state.rs` | 書く | `record: per-App tables with their own writers: tree (ViewNodes), windows (install_window, remove_window), requests (requests), modifiers and pointer_modality (input dispatch), app_motion (the AppMotion delivery), initial_render (true until start_mount writes false once); next_window only allocates window ids; devtools is fixed at construction` |
| 37 | `Handlers` | `view/tree.rs` | 変える | （手順 1。印を消す） |
| 38 | `CoreNode` | `view/tree.rs` | 書く | `independent: at is the builder's call site, written once at create (None for roots, fragments and wrappers); handlers and rare are boxes created on first use and kept until the node is removed; none constrains another` |
| 39 | `ViewNodes` | `view/tree.rs` | 他 unit（C07）→ 書く | C07 が `future_text_edits` を消した後の文：`identity: next_id only allocates node ids (create_root, create); devtools is fixed at construction`（`future_text_edits` が残っていれば差戻し） |

## L2：kivact-runtime と kivact-interaction の印を閉じる

- **wave / lane / 前提**：W8L / L2 / W8 の全 unit（runtime は W5a の R22・R23 の後）
- **所有**：`crates/kivact-runtime/`、`crates/kivact-interaction/`
- **手順**：
  1. `TimelineCell`（行 71、`motion/timeline/api.rs`）：`TimelinePhase` の `Idle`・`Paused`・`Finished` は `time` を持ち、`self.time` と同じ値を二度保存している。`apply(time)`（`self.time` の唯一の書き手）の後に、同じ値で `set_phase` するため（S-4）。次のように直す：
     - `TimelinePhase` を `{ Idle, Playing { start_ns: u64, from_time: f64 }, Paused, Finished, Driven }` にする。doc は「at `time`」を「at the shown time (`time`)」に直す。
     - `current_time` の `Idle { time } | Paused { time } | Finished { time } => time` の腕を `Idle | Paused | Finished => self.time.get_untracked()` にし、`Driven` の腕と一つにまとめる。
     - `set_phase(TimelinePhase::Finished { time: end })` などの payload を消す（`seek`、`pause`、`stop_for_reduced`、`finish_at_end`、constructor の `Idle { time: 0.0 }`）。`seek` の `Idle { .. } => Idle { time }` は `Idle => Idle`、`Paused { .. } \| Finished { .. } => Paused { time }` は `Paused \| Finished => Paused` にする。
     - `phase` を読む他の `match`（`rg -n "TimelinePhase::" crates/kivact-runtime/src`）は、payload の `{ .. }` を外す。
     - 前提：`apply` を経ずに `self.time` が変わる経路が無いこと（`rg -n "\.time\.set\(" crates/kivact-runtime/src/motion/timeline`）。`apply` の中の 1 件以外があれば差戻し。
  2. 他の行は共通手順 L。
- **確認**：共通の確認に加えて、次の 2 つ。
  ```sh
  cargo test -p kivact-runtime
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- timeline motion_easing task
  ```

| # | 項目 | file | 処置 | 印の文 |
|---|---|---|---|---|
| 63 | `State`（calendar） | interaction `calendar/state.rs` | 書く | `independent: selection_status is the announced selection text (State::pick and the range machine's effect write it), grid_state is the attached grid part (CalendarApi::grid), cell_states maps mounted days to their part state (CalendarApi::cell and its cleanup)`（U31 が field を消していれば未使用になり消える） |
| 64 | `DragApi` | interaction `drag/handle.rs` | 書く | `identity: sources allocates the source ids (DragApi::source_with); observed is the last attached source part, written only by attach_part_state from source_with` |
| 65 | `Press`（menu） | interaction `menu/press_state.rs` | 書く | `record: surfaces and rows are separate registries of mounted nodes, each written only by its own registrar (Press::surface, Press::row) and that registrar's cleanup; the gesture state is the machine` |
| 66 | `fn use_menubar` | interaction `menubar.rs` | 書く | `independent: focus is the trigger list's selection, written only through use_list's binding; menus holds the menus given at setup and nothing writes it afterwards` |
| 67 | `fn use_data_table_model` | interaction `table/model.rs` | 書く | `independent: six optional model settings, each set only by its own by-value builder (multi_sort or sort, filter, column_filters, arrange_rows, typed_column_filters, page); None skips that stage of ordered_rows, so every combination is valid` |
| 68 | `State`（table） | interaction `table/state.rs` | 書く | `record: tables of mounted parts with their own writers: grid_state (DataTableApi::grid), header_states (observe_header), row_states and cell_states (the row and cell builders with their cleanups), and headers, the per-column NodeRef cache filled by State::header_ref` |
| 69 | `Data`（text_edit） | interaction `text_edit/handle.rs` | 書く | `identity: next_control_id allocates the id of each mounted control (TextEditApi::control); observed is the attached control part, written only by attach_part_state in TextEditApi::control` |
| 71 | `TimelineCell` | runtime `motion/timeline/api.rs` | 変える → 書く | 手順 1 の後：`independent: phase is the only playback phase (written only by set_phase) and time the shown time (written only by apply); playback is changed only by reverse and set_rate; repeat, alternate, motion, segments, labels, tracks and completions are setup configuration; plan is a projection rebuilt only by rebuild; pending is the event queue that announce and fire_calls fill and run_pending empties` |
| 72 | `TrackState` | runtime `motion/timeline/plan.rs` | 書く | `cache: last holds the components that publish last wrote into value (set from the initial value by Timeline::track), written only by TrackSlot::publish so an unchanged frame skips the signal write` |
| 73 | `Runtime` | runtime `reactive/data.rs` | 書く | `record: graph, owners, objects and consts are the arenas; frames, collectors, sched, scratch, dep_pool and pass_touched are the stack, scheduler and scratch tables of stabilize; diag, later, graveyard, unsubscribed, unstamped and awaiting are separate queues and tables; seq and write_epoch only allocate creation and source-write identities; state (turn scheduling), exit (the exit drain) and clock (the latest host time) are independent facts` |
| 74 | `MemoCell` | runtime `reactive/data.rs` | 書く | `cache: value is the memo's last computed value, written only by the memo's run (None until the first run); eq is the comparison fixed by create_memo` |
| 75 | `OwnerEntry` | runtime `reactive/data.rs` | 書く | `independent: parent is the creating owner, written only by create_owner (None only for the root) and kept through disposal; extra holds the rare members, allocated on first use by extra_mut and emptied in place by dispose_one` |
| 76 | `Queue` | runtime `task/action.rs` | 他 unit（R22） | R22 が型を置き換える |
| 77 | `ActionCell` | runtime `task/action.rs` | 他 unit（R22） | R22 が `last`・`drained` を消して自分の印を書く |
| 78 | `AsyncCell` | runtime `task/async_state.rs` | 他 unit（R22） | R22 が一つの `Signal<AsyncPhase<T>>` にする |
| 79 | `JobGuard` | runtime `task/blocking.rs` | 書く | `independent: slot is the job's outcome handoff shared with its BlockingFuture and outstanding is the pool's statistic of unfinished jobs; JobGuard holds both handles from construction, and its drop settles one and decrements the other` |
| 80 | `LatestRun` | runtime `task/latest.rs` | 書く | `identity: generation only allocates run ids (start) and keeps counting across Idle so a stale run's id never matches; current is the only phase, Idle or Running with the task and the id start gave it, written by start, cancel and end` |
| 81 | `TaskState` | runtime `task/state.rs` | 書く | `record: tokio, time, finished, queues and pool are separate subsystems with their own writers (the tokio bridge, the timer scheduler, spawn, actions and the blocking pool); pool is the only lifecycle of the blocking pool; blocking_threads is the worker count fixed by new` |
| 82 | `Pool` | runtime `task/worker_pool.rs` | 書く | `independent: workers is the worker-thread lifecycle (new, take_panic, shutdown); fallbacks holds the threads submit starts when no worker takes a job; outstanding is a statistic of submitted unfinished jobs, read only by the task stats` |

## L3：kivact-engine・kivact-layout・kivact-editing の印を閉じる

- **wave / lane / 前提**：W8L / L3 / W8 の全 unit（engine は E17 の後）
- **所有**：`crates/kivact-engine/`、`crates/kivact-layout/`、`crates/kivact-editing/`
- **手順**：
  1. `EditorNode` の cache（行 54）：`model/interact/sync.rs` の、font が足されたときの loop（「New fonts can change fallback」の comment）で `editor.lines = Default::default();` の隣に `editor.measure_lines = Default::default();` を足す。E17 が field の型を変えていれば、同じ形で空にする。comment は「an input's lines (painted and measured) are shaped again」にする。
  2. 他の行は共通手順 L。
- **確認**：共通の確認に加えて、次を実行する。
  ```sh
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- launch_fonts text_editing custom_text_field
  ```

| # | 項目 | file | 処置 | 印の文 |
|---|---|---|---|---|
| 45 | `EditState` | editing `edit_state.rs` | 他 unit（E17） | E17 が `max_length` を消す。印は未使用になり消える |
| 46 | `SemanticsIdAllocator` | engine `a11y/state.rs` | 書く | `identity: next is the countdown allocator of synthetic semantics ids (next, fresh); empty_root keeps the id build_empty_semantics reserved for an empty tree, reused while no node takes it` |
| 47 | `Host` | engine `host/pump.rs` | 書く | `independent: app and engine are the shared models and text engine, set at launch and borrowed per call; frame_log is the benchmark log shared with renderer observers; reduced_motion_override is written only by the reduced-motion request` |
| 48 | `Counts` | engine `host/testing/test_host.rs` | 書く | `statistic: turns and frame_ticks are counted only by the counted() client wrapper and read by the test driver; they never steer scheduling` |
| 49 | `ArvaneTestHost` | engine `host/testing/test_host.rs` | 書く | `independent: host (the Launch), renderer (the observer's per-window notes), injected and controls (queues drained by SharedHost::event) are separate shared handles made once in ArvaneTestHost::new` |
| 50 | `SharedHost` | engine `host/testing/test_host.rs` | 書く | `independent: host, injected and controls are handles shared with ArvaneTestHost; event drains controls, then injected, then passes the event to host` |
| 51 | `Reader` | engine `host/write/reader.rs` | 書く | `independent: app (the window models) and engine (Arvane's text) are shared handles cloned from Writer and never reassigned; each call borrows one for its duration` |
| 52 | `Writer` | engine `host/write/writer.rs` | 書く | `independent: app (the window models) and engine (Arvane's text) are shared handles set by Writer::new and never reassigned; each write borrows them for its duration only` |
| 53 | `Context`（lower） | engine `lower/context.rs` | 書く | `independent: assets and scene_cache are borrowed stores; raster is the scale stack of paint_layers and paint_scene; dependencies, dropped and failures accumulate one operation's report (record, drop_command, fail; restore rewinds the last two); drawables is set by hosting` |
| 54 | `EditorNode` | engine `measure/editor.rs` | 変える → 書く | 手順 1 の後：`cache: lines (paint width) and measure_lines (width probes) are separate shaping caches filled lazily by refresh_lines, each on its own path; sync empties both when fonts are added` |
| 55 | `Model` | engine `model/aggregate.rs` | 書く | `independent: assets is the App's shared store; origins caches window origins (filled by Model::rect, cleared by forget_origins); window_request is set by Writer::set_window and taken by the host when the window is finished` |
| 56 | `AppModel` | engine `model/app_model.rs` | 書く | `independent: assets is the store shared with every Model; focused_window is the OS focus, written only by the Focused shell events and remove_window; text_ahead counts the text edits handed to the client and not yet dispatched (hold_text, delivered); font_generation counts font registrations` |
| 57 | `Changes` | engine `model/changes.rs` | 書く | `independent: paint (invalidate_paint, motion ticks), restack (tree edits and motion steps) and fonts (fonts_added) are separate pending flags that dispatch takes together` |
| 58 | `Admission` | engine `model/modal/state.rs` | 書く | `record: drags (press, end) and adoptions (adopt, take_adoption, forget_adoption) are separate per-pointer tables; forget_missing prunes both` |
| 59 | `OffscreenRenderer` | engine `offscreen.rs` | 書く | `independent: assets, scene_cache and engine are the output's own stores, made by Default; render and font use them, release_caches clears them, and engine is shared with TextMeasurer` |
| 60 | `PaintState` | engine `paint/state.rs` | 書く | `independent: scene_cache is a lowering cache; dropped_reported, misplaced_reported and twice_reported are report-once sets; dropped_draws is the DrawDropped mailbox; drawables is the frame plan (prepare_canvas); draw_owners indexes the slots holding a kept drawing (keep_draw, evict_draws, forget_draws)` |
| 61 | `NodeTree` | engine `tree/node_tree.rs` | 書く | `independent: flat caches flat child lists (filled by flat_children, cleared by forget_flat_through, forget_flat_around and remove); root is written only by install and remove` |
| 70 | `NodeStyle` | layout `style/node.rs` | 書く | `independent: aspect_ratio, align_self and grid_area are authored props written only by convert::taffy_style; the &mut self methods (set_min_width_floor, set_size_bounds) write only the size bounds` |

## L4：style・i18n・test・devtools の印を閉じる

- **wave / lane / 前提**：W8L / L4 / W8 の全 unit（style は S17〜S23 の後）
- **所有**：`crates/kivact-style/`、`crates/kivact-i18n/`、`crates/kivact-test/`、`crates/kivact-devtools/`、`crates/kivact-devtools-protocol/`
- **手順**：共通手順 L。S17 が `merge` を消した型（行 83〜90・92・94）は、印が未使用になって消える見込み。`Style`（行 93）は S17 が置き換えるので GONE の見込み。
- **確認**：共通の確認。

| # | 項目 | file | 処置 | 印の文 |
|---|---|---|---|---|
| 40 | `Connection` | devtools-protocol `client/connection.rs` | 書く | `independent: calls is the Open/Closed enum written by call_raw and the read loop; next_id allocates request ids; writer and notifications are Mutexes that only serialize the stream's two halves and are never replaced` |
| 41 | `fn view` | devtools `panels/elements/view.rs` | 書く | `independent: owners_on (the Owners checkbox) picks the tree shown; layout (the layout toggle) drives the highlight overlay of the details pane, which both trees share` |
| 42 | `fn new`（Session） | devtools `session.rs` | 書く | `record: target and status are handles made here and moved into Session; target is never written after this, status only by Session::report` |
| 43 | `Session` | devtools `state.rs` | 書く | `record: one handle per session fact, each with its own writer: target (Session::new only), connection (listen moves it to Lost), windows and inspected (on_windows, load_window, pick), turns, frames and errors (their notifications), panel and popped (the toolbar), status (report)` |
| 44 | `Pointing` | devtools `widgets/timeline.rs` | 書く | `independent: turns and frames are read-only inputs from the session; zoom and drag are written only by Pointing::handle, zoom_to and the Reset zoom button; width is a projection written only by the paint closure` |
| 62 | `SymbolCapture` | i18n `shared.rs` | 書く | `independent: symbols and digits are filled by the two DataProvider::load impls, one per ICU key, in whichever order ICU asks; NumberSymbols::new takes both once the formatter is built` |
| 83 | `GridStyle` | style `grid.rs` | 書く | `independent: each Option is one authored grid property (tracks, auto flow, item placement, justify), absent meaning inherited or default; merge and the grid setters write one property at a time` |
| 84 | `Sides` | style `sheet/fields.rs` | 書く | `independent: each side is its own authored length, absent meaning inherited or default; merge and the side setters overlay each side on its own` |
| 85 | `SizeFields` | style `sheet/fields.rs` | 書く | `independent: each Option is one authored size or flex property, absent meaning inherited or default; the setters and merge write one property at a time` |
| 86 | `SpacingFields` | style `sheet/fields.rs` | 書く | `independent: row_gap and column_gap are separate authored gaps, absent meaning inherited or default; the gap setters and merge write each on its own` |
| 87 | `FlowFields` | style `sheet/fields.rs` | 書く | `independent: each Option is one authored flow, grid, position or scroll property, absent meaning inherited or default; the setters and merge write one property at a time` |
| 88 | `PaintFields` | style `sheet/fields.rs` | 書く | `independent: each Option is one authored paint or effect property, absent meaning inherited or default; the setters and merge write one property at a time` |
| 89 | `TransformFields` | style `sheet/fields.rs` | 書く | `independent: each Option is one authored 2D or 3D transform property, absent meaning inherited or default; the setters and merge write one property at a time` |
| 90 | `TextFields` | style `sheet/fields.rs` | 書く | `independent: each Option is one authored text property, with the rare ones boxed in text, absent meaning inherited or default; the setters and merge write one property at a time` |
| 91 | `StateFields` | style `sheet/fields.rs` | 書く | `independent: each Option is the style of one interaction state and group marks a group container; any set of states may be styled, and merging combines each state's style on its own` |
| 92 | `MotionFields` | style `sheet/fields.rs` | 書く | `independent: enter_size, exit_size and layout_transition are separate authored motion properties, absent meaning default; the setters and merge write each on its own` |
| 93 | `Style` | style `sheet/fields.rs` | 他 unit（S17） | S17 が `Style { decls }` に置き換える |
| 94 | `TextExtras` | style `text.rs` | 書く | `independent: each Option is one rare authored text property, absent meaning inherited or default; the text setters and merge write one property at a time` |
| 95 | `TestApp` | test `screen/model.rs` | 書く | `record: separate test-session facts: host is the driver borrowed by each step, actions the last logged actions (Screen::log), contacts the contacts down (contact_down, contact_up, contact_cancel), last_contact the contact id allocator (contact_down)` |

## L5：arvane の印を閉じる

- **wave / lane / 前提**：W8L / L5 / W8 の全 unit（arvane は A07・A08・A12・A14 の後）
- **所有**：`crates/arvane-asset/`、`crates/arvane-native/`、`crates/arvane-shell/`、`crates/arvane-virtual/`、`dev/tests/gpu-interop-producer/`
- **手順**：共通手順 L。Linux 専用の file（`port/x11/*`）の印を書き換えたら、作業指示書 0.6 節の Linux の確認を行う。
- **確認**：共通の確認。

| # | 項目 | file | 処置 | 印の文 |
|---|---|---|---|---|
| 1 | `AssetStore` | arvane-asset `store/asset_store.rs` | 書く | `identity: next_id allocates AssetImage ids (new_pixels) and next_generation allocates entry generations (start_load, reparse) that drop stale job results; neither derives from the other` |
| 2 | `Counters` | arvane-native `mirror.rs` | 書く | `statistic: created, reused and presented are separate counts the test UI thread adds to (a new window, a pooled window, a shown frame); TestDisplay::stats only reads them` |
| 3 | `TestDisplay` | arvane-native `mirror.rs` | 書く | `identity: next allocates test window ids in TestDisplay::open; proxy is the event-loop sender behind a Mutex only for Sync and is never replaced` |
| 4 | `Retained` | arvane-native `module.rs` | 書く | `independent: mapping is the loader's answer, fixed when RETAINED is initialized; reported is the report-once flag swapped only by retain_module` |
| 5 | `NativeInput` | arvane-native `native_input/state.rs` | 書く | `record: per-window facts the subclass shares with the runner, one writer each: ime and text by the IME port, scale by the scale-change paths, reduced_motion, mouse and locale by the settings and focus messages, subclass_installed by the subclass install and remove, contacts by the pointer messages` |
| 6 | `Session`（embed） | arvane-native `port/embed/session.rs` | 書く | `independent: the child lifecycle, the key verdict ledger (keys), the one pending resize (resize) and the deferred-answer mailbox (answers) each have their own writer; resize::settle moves a request from resize to answers in one call` |
| 7 | `State`（win32） | arvane-native `port/win32/state.rs` | 他 unit（A14）→ 書く | A14 が `start`（`StartSlot`）を消した後の文：`record: separate facts of the embedded child, one writer each: surface (visibility), events (the queue), original_proc (the subclass), cursor, decoder, mouse, modifiers and clicks (input messages), environment (the last announced, compared on change), scale_source (set_scale), cpu, services and uia (released once by dispose)`（`start` が残っていれば差戻し） |
| 8 | `TestParent`（win32） | arvane-native `port/win32/test_parent.rs` | 書く | `independent: point is the last injected mouse position (button, move_to) and buttons the held-button mask (button only); any position pairs with any mask` |
| 9 | `WinitWindow` | arvane-native `port/winit/window.rs` | 書く | `independent: foreign and subclass are set once by open (foreign content, the Windows subclass install) and only dropped; monitor is the last-seen monitor, written only by the move and scale-change handler` |
| 10 | `WindowTable` | arvane-native `port/winit/windows.rs` | 書く | `independent: placement is the platform capability fixed by WindowTable::new; foreign is the loop's foreign-child connections, written only through foreign_connections_mut while a window opens` |
| 11 | `Notifier`（x11） | arvane-native `port/x11/notifier.rs` | 書く | `independent: reasons is the wake mailbox OR-ed in by request on any thread and swapped out by take on the UI thread; thread is the UI thread lifecycle enum written by spawn and join; a request is valid in every thread state` |
| 12 | `TestParent`（x11） | arvane-native `port/x11/test_parent.rs` | 書く | `independent: point is the last injected pointer position, covers the cover windows added by cover, wm_frame the WM frame recorded by open and taken by close; none constrains another` |
| 13 | `Window`（x11） | arvane-native `port/x11/window.rs` | 書く | `independent: redraw is the dirty bit set by Expose, resize and visibility and cleared by the present; ime is the XIM port and ime_driver the shell IME normalizer that drives it; neither stores the other's facts` |
| 14 | `ClickCounter` | arvane-shell `clicks.rs` | 書く | `independent: last is written only by a mouse Down and tap only by a touch or pen Down; a window may see either device first or only one of them` |
| 15 | `VirtualBackend` | arvane-virtual `backend.rs` | 書く | `record: windows is the open-window table and input the scripted input queue and clock; two tables shared by Rc with Running, Probe and WindowProbe` |
| 16 | `Running` | arvane-virtual `backend.rs` | 書く | `record: windows and input are the backend's open-window table and scripted input queue, shared by Rc; the loop's own phase is the State enum` |
| 17 | `Probe` | arvane-virtual `backend.rs` | 書く | `record: windows and input are read aliases of the backend's two tables; the probe writes only the logs the test reads (unhandled_keys)` |
| 18 | `WindowProbe` | arvane-virtual `backend.rs` | 書く | `record: windows and input are read aliases of the backend's two tables, narrowed to one window id` |
| 19 | `VirtualPlatform` | arvane-virtual `window.rs` | 書く | `record: scripted service records, one writer each: saved_state (the SaveState script item), responders (Script::respond), clipboard, dialogs, opened and revealed (service requests), next_foreign_child (the native child id allocator), fail_next (FailEffect, taken by the next effect)` |
| 20 | `VirtualWindow` | arvane-virtual `window.rs` | 書く | `record: what the app last asked of one scripted window, one ShellWindow setter per field (title, min_size, ime_purpose, ime, ime_text, cursor, soft_keyboard_shown, foreign_child, resizable); env and occluded are written by script items; redraw is the undelivered request; soft_keyboard_requests and focus_requests are statistics` |
| 166 | `Producer` | `dev/tests/gpu-interop-producer/src/producer.rs` | 書く | `independent: shared is the pool's handles, taken once by shared_textures or signal_prepared; next is the next frame number, advanced only by draw; frames may be drawn before or after the handles are taken` |

## L6：kivact-ui と examples/kivact の印を閉じる

- **wave / lane / 前提**：W8L / L6 / W8 の全 unit（U の C5 の規則で builder の struct が消えた後）
- **所有**：`crates/kivact-ui/`、`examples/kivact/`
- **手順**：共通手順 L。表の行の多くは GONE の見込み。U の unit が消すはずの struct が残っていても、共通手順 L のとおり印を書く（W8 の U の `--strict` の結果は別に扱う）。
- **確認**：共通の確認（`kivact-ui` と、`examples/kivact` の変えた package）。

| # | 項目 | file | 処置 | 印の文 |
|---|---|---|---|---|
| 131 | `Alert` | `alert/view.rs` | 書く | `independent: description, icon, actions and dismiss_content are separate optional slots, collapsible a setup flag and open an optional caller binding, each set only by its by-value setter; node_mut lends only root and render_contents only moves the slots out while into_view consumes the alert` |
| 132 | `Avatar` | `avatar/view.rs` | 書く | `independent: status_content is an optional slot drawn beside the status tone and name_tooltip an optional override of the theme's tooltip default; each is set only by its by-value setter and node_mut lends only root` |
| 133 | `AvatarGroup` | `avatar_group/view.rs` | 書く | `independent: overlap overrides the theme's member overlap and total adds not-yet-loaded participants to the overflow count; both are set only by their by-value setters and node_mut lends only root` |
| 134 | `Badge` | `badge/view.rs` | 書く | `independent: icon (leading) and trailing (from trailing_icon) are separate optional slots, each set only by its by-value setter; node_mut lends only root` |
| 135 | `Button` | `button/view.rs` | 書く | `independent: icon and trailing are separate optional slots, busy an optional caller value whose presence alone creates the busy clock, and hide_label an optional caller value; each is set only by its by-value setter and node_mut lends only root` |
| 136 | `Calendar` | `calendar/view.rs` | 他 unit（U54） | U54 が struct を消す。残っていれば差戻し（`month` と `displayed_month` は同じ事実の二つの出所で、閉じるには enum が要る） |
| 137 | `Card` | `card/view.rs` | 書く | `independent: header, media and footer are separate optional regions, each set only by its by-value setter; node_mut lends only root` |
| 138 | `Carousel` | `carousel/view.rs` | 書く | `independent: looping (wrap at the ends) and the optional per_view, gap and padding overrides of the theme layout are each set only by their by-value setter; node_mut lends only root` |
| 139 | `ChatComposer` | `chat_composer/view.rs` | 書く | `independent: drawer, header_actions, header_context, footer_actions, send_actions, send_icon and stop_icon are separate optional slots, each set only by its by-value setter; the render helpers only move slots out while into_view consumes the composer` |
| 140 | `ChatMessage` | `chat_message/view.rs` | 書く | `independent: status, time, align and bubble_width are optional caller values and the content, avatar, header, footer, metadata, media, icon, actions and reactions slots are optional, each set only by its by-value setter; render_body only moves slots out during into_view` |
| 141 | `ChatThread` | `chat_thread/view.rs` | 書く | `independent: viewport (a caller node ref), history_revision (the caller's dataset revision), composer (a slot), empty (its condition paired with its view) and gap (a spacing override) are optional inputs, each set only by its by-value setter` |
| 142 | `CodeBlock` | `code_block/view.rs` | 書く | `independent: title, language, end_content, footer, max_height, expanded (a caller binding) and highlight (a line styler) are optional inputs, each set only by its by-value setter; presentation and header only move them out during into_view` |
| 143 | `Combobox` | `combobox/data.rs` | 書く | `independent: query and open are optional caller bindings (absent means local signals), free_text, chips and exclude_selected setup flags, and mode, placeholder, empty, matcher and search optional settings, each set only by its by-value setter; Inline mode ignores open, as documented on open()` |
| 144 | `DataTable` | `data_table/types.rs` | 書く | `independent: each optional feature and the tree_expand_all flag is set only by its own by-value setter and taken once while into_view consumes the table; a feature whose base is absent (page_sizes without paging, expanded_rows without details, expand_all_control without tree) is ignored` |
| 145 | `DateInput` | `date_input/view.rs` | 書く | `independent: picker_label names the picker button, valid is an optional caller output binding for draft validity and week_start an optional calendar override; each is set only by its by-value setter` |
| 146 | `DateRangeInput` | `date_range_input/view.rs` | 書く | `independent: picker_label, placeholder (else derived from the format), valid (a caller output binding), week_start and presets are optional inputs, each set only by its by-value setter` |
| 147 | `DateTimeInput` | `date_time_input/view.rs` | 書く | `independent: picker_label names the picker button and week_start overrides the calendar's first weekday; both are set only by their by-value setters` |
| 148 | `Disclosure` | `disclosure/view.rs` | 書く | `independent: always_open, keep_mounted and animation are caller setup choices with documented precedence, and heading_level and group are set by DisclosureGroup through the by-value heading and group_position` |
| 149 | `Dropzone` | `dropzone/view.rs` | 書く | `independent: icon is an optional decorative slot, multiple governs how many files are accepted and pick_on_press adds the native picker on activation; each is set only by its by-value setter` |
| 150 | `EmptyState` | `empty_state/view.rs` | 書く | `independent: description, icon and actions are separate optional regions, each set only by its by-value setter; node_mut lends only root` |
| 151 | `Field` | `field/view.rs` | 書く | `independent: hint, required and error are optional caller values the field adds to its control (described_by, required, invalid with its message), each set only by its by-value setter` |
| 152 | `InputGroup` | `input_group/view.rs` | 書く | `independent: start, end, block_start and block_end are four separate adornment slots, each set only by its by-value setter` |
| 153 | `RowContents` | `item_list/row.rs` | 書く | `independent: the title, description, media, actions, header, footer and marker slots are optional (custom content wins over the text it replaces), delegate is an optional activation target, and position and the line counts optional overrides; the row's by-value setters write them and content only moves slots out while render consumes the row` |
| 154 | `NavigationMenuLink` | `navigation_menu/link.rs` | 書く | `independent: description (text under the title) and icon are separate optional slots, each set only by its by-value setter`（U23 が `description` を消していれば未使用になり消える） |
| 155 | `Pagination` | `pagination/view.rs` | 書く | `independent: page_size is an optional caller binding of items per page, and item and summary are optional view factories replacing cell and summary contents; each is set only by its by-value setter` |
| 156 | `RadioGroup` | `radio_group/view.rs` | 書く | `independent: item replaces an option's inner content and item_node modifies its outer radio node, documented to compose; each is set only by its by-value setter` |
| 157 | `Slider` | `slider/view.rs` | 書く | `independent: range overrides the default 0..=100 bounds and step the default continuous snapping; both are set only by their by-value setters` |
| 158 | `Splitter` | `splitter/view.rs` | 書く | `independent: start and end are the two panes, handle the handle's content, and collapse, snaps and key_step optional release and keyboard policies passed separately to the splitter behavior; each is set only by its by-value setter` |
| 159 | `Stat` | `stat/view.rs` | 書く | `independent: delta is optional change text, icon and chart are decorative slots, and label_content and value_content replace only the visible label and value while the required texts stay accessible; each is set only by its by-value setter` |
| 160 | `TableCell` | `table/view.rs` | 書く | `independent: header is fixed by the constructor (table_head or table_cell) and width is an optional fixed width set only by its by-value setter` |
| 161 | `Tabs` | `tabs/view.rs` | 書く | `independent: item replaces each tab's contents and keep_mounted keeps the panels built from content mounted; each is set only by its by-value setter` |
| 162 | `ToggleButton` | `toggle_button/view.rs` | 書く | `independent: icon is an optional leading slot and hide_label an optional caller value hiding the visible label; each is set only by its by-value setter` |
| 163 | `ToggleGroup` | `toggle_group/view.rs` | 書く | `independent: spacing overrides the theme gap and item replaces each toggle's contents; both are set only by their by-value setters` |
| 164 | `Toolbar` | `toolbar/view.rs` | 書く | `independent: start, center and end are the three region child lists (center also switches to the centred grid) and gap an optional spacing override; each is set only by its by-value setter` |
| 165 | `Tree` | `tree/view.rs` | 書く | `independent: item replaces the row body, header is an optional view before the rows and reveal an optional presence setting; each is set only by its by-value setter` |
| 194 | `fn app` | `examples/kivact/component/src/main.rs` | 書く | `independent: first and second say whether the First and Second cards are shown; each is cleared only by its own card's on_close and never set back` |

## L7：kivact-ui-basic とその example の印を閉じる

- **wave / lane / 前提**：W10 / K1（K35 の後）/ K35
- **所有**：`crates/kivact-ui-basic/`、`examples/kivact-ui-basic/`
- **手順**：共通手順 L。K の unit が builder を `#[component]` にしていれば、行の多くは GONE。example の関数は、K35 で file が移る・改名されることがあるので、共通手順 L の 2（field の並びで対応させる）で探す。
- **確認**：共通の確認（`kivact-ui-basic` と `kivact-ui-basic-catalog`）。

| # | 項目 | file | 処置 | 印の文 |
|---|---|---|---|---|
| 96 | `Attachment` | `attachment.rs` | 書く | `independent: media, description and actions are optional slots, each filled only by its own by-value setter; node_mut writes only root, so any subset of the three renders` |
| 97 | `CardHeader` | `card_header.rs` | 書く | `independent: description and action are optional parts set only by the by-value setters CardHeader::description and CardHeader::action; the title column and the end action render separately` |
| 98 | `FieldSet` | `field.rs` | 書く | `independent: legend and description are optional captions set only by FieldSet::legend and FieldSet::description; the legend row and the description line are drawn independently around the children` |
| 99 | `SidebarGroup` | `sidebar/menu.rs` | 書く | `independent: action (the label row's end control) and content (the group's body) are optional slots set only by the by-value setters SidebarGroup::action and SidebarGroup::content` |
| 100 | `SidebarMenuButton` | `sidebar/menu.rs` | 書く | `independent: icon, badge, action and sub are optional row parts, each set only by its own by-value setter; the leading icon, end badge, hover action and sub-menu render in separate slots` |
| 101 | `Sidebar` | `sidebar/panel.rs` | 書く | `independent: header, content and footer are optional panel sections set only by their by-value setters; build wraps each present one in its own section, so any subset is a valid panel` |
| 102 | `TraceLine` | `trace.rs` | 書く | `independent: icon and description are optional decorations of the trace title, set only by TraceLine::icon and TraceLine::description` |
| 172 | `fn destructive` | example `docs/alert_dialog.rs` | 書く | `independent: open is the alert's binding, set by the Delete Chat button and cleared by Cancel or Delete; deleted is set only by Delete; the alert can reopen after a deletion, so all four pairs occur` |
| 173 | `fn payments` | example `docs/data_table.rs` | 書く | `independent: sort is written by the column headers through the data table's sort binding, which also resets page to 0; page is then moved only by the pager, so every sort can be shown on every page` |
| 174 | `fn confirm` | example `docs/dialog.rs` | 書く | `independent: open is the dialog binding, set by the Delete account button and cleared by Cancel, Delete or the dialog's own close; deleted only turns true on Delete, and the dialog may be reopened afterwards` |
| 175 | `fn default` | example `docs/drawer.rs` | 書く | `independent: open is the drawer binding set by Open Drawer and cleared by Submit or Cancel; goal is changed only by the plus and minus buttons inside the drawer and is kept, not reverted, when it closes` |
| 176 | `fn checkboxes` | example `docs/dropdown_menu.rs` | 書く | `independent: status and activity back the Status Bar and Activity Bar menu checkboxes; on_activate flips only the one whose key was activated` |
| 177 | `fn steps` | example `docs/trace.rs` | 書く | `independent: first and second are the open bindings of two separate trace collapsibles, each toggled only by its own header` |

## L8：kivact-ui-pop とその example の印を閉じる

- **wave / lane / 前提**：W10 / K2（K37 の後）/ K37
- **所有**：`crates/kivact-ui-pop/`、`examples/kivact-ui-pop/`
- **手順**：共通手順 L（L7 と同じ注意）。行 191（`fn table`）は、K37 が shell の `sheet` を local の `signal(false)` にするので、状態の local が 3 つになる。その場合、印の文に `; sheet is the student drawer's binding, opened by the row press and closed by the drawer` を足す（共通手順 L の 5 の差戻しにしない）。
- **確認**：共通の確認（`kivact-ui-pop` と `kivact-ui-pop-catalog`）。

| # | 項目 | file | 処置 | 印の文 |
|---|---|---|---|---|
| 108 | `TimelineItem` | `timeline.rs` | 書く | `independent: title_content (a rich title replacing the text title) and glyph (the icon set by TimelineItem::icon) are set only by by-value setters and fill different slots of the row` |
| 188 | `fn switch_controls` | example `pop/components.rs` | 書く | `independent: volume and difficulty hold the values of the BGM volume slider and the difficulty slider, each written only by its own slider` |
| 189 | `fn check_controls` | example `pop/components.rs` | 書く | `independent: notifications and reports hold the values of the notifications and weekly-report checkboxes, each toggled only by its own checkbox` |
| 190 | `fn display` | example `pop/data.rs` | 書く | `independent: next_chip counts the chips added by the badge specimen's add button and picks the next suggestion; live is the progress value advanced only by the 900 ms loop` |
| 191 | `fn table` | example `pop/data.rs` | 書く | `independent: sort is written by the column headers, whose binding resets page to 0 as the status filter and the search do; page is then moved by the footer pager, so any sort pairs with any page` |
| 192 | `fn feedback` | example `pop/patterns.rs` | 書く | `independent: dismissed_info and dismissed_success hide the maintenance and success notices; each is set only by its own notice's close button and never cleared` |
| 193 | `fn student_drawer` | example `pop/shell.rs` | 書く | `independent: pin_student and auto_skill back the drawer's two switches (pin to main team, auto skill), each written only by its own switch` |

## L9：kivact-ui-creative とその example の印を閉じる

- **wave / lane / 前提**：W10 / K3（K38 の後）/ K38
- **所有**：`crates/kivact-ui-creative/`、`examples/kivact-ui-creative/`
- **手順**：共通手順 L（L7 と同じ注意）。行 179 は K38 が `src/creative/shell.rs` を消すので、GONE の見込み。
- **確認**：共通の確認（`kivact-ui-creative` と `kivact-ui-creative-catalog`）。

| # | 項目 | file | 処置 | 印の文 |
|---|---|---|---|---|
| 103 | `Numeric` | `numeric/builder.rs` | 書く | `independent: format, drag_distance, text_entry, timecode, accent and text_parser are construction options set only by make() and the by-value setters; prepare resolves each with its own fallback` |
| 104 | `NumericView` | `numeric/view.rs` | 書く | `independent: accent, drag_distance and text_entry are copied once from the builder in prepare(); wheel_rest is the sub-step wheel remainder, written only by NumericView::wheel` |
| 105 | `RowFactory` | `outliner/view.rs` | 書く | `identity: generation is the mount-id allocator advanced only by RowFactory::register, so a row's cleanup can tell its own mount from a newer one; row_height and has_activation are copied once from the outliner and never written` |
| 106 | `PropertyRow` | `property_row.rs` | 書く | `independent: leading and trailing are optional adornments on either side of the value, set only by PropertyRow::leading and PropertyRow::trailing` |
| 178 | `fn properties` | example `creative/layout.rs` | 書く | `independent: tab is the inspector tab strip's selection; display_name, display_axes and display_wireframe back three separate Viewport Display checkboxes; no handler writes more than one` |
| 179 | `fn new`（CatalogControls） | example `creative/shell.rs` | 書く | `independent: compact (the density button and its shortcut), current (the sidebar row press), sidebar_open (the top bar toggle), search (the search shortcut and buttons), shading and viewport_grid (choose_shading) are separate catalog controls with their own writers` |

## L10：kivact-ui-game とその example の印を閉じる

- **wave / lane / 前提**：W10 / K4（K40 の後）/ K40
- **所有**：`crates/kivact-ui-game/`、`examples/kivact-ui-game/`
- **手順**：共通手順 L（L7 と同じ注意）。
- **確認**：共通の確認（`kivact-ui-game` と `kivact-ui-game-catalog`）。

| # | 項目 | file | 処置 | 印の文 |
|---|---|---|---|---|
| 107 | `Kpi` | `kpi.rs` | 書く | `independent: change (the delta caption), heading (a label replacement) and trend (the sparkline samples) are optional, each set only by its by-value setter and passed to a separate part of the stat` |
| 180 | `fn buttons` | example `game/controls.rs` | 書く | `independent: grid_on and filter_on are the two toggle buttons of the button toolbar, each flipped only by its own on_press` |
| 181 | `fn navigation` | example `game/controls.rs` | 書く | `independent: step is the wizard step moved by the back and next buttons and the stepper; page is the separate pagination specimen's binding; no control writes both` |
| 182 | `NativeWindow` | example `game/frame.rs` | 書く | `independent: menu_bar, toolbar and status_bar are optional chrome slots set only by NativeWindow's by-value setters; node_mut touches root alone` |
| 183 | `Demo` | example `game/state.rs` | 書く | `record: three shared catalog signals with different writers: overlay (the one open overlay, via show, hide, toggle and open), rows (the approve, reassign and delete handlers) and selected (the table selection, cleared by delete)` |
| 184 | `fn feedback` | example `game/system.rs` | 書く | `independent: info, success, warning and danger are the open bindings of four separate inline alerts, each cleared only by its own dismiss button and never reopened` |
| 185 | `fn overlays` | example `game/system.rs` | 書く | `independent: pinned and archived are two dropdown checkbox entries flipped by their own keys; user_open is the user popover's dialog binding, opened by its trigger and closed by the dialog` |
| 186 | `fn layout` | example `game/system.rs` | 書く | `independent: notify is bound only to the inspector's notification switch and opacity only to its opacity slider` |
| 187 | `fn frame` | example `game/window.rs` | 書く | `independent: width is the explorer width edited only by the window's splitter; help is the About dialog's binding, set by the Help menu command and cleared by the dialog` |

## L11：kivact-ui-soft の印を閉じる

- **wave / lane / 前提**：W10 / K5（K43 の後）/ K43
- **所有**：`crates/kivact-ui-soft/`、`examples/kivact-ui-soft/`
- **手順**：共通手順 L（L7 と同じ注意）。`StyledText`（行 125）の field `heading` は書き手が無い（常に `None`）。型が残っていれば `heading` を消し、`type_style` には `None` を渡す。
- **確認**：共通の確認（`kivact-ui-soft` と `kivact-ui-soft-catalog`）。

| # | 項目 | file | 処置 | 印の文 |
|---|---|---|---|---|
| 109 | `AvatarStatusDot` | `avatar_status_dot.rs` | 書く | `independent: label (the accessible name) and icon (the glyph inside the dot) are by-value options; the dot names itself only when label is present and draws the icon only when it fits` |
| 110 | `LayoutGrid` | `grid.rs` | 書く | `independent: row_gap and column_gap each override gap on one axis and row_height fixes the row size; each is set only by its own by-value setter and read separately in into_view` |
| 111 | `Layout` | `layout.rs` | 書く | `independent: the region slots, the two resize handles, padding and content_width are optional parts set only by Layout's by-value setters; into_view derives has_start, has_end and the dividers from them, so any subset lays out` |
| 112 | `LayoutRegion` | `layout.rs` | 書く | `independent: divider, padding, padding_bottom and width are per-region overrides set by the by-value setters; Layout::into_view fills only an unset divider from the defaults, and padding_bottom overrides only the bottom edge of padding` |
| 113 | `Link` | `link.rs` | 書く | `independent: label (the accessible name), new_tab_label (a suffix used only when external), size and weight (typography overrides) are set by the constructor and by-value setters; each falls back on its own` |
| 114 | `MobileNav` | `mobile_nav.rs` | 書く | `independent: open (an app-owned binding instead of the shell's), label (the drawer name) and header (heading text, also the name fallback) are set only by MobileNav's by-value setters; into_view resolves open and label with their own fallbacks` |
| 115 | `MobileNavToggle` | `mobile_nav.rs` | 書く | `independent: label (an accessible label override) and icon (a replacement glyph) are set only by MobileNavToggle::label and ::icon; each falls back to the shell default on its own` |
| 116 | `SideNavHeading` | `nav_heading.rs` | 書く | `independent: superheading, subheading, icon, end and menu are optional heading parts, each set only by its own by-value setter; menu only adds the chevron and the menu popup, the others fill fixed slots` |
| 117 | `Outline` | `outline.rs` | 書く | `independent: active (the controlled id), viewport and content (scroll-tracking refs) are set only by their by-value setters; a controlled active wins over tracking, and content is read only when viewport is present` |
| 118 | `SideNavItem` | `side_navigation/items.rs` | 書く | `independent: icon, selected_icon (the icon while selected), end and actions are optional slots, each set only by its by-value setter` |
| 119 | `SideNavSection` | `side_navigation/items.rs` | 書く | `independent: subtitle and end are optional parts of the section header, set only by SideNavSection::subtitle and ::end_content` |
| 120 | `SideNav` | `side_navigation/shell.rs` | 書く | `independent: header, top, footer and icons are optional slots set by SideNav's by-value setters; collapse and resize are separate caller options, and the collapse ownership inside resize wins over collapse` |
| 121 | `SideNavCollapseButton` | `side_navigation/shell.rs` | 書く | `independent: binding (an override of the sidebar context's collapse binding) and label are set only by their by-value setters; each falls back on its own` |
| 122 | `ChatToolCalls` | `soft_chat_tools.rs` | 書く | `independent: label, expanded (the app-owned group disclosure binding) and detail (the per-call detail view factory) are set only by the by-value setters and used by separate parts of the group` |
| 123 | `TableSelectionToolbar` | `soft_table.rs` | 書く | `independent: selection_label (the count text factory) and actions (the bulk action views) are set only by their by-value setters and fill the two ends of the toolbar` |
| 124 | `FlexStack` | `stack.rs` | 書く | `independent: horizontal and vertical are screen-axis overrides set only by h_align and v_align; into_view maps them onto the main and cross axis for the initial direction, where each wins over justify or align alone` |
| 125 | `StyledText` | `text.rs` | 書く | `independent: size, color, weight and wrap are typography overrides set only by StyledText's by-value setters; the style applies each over the kind's defaults on its own`（手順の `heading` を消した後） |
| 126 | `Thumbnail` | `thumbnail.rs` | 書く | `independent: source, alt and label are optional, set only by Thumbnail::src, ::alt and ::label; the accessible name combines label and alt case by case, and a missing source shows the placeholder` |
| 127 | `Token` | `token.rs` | 書く | `independent: icon (leading), end (trailing content) and description (the accessible description of the main action) are set only by Token's by-value setters and fill separate parts` |
| 128 | `TopNavItem` | `top_nav.rs` | 書く | `independent: content (a replacement visible label) and icon are set only by TopNavItem::content and ::icon; icon_only hides the label view, never the icon` |
| 129 | `TopNavMegaMenuItem` | `top_nav.rs` | 書く | `independent: description and icon are optional parts of a mega menu entry, set only by their by-value setters` |
| 130 | `TopNavMegaMenuFeaturedCard` | `top_nav.rs` | 書く | `independent: description, image, content and link are optional parts of the featured card, set only by its by-value setters; link already pairs its label with its action in one payload` |

## L12：xtask の `legacy:` の仕組みを消す

- **wave / lane / 前提**：W11a / L / L1〜L11 が merge されていること
- **所有**：`dev/xtask/src/local_checks.rs`、`dev/xtask/src/state.rs`、`dev/xtask/src/state/check.rs`、`dev/xtask/src/state/tests.rs`
- **手順**：
  1. 前提の確認：`rg -n "xtask-allow\(state\): legacy:" crates examples templates dev/tests` が 0 件であること。残っていれば差戻し。
  2. `state/check.rs`：定数 `LEGACY` と `legacy_count` を消す。`state.rs`：`pub(crate) use check::{check, legacy_count};` を `pub(crate) use check::check;` にし、module の doc の「A marker whose text starts with `legacy:` ...」の文を消す。
  3. `local_checks.rs`：`legacy` の件数を数えて note を出す部分（`let mut legacy = 0;` から note の出力まで）を消す。
  4. `state/tests.rs`：
     - `legacy_markers_close_a_type_and_are_counted` を消す。
     - 他の試験の fixture の `// xtask-allow(state): legacy: Open.a, Open.b; probe` を `// xtask-allow(state): independent: probe` に直す。`// xtask-allow(state): legacy: a, b; probe` も同じく直す。
- **確認**：
  ```sh
  cargo fmt -p xtask
  cargo clippy -p xtask --all-targets -- -D warnings
  cargo test -p xtask
  cargo xtask check --no-write
  ```
- **完了条件**：`rg -n "xtask-allow\(state\): legacy:" crates dev examples templates` が 0 件（CP3 の検査と同じ）。上のコマンドが通る。

| # | 項目 | file | 処置 |
|---|---|---|---|
| 167 | 件数の note | `local_checks.rs` | 外す（手順 3） |
| 168 | `LEGACY` 定数 | `state/check.rs` | 外す（手順 2） |
| 169 | 試験 `legacy_markers_close_a_type_and_are_counted` | `state/tests.rs` | 外す（手順 4） |
| 170 | enum の試験の fixture | `state/tests.rs` | 外す（手順 4） |
| 171 | 関数の local の試験の fixture | `state/tests.rs` | 外す（手順 4） |
