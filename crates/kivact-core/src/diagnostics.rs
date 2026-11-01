//! Role: Diagnostics: error tracking and turn records.
//!
//! Diagnostics: errors routed to the default sink (read back with
//! [`errors`]), a ring buffer of
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

mod errors;
mod stats;
pub(crate) mod why;

#[doc(inline)]
pub use kivact_reactive::ErrorKind;
#[doc(inline)]
pub use kivact_reactive::{TraceRun, TraceWrite, TurnRecord};
#[doc(inline)]
pub use why::{Why, WhyDep, WhyProp, why};

pub use errors::errors;
pub use stats::Stats;
