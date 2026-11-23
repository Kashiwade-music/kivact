//! Role: The retained state and callbacks of one `foreign_window`
//! declaration, below the view and window modules.

use std::cell::Cell;
use std::marker::PhantomData;

use kivact_runtime::driver::Runtime;
use kivact_runtime::{Handler, Owner, Value};
use kivact_types::{NativeWindow, SurfaceSize, WindowId};

/// The child window of one opening of a `foreign_window`. A Copy handle (like
/// a `Signal`), valid while that opening lasts.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ForeignChild {
    pub(crate) window: WindowId,
    pub(crate) child: NativeWindow,
    pub(crate) size: SurfaceSize,
    /// The handle belongs to the UI thread, like a `Signal`.
    pub(crate) _thread: PhantomData<*const ()>,
}

/// One opening's callback lifetime, owned by its hooks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ForeignPhase {
    /// The window is opening; the native child is not reported yet.
    Waiting,
    /// The child exists and the app was told.
    Ready(ForeignChild),
    /// `on_destroy` ran (or the opening failed before the child existed).
    Released,
}

/// The retained callbacks and phase of one `foreign_window` declaration. The
/// callbacks are written by the builder and only read afterwards.
pub(crate) struct ForeignHooks {
    pub(crate) resizable: Value<bool>,
    pub(crate) ready: Option<Handler<ForeignChild>>,
    pub(crate) resize: Option<Handler<SurfaceSize>>,
    pub(crate) destroy: Option<Box<dyn Fn()>>,
    /// One opening's callback lifetime.
    pub(crate) phase: Cell<ForeignPhase>,
}

impl ForeignHooks {
    /// Run the registration callback of this opening.
    pub(crate) fn ready(&self, rt: &Runtime, owner: Owner, child: ForeignChild) {
        if let Some(handler) = &self.ready
            && let Err(error) = owner.run_handler(|| handler.call(child))
        {
            rt.report(Some(owner), error);
        }
    }

    /// Run the size callback of this opening.
    pub(crate) fn resized(&self, rt: &Runtime, owner: Owner, size: SurfaceSize) {
        if let Some(handler) = &self.resize
            && let Err(error) = owner.run_handler(|| handler.call(size))
        {
            rt.report(Some(owner), error);
        }
    }

    /// Run the destroy callback once for this opening.
    pub(crate) fn release(&self) {
        let phase = self.phase.get();
        if phase == ForeignPhase::Released {
            return;
        }
        self.phase.set(ForeignPhase::Released);
        if matches!(phase, ForeignPhase::Ready(_))
            && let Some(destroy) = self.destroy.as_ref()
        {
            destroy();
        }
    }
}
