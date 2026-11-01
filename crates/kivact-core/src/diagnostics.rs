//! Role: Diagnostics: error tracking and turn records.
//!
//! Diagnostics: errors routed to the default sink, a ring buffer of
//! structured turn records (causes, writes, re-evaluations, ops, duration),
//! mirrored to `tracing`, [`why`](fn@why) chains explaining a node's props, and the
//! [`Stats`] counters a test screen reports. Recording never changes
//! behaviour. Values are never recorded: only type names and source
//! locations. The ring lives in the reactive runtime, `why` with the views.
//!
//! A turn record holds the causes of the turn, the writes that changed a
//! value, the re-runs, the number of writes to the engine's tree and
//! notes. [`why`](fn@why) returns, for each dynamic prop of a node, the creation
//! site of the effect that set it and the memos and signals it read
//! (recursively). Every host node keeps where its builder was called, and
//! snapshots and diagnostics name nodes by it.

pub(crate) mod why;

#[doc(inline)]
pub use kivact_reactive::ErrorKind;
#[doc(inline)]
pub use kivact_reactive::{TraceRun, TraceWrite, TurnRecord};
#[doc(inline)]
pub use why::{Why, WhyDep, WhyProp, why};

/// Counters for granularity checks (`kivact::test`'s `Screen::stats`).
///
/// They are an internal performance diagnostic for changes to Kivact
/// itself; tests of an App check state, display and role/name instead.
/// The size of a counter proves neither that the screen is right nor OS
/// input, presentation or a performance target. Read them along the update
/// path: a signal write, the graph's dirty propagation, memo re-evaluation,
/// the render effect of each dynamic prop, the engine's typed setter that
/// drops equal writes, the commit's `Changes`/sync derivations, layout,
/// paint and accessibility, then the frame.
///
/// Read the creation counts against the owner boundaries: a component's
/// setup usually runs once, while a `show` branch or a keyed row factory
/// runs on every mount. A counter whose click changes a text but no host
/// node keeps `creates` equal and raises `set_props`. An equal write, or a
/// memo recomputing an equal value, needs no prop update. Filtering,
/// unmounting and re-adding the rows of a keyed list differ from changing
/// a label in which owners and creations they involve. Frame counts,
/// function calls and `creates` are different measures, and hostbench's
/// update-to-frame and redraw work are separate metrics: compare under the
/// same conditions.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Stats {
    /// Revisions committed (each one batch of writes).
    pub batches: u64,
    /// Cumulative accessibility payload construction, excluding debug references.
    #[cfg(feature = "test")]
    pub a11y_constructed: u64,
    /// Cumulative accessibility payloads sent, including rejected attempts.
    #[cfg(feature = "test")]
    pub a11y_published: u64,
    /// Writes given to the engine.
    pub ops: u64,
    /// Writes of the most recent revision.
    pub last_batch_ops: usize,
    pub creates: u64,
    pub set_props: u64,
    pub structural: u64,
    /// Nodes the most recent revision changed (render-count analogue).
    pub host_touched_last: usize,
    pub turns: u64,
    pub stale_events: u64,
    /// Wall time spent inside the core's `turn` callback (the host
    /// excluded), the `on_mount` work it runs included.
    pub core_ns: u64,
    /// Live reactive nodes (signals, memos, effects).
    pub live_reactive: usize,
    pub live_owners: usize,
    /// Live owner-bound objects (vars, callbacks, tasks, actions, ...).
    pub live_objects: usize,
    /// Live constant values (`Value`s of a constant; no graph node).
    pub live_constants: usize,
    /// Allocated graph, owner, object and constant arena slots (bounded growth check).
    pub arena_slots: usize,
    pub tree_nodes: usize,
    pub timers: usize,
    pub frame_loops: usize,
    pub animations: usize,
}
