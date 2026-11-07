//! Role: Drawables: view subtrees a canvas draws (HTML-in-Canvas), their handles and binding.
//!
//! A drawable is created in setup and mounted once (`Node::drawable` or
//! `Viewport::drawable`). The canvas draws it with `Draw::drawable`, and a
//! viewport-hosted drawable is exported to the app through its
//! [`DrawableShare`]. Each drawable's export has a turn-level wake: the
//! share's calls wake the drawable's task, which marks the drawable's
//! notifier; its effect then sends the export request to the engine.
//!
//! Invariants:
//! - A drawable is mounted at one place at a time; the binding names its
//!   root node and is cleared only when that same root is removed.
//! - The export registry (`LIVE`) holds the links of the drawables that
//!   are alive on this thread; engine events for other ids are stale.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::panic::Location;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use kivact_engine::contract::{DrawableTexturesHandle, ExportRequestMode, HostRequest};
use kivact_runtime::{AppId, Notifier, Owned, Owner, Runtime, Value, effect, memo, spawn_in};
use kivact_types::render::ExportedFrame;
use kivact_types::{DrawableId, Failure, NodeId, Size};

use crate::sync::Inbox;
use crate::view::drawable_share::{DrawableShare, ExportLink};
use crate::view::node_ref::NodeRef;

/// The next drawable id; ids are unique for the whole process.
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

thread_local! {
    /// The exports of the drawables alive on this thread, by id.
    static LIVE: RefCell<HashMap<DrawableId, Registered>> = RefCell::new(HashMap::new());
}

/// What the UI thread keeps of a live drawable to deliver engine events to it.
#[derive(Clone)]
struct Registered {
    inbox: Arc<Inbox<ExportLink>>,
    changed: Notifier,
}

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
    let state = Rc::new(DrawableState {
        owner,
        binding: Cell::new(DrawableBinding::Unbound),
        inbox: inbox.clone(),
        root,
        size,
    });
    let state = owner.store("Drawable", state, at);
    LIVE.with(|live| {
        live.borrow_mut().insert(
            id,
            Registered {
                inbox: inbox.clone(),
                changed,
            },
        );
    });
    spawn_in(owner, wake(inbox.clone(), changed), at);
    let sending = inbox.clone();
    let sent = RefCell::new(None);
    effect(move || {
        changed.track();
        send_requests(app, id, &sending, &sent);
    });
    owner.on_cleanup(move || {
        LIVE.with(|live| live.borrow_mut().remove(&id));
        let closed = inbox.update(|link| std::mem::replace(link, ExportLink::Closed));
        drop(closed);
        changed.notify();
    });
    Drawable { app, id, state }
}

/// The task of a drawable's share: each change the app made marks the
/// drawable's notifier, so its requests are sent in the next turn.
async fn wake(inbox: Arc<Inbox<ExportLink>>, changed: Notifier) {
    loop {
        inbox.next(ExportLink::take_changed).await;
        changed.notify();
    }
}

/// What the engine was last asked to export: mode, generation and scale.
type Sent = Option<(ExportRequestMode, u64, f64)>;

/// Send the export request of the drawable `id` when it differs from the
/// one sent last, and its geometry when the app set one.
fn send_requests(app: AppId, id: DrawableId, inbox: &Inbox<ExportLink>, sent: &RefCell<Sent>) {
    let Some(rt) = Runtime::get(app) else {
        return;
    };
    let window_scale = crate::platform::scale_factor().get();
    let writer = crate::view::state::ViewState::tree(&rt).engine().clone();
    let (request, geometry) = inbox.update(|link| match link {
        ExportLink::Open(open) => (
            Some((
                open.request_mode(),
                open.current(),
                open.scale.unwrap_or(window_scale),
            )),
            open.geometry.take().map(|g| (open.current(), g)),
        ),
        ExportLink::Closed => (None, None),
    });
    if let Some((mode, generation, scale)) = request {
        let now = (mode, generation, scale);
        if sent.borrow().as_ref() != Some(&now) {
            *sent.borrow_mut() = Some(now.clone());
            writer.drawable_request(HostRequest::DrawableExport {
                id,
                generation,
                mode: now.0,
                scale,
            });
        }
    }
    if let Some((generation, geometry)) = geometry {
        writer.drawable_request(HostRequest::SetDrawableGeometry {
            id,
            generation,
            geometry,
        });
    }
}

impl Drawable {
    /// The laid-out size of the content; zero until it is laid out.
    pub fn size(&self) -> Value<Size> {
        self.state.get().size
    }

    /// A thread-safe handle that exports frames of a viewport-hosted drawable (§10.5).
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

/// Run `f` on the link of the drawable `id` when it is alive on this thread.
fn with_link(id: DrawableId, f: impl FnOnce(&Inbox<ExportLink>, &Notifier)) {
    let registered = LIVE.with(|live| live.borrow().get(&id).cloned());
    if let Some(registered) = registered {
        f(&registered.inbox, &registered.changed);
    }
}

/// A frame the compositor exported for a drawable.
pub(crate) fn frame(frame: &ExportedFrame) {
    with_link(frame.sink, |inbox, changed| {
        inbox.update(|link| {
            if let ExportLink::Open(open) = link {
                open.on_frame(frame.clone());
            }
        });
        changed.notify();
    });
}

/// The engine answered the textures request of `generation` for `id`.
pub(crate) fn textures(
    id: DrawableId,
    generation: u64,
    result: &Result<DrawableTexturesHandle, Failure>,
) {
    #[cfg(feature = "gpu-interop")]
    {
        let result = match result {
            Ok(handle) => match handle.take() {
                Some(textures) => Ok(textures),
                None => return,
            },
            Err(failure) => Err(failure.clone()),
        };
        with_link(id, |inbox, changed| {
            inbox.update(|link| {
                if let ExportLink::Open(open) = link {
                    open.on_textures(generation, result);
                }
            });
            changed.notify();
        });
    }
    #[cfg(not(feature = "gpu-interop"))]
    let _ = (id, generation, result);
}

/// The drawable `id` was resized: a GPU export asks for a new pool.
pub(crate) fn resized(id: DrawableId) {
    with_link(id, |inbox, changed| {
        inbox.update(|link| {
            if let ExportLink::Open(open) = link {
                let current = open.current();
                #[cfg(feature = "gpu-interop")]
                open.on_gpu_renew(current);
                #[cfg(not(feature = "gpu-interop"))]
                let _ = current;
            }
        });
        changed.notify();
    });
}

/// The GPU pool of drawable `id` of `generation` was lost with the device.
pub(crate) fn lost(id: DrawableId, generation: u64) {
    with_link(id, |inbox, changed| {
        inbox.update(|link| {
            if let ExportLink::Open(open) = link {
                #[cfg(feature = "gpu-interop")]
                open.on_gpu_renew(generation);
                #[cfg(not(feature = "gpu-interop"))]
                let _ = (open, generation);
            }
        });
        changed.notify();
    });
}

/// The window can export to GPU again: every waiting drawable asks anew.
pub(crate) fn gpu_available() {
    let all: Vec<Registered> = LIVE.with(|live| live.borrow().values().cloned().collect());
    for registered in all {
        registered.inbox.update(|link| {
            if let ExportLink::Open(open) = link {
                #[cfg(feature = "gpu-interop")]
                open.on_gpu_available();
                #[cfg(not(feature = "gpu-interop"))]
                let _ = open;
            }
        });
        registered.changed.notify();
    }
}
