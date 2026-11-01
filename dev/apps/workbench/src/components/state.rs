//! Role: Passive shared workbench state.

use std::rc::Rc;

use kivact::prelude::Signal;

use crate::model::{Record, RecordKey};

/// Buckets of the record timeline (records per bucket = count / BUCKETS).
pub const BUCKETS: usize = 100;

/// Shared state of the workbench, provided as context to rows and detail.
#[derive(Clone, Copy)]
pub struct Board {
    pub records: Signal<Rc<Vec<Record>>>,
    pub selected: Signal<Option<RecordKey>>,
}
