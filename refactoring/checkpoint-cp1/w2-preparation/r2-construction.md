# R-2 construction recommendation (read-only)

Prepared 2026-11-04. Companion: r2-packet.md. No implementation/interface decision has been applied. R1 must merge and CP1 must pass before R2 work.

## Recommendation

If `Runtime::ext<T>() -> &T` and complete ViewState-in-Extensions remain requirements, prefer a **narrow typed construction allocator** over the proposed ordinary `build(&Runtime)` callback. The simple prebuilt-Extensions signature is attractive for today's empty extension containers, but it does not solve the C1 requirement: complete ViewState itself contains runtime-owned reactive handles. Moving signal creation after Runtime.new can only put those handles into an already installed ViewState through a late-filled slot, or move ViewState out of Extensions and rewrite its lookup/ownership model. Neither is an incremental S7-compliant solution to the agreed final design.

This recommendation does not propose a second runtime, a general alternate setup API, or a registry entry containing a partially usable Runtime. Limit the allocator to source/trigger allocation actually needed by subsystem construction; publish the ordinary registered Rc<Runtime> only once the finished Extensions and root exist. Root decides the actual signature and scope.

## Evidence distinguishing current constructors from final dependencies

- `core/client/app_client.rs:119–140` constructs ViewState::new, TaskState::new, MotionState::default, LifecycleState, HostState and optional GPU/trace before Runtime.new. These constructors need no signals only because ViewState has `started: OnceCell<Started>` and MotionState has `env: Cell<Option<MotionEnv>>`.
- `core/view/state.rs:199–224`: Started contains Tree, EnvState, capabilities, FocusKeys, minimum size and GPU adapter signal. ViewState::start installs it after runtime creation (`:268`).
- Work order C1 (`REFACTORING_WORK_ORDER.md:637–650`) explicitly finishes ViewState during Launch::Started construction. Appendix SM-C08 (`refactoring/A-state-machines.md:254` vicinity; locate heading rather than relying on shifted line numbers) says Started's contents become ViewState fields, eliminating the OnceCell.
- Design's App-state table (`REFACTORING_DESIGN.md:706–709`) keeps core subsystem state in runtime Extensions, all installed at construction, and requires currently lazy Tree/ViewState triggers to exist at construction.
- `core/view/state.rs:105–118`: EnvState::new allocates two owner signals. `core/view/focus.rs:26–35` allocates focus/visible signals plus a Selector. `core/view/tree.rs:298–313` constructs tree/root, and `:372–391` currently allocates two triggers lazily; design requires those eagerly.
- `core/view/root.rs:49–74` installs Started by first looking up ViewState via Runtime::ext. `:80–120` reads extensions, derives locale, attaches MotionEnv, installs theme/messages, then mounts user view. This assembly must be reordered, not simply moved verbatim into the constructor callback.
- `reactive/setup.rs:45–49` and `owner_handle.rs:140–155`: normal Signal/Trigger creation itself needs graph and owner arenas, not ext<T>. However free setup functions require a current registered runtime/setup frame, and Owner methods call registry::runtime(app).
- `reactive/registry.rs:48–57` currently publishes an Rc<Runtime> with immutable Extensions already inside; `reactive/ext.rs:17–23` returns references for the runtime lifetime. A strong registry Rc prevents later Rc::get_mut to finish ext safely. Rc::new_cyclic does not solve this: Weak::upgrade is unavailable while the constructor callback runs.

## Option 1: prebuild Extensions, then complete Started after Runtime.new

Suggested simplified signature: Runtime.new(host, waker, config, extensions). It removes the immediate callback/immutable-field paradox and matches current empty subsystem constructors. Runtime can allocate its root internally before registration; no root-owner Option is necessary.

But subsequent `Started` creation cannot complete an immutable extension ViewState using those runtime handles without either:

1. retaining/refactoring `ViewState::start`, OnceCell, RefCell<Option<...>> or an equivalent hidden late-filled field (forbidden S7; C1 will remove it);
2. storing complete ViewState solely in core Launch::Started and changing every ViewState::of(rt) lookup away from runtime Extensions (a larger architectural change, not merely an internal Runtime.new signature change);
3. making Extensions dynamically mutable and changing borrowing/lifetime of ext<T> (R7's use_app_state insertion does not by itself authorize a missing core dependency to be installed after startup).

Thus option 1 is acceptable only if root deliberately changes the final ViewState ownership design, or accepts an explicit temporary migration state that is removed in the same R2/C1 construction transaction. It is not the durable simple solution under the current no-partial-object rule. Do not present current constructor independence as proof that finished ViewState is independent of Runtime.

## Option 2: narrow RuntimeSetup / construction parts

Conceptual ownership sequence (not an implemented API):

1. Reserve the AppId and allocate root/graph/owner arenas in a non-running construction value. An unused/reserved registry identity may be represented separately, but do not expose a partially functioning Runtime to public lookup.
2. Use explicit construction operations to allocate root-owned source/trigger handles directly against those arenas. They do not call Owner::signal, free signal, runtime lookup, effects, tasks or user callbacks. AppId and handles remain stable after arenas are moved into Runtime.
3. Core assembles complete ViewState/Tree/EnvState and other Extensions from those values plus ready host info/Writer/waker.
4. Consume construction parts plus finished Extensions into one complete Rc<Runtime>, register it, then run ordinary root setup/mount while the Started transition is still local and before user-visible Launch state is published.

This separates construction allocation from execution, keeps one registry/runtime identity, avoids OnceCell/ext placeholder/interior-reference unsafety, and lets C1 reuse the completed ViewState constructor without undoing R2. Prefer a small constructor-only surface over imitating the whole Runtime API or allowing arbitrary setup closures.

## Nontrivial issue that must be addressed, not hidden

FocusKeys currently includes Selector, not just inert signal handles. `reactive/selector.rs:47–100` creates an Internal effect. Its closure immediately uses `try_runtime(app)` and src.get; create_effect executes Internal effects during creation (`reactive/graph.rs` effect path). A construction allocator cannot blindly call this existing API before registration. A general typed builder promising arbitrary current reactive constructors would therefore be misleading.

Smallest root choices for this specific dependency:

- Make the constructor's focus selector creation use an internal preallocated initialized selector state, with its source-following effect activated only after completed runtime publication but before any user code; this needs explicit first-run equivalence, not an arbitrary uninitialized extension field.
- Or avoid storing the derived selector in ViewState during construction: keep the authoritative focus source there and create/store the derived selector under the root owner in a completed registration/setup phase, with its ownership/access contract deliberately updated. This changes where the cache lives; do not reintroduce a late-filled ViewState dependency disguised as a cache.

Which is smaller should be checked after R1's actual private module layout. No new general typestate framework is justified for one selector.

The allocator can handle eager Tree notification triggers without this complication: owner.trigger is only a source with unit payload. EnvState fields, adapter signal, focus source and visible signal likewise need no effect evaluation. Theme/messages and root view mounting can remain ordinary setup after the complete runtime exists; they are owner contexts, not unfinished Extensions.

## Other construction contracts to decide alongside this

- Initial host time must be available before user root setup invokes sleep/animation. New constructor RuntimeConfig lacks now, while design allows only TurnInput.now to transport clock. Decide a first setup turn or explicit construction-clock input without allowing a second clock owner.
- Preserve diagnostic root source location: current create_root(self.at) uses App::run/test::render location, not the runtime constructor's file.
- Construction failure/unwind before publication must not leak a registered half-App or invoke user cleanup of a partially installed extension set. Inert construction handles must never accidentally resolve to another AppId; identities remain never reused.
- Extension drop order is currently installation order (`reactive/ext.rs` module invariant); finishing construction must not accidentally reverse subsystem disposal order.

## Root decision requested, not user clarification

Decide whether immutable ext()->&T plus completed ViewState-in-Extensions remains fixed. If yes, authorize a minimal constructor allocation type and a precise focus-selector startup rule. If no, explicitly choose and document an alternative ViewState ownership/access path before selecting the simple prebuilt-Extensions Runtime.new signature. No code should emulate either option with hidden OnceCell/Option storage.
