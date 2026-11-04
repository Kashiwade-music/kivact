# R-2 Runtime placement and DAG boundary (read-only)

Prepared 2026-11-04 against the current pre-R1 tree. No source/interface/git/build/test modification. Companion artifacts: r2-packet.md and r2-construction.md. Root decision only; implementation waits for R1 merge/CP1 PASS.

## Constraint and source evidence

`REFACTORING_DESIGN.md:320–327` specifies reactive=[]; handler->[reactive]; turn->[reactive,handler]; time->[reactive,turn]; task->[reactive,handler,turn,time]; host may use all of them. Host contains orchestration bodies for Runtime::new and Runtime::turn, but that does not require the nominal Runtime definition to live there.

Current Runtime is a single struct in `crates/kivact-reactive/src/data.rs:47–82`. Twenty-two current reactive source files mention Runtime. Graph, owner, frames, handles, selectors, setup and diagnostics add inherent methods to this type and access its arenas/cells directly. Representative uses:

- graph.rs:73–180 inserts graph nodes and immediately runs initial Internal/Render effects;
- owner.rs:64–99 allocates owners;
- frame.rs guards hold a reference to Runtime and pop its frame stack;
- data.rs Object disposal callbacks accept &Runtime, Frame guard stores &Runtime;
- registry.rs:48–57 stores Rc<Runtime>, and Runtime::current/get return that same registered type;
- owner_handle.rs:102,133–155 obtains the registered Runtime, then creates sources/triggers/frames;
- setup.rs:45–49 uses current runtime and require_setup;
- handles.rs callbacks use Runtime frames;
- driver.rs:35 and graph writes call request_turn; clock_now delegates to Runtime::now_ns;
- ext.rs:17 returns &T from Runtime-owned immutable Extensions.

Core/task/motion also use the same Runtime for graph context, ext lookup, root owner, clock and work scheduling. Core diagnostics use Runtime::current/get (`core/diagnostics/errors.rs`, why.rs), so introducing a second outward runtime identity would affect more than only AppClient.

## Recommended direction: one Runtime defined in lower reactive

Keep one nominal Runtime and the existing graph/owner/frame/registry identity. Define only the pure contracts needed by its fields and reactive write checks at that lower layer. Implement upper orchestration in host via inherent `impl Runtime`; Rust permits this within one crate. Such upper impls do not require reactive source to import host.

The minimal contracts are:

1. Host's wake-only trait. Runtime needs to store and invoke it from request_turn, so the trait definition must be below host. Define exactly one Host trait in a small reactive contract leaf; host can directly reexport that definition at the intended user-facing runtime module path. This is dependency inversion, not a compatibility wrapper or duplicated trait.
2. TurnState/Stage if Runtime stores the state and reactive write paths check Stage. These cannot be defined solely in turn and used by Runtime fields while reactive=[] holds. Put the execution-state tags at the same lower invariant layer; turn may reexport/use the exact type. TurnClient/TurnInput/TurnItem/TurnReport can remain defined in turn and Runtime::turn's body remains in host.
3. ExitState is pure. It can remain turn-defined if upper host-state storage holds it as a normal construction-time extension and reactive never inspects it. If it is a direct Runtime field, it too needs a lower definition; do not add a reactive->turn dependency by accident.
4. ThreadWaker is a pure cross-thread wake handle. Either define it once in the lower contract leaf and reexport from host, or keep it host-defined while passing its underlying existing Arc<dyn Fn()+Send+Sync> into the task executor. The first gives consumers one consistent nominal handle; the second minimizes lower fields if Runtime never stores ThreadWaker. No late waker slot is needed in either case.
5. RuntimeConfig need not move lower merely because Runtime::new takes it. A constructor implemented in host can destructure host::RuntimeConfig and pass its individual values into lower construction parts. Move config only if the lower Runtime actually stores that nominal type, which is unnecessary for blocking_threads/reduced_motion configuration.

Do not place task Scheduler, executor, MotionState or Machine concrete fields inside lower Runtime if they create upper imports. Continue to construct upper subsystem values as complete owned Extensions and access them from host orchestration. Runtime's generic Extensions storage does not depend on their defining modules.

The original DAG remains intact. The documentation's statement that certain pure types are *defined* in host/turn needs a precise adjustment, while their canonical outward exposure can stay host/turn through direct reexports. There is only one implementation and one route to behavior; no alias of an old interface, no alternate mode and no Deref forwarding layer.

## Construction allocator fit

The typed construction allocator from r2-construction.md belongs beside the lower graph/owner data because it allocates those arenas and source/trigger handles. The host constructor consumes its finished state plus upper-created Extensions and publishes the single Runtime. A lower allocator must not take upper RuntimeConfig or refer to host orchestration.

Root-owned owner identity/source handles are allocated before final publication. The previously identified initial selector execution issue still requires a deliberate startup rule; moving Host trait alone does not solve the immutable-extension construction cycle. Keep this issue separate from module placement.

## Alternative: host::Runtime containing ReactiveState

A genuine composition can preserve reactive=[] only if all lower APIs, guard types, callbacks, handles and the registry use ReactiveState rather than host::Runtime. Upper Runtime would own ReactiveState plus orchestration state. That is a valid new architecture, but has these concrete costs:

- rename/refactor graph/owner/frame/setup/selector/handles references across the 22 lower files;
- decide whether registry current/get returns only ReactiveState or the outer Runtime; lower registry cannot name host::Runtime;
- move extension and clock/wake lookups consistently, since all lower handle callbacks currently receive the same Runtime that exposes ext<T> and now_ns;
- change core/task/motion call sites to choose the correct object and ensure Runtime::current/get does not leave two App identities;
- avoid adding a broad Deref or forwarding facade to make old calls compile, which would defeat the requested single-path simplification.

If ReactiveState still ends up owning registry identity, arenas, ext map, host wake and clock so existing callbacks work, the outer host Runtime is mostly a second name plus forwarding shell. That is more work and more conceptual surface than keeping one lower Runtime with host-provided inherent orchestration methods.

Recommend composition only if root explicitly chooses to change all those ownership contracts for a substantive reason, not solely to honor the prose location of Runtime::new.

## Root choices to make explicit

- Confirm one lower-defined Runtime, with host inherent orchestration methods and no lower imports of host/turn.
- Authorize moving the nominal wake-only Host and execution tags to the lower pure contract leaf, preserving intended public host/turn paths via direct reexport.
- Choose whether ThreadWaker is lower nominal contract or host handle converted once to executor's Arc; keep RuntimeConfig host-defined unless a real lower field needs it.
- Choose ExitState storage location; its deadline must remain single-owned regardless of struct placement.
- Apply the same direction to the construction allocator, and separately settle the focus Selector's initial execution and initial clock/startup-turn questions.

No DAG exception, reactive->host cycle, compatibility Deref, extra registered Runtime object or partial usable Runtime is recommended.
