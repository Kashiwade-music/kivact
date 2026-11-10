//! Role: Passive runtime state and handle identities.
//!
//! The handles (`Signal`, `Value`, `Var`, `Callback`, `Binding`, `Selector`,
//! ...) are `(AppId, Key{slot, generation})` and `Copy + !Send + !Sync`
//! through the [`UiThreadOnly`] marker: moving one off the UI thread fails
//! to compile. The App, the generation and the owner's lifetime are checked
//! by the operations that actually reach the state (reading and writing a
//! value, calling a `Callback`): a handle of a dropped App, or of another
//! App than the executing one, is `WrongApp`; a freed one is `Disposed`.
//! Converting an existing handle with `IntoValue` (identity) creates no node
//! and checks nothing; the checks happen when the value is read or written.
//!
//! Invariants:
//! - User code never runs while an arena table is borrowed: a value is
//!   reached by cloning its `Rc` before the closure is called, and a
//!   disposed value is dropped after the borrow is released.

use crate::reactive::{AppId, NodeId};
use std::any::{Any, TypeId};
use std::cell::{Cell, RefCell};
use std::collections::{BTreeSet, HashSet, VecDeque};
use std::fmt;
use std::marker::PhantomData;
use std::panic::Location;
use std::rc::Rc;

use crate::reactive::arena::{Arena, Key};
use crate::reactive::detail::DetailLog;
use crate::reactive::error::UiError;
use crate::reactive::links::Links;

/// Upper bound of effect runs per stabilisation and of user-effect rounds per turn.
pub(crate) const CYCLE_LIMIT: usize = 100;

/// A running turn and its trace record.
#[must_use]
pub struct TurnGuard<'a> {
    pub(crate) rt: &'a Runtime,
    pub(crate) started: std::time::Instant,
}

/// Live-object counts (lifecycle leak checks).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LiveCounts {
    /// Live graph nodes (sources, memos, effects).
    pub reactive: usize,
    /// Live owners.
    pub owners: usize,
    /// Live owner-bound objects.
    pub objects: usize,
    /// Live constant values ([`Value`]s of a constant, no graph node).
    pub constants: usize,
    /// Allocated arena slots of the four arenas (live + free + retired).
    pub slots: usize,
}

/// The reactive runtime of one App: arenas, frame stack, scheduler, trace
/// ring, its driver and the extensions of the higher crates.
pub struct Runtime {
    pub(crate) id: AppId,
    pub(crate) graph: RefCell<Arena<RNode>>,
    pub(crate) owners: RefCell<Arena<OwnerEntry>>,
    pub(crate) objects: RefCell<Arena<Object>>,
    /// Constant values (`Value::Const`): no node, no notification.
    pub(crate) consts: RefCell<Arena<ConstEntry>>,
    pub(crate) frames: RefCell<Vec<Frame>>,
    pub(crate) collectors: RefCell<Vec<Collector>>,
    pub(crate) sched: RefCell<Sched>,
    pub(crate) scratch: RefCell<Vec<(Key, Staleness)>>,
    // xtask-allow(state-fields): statistic: the effect creation-order ID counter; any value is valid on its own
    pub(crate) seq: Cell<u64>,
    // xtask-allow(state-fields): independent: the write counter labels every notification; it is valid at any value beside the extension set
    pub(crate) write_epoch: Cell<kivact_types::cell::Generation>,
    pub(crate) root: Key,
    // xtask-allow(state-fields): independent: the extensions installed when the App is created; the write counter is unrelated
    #[expect(
        clippy::disallowed_types,
        reason = "Construction fills this cell before returning or polling; its empty state is inaccessible."
    )]
    pub(crate) ext: std::cell::OnceCell<Extensions>,
    /// Last stamp remains meaningful after every acknowledged callback was removed.
    pub(crate) mount_revision: Cell<kivact_types::Revision>,
    pub(crate) unstamped: RefCell<Vec<Mount>>,
    pub(crate) awaiting: RefCell<VecDeque<(kivact_types::Revision, Mount)>>,
    pub(crate) host: Rc<dyn crate::reactive::host_types::Host>,
    pub(crate) state: Cell<crate::reactive::host_types::TurnState>,
    pub(crate) exit: Cell<crate::reactive::host_types::ExitState>,
    // xtask-allow(state-fields): statistic: the runtime's clock reading in nanoseconds; any value is valid on its own
    pub(crate) clock: Cell<u64>,
    pub(crate) diag: RefCell<Diag>,
    /// Errors raised where no user code may run (inside the engine's
    /// layout), delivered at the start of the next turn
    /// ([`Runtime::report_later`]).
    pub(crate) later: RefCell<Vec<(Option<Key>, UiError)>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Staleness {
    Check,
    Dirty,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CompState {
    Clean,
    Stale(Staleness),
    Queued(Staleness),
    Running,
}
impl CompState {
    pub(crate) fn staleness(self) -> Option<Staleness> {
        match self {
            Self::Stale(s) | Self::Queued(s) => Some(s),
            _ => None,
        }
    }
    pub(crate) fn mark(&mut self, stale: Staleness) -> bool {
        match *self {
            Self::Running => false,
            Self::Clean => {
                *self = Self::Stale(stale);
                true
            }
            Self::Stale(old) if old < stale => {
                *self = Self::Stale(stale);
                true
            }
            Self::Queued(old) if old < stale => {
                *self = Self::Queued(stale);
                true
            }
            _ => false,
        }
    }
}
/// When an effect runs and whether it may write.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectKind {
    /// Node prop effects: pure, run during stabilisation.
    Render,
    /// Flow/list/resource drivers: may mount and write, run during stabilisation.
    Internal,
    /// `effect`: run after the batch is flushed.
    User,
}

/// What a graph node is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A writable source (signal, var storage).
    Source,
    /// A cached derived value.
    Memo,
    /// An effect of the given kind.
    Effect(EffectKind),
}

/// Erased computation of a memo or effect. Returns whether the value changed.
pub(crate) trait Compute {
    fn run(&self) -> bool;
    /// The value is borrowed (a `with` closure is running), so it cannot be
    /// recomputed now (memos).
    fn busy(&self) -> bool {
        false
    }
}

/// Storage of a source node.
pub(crate) struct SourceCell<T> {
    /// The current value.
    pub(crate) value: RefCell<T>,
}

pub(crate) struct MemoCell<T> {
    // xtask-allow(state-fields): cache: the memo's last computed value; the graph state is CompState
    pub(crate) value: RefCell<Option<T>>,
    pub(crate) f: Box<dyn Fn(Option<&T>) -> T>,
    // xtask-allow(state-fields): cache: part of the memo's cache entry; the value and its equality function are used only together
    pub(crate) eq: Option<fn(&T, &T) -> bool>,
}

pub(crate) struct EffectCell {
    pub(crate) f: Box<dyn Fn()>,
}

pub(crate) struct RNode {
    pub(crate) owner: Key,
    pub(crate) owner_index: u32,
    pub(crate) cell: Rc<dyn Any>,
    pub(crate) at: &'static Location<'static>,
    /// Value type name (diagnostics; values themselves are never recorded).
    pub(crate) ty: &'static str,
    pub(crate) data: NodeData,
}

/// What a graph node is, with the data only that kind has.
pub(crate) enum NodeData {
    /// A writable source (signal, var storage, trigger, selector key).
    Source(SourceNode),
    /// A cached derived value.
    Memo(MemoNode),
    /// An effect.
    Effect(EffectNode),
    Watcher(EffectNode),
}

pub(crate) struct SourceNode {
    /// (subscriber, index of this node in subscriber.deps).
    pub(crate) subs: Links,
    // xtask-allow(state-fields): independent: the write version the graph comparisons read; the diagnostic record below may be absent
    /// Write generation of the last change.
    pub(crate) changed_at: kivact_types::cell::Generation,
    // xtask-allow(state-fields): independent: the diagnostic record of the last value-changing write; absent until one happens
    /// Location and turn of the last value-changing write (why chains).
    pub(crate) written: Option<(&'static Location<'static>, Turn)>,
}

pub(crate) struct MemoNode {
    /// (subscriber, index of this node in subscriber.deps).
    pub(crate) subs: Links,
    pub(crate) run: Computation,
}

pub(crate) struct EffectNode {
    pub(crate) kind: EffectKind,
    /// Creation order: queued effects run in it.
    pub(crate) seq: u64,
    pub(crate) run: Computation,
    pub(crate) cleanups: Vec<Box<dyn FnOnce()>>,
}

/// The part of a memo or an effect that computes.
pub(crate) struct Computation {
    pub(crate) compute: Rc<dyn Compute>,
    /// (source, index of this node in source.subs), in read order.
    pub(crate) deps: Links,
    pub(crate) state: CompState,
}

impl Computation {
    /// A computation that has not run yet (dirty).
    pub(crate) fn new(compute: Rc<dyn Compute>) -> Self {
        Computation {
            compute,
            deps: Links::default(),
            state: CompState::Stale(Staleness::Dirty),
        }
    }
}

impl RNode {
    /// The public kind (diagnostics).
    pub(crate) fn kind(&self) -> Kind {
        match &self.data {
            NodeData::Source(_) => Kind::Source,
            NodeData::Memo(_) => Kind::Memo,
            NodeData::Effect(e) | NodeData::Watcher(e) => Kind::Effect(e.kind),
        }
    }

    /// The readers of a source or a memo; nothing reads an effect.
    pub(crate) fn subs(&self) -> &[(Key, u32)] {
        match &self.data {
            NodeData::Source(s) => &s.subs,
            NodeData::Memo(m) => &m.subs,
            NodeData::Effect(_) | NodeData::Watcher(_) => &[],
        }
    }

    pub(crate) fn subs_mut(&mut self) -> Option<&mut Links> {
        match &mut self.data {
            NodeData::Source(s) => Some(&mut s.subs),
            NodeData::Memo(m) => Some(&mut m.subs),
            NodeData::Effect(_) | NodeData::Watcher(_) => None,
        }
    }

    /// The computation of a memo or an effect.
    pub(crate) fn computation(&self) -> Option<&Computation> {
        match &self.data {
            NodeData::Source(_) => None,
            NodeData::Memo(m) => Some(&m.run),
            NodeData::Effect(e) | NodeData::Watcher(e) => Some(&e.run),
        }
    }

    pub(crate) fn computation_mut(&mut self) -> Option<&mut Computation> {
        match &mut self.data {
            NodeData::Source(_) => None,
            NodeData::Memo(m) => Some(&mut m.run),
            NodeData::Effect(e) | NodeData::Watcher(e) => Some(&mut e.run),
        }
    }

    /// What a memo or an effect read during its last run (empty for a source).
    pub(crate) fn deps(&self) -> &[(Key, u32)] {
        match self.computation() {
            Some(c) => &c.deps,
            None => &[],
        }
    }
}

/// Read-only snapshot of one graph node, for diagnostics (`why` chains).
/// Values are never exposed: only kind, type name and locations.
pub struct NodeInfo {
    /// Source, memo or effect.
    pub kind: Kind,
    /// Value type name.
    pub ty: &'static str,
    /// Where the node was created.
    // xtask-allow(api-shape): Diagnostic source positions retain Rust caller locations.
    pub at: &'static Location<'static>,
    /// Location and turn of the last value-changing write (sources).
    // xtask-allow(api-shape): Diagnostic source positions retain Rust caller locations.
    pub written: Option<(&'static Location<'static>, Turn)>,
    /// What the node read during its last run, in read order.
    pub deps: Vec<NodeId>,
}

/// What only some owners have: objects, constants, cleanups, contexts and
/// an error handler (`OwnerEntry::extra`, created by the first of them).
#[derive(Default)]
pub(crate) struct OwnerExtra {
    pub(crate) objects: Vec<Key>,
    pub(crate) consts: Vec<Key>,
    pub(crate) cleanups: Vec<Box<dyn FnOnce()>>,
    pub(crate) contexts: Vec<(TypeId, Rc<dyn Any>)>,
    pub(crate) on_error: Option<Rc<dyn Fn(UiError)>>,
}

pub(crate) struct OwnerEntry {
    // xtask-allow(state-fields): option: absent means the root owner, which has no parent
    pub(crate) parent: Option<Key>,
    pub(crate) parent_index: u32,
    pub(crate) children: Vec<Key>,
    pub(crate) nodes: Vec<Key>,
    pub(crate) extra: Option<Box<OwnerExtra>>,
    /// Where the boundary that created this owner was called (root, flow
    /// branch, list row, `provide`, `error_boundary`): how diagnostics name
    /// the owner.
    pub(crate) created_at: &'static Location<'static>,
    /// Disposal has started: registrations are released by the disposal
    /// (`owner.rs`).
    pub(crate) phase: OwnerPhase,
}

impl OwnerEntry {
    /// The rare members, created on first use.
    pub(crate) fn extra_mut(&mut self) -> &mut OwnerExtra {
        self.extra.get_or_insert_default()
    }
}

const _: () = assert!(
    std::mem::size_of::<OwnerEntry>() <= 88,
    "OwnerEntry grew: put the new member in OwnerExtra"
);

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum OwnerPhase {
    Live,
    Disposing { round: u8 },
}

/// A constant value of an owner (`Value::Const`): released with its owner.
pub(crate) struct ConstEntry {
    pub(crate) created_at: &'static Location<'static>,
    /// The value, an `Rc<T>` behind `dyn Any`.
    pub(crate) data: Rc<dyn Any>,
}

/// An owner-bound object (Var, Callback, NodeRef, Task, Action, ...).
pub(crate) struct Object {
    pub(crate) owner: Key,
    pub(crate) owner_index: u32,
    pub(crate) data: Rc<dyn Any>,
    /// The name the owner stored it under (diagnostics).
    pub(crate) name: &'static str,
    pub(crate) on_dispose: Option<Box<dyn FnOnce(&Runtime)>>,
    pub(crate) created_at: &'static Location<'static>,
}

/// One execution context. Copies never own dependency storage.
#[derive(Clone, Copy, Default)]
pub(crate) struct Frame {
    // xtask-allow(state-fields): independent: ADR-20: the frame kind and pure/tracking/owner/writer are separate facts
    pub(crate) owner: Option<Key>,
    pub(crate) kind: FrameKind,
    // xtask-allow(state-fields): independent: ADR-20: the frame kind and pure/tracking/owner/writer are separate facts
    pub(crate) pure: bool,
    // xtask-allow(state-fields): independent: ADR-20: the frame kind and pure/tracking/owner/writer are separate facts
    pub(crate) writer: Option<Key>,
    // xtask-allow(state-fields): independent: ADR-20: the frame kind and pure/tracking/owner/writer are separate facts
    pub(crate) tracking: Option<CollectorSlot>,
}

#[derive(Clone, Copy)]
pub(crate) struct CollectorSlot(pub(super) usize);
#[derive(Clone, Copy, Default)]
pub(crate) enum FrameKind {
    #[default]
    Handler,
    Setup(Key),
    Computation {
        node: Key,
        effect: bool,
    },
}

/// Dependencies of one active computation, retained while it is suspended.
pub(crate) struct Collector {
    pub(crate) node: Key,
    pub(crate) deps: Vec<Key>,
    pub(crate) set: Option<HashSet<Key>>,
}

/// Pops its context and, only for a new computation, its owned collector.
pub(crate) struct FrameGuard<'a> {
    pub(crate) rt: &'a Runtime,
    pub(crate) depth: usize,
    pub(crate) outer_app: AppId,
    pub(crate) collector: Option<CollectorSlot>,
}

/// Queued effects by creation order: being in a set is being queued.
#[derive(Default)]
pub(crate) struct Sched {
    /// Render and Internal effects (run by `stabilize`).
    pub(crate) render: BTreeSet<(u64, Key)>,
    /// User effects (run by `run_user_effects`).
    pub(crate) user: BTreeSet<(u64, Key)>,
}

/// A value-changing write during a turn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TraceWrite {
    /// Type name of the written value.
    pub ty: &'static str,
    /// Where the signal was created.
    // xtask-allow(api-shape): Diagnostic source positions retain Rust caller locations.
    pub created_at: &'static Location<'static>,
    /// Where it was written.
    // xtask-allow(api-shape): Diagnostic source positions retain Rust caller locations.
    pub at: &'static Location<'static>,
}

/// A memo or effect re-evaluated during a turn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TraceRun {
    /// `"memo"`, `"render"`, `"internal"` or `"effect"`.
    pub kind: &'static str,
    /// Where the memo or effect was created.
    // xtask-allow(api-shape): Diagnostic source positions retain Rust caller locations.
    pub at: &'static Location<'static>,
}

/// Which turn something happened in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Turn {
    /// While the App was set up (its first mount), before the first turn.
    #[default]
    Setup,
    /// A turn, numbered from 1.
    Number(u64),
}

/// One turn of the structured trace.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TurnRecord {
    /// Which turn: `Turn::Setup` for the notes recorded while the App was set
    /// up, else the turn's number.
    pub turn: Turn,
    /// Host events delivered in this turn.
    pub events: usize,
    /// What started the work: host events (`"Activate n5"`), `"timer"`, `"task"`.
    pub causes: Vec<String>,
    /// Value-changing writes (first 64).
    pub writes: Vec<TraceWrite>,
    /// Re-evaluated memos and effects (first 64; `runs` counts all).
    pub reruns: Vec<TraceRun>,
    /// Number of memo and effect runs in this turn.
    pub runs: usize,
    /// Writes committed to the engine at the end of this turn.
    pub ops: usize,
    /// Time spent in the turn, in nanoseconds.
    pub duration_ns: u64,
    /// Notes recorded during the turn (stale events, duplicate styles, ...).
    pub notes: Vec<String>,
}

/// Ring buffers of errors, diagnostics and turn records, plus the turn
/// being recorded and the detailed trace the DevTools reads.
#[derive(Default)]
pub(crate) struct Diag {
    /// Errors that reached the default sink, with their report sequence.
    pub(crate) errors: VecDeque<(u64, UiError)>,
    /// Non-fatal diagnostics, with their report sequence.
    pub(crate) diagnostics: VecDeque<(u64, UiError)>,
    pub(crate) turns: VecDeque<TurnRecord>,
    /// The record being filled: the setup's, then always the next turn's
    /// (what happens between two turns belongs to the next one).
    pub(crate) current: TurnRecord,
    pub(crate) total_errors: u64,
    /// The detail of the turn being recorded: off unless the DevTools asked.
    pub(crate) detail: DetailLog,
    /// The sequence number of the last error or diagnostic put in a ring.
    pub(crate) next_seq: u64,
}

/// The per-App extension values, at most one per type.
#[derive(Default)]
pub struct Extensions(pub(crate) Vec<(TypeId, Box<dyn Any>)>);

/// An `on_mount` callback waiting for the driver's acknowledgement.
pub(crate) struct Mount {
    /// The owner it runs for (skipped once that owner is gone).
    pub(crate) owner: Key,
    pub(crate) run: Box<dyn FnOnce()>,
}

/// The marker that keeps a handle on the UI thread: it is neither `Send` nor
/// `Sync`, so a closure that captures a `Signal` (or any other handle) cannot
/// move to another thread. Read the value outside and pass plain data in.
#[derive(Clone, Copy, Debug)]
pub struct UiThreadOnly(PhantomData<*const ()>);

/// A writable reactive value. Writes change the value immediately and notify
/// readers at the end of the turn.
pub struct Signal<T: 'static> {
    pub(crate) app: AppId,
    pub(crate) key: Key,
    pub(crate) _m: PhantomData<(fn() -> T, UiThreadOnly)>,
}

/// Where a [`Value`] lives: a graph node (signal, memo) or a constant of the
/// App's constant arena.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ValueRepr {
    /// A constant: no graph node, never notifies.
    Const(Key),
    /// A graph node.
    Node(Key),
}

/// The one read-only reactive type: memos, signals, constants, props and
/// animation results are all `Value<T>`.
///
/// Copying the handle requires neither `T: Copy` nor `T: Clone` and does not
/// extend its owner lifetime. Constants occupy the owner's constant arena,
/// never a graph node; reading them creates no dependency.
pub struct Value<T: 'static> {
    pub(crate) app: AppId,
    pub(crate) repr: ValueRepr,
    pub(crate) _m: PhantomData<(fn() -> T, UiThreadOnly)>,
}

/// Untracked local variable (`useRef`). Never notifies.
///
/// A `Var` shares a value that is not `Copy` (a path, a configuration
/// struct) between several `Fn` closures: a closure that moves the `Var`
/// moves a `Copy` handle, and each one reads the value with `get`/`with`.
/// It holds state that is not shown; a value that changes the screen is a
/// `Signal`.
pub struct Var<T: 'static> {
    pub(crate) app: AppId,
    pub(crate) key: Key,
    pub(crate) _m: PhantomData<(fn() -> T, UiThreadOnly)>,
}

pub(crate) struct CallbackFn<E> {
    pub(crate) owner: Key,
    pub(crate) f: Box<dyn Fn(E) -> Result<(), UiError>>,
}

/// A Copy callback (child → parent), made by [`callback`](crate::reactive::callback).
/// `call` routes failures to the error sink; `try_call` returns them.
/// Equality identifies the same callback registration (App and registration key),
/// so a copied callback stays equal even after its owner is disposed.
///
/// A `Callback` passes the same closure to several places: a closure is
/// moved into the first control that takes it, but a `Callback` is `Copy`,
/// so one `callback(..)` can be given to many controls (and to child
/// components that report to their parent). A control that takes a closure
/// takes a plain closure; make a `Callback` only to share one.
pub struct Callback<E: 'static> {
    pub(crate) app: AppId,
    pub(crate) key: Key,
    pub(crate) _m: PhantomData<(fn(E), UiThreadOnly)>,
}

pub(crate) enum BindingRepr<T: 'static> {
    Signal(Signal<T>),
    Pair(Value<T>, Callback<T>),
}

/// A two-way input binding: a value and a setter.
pub struct Binding<T: 'static> {
    pub(crate) repr: BindingRepr<T>,
}

impl Extensions {
    /// No extensions.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add `value`. Panics (invariant) if a value of the same type is
    /// already present: each type is installed by exactly one subsystem.
    pub fn with<T: 'static>(mut self, value: T) -> Self {
        let id = TypeId::of::<T>();
        assert!(
            self.0.iter().all(|(k, _)| *k != id),
            "invariant: extension {} installed twice",
            std::any::type_name::<T>()
        );
        self.0.push((id, Box::new(value)));
        self
    }

    /// The value of type `T`, if installed.
    #[inline]
    pub(crate) fn get<T: 'static>(&self) -> Option<&T> {
        let id = TypeId::of::<T>();
        self.0
            .iter()
            .find(|(k, _)| *k == id)
            .and_then(|(_, v)| v.downcast_ref::<T>())
    }
}

impl<T: 'static> Clone for BindingRepr<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Copy for BindingRepr<T> {}

impl<T: 'static> fmt::Debug for Signal<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Signal({}:{})", self.key.slot, self.key.generation)
    }
}

impl<T: 'static> fmt::Debug for Value<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.repr {
            ValueRepr::Node(k) => write!(f, "Value({}:{})", k.slot, k.generation),
            ValueRepr::Const(k) => write!(f, "Value(const {}:{})", k.slot, k.generation),
        }
    }
}

macro_rules! copy_handle {
    ($name:ident < $($g:ident),* >) => {
        impl<$($g: 'static),*> Clone for $name<$($g),*> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<$($g: 'static),*> Copy for $name<$($g),*> {}
    };
}

copy_handle!(Signal<T>);

copy_handle!(Value<T>);

/// The same value: the same node or the same constant.
impl<T: 'static> PartialEq for Value<T> {
    fn eq(&self, other: &Self) -> bool {
        self.app == other.app && self.repr == other.repr
    }
}

copy_handle!(Var<T>);

copy_handle!(Callback<E>);

/// Equality identifies the same callback registration, independently of its payload type.
impl<E: 'static> PartialEq for Callback<E> {
    fn eq(&self, other: &Self) -> bool {
        self.app == other.app && self.key == other.key
    }
}
impl<E: 'static> Eq for Callback<E> {}

copy_handle!(Binding<T>);
