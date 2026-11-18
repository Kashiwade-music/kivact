# Engine audit (kivact-engine, kivact-layout, kivact-editing)

## 0. Summary

1. **Frames do too much.** Each frame lowers every visible node from scratch. For each node it clones and sanitizes its `PaintProps` (`effective_paint`). The paint list is also rebuilt in full (O(visible·depth)) on every layout pass, scroll, hover on a node with state styles, canvas scene write and **every tick while any state transition runs**. So a hover color fade rebuilds the whole list each frame (EN-03, EN-04).
2. **Time has three bases and is fixed up after the fact.** These are host ns, `TimeScale` motion ns and per-window pause. `Frame.now` holds whichever was written last. A pause is repaired by shifting start times in 3 of 5 motion kinds. Image playback, smooth scroll and fling are not shifted. This causes bugs (EN-02).
3. **Recent additions brought bugs and special cases.**
   - Double/triple click selection collapses on the first drag move, and drag-after-double-click extends by character (EN-01).
   - Word segmentation and selection policy sit in the engine. This contradicts "engine holds no editing policy" (EN-21).
   - Vertical caret movement has no goal column (EN-31).
4. **PERF-1 is not met on many paths.** These walk everything on ordinary input:
   - `set_scene` on the focused node computes the whole Tab order each write (EN-05).
   - Opening or closing an overlay, or toggling Contents or fixed, derives layer facts for the whole tree (EN-06).
   - Every frame scrolls every text input (EN-07).
   - Every layout pass re-reports every observed node and scroller (EN-08).
   - Hit testing iterates all children per level (EN-09).
   - Visibility is re-checked by a walk to the root for every node a walk visits (EN-24).
5. **Duplicated owners.**
   - Window maps are kept in parallel (`Host.windows` and `AppModel.windows`).
   - `HostRequest` is mirrored by `DeferredRequest`, and App-wide requests sit in the main window's queue.
   - Three motion-run structs have identical shapes.
   - Two frame logs.
   - Write statistics keep HashSets per write, and the a11y counters silently read 0 outside the `testing` feature.
   - Five per-slot column owners must be pushed and reset in lockstep.

**Target shape.** One `kivact-engine` crate (tree/model + host), organized around four things:
- **One `Slots` column owner.** One push/reset site for per-slot columns.
- **One time owner per window.** A `Clock` maps host ns to motion ns with scale and pause offset; no start times are shifted afterwards.
- **A retained per-node draw cache.** It is invalidated by the same `Dirty` bits, so a frame re-lowers only the nodes that were redrawn.
- **A paint list that is patched rather than rebuilt.** Restacking happens only for real stacking changes.

`kivact-layout` and `kivact-editing` stay as crates:
- `kivact-editing` gains word/line segmentation, and its host-only items go under a hidden `host` module.
- `kivact-layout` drops its testing-only comparison API from the release surface.

## 1. Current structure map

| unit | responsibility | lines | used by | boundary |
|---|---|---|---|---|
| kivact-engine::tree | NodeStore (slots, IdMap), links, ChildOrder, flat-children cache, index sets (scrollers, images, sticky, fixed, observed, overlays, presence) | 1.6k | model, paint, a11y, measure | ok |
| ::model | per-window `Model` (focus, pointer, keys, edit/IME, states, state/layout/presence motion, frame clock, sync dispatcher, geometry/layout bridge, hit test) + `AppModel` | 13.2k | everything | core; too big but cohesive |
| ::measure | layout leaf measurement, editor payloads, computed text column | 0.9k | model, layout pass | fine as module |
| ::lower | values → Arvane render cmds (fill, border, glyphs, assets, scene cache) | 5.5k | paint, measure, offscreen | **leaf: zero `crate::` deps outside itself** (contradicts ADR-10's reason) |
| ::text_input | editor line shaping cache (`EditLines`) | 2.0k | measure, model, paint, a11y | **leaf: zero `crate::` deps** |
| ::paint | retained paint list, ink, child index, frame composition | 3.1k | host | depends on model internals both ways (`HitInk`, `LayerView` single-impl traits) |
| ::a11y | dirt tracking, identity metadata, payload build, publish | 2.2k | host | ok as module |
| ::check | text layout findings (DevTools, test driver, debug lint) | 0.8k | reader, testing | always compiled |
| ::host | pump, turn, effects, window changes, events, devtools host, write API (Writer/Reader/props table), launch/embed | 6.0k (+1.7k testing) | core, kivact-test | parallel state with AppModel |
| ::contract | engine↔core protocol (HostEvent 50+ variants, NodeProperties, HostRequest) | 1.1k | core | internal protocol |
| kivact-layout | Taffy wrapper over a borrowed tree, slot-indexed columns, geometry changes, overlay placement | 3.9k | engine only | real (isolates Taffy), but **shares the slot domain** with engine (push/reset lockstep) |
| kivact-editing | EditState/TextSession/Journal/Editor/Damage, EditBehavior/EditContext/Lines, LineColors, TextLayout | 1.5k | engine, core (custom), style, interaction | real (pure text), but exposes host-only machinery publicly |

Fake or missing boundaries:
- Engine host ↔ model: the host pokes into `model.paint`, `model.drawables`, `model.effects`, `model.external_writes`, `model.window_request`, `model.frame`, `model.pointer`. This is not a boundary; it should stay one crate.
- The `HitInk` and `LayerView` traits exist only to satisfy module direction (one implementor each).
- kivact-layout's "slot allocation domain" contract is shared mutable numbering with the engine.

## 2. Findings

### EN-01 Double/triple-click selection collapses on drag; granularity lost
- kind: BUG
- where: crates/kivact-engine/src/model/pointer/machine.rs:406-423 (`press_text_input`, `select_text`), :379-398, :638-667 (`pointer_move`); model/pointer/state.rs `MouseGesture::SelectText { editor, route }`
- evidence: A double click selects `word.start..word.end` and then sets `MouseGesture::SelectText`. Any later `Move` with Primary held calls `self.place_caret(i, at, true)`. That keeps the anchor at `word.start` and moves the caret to the pointer, so the smallest jitter shrinks the word selection. A drag after a double or triple click extends by character, not by word or line. Touch and pen use a second, separate "selecting" path (focused editor ∈ pressed chain, :651-661).
- fix: `MouseGesture::SelectText { editor, route, unit: SelectUnit }` with `enum SelectUnit { Char, Word(Range<usize>), Line(Range<usize>) }`, set by `press_text_input`. The move calls `extend_selection(editor, at, unit)`. That function keeps the original unit range and extends to the unit boundary at `at` (the same as the native platforms). Touch and pen get the same gesture variant (`TouchGesture::SelectText`), and the derived "pressed contains focused editor" branch is deleted.
- breaking: no   kpi: -   size: S
- confidence: high (code path); medium (how often a move event fires between down and up)

### EN-02 Three time bases; pause handled by shifting starts in 3 of 5 motion kinds
- kind: STATE / BUG / DUP
- where:
  - model/frame/ticks.rs:151-152 (`self.frame.now = self.frame.time_scale.motion_ns(now)`)
  - model/frame/state.rs:97 (`set_now` with host ns, called from host/events.rs:173, pump.rs:464, window_changes.rs:119)
  - shift repairs: state_motion/steps.rs:335, presence/size.rs:108, layout_motion/steps.rs:211
  - not shifted: playback/tick.rs:35 (`Playing { start: now }`), frame/ticks.rs:194-218 (`Fling { last_ns }`, `Running { began_ns }`)
  - also: `AppModel.clock` + `MotionClock` (app_model.rs:28-47), `Frame.clock` (frame/state.rs:40), `Host.app_motion` Sent, `HostWindow.surface`, `Host.presence`
- evidence:
  - With a DevTools time scale ≠ 1, `Frame.now` alternates between scaled motion ns (after a tick) and host ns (after input or a turn). Transitions started from input are compared with ticks in the other base.
  - On resume, `update_env` shifts state, size and layout runs. Animated images and smooth scrolls/flings are not shifted. A fling then computes `dt` = the whole pause, and image playback jumps ahead.
  - "Drawability" is stored six times: HostWindow.surface, Host.presence, Frame.clock, env surface flag, AppClock, app_motion Sent.
- fix:
  - Add one `Clock` type in kivact-types: `struct Clock { scale: TimeScale, paused: Option<u64>, offset_ns: u64 }`, with `fn motion_ns(&self, host_ns) -> u64` (pause- and scale-adjusted) and `pause(at)` / `resume(at)`.
  - `AppModel` owns the App clock. Each `Model` owns its window clock, the same type.
  - `Frame` stores only `host_now: u64`. Every motion record (state tracks, SizeMotion, LayoutAnimation, Playback, SmoothScroll) stores motion ns read through `clock.motion_ns(host_now)`.
  - Delete `shift_state_times`, `shift_size_times`, `shift_layout_times`, `AppClock`, `Frame::clock`, `TickHistory` (it becomes `Option<u64>`), and the `set_now`/scaled overwrite.
  - Drawability is one derived fn `Host::drawable(id)`. Its result feeds `Clock::pause/resume` only at transitions.
- breaking: no   kpi: K3 (fewer repair loops)   size: M
- confidence: high

### EN-03 Every frame re-lowers every visible node and clones its paint
- kind: INEFF
- where: paint/frame_scene.rs:94-150 (`compose_list`), :144 `visual::paint(self, id)`; model/interact/visual.rs:16-20; model/states.rs:191-215 (`paint_with`: `.cloned()` + merge; `effective_paint`); pointer/hover.rs:117 (`cursor` clones per ancestor per pump); hit.rs:177
- evidence:
  - `compose_list` walks all list entries each frame and lowers box, border, text and scenes into a fresh `Composition`.
  - Per node it calls `visual::paint`, which runs `effective_paint` (a full `PaintProps` clone with Vec shadows and backgrounds, and boxed effects) and then `sanitize::paint`. Only DrawList scenes are cached (`SceneCache`).
  - A caret blink or one color change costs O(visible) lowering plus allocations.
- fix:
  - Per-slot `DrawCache` column: `Option<Rc<NodeDraw>>`, where `NodeDraw { paint: Rc<PaintProps> /*sanitized, deltas applied*/, open: Vec<RenderCmd>, close: Vec<RenderCmd> }`.
  - Invalidate it from the `Dirty::WRITTEN|REDRAW` set in `PaintState::outdate_paint`, plus running state motion nodes.
  - `compose_list` copies the cached commands of clean nodes and re-lowers only invalidated ones.
  - `effective_paint` returns `Cow<'_, PaintProps>`: borrowed when there is no motion and no active delta.
  - `cursor()` reads `effective_cursor(id) -> Cursor` without building a PaintProps.
- breaking: no   kpi: K3, K4, K5   size: L
- confidence: high

### EN-04 Paint list rebuilt in full far too often; each visited node walks to the root
- kind: INEFF / CONTRA (paint/list.rs doc: "rebuilt only when something may move …")
- where:
  - model/interact/sync.rs:162-175 (`restack = changes.restack || moved || !restyled.is_empty()`)
  - model/frame/ticks.rs:163-164 (`if self.motion.is_active() { self.changes.restack = true; … }` on every tick)
  - model/state_motion/steps.rs:254-261 (`motion_changed` → restack)
  - model/apply.rs:64 (`move_before`), :79 (`set_scene` → `restack = true` unconditionally)
  - paint/list/walk.rs:119 `host.tree().drawn(id)`; tree/node_tree.rs:300-383 (`attached_where` loops to the root)
- evidence:
  - Any layout pass or scroll (`moved`), any hover or focus change on a node with state styles (`restyled`), any canvas scene write, and every frame of any state transition (a background-color fade included) rebuilds the whole list.
  - The rebuild is O(visible) only in principle. Each visited node calls `drawn(id)`, which walks every ancestor (two hash lookups per step: index→id→`IdMap`). The real cost is O(visible·depth).
- fix:
  - Restack only on stacking-fact changes: `Look::stacking()` or `Place` differs, children regrouped, layer facts changed.
  - Motion ticks of color, opacity-in-layer or transform-of-layer go through redraw (EN-03 cache plus the layer matrix). They do not go through restack.
  - `moved` from scroll updates the scroller's step origin and clip only. That needs a subtree re-walk from the scroller (`Walk::node(scroller)` splice), not a root walk.
  - The walk passes `drawn: bool` down instead of calling `tree.drawn(id)`.
  - Delete `restack = true` from `set_scene`: a scene's ink change already forgets ink via `WRITTEN`.
- breaking: no   kpi: K3, K4, K5   size: L
- confidence: high

### EN-05 Every scene write to the focused node computes the full Tab order
- kind: INEFF (PERF-1)
- where: model/apply.rs:76-121 (`set_scene`: `let old_order = if self.focused()… { self.focus_order() }`)
- evidence: A focused canvas (chart, slider, custom control) that redraws each frame runs `focus_order()` (a full `flat_preorder` plus region scan, O(N)) per write. This happens even when the focus stays valid.
- fix: Capture only the focused node's own region order before the write (`node.regions_in_focus_order()`, O(regions)). After the write, if `!is_focus_target(focus)`, pick the next region of the same node by index, and only then fall back to `next_tab_stop_after(focus.node)` (one O(N) call, only when focus is really lost).
- breaking: no   kpi: K3, K4   size: S
- confidence: high

### EN-06 Overlay open/close, a Contents toggle or a fixed toggle re-derive everything
- kind: INEFF (PERF-1)
- where: model/apply.rs:275-281 (`reflattened`: `tree.forget_flat()` + `stale_all_facts()`), :362, :511, :543, :634; model/lifecycle.rs:24; paint/list/walk.rs:483 (`StaleFacts::All => *facts = self.layer_facts()`)
- evidence:
  - Opening or closing any tooltip, menu, popover or dialog (an Overlay) marks all facts stale. The next list runs `layer_facts()` over every drawn subtree.
  - Toggling a Contents node clears every cached flat-children list. A targeted `forget_flat_around` exists but is not used.
- fix:
  - Layer roots are kept in `NodeTree::overlays`/`fixed`.
  - Adding or removing one layer root stales that root's facts and removes its entry (`facts.remove`). It does not stale all.
  - `reflattened(id)` uses `forget_flat_around(id)` and `stale_fact(flat_parent)`.
  - `StaleFacts::All` remains only for a new root and for fonts.
- breaking: no   kpi: K3, K5   size: S
- confidence: high

### EN-07 Every text input is scrolled to its caret every frame and every pump
- kind: INEFF / STATE (an effect that repairs derived state)
- where: model/interact/text_geometry.rs:296-338 (`scroll_inputs` loops `input_ids()`), host/pump.rs:355 (each `present`), model/interact/caret.rs:8 (each `finish_window` → `synchronize_ime`), each run ending in `self.sync()`
- evidence: For every editor it shapes the caret (`caret_local`), refreshes lines, reads `rect()` (an ancestor walk) and writes `scroll_x`/`scroll_y`. It runs twice per pump+frame, even when nothing changed. A form with 100 fields does 100 caret shapings per frame.
- fix: Add `Dirty::CARET` (set by `edit_editor_with`, controlled value, `LayoutReasons` on the editor, resize). `scroll_inputs` iterates `changes.take(Dirty::CARET)` only, and is called once from `settle` (not from `present` or `caret()`).
- breaking: no   kpi: K3, K4   size: S
- confidence: high

### EN-08 The geometry report walks every scroller and every observed node per layout pass
- kind: INEFF (PERF-1)
- where: model/interact/geometry.rs:562-631 (`report_geometry`: `scrollers ∪ observed` loop, `bounds()`, `window_layout_rect()`, `text_truncated()` per node), :153 (`refit_tooltips(engine, !computed)` re-lays out every keep-anchor-clear tooltip on scroll-only passes)
- evidence:
  - core observes every virtual-list row wrapper (kivact-core view/flow/virtual_list.rs:522) and every node with an `on_layout` handler.
  - Each layout pass recomputes window rects (O(depth) each) and shapes text for truncation for all of them, even when one node moved.
- fix: Report only observed or scroll nodes that are in the layout's `GeometryChanges`, have an ancestor in it, or sit under a scrolled container (`Dirty::SCROLLED`). Keep a `BTreeSet` of observed nodes per nearest scroller so a scroll touches only its own. Refit a tooltip only when its anchor moved or its own geometry changed.
- breaking: no   kpi: K3, K5   size: M
- confidence: high (code); medium (how common large observer counts are)

### EN-09 Hit testing iterates all children per level and sorts per hit
- kind: INEFF
- where: model/interact/hit.rs:182-232 (`for c in self.stacked(n).rev()` + `may_hit`), model/transform.rs:545-557 (`stacked` collects and sorts when the container stacks)
- evidence:
  - Ink pruning skips descending into a child, but every child of every node on the path is still visited. That means 10k rows → 10k `may_hit` checks plus id hashing per pointer move.
  - Containers with z-indices allocate and sort their children on every hit.
  - The paint list already keeps a `ChildIndex` (sorted starts + furthest) for containers with ≥32 children, but hit testing does not use it.
- fix:
  - `hit_box` asks `paint.list.indices.get(&id)` and, when present, binary-searches the candidate children by `p` along the index axis (with `furthest` for overlap). Otherwise it uses the linear loop.
  - The stacked order is cached per container in the paint list (`PaintList::stack_order: HashMap<NodeId, Rc<[NodeId]>>`, dropped on `regroup` or a z change) and shared by walk and hit.
- breaking: no   kpi: K3, K5   size: M
- confidence: high

### EN-10 The a11y Metadata path enumerates twice; the PERF-1 exception is too broad
- kind: INEFF / PROCESS
- where: a11y/build/dirty.rs:107 and :183 (two `semantic_order()` calls per Metadata build), a11y/build/node.rs:414-421 (`clipping_of_all` iterates every slot, `tree().iter()`)
- evidence: Any structural write (one row appended or removed, a virtual list scrolling and remounting rows) while assistive technology is active runs three full walks. PERF-1 explicitly exempts "a11y の構造・role・名前の変化（Metadata の経路）", so this is legal but costly exactly where lists change.
- fix:
  - Compute `order` once and pass it to `build_semantics_tail`. `clipping_of_all` iterates the order (only attached nodes), not all slots.
  - Make metadata incremental per regrouped parent: `Metadata.parents`/`children` updated for `Dispatched::regrouped()` subtrees only.
  - Change PERF-1 to drop the Metadata exception.
- breaking: no   kpi: K3, K5   size: S (dedupe) / M (incremental)
- confidence: high

### EN-11 Tab order is O(N) per Tab press, revalidation and focused-removal
- kind: INEFF (PERF-1 lists no exception)
- where: model/focus/navigation.rs:67 (`focus_order`), :116, :370 (`next_tab_stop_outside`), :385-405 (`revalidate_focus`); model/apply.rs:578-583
- evidence: Each call is a full `flat_preorder` with a region scan and `HashSet` per node. Each candidate check calls `tree.shown()` (an ancestor walk).
- fix: Implement `next_tab_stop(from: FocusTarget, forward) -> Option<FocusTarget>` as a preorder walk that starts at `from` and stops at the first eligible target (skip subtrees that are hidden or out of modal scope). Then delete `focus_order()` except for the DevTools/testing listing.
- breaking: no   kpi: K3, K5   size: M
- confidence: high

### EN-12 Wrapped editors rebuild all lines on every edit; the width-probe cache never receives damage
- kind: INEFF / CONTRA (text_input.rs doc: "an edit … cost the lines they touch")
- where: text_input/refresh.rs:95-103 (`fits = wrap_width().is_none() && …splice`), measure/input.rs:82-88 (`damage: if measurement { None } …`), measure/editor.rs:52-58 (`flush_damage` feeds only `lines`)
- evidence:
  - With `wrap` on, every revision takes `rebuild_with_metrics` (split and count the whole document, drop shaped lines).
  - `measure_lines` (the content-sized wrapped editor's width probe) always gets `damage: None`, so it rebuilds on every edit.
  - Damage travels EditState → Editor::take_damage → EditorNode::flush_damage → EditLines.damage → Source: three holders, one consumer.
- fix:
  - `EditState` keeps a bounded change log addressable by revision: `fn damage_since(&self, rev: u64) -> Option<Damage>`.
  - Each `EditLines` asks with its own `revision`, so any number of caches catch up. Delete `EditState.damage`, `take_damage`, `flush_damage`, `EditLines.damage`, `hold_damage`.
  - Implement splice for wrapped hard lines (a wrapped hard line is still one paragraph per hard line, so the same splice applies).
- breaking: no (`Editor::take_damage` is engine-only)   kpi: K5   size: M
- confidence: medium (rebuild cost depends on lazy shaping)

### EN-13 `ImeTarget` is a cached derivation of focus + editor + props
- kind: STATE
- where: model/edit/state.rs:85-95, model/edit/session.rs:36-51 (`update_ime_target`, written from focus/navigation.rs:311, :386); read at keys.rs:206, session.rs:30, host/write/reader.rs:433
- evidence: `ImeTarget::{None, Native(slot), Session(id)}` is recomputed from `focused()`, `disabled()`, `editors[slot]` and `properties.ime`. It is refreshed only on focus change and commit. A disable, editor gain/loss or `ime` prop write between commits leaves it stale. Every IME entry also duplicates the dispatch (`session_*` early-return then native path; `ime_select` runs two journals through two `place_caret` calls).
- fix: Delete the field. Add `fn ime_target(&self) -> ImeTarget`, computed on demand (O(1)). Every IME entry starts with `match self.ime_target()`. `ime_select` does one `edit_input(|e| e.select(start..end))`.
- breaking: no   kpi: -   size: S
- confidence: high

### EN-14 Edit entry repairs the behavior's claim; the limit is stored as state
- kind: STATE / API
- where: model/keys.rs:240-298 (`edit_editor_with`: `reported` mutated by revert, truncate, read-only revert, adopt); kivact-editing/src/behavior.rs `EditOutcome::Done { changed }`, `Copy { changed }`; edit_state.rs `max_length` (set each journal by `Editor::journal`)
- evidence:
  - A behavior says `changed: false` while changing the text, and the host reverts it.
  - `changed` duplicates `journal.text_changed()`.
  - `max_length` is a node prop copied into the document before each edit.
- fix:
  - `EditOutcome::{NotEditing, Done, Copy(String), Paste}`. The host reports `TextChanged` iff `journal.text_changed()` after the limit and read-only steps.
  - `Journal::new(state, Limits { max_length, read_only })` holds the limits for the journal's lifetime. Delete `EditState.max_length`.
  - The order of steps becomes a fixed function `fn close_edit(journal, limits) -> Option<Vec<TextChange>>`.
- breaking: yes (`kivact::custom::EditOutcome`)   kpi: -   size: S
- confidence: high

### EN-15 Parallel window maps; a close flag synced into the phase
- kind: STATE / DUP
- where: host/pump.rs:108 (`Host.windows: BTreeMap<WindowId, HostWindow>`) vs model/app_model.rs:38 (`AppModel.windows: BTreeMap<WindowId, Model>`), with `.expect("invariant: a presented window has a model")` patterns throughout pump.rs; pump.rs:87 `close: CloseSignal` + `HostPhase::Closed` + `observe_close()` (20 sites)
- evidence: The two maps must contain the same keys (insert in window_changes, remove in two places). The closed fact lives in the signal and is copied into `phase` by polling.
- fix:
  - `AppModel.windows: BTreeMap<WindowId, Window { model: Model, host: HostWindow }>`. HostWindow fields stay `pub(in crate::host)`.
  - `HostPhase::{Starting{next_wake}, Running{next_wake}, Exiting}`, plus `fn closed(&self) -> bool { self.close.is_set() }`.
  - Delete `HostPhase::Closed` and `observe_close`.
- breaking: no   kpi: -   size: M
- confidence: high

### EN-16 `DeferredRequest` mirrors `HostRequest`; App-wide requests sit in a window queue
- kind: DUP
- where: model/aggregate.rs:14-24 (`DeferredRequest`), host/write/writer.rs:420-470 (`queue_main(.., DeferredRequest::ReducedMotion/Emulate/TimeScale)`), host/pump.rs:170-266
- evidence: Seven variants are copied from `HostRequest` minus `window`. App-wide effects (reduced motion, emulation, time scale) are pushed into the **main window's** `Model.effects`.
- fix: Use `AppModel.effects: Vec<(RequestId, HostRequest)>` (one FIFO, App-owned). Delete `DeferredRequest` and `Model.effects`. `Host::effects` matches `HostRequest` directly.
- breaking: no   kpi: -   size: S
- confidence: high

### EN-17 Three motion-run structs with one shape
- kind: DUP
- where: model/presence/state.rs:17-25 (`SizeMotion`), model/layout_motion/state.rs:24-38 (`LayoutAnimation`), model/state_motion/state.rs (`TrackPhase::Running{start, spring}`), plus `SmoothScroll` and `playback::Entry`
- evidence: Each has `{ from: Vec<f64>, to: Vec<f64>, current: Vec<f64>, timing: Transition, start: u64, spring: SpringRun }` (SizeMotion uses scalar f64 and `Vec` current). Each has its own step/shift/settle code.
- fix: Put one `MotionRun { from: SmallVec<[f64;4]>, to, current, timing: Transition, start_motion_ns: u64, spring: SpringRun }` with `fn step(&mut self, motion_ns) -> Settled` in `model/motion_run.rs`. The three owners keep their keyed maps of `MotionRun`. Delete the triplicated step and shift code (with EN-02).
- breaking: no   kpi: K6 (code)   size: M
- confidence: medium (state tracks have per-prop interp extras)

### EN-18 Write statistics keep HashSets per write; the public a11y stats lie
- kind: INEFF / DUP / CONTRA
- where: crates/kivact-engine/src/mutation.rs:23-90 (`Writes { created, updated, touched: HashSet<NodeId>, sparse_touched_revisions }`), host/write/reader.rs:70-90 and writer.rs:135-153 (identical `stats()`), `#[cfg(feature = "testing")]` filling `a11y_*`
- evidence:
  - Every prop write does up to two HashSet inserts only to report `last_ops`/`touched_last`. `touched` duplicates `Changes.dirty` WRITTEN.
  - `kivact::diagnostics::Stats.a11y_constructed/published/enumerated` are always 0 in non-test builds.
- fix:
  - `Writes { stats: WriteStats, pending: u32 }` with counters only. `touched_last` = `changes.touched().count()` at commit.
  - Count a11y always (three u64 adds) and drop the cfg.
  - Keep one `stats()` on `Reader`. `Writer::stats` delegates or is deleted.
- breaking: no   kpi: K3   size: S
- confidence: high

### EN-19 Two frame logs; duplicated launch code
- kind: DUP
- where: frame_log.rs (`KIVACT_FRAME_LOG` file) vs `EngineLog::frames_recording` (DevTools ring) in pump.rs:404-439; renderer-change closure duplicated host/launch.rs:39-45 and embed.rs:76-82
- evidence: Two recorders of the same `FrameRecord` facts, with different formats.
- fix: One `FrameRecord` stream in `EngineLog`, and the file writer as one optional sink (`EngineLog::with_file_sink`). One `fn renderer_logger(log) -> impl FnMut` shared by launch and embed.
- breaking: no   kpi: K6   size: S
- confidence: high

### EN-20 `KeyResult::Text` is documented wrong and carries a dead payload
- kind: CONTRA / DEAD
- where: model/keys.rs:79-85 (doc: "the platform's text input inserts it; a host without one calls insert_text"), :150-152 (`key()` always calls `insert_text`), host/events.rs:493 (`KeyResult::Text(_) => {}`)
- fix: `enum KeyResult { Reported, Consumed, Copy(String), Paste(NodeId) }`. `handle_key` inserts the text itself and returns `Consumed`.
- breaking: no   kpi: -   size: S
- confidence: high

### EN-21 Word/line selection is engine policy; word means a whitespace run
- kind: CONTRA / STRUCT
- where: text_input/word.rs (216 lines), model/pointer/machine.rs:406; keys.rs module doc ("The engine holds no editing policy of its own")
- evidence:
  - The double-click word is "a run of non-whitespace": `hello,` includes the comma, and a whole Japanese sentence is one word.
  - It lives in the engine although `kivact-editing` already depends on `unicode-segmentation`.
- fix:
  - Move it into kivact-editing: `EditState::word_at(at) -> Range<usize>` via `split_word_bound_indices` (UAX #29; a whitespace run stays its own unit), and `EditState::line_at(at, multiline) -> Range<usize>`.
  - The engine calls them. Delete engine `text_input/word.rs`.
- breaking: no   kpi: -   size: S
- confidence: high

### EN-22 Two different types named `LeafMeasure`
- kind: CONTRA (VOC-3)
- where: contract/measure.rs (`pub struct LeafMeasure`, the leaf measurement callback) vs measure/state.rs:19 (`enum LeafMeasure { None, Natural, Text }`)
- fix: Rename the column enum to `LeafCache`.
- breaking: no   kpi: -   size: S
- confidence: high

### EN-23 Per-node memory is about 1.5 KB
- kind: INEFF (K2)
- where:
  - tree/node.rs:22-43 (HostNode ≤312 B; `scroll: Point` and `shift: Point` inline on every node)
  - contract/node.rs:36-74 (NodeProperties ≤240 B; `NodeInput` = 8 `Option<bool>` + `Option<FocusTarget>` + `Option<RovingFocus>`)
  - layer.rs:47-109 (`Look` ≤72 B, 14 bools) in `paint.look: Vec<Option<Look>>`
  - paint/state.rs:69 (`ink: Vec<Cell<Option<Ink>>>` ≈ 90 B)
  - kivact-layout columns.rs:224-260: `taffy::Cache` per slot (≈520 B: 1 final + 9 measure entries), `unrounded: Vec<taffy::Layout>` and `final_layout: Vec<Option<taffy::Layout>>` (two ≈90 B copies), `ChildCursor`, `NodeLayoutState { calcs: Vec, measured: Record { Vec } }`
- fix:
  - Move `scroll` to `NodeTree.scroll: HashMap<NodeId, Point>` (keys = `scrollers`) and `shift` to a map keyed by `sticky` (MEM-1).
  - `NodeInput` becomes `flags: InputFlags(u16)` (two bits per tri-state) + `focus_on_press: Option<FocusTarget>`; roving stays.
  - `Look` bools become `LookFlags(u16)`.
  - kivact-layout: keep `final_layout` only as the rounded `NodeLayout` (f64 values the engine reads) and drop `unrounded` after rounding (taffy needs it only during a pass: use a pass-scoped scratch `Vec` indexed by slot).
  - Allocate `taffy::Cache` only for slots that took part in a pass (`Vec<Option<Box<Cache>>>`).
  - Tighten the MEM-6 asserts to the new sizes.
- breaking: no   kpi: K2   size: M
- confidence: medium (sizes estimated from field types, not measured)

### EN-24 Ancestor walks hash every step; window rects are never cached
- kind: INEFF
- where: tree/node_tree.rs:216-295, 370-383 (`parent_id` → `id_at` → `nodes.get(&id)` per step); model/interact/geometry.rs:244-283 (`rect()` O(depth), called by a11y bounds, hit, text_box, scroll_inputs, IME, geometry reports)
- evidence: Each ancestor step converts NodeIndex → NodeId → IdMap hash → node, although links already hold `NodeIndex`. `shown()`, `drawn()` and `chain()` run inside per-node loops (paint walk, focus order, a11y).
- fix:
  - Add `NodeTree::ancestors(index) -> impl Iterator<Item = (NodeIndex, &HostNode)>` using links only, and rewrite `attached_where`, `flat_parent`, `chain`, `is_in_subtree` and `depth` on it.
  - Top-down walks pass inherited `shown`/`drawn` flags (EN-04).
  - Add a per-slot `window_origin: Cell<Option<(Generation, Point)>>` column invalidated by `Geometry::Moved/LaidOut`.
- breaking: no   kpi: K3, K4, K5   size: M
- confidence: high

### EN-25 Five per-slot column owners pushed and reset in lockstep
- kind: STRUCT / STATE
- where: model/lifecycle.rs:8-50 (`create_slot`: `layout.push(); measure.push(); paint.push_slot(); editors.push(None)`; `remove_slots` resets each), tree/node_tree.rs (`flat` cache sized lazily), kivact-layout `Layout::push/reset` ("Each instance belongs to one structural slot-allocation domain")
- evidence: Five owners each keep an index-aligned Vec. Forgetting one push/reset is a silent misalignment. The layout crate has to trust the engine's numbering.
- fix:
  - `tree::Slots` owns `NodeStore` plus one `SlotColumns { measure, ink, look, editor, flat, draw /*EN-03*/ }` with a single `push()`/`reset(slot)`.
  - kivact-layout columns grow lazily from `LayoutTree::slot_count()` (`ensure(slot)`) and are reset through `Layout::forget(slot)` from the same single reset site.
- breaking: no   kpi: -   size: M
- confidence: high

### EN-26 The engine copies shell input state; dispatch-order state lives in the engine
- kind: DUP / CONTRA (architecture "modifiers / mouse: shell の ModifierState / Mouse。engine に同じ入力状態を複写しない")
- where: model/pointer/state.rs:430-445 (`PointerSource { buttons, pressure, tilt, time_ns }` copied from each sample), model/modal/state.rs (`Admission { drags, adoptions }`: a second per-pointer table with `RefCell` for dispatch order), host/write/writer.rs:94-108 (`adopt_pointer` and `adopt_pointer_target`, two entry points)
- fix:
  - `Track` stores only what the engine decides (over, hover, gesture). Samples are passed into `pointer_down/move/up` as `&PointerSample` and forwarded.
  - Move `Admission` (drags plus adoptions) to core's dispatcher, which owns dispatch order. The engine `Activate` stays plain.
  - Delete `Writer::adopt_pointer*`, `forget_adopted_pointer`, `prepare_input`, `contract::Activation`.
- breaking: no (engine↔core internal)   kpi: -   size: M
- confidence: medium (core side not audited)

### EN-27 Redundant syncs per frame and per pump
- kind: INEFF
- where: model/interact/sync.rs:107 `sync()`; it is called from `settle` (twice), `scroll_inputs`, `key`, `paste`, `preedit`, `commit`, `ime_select`, `poll_assets`, `tick`. pump.rs:494-498 runs `poll_assets` + `settle` per window per pump-loop iteration. sync.rs:121-126 (`paint.forget_missing`, `assets.prune_reports` scan their whole report sets each sync). sync.rs:251-254 (`report_images` over **all** images after each layout pass).
- fix:
  - Let `sync` early-return when `Changes` is empty and nothing else is pending (`Changes::is_clean()`).
  - Remove `forget_missing`/`prune_reports` from sync and call them from `remove_now` with the removed ids.
  - `report_images` only for images in `Dispatched` or polled as changed.
  - Input entry points don't sync; the pump's settle does.
- breaking: no   kpi: K3, K5   size: S
- confidence: high

### EN-28 Dead or testing-only code in the release surface
- kind: DEAD
- where:
  - host/window.rs:17-22 (`Content::Foreign { child }` under `#[allow(dead_code)]`)
  - kivact-layout layout.rs:118-131 + compare.rs (`compare`, `compare_measurements`, `rebuild_all`, `Clone` on columns: used only by engine `check/layout_reference.rs`, cfg(debug+testing))
  - `RefreshMode::Verify` (text_input/refresh.rs:17-24, compiled in release with `expect(dead_code)`)
  - `check` module plus `Reader::layout_findings`, and `DevtoolsHost` overlay/pick (host/devtools.rs), always compiled though only core's `devtools` feature calls them
- fix:
  - Drop `child` (it is in the ready event).
  - Move the layout comparison behind kivact-layout feature `reference` (enabled by engine `testing`).
  - `RefreshMode` becomes cfg(debug_assertions) only.
  - Engine feature `devtools` (forwarded from core `devtools`) gates check, overlay and pick.
- breaking: no   kpi: K6   size: S
- confidence: high

### EN-29 The state-field and direction rules create noise and single-implementor traits
- kind: PROCESS
- where:
  - mutation.rs:96-135 (10 byte-identical `xtask-allow(state-fields): independent:` lines)
  - layer.rs (19 markers, each a copy of the doc line)
  - contract/node.rs (24 `option: absent means …` on passive props)
  - layer.rs:31-45 (`LayerView` trait, one implementor), model/interact/hit.rs:41-45 (`HitInk`, one implementor)
- fix:
  - `Mutation` becomes a `u16` bitset like `Dirty` (`const STYLE: Mutation = Mutation(1)`, …).
  - `Look`/`SubtreeFacts` flags become bitsets (no markers needed).
  - The S-2 checker should skip types that are never mutated through `&mut self` methods (passive records, props, events). Delete the per-field markers there.
  - Delete `LayerView` and `HitInk`: paint and hit test are the same unit (the paint list owns ink), so `model::interact::hit` moves to `paint::hit`.
- breaking: no   kpi: -   size: S
- confidence: high

### EN-30 Host lifecycle phases are split across two enums
- kind: STATE (minor)
- where: host/launch.rs:85-89 (`Launch::{Pending, Starting, Running}`) and pump.rs:39-59 (`HostPhase::Starting`)
- fix: `Launch::{Pending, Running(Box<Host>)}`, with `Host::begin` returning the host. `HostPhase::Starting` remains the single "inside the shell start" fact.
- breaking: no   kpi: -   size: S
- confidence: medium

### EN-31 Vertical caret movement has no goal column (cross-area)
- kind: BUG
- where: kivact-interaction/src/text_edit/policy.rs `vertical_target` (uses the current caret x each time); kivact-editing session.rs (no goal)
- evidence: Down through a short line and on to a long one drifts to the short line's end x.
- fix: `TextSession { anchor, caret, marked, goal_x: Option<f64> }`. `EditContext::x_of` is used only when `goal_x` is None. Every non-vertical `move_to` clears it, and vertical moves set it.
- breaking: no   kpi: -   size: S
- confidence: high

### EN-32 Host-only editing machinery is public API of kivact-editing
- kind: API / STRUCT
- where: kivact-editing lib.rs:55-64 (`Editor`, `Journal`, `Incoming`, `Taken`, `Damage` pub; only the engine uses them)
- fix: Put them in `pub mod host` with `#[doc(hidden)]` plus a reasoned internal-contract (API-8). The user surface stays EditState, TextSession, TextChange, EditOutcome, EditRequest, EditContext, TextLayout, EditLine, EditCaret, LineColors…
- breaking: no (not re-exported by the facade)   kpi: -   size: S
- confidence: high

## 3. Target structure for this area

```
kivact-types        (+ Clock: pausable, time-scaled motion time)       [EN-02]
kivact-editing      pure document: EditState(+damage_since, word_at,  [EN-12, EN-21, EN-31]
                    line_at, goal_x), TextSession, TextChange, Journal(limits),
                    behavior contracts; `host` hidden module (Editor, Incoming, Taken)
kivact-layout       Taffy wrapper; lazily grown columns; `reference`  [EN-23, EN-25, EN-28]
                    feature for comparisons
kivact-engine
  tree/             NodeStore + Slots (single push/reset of all per-slot columns),
                    ancestor iterator over NodeIndex, sparse scroll/shift maps
  model/            Model, AppModel{windows: Window{model, host}, effects, clock}
    input/          pointer (Track + SelectText{unit}), keys, edit/IME (ime_target() derived)
    focus/          next_tab_stop walk (no full order)
    motion/         MotionRun + state/presence/layout owners, playback, smooth scroll (motion ns)
    changes, sync   Dirty bits incl. CARET; sync no-op when clean
  measure/          leaf measurement, LeafCache column
  text_input/       EditLines (damage via EditState::damage_since)
  lower/            value → RenderCmd
  paint/            list (patched; restack only on stacking facts), ink, child index,
                    stack order cache, hit test (moved here), DrawCache, frame composition
  a11y/             dirt, metadata (single enumeration; incremental per regroup), publish
  host/             pump/turn/effects (HostRequest FIFO), events, window changes,
                    write API; devtools + check behind engine feature `devtools`
```

Boundary decisions:
- **kivact-engine stays one crate.** The host reads model groups directly (paint redraw, drawables, effects, IME). The model and paint share ink, stacking and hit testing. Neither split passes B-1.
- **kivact-layout stays.** It isolates Taffy (generic-heavy compile unit), its inputs and outputs fit in a few types (`LayoutTree`, `LeafMeasurer`, `NodeLayout`, `GeometryChanges`), and it has one reason to change. Fix the slot coupling (EN-25) and the testing surface (EN-28).
- **kivact-editing stays.** It is a pure document model shared by engine, core, style and interaction. It absorbs word/line segmentation and the goal column. It does not merge into the engine, because core and style must not depend on the engine.
- **`lower` and `text_input`: owner decision** (§5). Evidence: both have zero `crate::` dependencies outside themselves. ADR-10's reason ("計測・model・paint が内部情報を共有") no longer holds in code. But `lower`'s surface used by paint is 15 `pub(crate)` modules, which would need narrowing first.

Delete:
- `DeferredRequest`, `Model.effects`, `AppClock`, `Frame.clock`, `TickHistory`, `shift_*_times` ×3, `ImeTarget` field.
- `KeyResult::Text`, `HostPhase::Closed`, `observe_close`, `Launch::Starting`.
- `Writes.{created,updated,touched,sparse_touched_revisions}`, `Writer::stats` duplicate, `FrameLog` as a separate recorder.
- `LayerView`, `HitInk`, engine `text_input/word.rs`, `Content::Foreign.child`, `EditState.max_length`, `EditState.damage`, `EditorNode::flush_damage`, `EditOutcome::*{changed}`.
- `Admission`/adoption (moved to core).

## 4. Public API (kivact-editing items re-exported via `kivact::custom`)

Custom editor behavior:
```rust
// before
TextEditing::new().behavior(|state: &mut EditState, req: EditRequest<'_>, cx: &EditContext<'_>| {
    match req { EditRequest::Key { key, .. } if key == Key::Tab => {
        state.replace(state.selection(), "  ");
        EditOutcome::Done { changed: true }          // must match reality or the host reverts
    } _ => EditOutcome::NotEditing }
})
// after
TextEditing::new().behavior(|state: &mut EditState, req: EditRequest, cx: &EditContext| {
    match req { EditRequest::Key { key, .. } if key == Key::Tab => {
        state.replace(state.selection(), "  ");
        EditOutcome::Done                             // the host sees the change itself
    } _ => EditOutcome::NotEditing }
})
```
- `EditRequest::Paste(String)` (owned) removes the lifetime from `EditRequest`.
- `EditContext` becomes an opaque handle with no lifetime parameter in the user signature (it holds an `Rc` to the line snapshot).
- `EditContext::new(bool, bool, bool, Option<EditLayout>)` becomes `pub(crate)` (users never build one). `EditLayout` and the `Lines` trait become `#[doc(hidden)]` host contracts.
- `EditOutcome::{NotEditing, Done, Copy(String), Paste}` drops `changed`.
- New: `EditState::word_at(at)`, `EditState::line_at(at)`, `EditState::select(range)` (replaces two `move_to` calls).

Names to delete or merge:
- `EditOutcome::Done{changed}` / `Copy{changed}` → `Done` / `Copy(String)`.
- `Editor`, `Journal`, `Incoming`, `Taken`, `Damage` → `kivact_editing::host` (hidden).
- core `custom::Decoration`/`DecorationKind` vs kivact-editing `RangeDecoration`/`RangeDecorationKind`: one type, re-exported (VOC-1). Decide in the core audit.

## 5. Open questions for the owner

1. **ADR-10 revisit.** Should `lower` (+ `text_input`) become a crate such as `kivact-lower` after its surface is narrowed to about 5 entry functions? The code no longer supports ADR-10's reason, but a split only pays off with a narrow API.
2. **Per-window vs App-wide pause** (EN-02). Should engine state transitions pause per window (current) or with the App clock (ADR-49 already made client motion App-wide)? One App `Clock` removes the per-window clock entirely.
3. **PERF-1 Metadata exception** (EN-10). Should it be removed, making a11y metadata incremental per regrouped parent, given that virtual lists remount rows on every scroll?
4. **Retained per-node draw cache** (EN-03). This is the largest K4 win but adds per-node memory (≈ the lowered command bytes of visible nodes only, evicted for culled nodes). Accept the K2 trade-off?
