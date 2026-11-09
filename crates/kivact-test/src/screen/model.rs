//! Role: Store screen handles, queries and core timing.

use kivact_core::launch::{Inspect, LaunchHandle};
use kivact_core::view::PointerKind;
use kivact_core::view::Role;
use kivact_engine::testing::ArvaneTestHost;
use kivact_style::draw::RegionId;
use kivact_types::{NodeId, WindowId};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

/// How long `find_by_*` and `wait_for` wait by default (virtual time).
pub(super) const FIND_TIMEOUT: Duration = Duration::from_secs(5);

/// A rendered App on real windows of the engine's test driver, shared by
/// every [`Screen`] handle of its windows.
pub(crate) struct TestApp {
    pub(super) host: RefCell<ArvaneTestHost>,
    pub(super) launch: LaunchHandle,
    pub(super) record: Rc<Record>,
    pub(super) actions: RefCell<Vec<String>>,
    /// The file name failure artifacts are written under.
    pub(super) name: String,
    /// Where the screen was rendered (the location of a node created
    /// outside every builder).
    pub(super) at: &'static std::panic::Location<'static>,
    /// Failure artifacts were written (by a failing query); they are written once.
    pub(super) artifacts_written: kivact_types::cell::ReportOnce,
    /// The touch and pen contacts the test has down (`contact_*`), in the
    /// order they went down, each at its last position.
    pub(super) contacts: RefCell<Vec<(Contact, kivact_style::draw::Point)>>,
    // xtask-allow(state-fields): identity: the platform pointer id of the last contact
    /// The platform's id of the last contact (the window gives each
    /// contact a pointer id of its own, never the mouse's).
    pub(super) last_contact: Cell<u64>,
}

/// A handle to one window of a rendered App on the engine's test driver.
#[derive(Clone)]
pub struct Screen {
    pub(super) app: Rc<TestApp>,
    pub(super) window: WindowId,
}

/// A touch or pen contact a test put down with [`Screen::contact_down`].
/// The calls that go on with it name it, so several contacts can be down
/// at once; each is a pointer of its own, with an id the window gives it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Contact {
    /// The platform's id of the contact.
    pub(super) id: u64,
    /// The window the contact went down in.
    pub(super) window: WindowId,
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
pub struct Locator<'a> {
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
        self.check_open();
        crate::Script {
            platform: self.app.host.borrow().script(),
            window: self.window,
        }
    }

    /// Fail unless this handle's window is still open.
    pub(super) fn check_open(&self) {
        if !self.is_open() {
            self.fail(&format!("window {} is closed", self.window.get()));
        }
    }

    /// The test host, failing when this handle's window is closed.
    pub(super) fn host(&self) -> std::cell::Ref<'_, ArvaneTestHost> {
        self.check_open();
        self.app.host.borrow()
    }

    /// The test host, failing when this handle's window is closed.
    pub(super) fn host_mut(&self) -> std::cell::RefMut<'_, ArvaneTestHost> {
        self.check_open();
        self.app.host.borrow_mut()
    }

    pub(super) fn inspect(&self) -> Inspect<'_> {
        self.app
            .launch
            .inspect()
            .expect("invariant: screen host started the app")
    }

    pub(super) fn facts(&self) -> crate::facts::Facts {
        self.check_open();
        crate::facts::Facts::new(self.inspect(), self.window)
    }
}
