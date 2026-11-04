//! Role: Diagnostics: error tracking and turn records.
//!
//! In debug builds, test screens can also write comparison traces under
//! `target/kivact-trace/<test-name>.txt`. `KIVACT_TRACE` selects `stage`,
//! `semantics`, and `host_events` with a comma-separated list. These traces
//! include semantic and host event values, in observed order, and omit wall
//! clock durations. `stage` records phase entries and structured turn summaries;
//! `semantics` records node changes observed at phase and commit boundaries;
//! `host_events` records incoming events, commits, and turn outputs. Files keep
//! observation order. Test names come from the test thread, with the render
//! location as a fallback; multiple screens use the same numeric suffixes as
//! failure artifacts. `CARGO_TARGET_DIR` overrides the target directory.
//! Release builds do not read this variable or record comparison traces.
//!
//! Diagnostics: errors routed to the default sink (read back with
//! [`errors`]), a ring buffer of
//! structured turn records (causes, writes, re-evaluations, ops, duration),
//! mirrored to `tracing`, [`why`](fn@why) chains explaining a node's props, and the
//! [`Stats`] counters a test screen reports. Recording never changes
//! behaviour. Structured turn records never record values: only type names and source
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
#[cfg(debug_assertions)]
pub(crate) mod trace;
pub(crate) mod why;

#[doc(inline)]
pub use kivact_reactive::ErrorKind;
#[doc(inline)]
pub use kivact_reactive::{TraceRun, TraceWrite, Turn, TurnRecord};
#[doc(inline)]
pub use why::{Why, WhyDep, WhyProp, why};

pub use errors::errors;
pub use stats::Stats;
