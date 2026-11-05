//! Role: Runtime construction and shared arena queries.

use crate::reactive::AppId;
use std::cell::{Cell, RefCell};

use crate::reactive::arena::Arena;
use crate::reactive::data::{Diag, LiveCounts, Runtime, Sched};
use crate::reactive::host_types::{ExitState, Host, Stage, TurnState};
use std::rc::Rc;

impl Runtime {
    /// A runtime for App `id` with its extensions (registered by
    /// [`Runtime::register`]).
    pub(crate) fn with_id(id: AppId, host: Rc<dyn Host>, start_ns: u64) -> Runtime {
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
            // Registration precedes root creation, which always takes the first fresh slot.
            root: crate::reactive::arena::Key {
                slot: 0,
                generation: 0,
            },
            #[expect(
                clippy::disallowed_types,
                reason = "Construction fills this cell before returning or polling; its empty state is inaccessible (design 19, SM-R17)."
            )]
            ext: std::cell::OnceCell::new(),
            mount_revision: Cell::new(kivact_types::Revision(0)),
            unstamped: RefCell::new(Vec::new()),
            awaiting: RefCell::new(std::collections::VecDeque::new()),
            host,
            state: Cell::new(TurnState::Running(Stage::Setup)),
            exit: Cell::new(ExitState::Running),
            clock: Cell::new(start_ns),
            diag: RefCell::new(Diag::default()),
            later: RefCell::new(Vec::new()),
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
