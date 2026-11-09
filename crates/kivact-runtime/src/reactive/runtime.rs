//! Role: Runtime construction and shared arena queries.

use crate::reactive::AppId;
use std::cell::{Cell, RefCell};
use std::panic::Location;

use crate::reactive::arena::Arena;
use crate::reactive::data::{Diag, LiveCounts, Runtime, Sched};
use crate::reactive::host_types::{ExitState, Host, Stage, TurnState};
use crate::reactive::identity::{OwnerId, key_raw};
use std::rc::Rc;

/// One live owner: its identity, its parent and where it was created.
pub struct OwnerInfo {
    /// The parent owner.
    pub parent: Option<OwnerId>,
    /// Where the owner was created.
    // xtask-allow(api-shape): diagnostic source positions retain Rust caller locations.
    pub created_at: &'static Location<'static>,
}

/// One live owner in a [`LiveSnapshot`].
pub struct LiveOwner {
    /// The owner.
    pub id: OwnerId,
    /// The parent owner.
    pub parent: Option<OwnerId>,
    /// Where the owner was created.
    // xtask-allow(api-shape): diagnostic source positions retain Rust caller locations.
    pub created_at: &'static Location<'static>,
}

/// One live owner-bound object in a [`LiveSnapshot`].
pub struct LiveObject {
    /// The object's protocol id.
    pub id: u64,
    /// The owner it is registered under.
    pub owner: OwnerId,
    /// The name it was stored under.
    pub name: &'static str,
    /// Where it was created.
    // xtask-allow(api-shape): diagnostic source positions retain Rust caller locations.
    pub created_at: &'static Location<'static>,
}

/// The live owners and owner-bound objects of one App.
pub struct LiveSnapshot {
    /// Live owners.
    pub owners: Vec<LiveOwner>,
    /// Live objects.
    pub objects: Vec<LiveObject>,
}

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
            write_epoch: Cell::new(kivact_types::cell::Generation::new()),
            // Registration precedes root creation, which always takes the first fresh slot.
            root: crate::reactive::arena::Key {
                slot: 0,
                generation: 0,
            },
            #[expect(
                clippy::disallowed_types,
                reason = "Construction fills this cell before returning or polling; its empty state is inaccessible."
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
        }
    }

    /// The live parent and creation location of owner `id`, or `None` once it is gone.
    pub fn owner_info(&self, id: OwnerId) -> Option<OwnerInfo> {
        self.owners.borrow().get(id.0).map(|o| OwnerInfo {
            parent: o.parent.map(OwnerId),
            created_at: o.created_at,
        })
    }

    /// The live owners and owner-bound objects of this App.
    pub fn live_snapshot(&self) -> LiveSnapshot {
        let owners = self
            .owners
            .borrow()
            .iter()
            .map(|(key, o)| LiveOwner {
                id: OwnerId(key),
                parent: o.parent.map(OwnerId),
                created_at: o.created_at,
            })
            .collect();
        let objects = self
            .objects
            .borrow()
            .iter()
            .map(|(key, o)| LiveObject {
                id: key_raw(key),
                owner: OwnerId(o.owner),
                name: o.name,
                created_at: o.created_at,
            })
            .collect();
        LiveSnapshot { owners, objects }
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
