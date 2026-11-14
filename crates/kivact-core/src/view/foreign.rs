//! Role: The retained state and callbacks of one `foreign_window`
//! declaration, below the view and window modules.

use std::cell::{Cell, RefCell};
use std::marker::PhantomData;
use std::rc::Rc;
use std::rc::Weak;

use kivact_runtime::{Binding, Handler, Owner, Runtime, Value};
use kivact_types::{
    DismissReason, NativeWindow, Size, SurfaceSize, WindowId, WindowMode, WindowPlacement,
};

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

/// The retained callbacks and reactive values of one `foreign_window`
/// declaration.
pub(crate) struct ForeignHooks {
    pub(crate) title: Value<String>,
    pub(crate) open: Binding<bool>,
    pub(crate) size: Size,
    // xtask-allow(state-fields): option: read once when the window opens; absent means the engine's minimum
    pub(crate) min_size: Option<Size>,
    pub(crate) mode: WindowMode,
    pub(crate) placement: WindowPlacement,
    pub(crate) resizable: Value<bool>,
    pub(crate) dismiss: Option<Handler<DismissReason>>,
    // xtask-allow(state-fields): option: the app's handler; absent when it gave none
    pub(crate) ready: RefCell<Option<Handler<ForeignChild>>>,
    // xtask-allow(state-fields): option: the app's handler; absent when it gave none
    pub(crate) resize: RefCell<Option<Handler<SurfaceSize>>>,
    // xtask-allow(state-fields): option: the app's callback; absent when it gave none
    pub(crate) destroy: RefCell<Option<Box<dyn Fn()>>>,
    /// One opening's callback lifetime.
    pub(crate) phase: Cell<ForeignPhase>,
}

impl ForeignHooks {
    /// Run the registration callback of this opening.
    pub(crate) fn ready(&self, rt: &Runtime, owner: Owner, child: ForeignChild) {
        let handler = self.ready.borrow().clone();
        if let Some(handler) = handler
            && let Err(error) = owner.run_handler(|| handler.call(child))
        {
            rt.report(Some(owner), error);
        }
    }

    /// Run the size callback of this opening.
    pub(crate) fn resized(&self, rt: &Runtime, owner: Owner, size: SurfaceSize) {
        let handler = self.resize.borrow().clone();
        if let Some(handler) = handler
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
            && let Some(destroy) = self.destroy.borrow().as_ref()
        {
            destroy();
        }
    }
}

/// The App's open foreign windows, for the exit path.
#[derive(Default)]
pub(crate) struct ForeignWindows {
    slots: RefCell<Vec<Weak<ForeignHooks>>>,
}

impl ForeignWindows {
    /// Record `hooks` as an open foreign window.
    pub(crate) fn register(rt: &Runtime, hooks: &Rc<ForeignHooks>) {
        let windows = rt.ext::<ForeignWindows>();
        let mut slots = windows.slots.borrow_mut();
        slots.retain(|slot| slot.strong_count() > 0);
        slots.push(Rc::downgrade(hooks));
    }

    /// Write `false` to the `open` binding of every foreign window, as the
    /// App's exit turn does. Owners are disposed by the following
    /// stabilization, so `on_destroy` runs while the child is still alive.
    pub(crate) fn close_all_for_exit(rt: &Runtime) {
        let windows = rt.ext::<ForeignWindows>();
        let open: Vec<Rc<ForeignHooks>> = windows
            .slots
            .borrow()
            .iter()
            .filter_map(Weak::upgrade)
            .collect();
        for hooks in open {
            if hooks.phase.get() != ForeignPhase::Released && hooks.open.get_untracked() {
                hooks.open.set(false);
            }
        }
        windows
            .slots
            .borrow_mut()
            .retain(|slot| slot.strong_count() > 0);
    }
}
