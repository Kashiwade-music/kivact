//! Role: The one way to run a user's function inside an engine borrow.
//!
//! The engine calls a few user functions synchronously while it holds its
//! model or tree borrowed: a leaf's measure function, a text editor's
//! behavior and its line colors. Each is wrapped in a [`SyncHook`] and
//! nothing else may be handed to the engine for this purpose.
//!
//! Invariants:
//! - The provider (the mount's owner cleanup and the binding's current
//!   slot) holds the [`Hooked`] content and function; the engine's callback
//!   holds only a [`SyncHook`], which has a `Weak<Hooked>`. The engine
//!   dropping its callback while it is borrowed therefore drops no user
//!   value, and the provider drops the old [`Hooked`] before the view tree
//!   is borrowed (its `Drop` may read the tree).
//! - [`SyncHook::call`] runs the function as a handler of the hook's owner
//!   in a pure, untracked frame: a signal write or other side effect is
//!   rejected as `WriteInPureContext` and a read subscribes nothing.
//! - A panic (a rejected write included) is caught, never unwinds into the
//!   engine, and is reported once per [`Hooked`] through
//!   `Runtime::report_later` (the engine has let go of its model by the
//!   next turn). `call` then returns `None`; the caller keeps its fallback.

use std::cell::{Cell, RefCell};
use std::panic::{AssertUnwindSafe, Location, catch_unwind};
use std::rc::{Rc, Weak};

use kivact_reactive::{Owner, Runtime, UiError, pure, untrack};

/// One content of a hook and its function, kept alive by the provider.
pub(crate) struct Hooked<C, F> {
    pub(crate) content: C,
    f: F,
    /// The failure of this content was reported.
    reported: Cell<bool>,
}

/// The [`Hooked`] the engine calls now, shared by the binding that
/// replaces it and the owner's cleanup that drops it.
pub(crate) type Current<C, F> = Rc<RefCell<Option<Rc<Hooked<C, F>>>>>;

/// What the engine's callback holds: a weak [`Hooked`], the runtime to
/// report to, the owner it runs for and the author's call site.
pub(crate) struct SyncHook<C, F> {
    hooked: Weak<Hooked<C, F>>,
    rt: Weak<Runtime>,
    owner: Owner,
    at: &'static Location<'static>,
}

impl<C, F> SyncHook<C, F> {
    /// A hook for `content` and `f`, and the [`Hooked`] its provider must
    /// keep alive for as long as the engine may call the hook.
    pub(crate) fn new(
        rt: Weak<Runtime>,
        owner: Owner,
        at: &'static Location<'static>,
        content: C,
        f: F,
    ) -> (Self, Rc<Hooked<C, F>>) {
        let hooked = Rc::new(Hooked {
            content,
            f,
            reported: Cell::new(false),
        });
        let hook = Self {
            hooked: Rc::downgrade(&hooked),
            rt,
            owner,
            at,
        };
        (hook, hooked)
    }

    /// Run `run` on the content and the function: `None` when the provider
    /// dropped them or `run` panicked (reported once, see the module doc).
    pub(crate) fn call<R>(&self, run: impl FnOnce(&C, &F) -> R) -> Option<R> {
        let hooked = self.hooked.upgrade()?;
        let result = self.owner.run_handler(|| {
            pure(|| untrack(|| catch_unwind(AssertUnwindSafe(|| run(&hooked.content, &hooked.f)))))
        });
        match result {
            Ok(r) => Some(r),
            Err(panic) => {
                if !hooked.reported.replace(true)
                    && let Some(rt) = self.rt.upgrade()
                {
                    let error = UiError::from_panic(panic, Some(self.at));
                    rt.report_later(Some(self.owner), error);
                }
                None
            }
        }
    }
}
