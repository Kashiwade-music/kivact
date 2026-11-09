//! Role: Drawable handles: the state, the binding and the export share.
//!
//! A handle is created in setup and knows nothing of builders or mounting:
//! `mount` checks and records the binding through the methods here.

use std::cell::{Cell, RefCell};
use std::panic::Location;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use kivact_runtime::{AppId, Owned, Owner, Runtime, Value, effect, memo, spawn_in};
use kivact_types::{DrawableId, NodeId, Size};

use super::registry::{self, Registered};
use super::requests::{send_requests, wake};
use crate::sync::Inbox;
use crate::view::drawable_share::{DrawableShare, ExportLink};
use crate::view::node_ref::NodeRef;

/// The next drawable id; ids are unique for the whole process.
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// Where a drawable is mounted. Only `root` is kept: the host is the root's
/// parent in the engine, which knows it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DrawableBinding {
    /// Not mounted anywhere.
    Unbound,
    /// Mounted: its root node.
    Bound { root: NodeId },
}

/// The state of one drawable, owned by its setup owner.
pub(crate) struct DrawableState {
    owner: Owner,
    binding: Cell<DrawableBinding>,
    inbox: Arc<Inbox<ExportLink>>,
    /// The root node of the content, bound where the drawable is mounted.
    root: NodeRef,
    size: Value<Size>,
}

/// A view subtree a canvas draws (HTML-in-Canvas). Create it in setup, mount
/// its content with `Node::drawable` (or `Viewport::drawable` to export it),
/// draw it with `Draw::drawable`.
#[derive(Clone, Copy, PartialEq)]
pub struct Drawable {
    app: AppId,
    id: DrawableId,
    state: Owned<Rc<DrawableState>>,
}

/// Create a drawable in setup; its content is mounted once, elsewhere.
#[track_caller]
pub fn drawable() -> Drawable {
    let at = Location::caller();
    let owner = Owner::current("drawable", at);
    let id = DrawableId(NEXT_ID.fetch_add(1, Ordering::Relaxed));
    let inbox = Arc::new(Inbox::new(ExportLink::initial()));
    let root = NodeRef::owned_by(owner, at);
    let rect = root.rect();
    let size = memo(move || {
        rect.get()
            .map_or(Size::new(0.0, 0.0), |r| Size::new(r.width, r.height))
    });
    let changed = owner.notifier(at);
    let app = owner.app();
    let window = crate::view::state::current_window(owner);
    let state = Rc::new(DrawableState {
        owner,
        binding: Cell::new(DrawableBinding::Unbound),
        inbox: inbox.clone(),
        root,
        size,
    });
    let state = owner.store("Drawable", state, at);
    registry::insert(
        id,
        Registered {
            inbox: inbox.clone(),
            changed,
        },
    );
    spawn_in(owner, wake(inbox.clone(), changed), at);
    let sending = inbox.clone();
    let sent = RefCell::new(kivact_types::cell::Sent::new());
    effect(move || {
        changed.track();
        send_requests(app, id, window, &sending, &sent);
    });
    owner.on_cleanup(move || {
        registry::remove(id);
        let closed = inbox.update(|link| std::mem::replace(link, ExportLink::Closed));
        drop(closed);
        changed.notify();
    });
    Drawable { app, id, state }
}

impl Drawable {
    /// The laid-out size of the content; zero until it is laid out.
    pub fn size(&self) -> Value<Size> {
        self.state.get().size
    }

    /// A thread-safe handle that exports frames of a viewport-hosted drawable.
    pub fn share(&self) -> DrawableShare {
        DrawableShare {
            inner: self.state.get().inbox.clone(),
        }
    }

    /// The drawable's id (the engine's name for it).
    pub(crate) fn id(&self) -> DrawableId {
        self.id
    }

    /// The node reference of the content's root, bound where it is mounted.
    pub(crate) fn root_ref(&self) -> NodeRef {
        self.state.get().root
    }

    /// Check the mount against the binding; reports and returns `false` when it
    /// must not be mounted (another App, disposed, or already mounted).
    pub(crate) fn can_mount(&self, rt: &Rc<Runtime>, at: &'static Location<'static>) -> bool {
        use kivact_runtime::{ErrorKind, UiError};
        if self.app != rt.id() {
            rt.report(
                None,
                UiError::new(
                    ErrorKind::WrongApp,
                    "`.drawable(d)` got a Drawable of another App; it was not mounted",
                    Some(at),
                ),
            );
            return false;
        }
        let Some(state) = self.state.find() else {
            rt.report(
                None,
                UiError::new(
                    ErrorKind::Disposed {
                        created_at: None,
                        disposed_by: None,
                    },
                    "`.drawable(d)` got a Drawable whose owner was disposed; it was not mounted",
                    Some(at),
                ),
            );
            return false;
        };
        if let DrawableBinding::Bound { root } = state.binding.get()
            && crate::view::state::ViewState::tree(rt).get(root).is_some()
        {
            rt.report(
                Some(state.owner),
                UiError::new(ErrorKind::Message, "Drawable is already mounted", Some(at)),
            );
            return false;
        }
        true
    }

    /// Record the mount of the root `root`; the binding is cleared when that
    /// same root is removed (a later mount's binding stays).
    pub(crate) fn bind(&self, root: NodeId, owner: Owner) {
        let Some(state) = self.state.find() else {
            return;
        };
        state.binding.set(DrawableBinding::Bound { root });
        let handle = self.state;
        owner.on_cleanup(move || {
            let Some(state) = handle.find() else {
                return;
            };
            if state.binding.get() == (DrawableBinding::Bound { root }) {
                state.binding.set(DrawableBinding::Unbound);
            }
        });
    }
}

impl kivact_style::draw::DrawableSource for Drawable {
    fn drawable_id(&self) -> DrawableId {
        self.id
    }
}
