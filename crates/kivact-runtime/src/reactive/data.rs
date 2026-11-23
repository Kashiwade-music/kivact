//! Role: Passive runtime state and handle identities.
//!
//! The handles (`Signal`, `Value`, `Var`, `Callback`, `Selector`, ...) are `(AppId, Key{slot, generation})` and `Copy + !Send + !Sync`
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
use crate::reactive::detail::{RecordBits, Turn, TurnRecord};
use crate::reactive::error::UiError;
use crate::reactive::host_types::Stage;
use crate::reactive::links::Links;

/// Upper bound of effect runs per stabilisation and of user-effect rounds per turn.
pub(crate) const CYCLE_LIMIT: usize = 100;

/// A running turn and its trace record.
#[must_use]
pub struct TurnGuard<'a> {
    pub(crate) rt: &'a Runtime,
    pub(crate) started: std::time::Instant,
    /// The deferred, timer and task rounds this turn ran.
    pub(crate) work_rounds: std::cell::Cell<u32>,
}

/// One value per type. It is filled when the App is created and never
/// changed afterwards.
#[derive(Default)]
pub(crate) struct Extensions(Vec<(TypeId, Box<dyn Any>)>);

impl Extensions {
    /// No extension.
    pub(crate) fn new() -> Self {
        Self(Vec::new())
    }

    /// With `value` installed, replacing an earlier value of its type.
    pub(crate) fn with<T: 'static>(mut self, value: T) -> Self {
        let id = TypeId::of::<T>();
        self.0.retain(|(ty, _)| *ty != id);
        self.0.push((id, Box::new(value)));
        self
    }

    /// The extension of type `T`, if it was installed.
    pub(crate) fn get<T: 'static>(&self) -> Option<&T> {
        let id = TypeId::of::<T>();
        let (_, value) = self.0.iter().find(|(ty, _)| *ty == id)?;
        let value: &dyn Any = &**value;
        value.downcast_ref::<T>()
    }
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

// xtask-allow(state): record: graph, owners, objects and consts are the arenas; frames, collectors, sched, scratch, dep_pool and pass_touched are the stack, scheduler and scratch tables of stabilize; diag, later, graveyard, unsubscribed, unstamped and awaiting are separate queues and tables; deferred is the queue that Runtime::defer adds to and run_deferred takes from; seq and write_epoch only allocate creation and source-write identities; state (turn scheduling), exit (the exit drain) and clock (the latest host time) are independent facts; app is written once inside Runtime::new before it returns and only read afterwards
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
    /// Emptied dependency lists kept for the next collector.
    pub(crate) dep_pool: RefCell<Vec<Vec<Key>>>,
    /// The effects that ran in the current stabilize pass.
    pub(crate) pass_touched: RefCell<Vec<Key>>,
    pub(crate) seq: Cell<u64>,
    pub(crate) write_epoch: Cell<kivact_types::cell::Generation>,
    pub(crate) root: Key,
    pub(crate) unstamped: RefCell<Vec<Mount>>,
    pub(crate) awaiting: RefCell<VecDeque<(kivact_types::Revision, Mount)>>,
    pub(crate) host: Rc<dyn crate::reactive::host_types::Host>,
    pub(crate) state: Cell<crate::reactive::host_types::TurnState>,
    pub(crate) exit: Cell<crate::reactive::host_types::ExitState>,
    pub(crate) clock: Cell<u64>,
    pub(crate) diag: RefCell<Diag>,
    /// Errors raised where no user code may run (inside the engine's
    /// layout), delivered at the start of the next turn
    /// ([`Runtime::report_later`]).
    pub(crate) later: RefCell<Vec<(Option<Key>, UiError)>>,
    /// Handlers deferred until the running one returns (`Runtime::defer`),
    /// each with the owner it runs for and where it was deferred.
    pub(crate) deferred: RefCell<VecDeque<(Key, &'static Location<'static>, Box<dyn FnOnce()>)>>,
    /// The state of the layers above the graph (tasks, motion), installed
    /// when the App is created: see `extensions`.
    pub(crate) extensions: Extensions,
    /// What the host layer installs while `Runtime::new` runs: written once,
    /// before `Runtime::new` returns, and only read afterwards.
    pub(crate) app: kivact_types::cell::Deferred<AppSlot>,
    pub(crate) graveyard: RefCell<Graveyard>,
    /// What runs when a watched source loses its last subscriber (selector
    /// key nodes): run once, then forgotten.
    pub(crate) unsubscribed: RefCell<std::collections::HashMap<Key, Box<dyn FnOnce(&Runtime)>>>,
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
    /// The cell itself as `Any` (a memo's value is read through it).
    fn into_any(self: Rc<Self>) -> Rc<dyn Any>;
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

// xtask-allow(state): cache: value is the memo's last computed value, written only by the memo's run (None until the first run); eq is the comparison fixed by create_memo
pub(crate) struct MemoCell<T> {
    pub(crate) value: RefCell<Option<T>>,
    pub(crate) f: Box<dyn Fn(Option<&T>) -> T>,
    pub(crate) eq: Option<fn(&T, &T) -> bool>,
}

pub(crate) struct EffectCell {
    pub(crate) f: Box<dyn Fn()>,
}

pub(crate) struct RNode {
    pub(crate) owner: Key,
    pub(crate) owner_index: u32,
    /// Where a memo or an effect stands; a source is always `Clean`.
    pub(crate) state: CompState,
    pub(crate) data: NodeData,
}

const _: () = assert!(
    std::mem::size_of::<RNode>() <= 96,
    "RNode grew: diagnostics belong in Diag::node_meta, rare members in a box"
);

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
    /// The value (a `SourceCell<T>`).
    pub(crate) cell: Rc<dyn Any>,
    /// (subscriber, index of this node in subscriber.deps).
    pub(crate) subs: Links,
    /// Write generation of the last change.
    pub(crate) changed_at: kivact_types::cell::Generation,
}

pub(crate) struct MemoNode {
    /// The memo (a `MemoCell<T>`; its value is read through `into_any`).
    pub(crate) comp: Rc<dyn Compute>,
    /// (subscriber, index of this node in subscriber.deps).
    pub(crate) subs: Links,
    /// (source, index of this node in source.subs), in read order.
    pub(crate) deps: Links,
}

pub(crate) struct EffectNode {
    pub(crate) comp: Rc<dyn Compute>,
    /// (source, index of this node in source.subs), in read order.
    pub(crate) deps: Links,
    pub(crate) kind: EffectKind,
    /// Runs in the current stabilize pass (R03).
    pub(crate) pass_runs: u8,
    /// Creation order: queued effects run in it.
    pub(crate) seq: u64,
    /// Cleanups registered by the running effect (most effects have none).
    pub(crate) cleanups: Option<Box<Vec<Box<dyn FnOnce()>>>>,
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

    /// Whether this node computes (a memo or an effect).
    pub(crate) fn is_computation(&self) -> bool {
        !matches!(self.data, NodeData::Source(_))
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
    pub(crate) fn comp(&self) -> Option<&Rc<dyn Compute>> {
        match &self.data {
            NodeData::Source(_) => None,
            NodeData::Memo(m) => Some(&m.comp),
            NodeData::Effect(e) | NodeData::Watcher(e) => Some(&e.comp),
        }
    }

    /// What a memo or an effect read during its last run (empty for a source).
    pub(crate) fn deps(&self) -> &[(Key, u32)] {
        match &self.data {
            NodeData::Source(_) => &[],
            NodeData::Memo(m) => &m.deps,
            NodeData::Effect(e) | NodeData::Watcher(e) => &e.deps,
        }
    }

    pub(crate) fn deps_mut(&mut self) -> Option<&mut Links> {
        match &mut self.data {
            NodeData::Source(_) => None,
            NodeData::Memo(m) => Some(&mut m.deps),
            NodeData::Effect(e) | NodeData::Watcher(e) => Some(&mut e.deps),
        }
    }

    /// The cell a handle reads: a source's value or a memo's cell.
    pub(crate) fn cell(&self) -> Rc<dyn Any> {
        match &self.data {
            NodeData::Source(s) => Rc::clone(&s.cell),
            NodeData::Memo(m) => Rc::clone(&m.comp).into_any(),
            NodeData::Effect(e) | NodeData::Watcher(e) => Rc::clone(&e.comp).into_any(),
        }
    }
}

/// The type and creation site of one graph node (`Diag::node_meta`):
/// always kept in a build with the `devtools` feature (recording may
/// start later, and `why` still names every node), else only for nodes
/// created while a recorder is on. Never in the node itself.
#[derive(Clone, Copy)]
pub(crate) struct NodeMeta {
    pub(crate) at: &'static Location<'static>,
    pub(crate) ty: &'static str,
}

/// Read-only snapshot of one graph node, for diagnostics (`why` chains).
/// Values are never exposed: only kind, type name and locations.
pub struct NodeInfo {
    /// Source, memo or effect.
    pub kind: Kind,
    /// Value type name.
    pub ty: &'static str,
    /// Where the node was created.
    pub at: &'static Location<'static>,
    /// Location and turn of the last value-changing write (sources).
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

// xtask-allow(state): independent: parent is the creating owner, written only by create_owner (None only for the root) and kept through disposal; extra holds the rare members, allocated on first use by extra_mut and emptied in place by dispose_one
pub(crate) struct OwnerEntry {
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
    pub(crate) owner: Option<Key>,
    pub(crate) kind: FrameKind,
    pub(crate) pure: bool,
    pub(crate) writer: Option<Key>,
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

// xtask-allow(state): independent: cause is the index of the last cause recorded in the current turn, written by cause() and cleared at turn boundaries; open_stage is the stage being timed, written by stage() and taken by end_turn; next_seq is the report sequence, written by record_error and record_diagnostic; no field's value decides another's
/// Ring buffers of errors, diagnostics and turn records, plus the turn
/// being recorded.
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
    /// The recorders that are on (none: no turn trace is kept).
    pub(crate) record: RecordBits,
    /// The index of the last cause recorded in the current turn.
    pub(crate) cause: Option<u32>,
    /// The stage being timed and the turn-relative ns it started at.
    pub(crate) open_stage: Option<(Stage, u64)>,
    /// The sequence number of the last error or diagnostic put in a ring.
    pub(crate) next_seq: u64,
    /// The type and creation site of nodes (see `NodeMeta`).
    pub(crate) node_meta: std::collections::HashMap<Key, NodeMeta>,
    /// Location and turn of each source's last value-changing write while a
    /// recorder is on (`why` chains).
    pub(crate) node_written: std::collections::HashMap<Key, (&'static Location<'static>, Turn)>,
}

/// What an owner's disposal released, dropped after every borrow ends
/// (user `Drop` code may run). Kept on the runtime to reuse its capacity.
#[derive(Default)]
pub(crate) struct Graveyard {
    pub(crate) nodes: Vec<RNode>,
    pub(crate) data: Vec<Rc<dyn Any>>,
}

/// The host layer's part of the runtime (`AppParts`, stored).
pub(crate) struct AppSlot {
    pub(crate) state: Box<dyn Any>,
}

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

/// A writable reactive value: a signal (`signal`) or a two-way view of
/// another value (`binding`, `bind!`, `optional`). Writes change the
/// value immediately and notify readers at the end of the turn.
pub struct Signal<T: 'static> {
    pub(crate) app: AppId,
    pub(crate) repr: SignalRepr,
    pub(crate) _m: PhantomData<(fn() -> T, UiThreadOnly)>,
}

/// Where a [`Signal`] lives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SignalRepr {
    /// A source node of the graph.
    Source(Key),
    /// A lens: `read` is the value it shows, `write` the `LensCell` object that writes it.
    Lens { read: ValueRepr, write: Key },
}

/// How a lens writes: replace the whole value, or edit the current one
/// in place (both decide themselves whether readers are notified).
pub(crate) struct LensCell<T> {
    pub(crate) owner: Key,
    pub(crate) set: Box<dyn Fn(T) -> Result<(), UiError>>,
    pub(crate) edit: Box<dyn Fn(&mut dyn FnMut(&mut T)) -> Result<(), UiError>>,
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

impl<T: 'static> fmt::Debug for Signal<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.repr {
            SignalRepr::Source(k) => write!(f, "Signal({}:{})", k.slot, k.generation),
            SignalRepr::Lens { write, .. } => {
                write!(f, "Signal(lens {}:{})", write.slot, write.generation)
            }
        }
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
