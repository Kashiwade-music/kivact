# R-2 read-only preparation packet

Prepared 2026-11-04 against the pre-R1 source paths. R1 must merge before implementation. CP1 PASS is required before W2 implementation. No source edits/build/test/git changes were made for this investigation; this file is an untracked generated artifact only.

## Required design and ownership

Design 6.1–6.2 (`REFACTORING_DESIGN.md:512–607`), work order R-2 (`REFACTORING_WORK_ORDER.md:574–587`), appendix SM-R04/R05/R06/R09/R10/R19 and SM-C02/C05/C06 are authoritative.

Runtime owns TurnState, monotonically advancing clock, root owner, executor/waker, exit/deadline, unstamped/awaiting mounts, and later reports. Core retains engine Writer, event routing, focus/activation requests unless deliberately moved as discussed below, view/removal processing, asset-failure dedup, and embedded-close observation.

Host::wake is UI-thread-only and only called on Quiet -> WakeSent. ThreadWaker calls the engine's thread-safe wake directly, never TurnState. RuntimeConfig has blocking_threads and initial reduced_motion. Runtime::new(host, waker, config, build(&Runtime)->Extensions) must create root before build. Runtime creation moves from AppClient::new into Launch::Started construction after native start. AppConfig remains unchanged in R2 (settings consolidation is C1).

## Exact current startup flow

`crates/kivact-core/src/client/app_client.rs:119–140`: AppClient::new currently creates HostState/HostLink, ViewState::new, optional GPU and trace extensions, TaskState, MotionState, LifecycleState; it registers Runtime before host start.

`app_client.rs:223–272`: start consumes Waiting config; installs task ReadyWaker; installs HostLink clock/waker; creates root owner; view::root::start establishes tree/env/focus/capabilities; writes title/min-size; registers fonts before mounting; mounts settings/view; reports missing capabilities; stabilizes; first commit stamps mounts; completes setup notes/trace; clears request and wakes only for user/ready work.

The replacement must retain root owner -> complete extension signals/tree/environment/focus -> application fonts -> view mount dependencies, with the host clock already correct for setup code.

## Exact current turn flow (prose, not a replacement implementation table)

Sources: `app_client.rs:280–377`, `client/phases.rs:48–105`, `client/settle.rs:16–56`.

1. Advance clock with max(previous, now_ns). If no events and last acknowledgement releases no mount, return output immediately: no turn recorder, Reports, or normal dispatch/settle runs.
2. Compute `before` as first boundary revision minus one (saturating), or Writer's current revision with no boundary. Clear request, enter HostLink turn scope, increment turns, create recorder. Run mounts acknowledged through before, THEN deliver later reports, THEN stabilize.
3. Acknowledge boundaries at delivered=0. For every event, stop early if embedded close flag is set. Deliver TextEditsAhead bookkeeping, record cause, route event, drain requested activations (one snapshot), stabilize if that snapshot was nonempty, ordinary stabilize, then release all boundaries at or before the number delivered and stabilize after each boundary. Finally acknowledge through events.len(), including boundaries after events skipped by embedded close.
4. Capture done=Writer's current revision ONCE. Check draining action deadlines, then settle async tasks/timers.
5. Commit loop: drain requested activations; if snapshot nonempty, stabilize AND settle async. Commit order is motion plan_frame/request_frame -> flush focus -> remove nodes and prune their asset failures -> pending write count -> Writer.commit if nonempty, else current revision -> stamp previously unstamped mounts with that revision.
6. Check mounts due at the saved done revision plus pending user effects. If neither, stop. Increment round; if round > 100, abandon queued user effects, collect locations, report Cycle, and stop. Otherwise run due mounts -> stabilize; run one user-effect batch -> stabilize; settle async; repeat.
7. Compute output, finish trace/recorder. Leave in-turn scope. Calculate pending user effects/ready tasks/later reports/activations; clear request and wake only if something remains. A request raised and fully consumed during the turn does NOT by itself cause another wake.

Do not replace done with the newly committed revision: newly written nodes have not yet been acknowledged by the engine.

## Limits (independent)

- Async settle is at most 1000 rounds (`client/settle.rs:8–55`). Each round takes due timers, records timer cause, wakes all extracted wakers, polls tasks, records task cause if polled, then stabilizes. It stops when no timer woke and no task polled. If still ready/due at limit, report Cycle and retain the remaining work for the next turn.
- Mount/user-effect loop permits 100 rounds, tests `round > USER_EFFECT_ROUNDS` after increment (`app_client.rs:323–347`). It abandons user effects at the limit, not async work.
- Reactive stabilize separately limits one effect to 100 runs and the whole pass to 100*(graph.len()+1) (`crates/kivact-reactive/src/turn.rs:64–99`). R2 must preserve this independent safeguard.
- Executor poll's own 100000 runnable guard exists in `crates/kivact-task/src/spawn.rs:109`; do not silently collapse it into either outer limit.

## Mount, report, exit and drop contracts

`reactive/mounts.rs:27–91`: pending callbacks currently have optional token; stamping asserts monotonic tokens. Ack partitions/takes due callbacks before running user code, preserves order, and skips disposed owners. Newly queued callbacks do not join the current snapshot. Replace with unstamped Vec and awaiting VecDeque<(Revision, Mount)> while preserving these rules.

`reactive/report.rs:61–66`: take later-report snapshot before dispatch; reports queued by report handlers wait for the next report stage.

`core/client/state.rs:62–68`: first close starts now+DEFAULT_DRAIN_NS via saturating_add, repeats retain deadline. `app_client.rs:382–417`: if queues no longer busy, Exited; otherwise deadline expiry abandons queues and reports Closed; next wake is min(task deadline, drain deadline). Runtime alone must own this deadline.

`app_client.rs:422–450`: normal Drop disposes root, abandons queues, polls ready cancellation runnables on UI thread, stops task workers, unregisters runtime; propagating panic skips application code and only stops workers. Embedded detached-module retention remains intact.

## Extension construction: actual accesses

Current constructors do not need runtime extensions:
- ViewState::new (`core/view/state.rs:255–266`) initializes OnceCell/plain cells/Requests containers; it creates no signals and has no Runtime argument.
- TaskState::new (`task/state.rs:79–93`) creates Executor, timers, queues and pool metadata, with no Runtime::ext call. Executor::new (`task/executor.rs:48–53`) creates Shared; set_waker is currently later (`:55`).
- MotionState::default, HostState::default, LifecycleState::default, optional GPU state and trace default construct containers. Their runtime lookups happen through later `of(rt)` calls, not the constructor.

Signal/trigger creation itself requires runtime registry and arenas, NOT Runtime::ext:
- Free signal (`reactive/setup.rs:45–49`) calls setup_scope -> registered current runtime/current owner -> Runtime::create_source.
- Owner::signal and Owner::trigger (`reactive/owner_handle.rs:140–155`) call Owner::rt -> registry::runtime(app, at), then create_source; no extension access.
- Owner::with_setup (`owner_handle.rs:133–137`) needs registry lookup and setup_frame. A free signal constructor additionally requires this live Setup frame; Owner::signal does not require that free-function lookup.
- create_source/insert_node in `reactive/graph.rs` touch graph/owner arenas and diagnostics; create_owner (`reactive/owner.rs:64–99`) touches owner arenas. These normal construction paths do not ask for ext<T>.
- EnvState::new (`core/view/state.rs:105–118`) calls owner.signal twice and creates memo cache metadata. Focus keys (`core/view/focus.rs:keys`) create owner signals and a selector under with_setup, also using registry/arenas.

Actual extension access begins in current startup assembly:
- view::root::start (`core/view/root.rs:49–74`) first calls ViewState::of(rt) -> rt.ext::<ViewState>(), then installs Started containing Tree, EnvState, focus keys, capabilities, min size and optional adapter signal.
- view::root::mount_root (`root.rs:80–120`) reads ViewState extension, creates locale memo, calls MotionState::of(rt).attach_env, creates/provides theme and Messages signals and mounts view.
- `Runtime::ext<T>` (`reactive/ext.rs:17–23`) returns &T from immutable Extensions, panicking for absence. Extensions are `Vec<(TypeId, Box<dyn Any>)>` (`data.rs:421`) and presently never change after runtime registration.
- registry::register (`reactive/registry.rs:48–57`) constructs complete Rc<Runtime> including ext, then publishes it in REGISTRY. This stored Rc is required for Owner-based signal creation during build.

Therefore the earlier question is narrower than 'signals need extensions': they do not. The unresolved construction issue is how build(&Runtime) can create registry-backed signals before installing the returned immutable Extensions, without a forbidden late-filled slot and while preserving ext()->&T. Construction-phase access restrictions or a revised construction protocol still require root decision. Do not introduce OnceCell<Extensions> or unsafe interior references as a workaround.

## Activation queue: exact behavior and options retaining two TurnClient methods

Current enqueue: `core/view/requests.rs:211–233`, request_activation pushes FocusTarget then request_turn. Current execution: `core/client/dispatch/pointer.rs:29–42` takes ONE snapshot, returns true if the snapshot was nonempty even if every target is stale/disabled, rechecks modal scope/mounted/disabled at execution time, and invokes node-owner handlers. An activation enqueued by an activation handler waits until the next drain, not recursively in this batch.

Option A: runtime-owned opaque deferred callback batch (no TurnClient method added). NodeRef activation enqueues a `FnOnce(&Runtime)` payload capturing FocusTarget; the callback stays core-defined and resolves ViewState/disabled/modal/handler only on execution. Runtime drains one snapshot at the existing post-dispatch and precommit positions and knows whether it was nonempty, so it can preserve the conditional stabilize/async settle. Core pending_activations list must be removed, not mirrored. Pass &Runtime at execution rather than capturing strong Rc<Runtime> to avoid a cycle. This preserves the two-method TurnClient contract and keeps runtime independent of core, but is a new runtime queue/API and requires root approval. Source references retain the exact stale/disabled/owner semantics to implement later.

Option B: retain the typed core queue, expose its drain as a construction-time callback in Extensions/runtime hooks rather than a TurnClient method. Runtime invokes this callback at the same two positions and receives the nonempty result. This can retain exactly dispatch/commit as TurnClient methods, but adds a layer-neutral host-work hook contract; extension ownership must not create a Runtime/core cycle or hold RefCell borrows while user code runs. It is broader than Option A and depends on the extension-construction decision.

Putting activation draining only in TurnClient::dispatch and commit is NOT by itself equivalent: there may be no event, commits need conditional async settling BEFORE frame/focus/removal/engine commit, and core would again own choreography. Converting activations to normal events for the next turn likewise changes same-turn timing. A synthetic event/generic Event polling scheme needs an additional event-source contract, not merely the existing two methods.

## Other root decisions required after R1

1. Startup clock and first commit/stamp: new RuntimeConfig has no initial now; only TurnInput.now may carry time. Existing root setup already reads host time for tasks/motion. Define an explicit setup-turn protocol or adjust constructor inputs, and ensure initial mounts are stamped by runtime-owned commit handling.
2. Frame position: existing HostEvent::FrameTick updates HostLink clock and motion in batch order (`core/client/dispatch/env.rs:29–36`), while proposed TurnInput.frame carries at most one frame. Moving all frame work ahead of events can change ordering; decide whether R2 retains frame events until R4 or revises ordered items.
3. Stage misuse matrix: Reports/Mounts/UserEffects callbacks must keep their allowed writes. Existing commit contains focus/removal processing before Writer.commit; locate which operations actually write signals before applying a blanket Commit write prohibition.
4. Wake completion: RunningRequested must not force an empty extra turn when its work settled; preserve the explicit pending-work test and Quiet/WakeSent outcome.
5. Before/done revisions: runtime must receive or retain the committed/acknowledged baseline needed by the before/done logic, including startup. TurnClient.commit returns Revision rather than current writes count; trace operation counting needs a declared source without changing commit order.
6. Embedded close: dispatch suppression checks an Arc<AtomicBool> currently owned by core, but end-of-batch acknowledgements still run. Keep that distinction through generic TurnItem dispatch without quietly dropping mount acknowledgements.

No interface decision or implementation has been made by this packet. Root will decide autonomously after R1; implementation remains blocked on the project sequencing gate, not on user clarification.
