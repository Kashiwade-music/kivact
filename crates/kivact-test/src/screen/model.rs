//! Role: Store screen handles, queries and core timing.

use kivact_core::launch::{Inspect, LaunchHandle};
use kivact_core::view::PointerKind;
use kivact_core::view::Role;
use kivact_engine::testing::ArvaneTestHost;
use kivact_style::draw::RegionId;
use kivact_types::NodeId;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

/// How long `find_by_*` and `wait_for` wait by default (virtual time).
pub(super) const FIND_TIMEOUT: Duration = Duration::from_secs(5);

/// A rendered App on a real window of the engine's test driver.
pub struct Screen {
    pub(super) host: RefCell<ArvaneTestHost>,
    pub(super) launch: LaunchHandle,
    pub(super) record: Rc<Record>,
    pub(super) actions: RefCell<Vec<String>>,
    /// The file name failure artifacts are written under.
    pub(super) name: String,
    /// Where the screen was rendered (the location of a node created
    /// outside every builder).
    pub(super) at: &'static std::panic::Location<'static>,
    // xtask-allow(state-fields): independent report-once flag for failure artifact publication.
    /// Failure artifacts were written (by a failing query).
    pub(super) artifacts_written: Cell<bool>,
    /// Physical pixels per logical pixel.
    pub(super) scale: Cell<f64>,
    /// The touch and pen contacts the test has down (`contact_*`), in the
    /// order they went down, each at its last position.
    pub(super) contacts: RefCell<Vec<(Contact, kivact_style::draw::Point)>>,
    // xtask-allow(state-fields): identifier allocator, not an event or phase.
    /// The platform's id of the last contact (the window gives each
    /// contact a pointer id of its own, never the mouse's).
    pub(super) last_contact: Cell<u64>,
}

/// A touch or pen contact a test put down with [`Screen::contact_down`].
/// The calls that go on with it name it, so several contacts can be down
/// at once; each is a pointer of its own, with an id the window gives it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Contact {
    /// The platform's id of the contact.
    pub(super) id: u64,
    pub(super) kind: PointerKind,
}

impl Contact {
    /// A finger (`PointerKind::Touch`) or a pen (`PointerKind::Pen`).
    pub fn kind(self) -> PointerKind {
        self.kind
    }

    /// How the action log names it.
    pub(super) fn name(self) -> String {
        format!("{:?} {}", self.kind, self.id)
    }
}

/// A node (or a scene region) found by a query.
#[derive(Clone, Copy)]
pub struct Element<'a> {
    pub(super) screen: &'a Screen,
    pub(super) node: NodeId,
    pub(super) region: Option<RegionId>,
}

#[derive(Clone)]
pub(super) enum Query {
    Role(Role, String),
    Text(String),
    TestId(String),
}

impl std::fmt::Display for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Query::Role(r, n) => write!(f, "role {r:?} named {n:?}"),
            Query::Text(t) => write!(f, "text {t:?}"),
            Query::TestId(t) => write!(f, "test_id {t:?}"),
        }
    }
}

/// Queries scoped to a subtree: `screen.within(list).get_by_text("a")`.
#[derive(Clone, Copy)]
pub struct Within<'a> {
    pub(super) screen: &'a Screen,
    pub(super) scope: NodeId,
}

/// Time spent in the core's turns; the engine owns commit counters.
#[derive(Default)]
pub(super) struct Record {
    pub(super) core_ns: Cell<u64>,
}

impl Screen {
    pub fn script(&self) -> crate::Script {
        crate::Script {
            platform: self.host.borrow().script(),
        }
    }
    pub(super) fn inspect(&self) -> Inspect<'_> {
        self.launch
            .inspect()
            .expect("invariant: screen host started the app")
    }
    pub(super) fn facts(&self) -> crate::facts::Facts {
        crate::facts::Facts::new(self.inspect())
    }
}
