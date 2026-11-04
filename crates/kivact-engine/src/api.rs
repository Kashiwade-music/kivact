//! Role: Re-export engine handles used by the core client.
//!
//! The host supplies owned write access at startup and invokes callbacks
//! without holding model borrows. Every later call is a turn on a laid-out
//! tree: the whole ordered event queue, with the commit boundaries
//! among its events, so the client runs `on_mount` after the events of its
//! revision.

pub use crate::host::{Boundary, Client, HostWaker};

pub use crate::host::write::typed::{Prop, PropValue};
pub use crate::host::write::{Reader, Writer};
pub use crate::offscreen::{OffscreenError, OffscreenImage, OffscreenRenderer};

pub use crate::text_measure::TextMeasurer;

pub use crate::mutation::WriteStats;

pub use crate::host::options::HostOptions;
