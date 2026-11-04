//! Role: Error boundary for catching child errors.
//!
//! `error_boundary`: an owner boundary around a child that receives the
//! errors reported in it: handler and task `Err`s, `Owner::report`, failed
//! platform requests, lifecycle `Err`s. A panic is not an error: it is not
//! caught here and ends the App.
//!
//! What happens to an error raised under the boundary while the child is
//! shown:
//! - neither handler set: it goes on to the parent's handler;
//! - `.on_error(f)`: `f(err)` runs and the child stays;
//! - `.fallback(f)`: the child is disposed and `f(err, reset)` is shown;
//!   calling `reset()` remounts a fresh child;
//! - both: `on_error` runs first, then the fallback replaces the child.
//!
//! While the fallback is shown, errors (from the fallback, or late ones
//! from the child) go to the parent's handler instead of being dropped.
//! `on_error` runs as a handler of the boundary; a panic in it propagates.

use std::cell::RefCell;
use std::panic::Location;
use std::rc::Rc;

use crate::view::descriptor::Mcx;
use crate::view::descriptor::{IntoView, View, view_fn};
use kivact_runtime::EffectKind;
use kivact_runtime::UiError;

use crate::view::flow::branch::{Branch, Build, build, mount_branch, unmount_branch};

type FallbackFn = Rc<dyn Fn(UiError, Retry) -> View>;

/// A boundary catching the errors of its child. Built by
/// [`error_boundary`]; see the module documentation for the semantics.
pub struct ErrorBoundary {
    child: Build,
    fallback: Option<FallbackFn>,
    on_error: Option<Rc<dyn Fn(UiError)>>,
    at: &'static Location<'static>,
}

/// Catch the errors raised in `child()`'s subtree; choose what happens with
/// [`ErrorBoundary::fallback`] and [`ErrorBoundary::on_error`].
///
/// ```ignore
/// error_boundary(editor)
///     .on_error(move |e| log.push(e))
///     .fallback(|e, reset| column((text(e.to_string()), button("Retry").on_press(move || reset()))))
/// ```
#[track_caller]
pub fn error_boundary<V: IntoView>(child: impl Fn() -> V + 'static) -> ErrorBoundary {
    ErrorBoundary {
        child: build(child),
        fallback: None,
        on_error: None,
        at: Location::caller(),
    }
}

impl ErrorBoundary {
    /// On the first error, replace the child with `f(err, reset)`;
    /// calling `reset()` mounts a fresh child again.
    pub fn fallback<V: IntoView>(mut self, f: impl Fn(UiError, Retry) -> V + 'static) -> Self {
        self.fallback = Some(Rc::new(move |e, reset| f(e, reset).into_view()));
        self
    }

    /// Receive each error raised under the boundary while the child is
    /// shown; the child keeps being shown (unless a fallback is also set).
    pub fn on_error(mut self, f: impl Fn(UiError) + 'static) -> Self {
        self.on_error = Some(Rc::new(f));
        self
    }
}

/// What the boundary does with an error raised under it while the child is shown.
enum Catch {
    /// `.on_error(f)` alone: `f` runs and the child stays.
    Observe(Rc<dyn Fn(UiError)>),
    /// `.fallback(f)`, with or without `.on_error`: the fallback replaces the child.
    Replace {
        on_error: Option<Rc<dyn Fn(UiError)>>,
        fallback: FallbackFn,
    },
}

/// What a boundary with a fallback shows.
#[derive(Clone, PartialEq)]
enum Shown {
    Child,
    Fallback(UiError),
}

impl IntoView for ErrorBoundary {
    fn into_view(self) -> View {
        let ErrorBoundary {
            child,
            fallback,
            on_error,
            at,
        } = self;
        let catch = match (fallback, on_error) {
            (None, None) => None,
            (None, Some(f)) => Some(Catch::Observe(f)),
            (Some(fallback), on_error) => Some(Catch::Replace { on_error, fallback }),
        };
        view_fn(move |cx| {
            let frag = cx.fragment();
            let rt = cx.rt.clone();
            let parent = cx.owner;
            let boundary = parent.child(at);
            let (on_error, fallback) = match catch {
                // No handler: errors go on to the parent; the child is mounted once and stays.
                None => {
                    mount_branch(&rt, boundary.child(at), frag, || child());
                    return frag;
                }
                Some(Catch::Observe(f)) => {
                    boundary.on_error(move |e| f(e));
                    mount_branch(&rt, boundary.child(at), frag, || child());
                    return frag;
                }
                Some(Catch::Replace { on_error, fallback }) => (on_error, fallback),
            };
            let shown = boundary.signal(Shown::Child, at);
            let weak_rt = Rc::downgrade(&rt);
            boundary.on_error(move |e: UiError| {
                let Some(rt) = weak_rt.upgrade() else { return };
                match shown.try_get_untracked() {
                    Some(Shown::Child) => {
                        if let Some(f) = &on_error {
                            f(e.clone());
                        }
                        shown.set(Shown::Fallback(e));
                    }
                    // While the fallback is shown, and once the boundary is disposed, the error
                    // goes to the parent.
                    Some(Shown::Fallback(_)) | None => rt.report(Some(parent), e),
                }
            });
            let weak = Rc::downgrade(&rt);
            let current: RefCell<Option<Branch>> = RefCell::new(None);
            let inner = Mcx {
                rt: &rt,
                owner: boundary,
            };
            inner.effect(EffectKind::Internal, at, move || {
                let now = shown.get();
                let Some(rt) = weak.upgrade() else { return };
                if let Some(b) = current.borrow_mut().take() {
                    unmount_branch(&rt, b);
                }
                let b = match now {
                    Shown::Child => mount_branch(&rt, boundary.child(at), frag, || child()),
                    Shown::Fallback(e) => {
                        let fallback = fallback.clone();
                        mount_branch(&rt, boundary.child(at), frag, move || {
                            let reset = Retry::new(move || shown.set(Shown::Child));
                            fallback(e, reset)
                        })
                    }
                };
                *current.borrow_mut() = Some(b);
            });
            frag
        })
    }
}

/// A Copy handle that retries the operation associated with a fallback.
#[derive(Clone, Copy)]
pub struct Retry(kivact_runtime::Callback<()>);
impl Retry {
    /// Store a retry action under the current owner.
    #[track_caller]
    pub(crate) fn new(f: impl Fn() + 'static) -> Self {
        Self(f.into())
    }
    /// Try the failed operation again.
    pub fn run(&self) {
        self.0.call(());
    }
}
