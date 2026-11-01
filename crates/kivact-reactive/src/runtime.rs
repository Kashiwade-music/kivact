//! Role: Runtime construction and shared arena queries.

use crate::AppId;
use std::cell::{Cell, OnceCell, RefCell};

use crate::arena::Arena;
use crate::data::{Diag, Extensions, LiveCounts, Runtime, Sched};

impl Runtime {
    /// A runtime for App `id` with its extensions (registered by
    /// [`Runtime::register`]).
    pub(crate) fn with_id(id: AppId, ext: Extensions) -> Runtime {
        Runtime {
            id,
            graph: RefCell::new(Arena::new()),
            owners: RefCell::new(Arena::new()),
            objects: RefCell::new(Arena::new()),
            consts: RefCell::new(Arena::new()),
            frames: RefCell::new(Vec::new()),
            collectors: RefCell::new(Vec::new()),
            sched: RefCell::new(Sched::default()),
            scratch: RefCell::new(Vec::new()),
            seq: Cell::new(0),
            write_epoch: Cell::new(0),
            root_owner: Cell::new(None),
            ext,
            mounts: RefCell::new(Vec::new()),
            driver: OnceCell::new(),
            diag: RefCell::new(Diag::default()),
            later: RefCell::new(Vec::new()),
            turn_requested: Cell::new(false),
            writer: Cell::new(None),
            stab_epoch: Cell::new(0),
        }
    }

    /// Live arena counts of this App (lifecycle leak checks).
    pub fn live_counts(&self) -> LiveCounts {
        let (g, o, ob, c) = (
            self.graph.borrow(),
            self.owners.borrow(),
            self.objects.borrow(),
            self.consts.borrow(),
        );
        LiveCounts {
            reactive: g.len(),
            owners: o.len(),
            objects: ob.len(),
            constants: c.len(),
            slots: g.capacity() + o.capacity() + ob.capacity() + c.capacity(),
        }
    }

    pub(crate) fn next_seq(&self) -> u64 {
        let s = self.seq.get() + 1;
        self.seq.set(s);
        s
    }
}
