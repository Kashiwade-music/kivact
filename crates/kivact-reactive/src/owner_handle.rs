//! Role: Copy handles to owners and to the objects stored under them.
//!
//! [`Owner`] names one owner of one App; [`Owned`] names one object stored
//! under an owner. Neither retains the runtime: every operation looks the
//! App up again and raises a located `WrongApp` when it is gone or when
//! another App is running.
//!
//! The operations keep the runtime's registration rules (`owner.rs`): on an
//! owner that is being disposed or is gone, a cleanup runs at once, and a
//! node, object or context is released by the same disposal or at once.
//! They do not check purity or liveness themselves; a layer that hands an
//! owner to component authors adds the checks its API promises.
//!
//! Invariants:
//! - The creation methods take the location diagnostics name the created
//!   owner, node or object by (`Disposed` errors, `why` chains, the trace).
//! - [`Owned::try_get`] is `None` from the start of its owner's disposal;
//!   [`Owned::get`] and [`Owned::find`] still resolve an object that the
//!   disposal has not released yet (cleanups may use it).

use std::any::TypeId;
use std::marker::PhantomData;
use std::panic::Location;
use std::rc::Rc;

use crate::arena::Key;
use crate::data::{EffectKind, Runtime, Signal, UiThreadOnly, Value};
use crate::error::UiError;
use crate::execution::handle_usable_here;
use crate::registry::{check_not_foreign, runtime, try_runtime};
use crate::{AppId, NodeId, OwnerId, Trigger};

/// One owner of one App: the unit that disposes everything registered under
/// it (nodes, objects, cleanups, contexts, child owners). Copy; it does not
/// keep the owner or the App alive. Equality compares the identity only.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Owner {
    pub(crate) app: AppId,
    pub(crate) key: Key,
    pub(crate) marker: PhantomData<UiThreadOnly>,
}

/// An object stored under an [`Owner`] ([`Owner::store`]), released when the
/// owner is disposed or by [`Owned::remove`]. Copy; it does not keep the
/// object or the App alive. Equality compares the identity only.
pub struct Owned<T: 'static> {
    app: AppId,
    key: Key,
    name: &'static str,
    marker: PhantomData<(fn() -> T, UiThreadOnly)>,
}

impl Owner {
    pub(crate) fn from_key(app: AppId, key: Key) -> Self {
        Self {
            app,
            key,
            marker: PhantomData,
        }
    }

    /// The owner of the component setup that is running. A located
    /// `SetupOnly` naming `api` outside setup (also outside every App). It
    /// never falls back to the App's root.
    pub fn current(api: &'static str, at: &'static Location<'static>) -> Self {
        let (rt, key) = crate::scope::setup_scope(api, at);
        Self::from_key(rt.id, key)
    }

    /// The nearest owner of the code that is running: the setup owner, the
    /// owner a handler, task, frame callback or cleanup runs for, the owner
    /// of a running `Callback` or effect. A located `Disposed` when that
    /// owner is already disposed, a located `SetupOnly` naming `api` when
    /// no owner applies.
    pub fn ambient(api: &'static str, at: &'static Location<'static>) -> Self {
        let (rt, key) = crate::scope::owner_scope(api, at);
        Self::from_key(rt.id, key)
    }

    /// The App of this owner.
    pub fn app(self) -> AppId {
        self.app
    }

    /// The identity of this owner within its App.
    pub fn id(self) -> OwnerId {
        OwnerId(self.key)
    }

    /// The runtime of this owner's App. A located `WrongApp` when the App is
    /// gone or another App is running, a located `Disposed` when the owner
    /// is gone (an owner being disposed is still there).
    #[track_caller]
    pub fn runtime(self) -> Rc<Runtime> {
        let at = Location::caller();
        let rt = runtime(self.app, at);
        rt.require_owner(self.key, "Owner", at);
        rt
    }

    /// The runtime without the owner check (the registration rules apply).
    fn rt(self, at: &'static Location<'static>) -> Rc<Runtime> {
        runtime(self.app, at)
    }

    /// A child owner, disposed with this one; `at` names it in diagnostics.
    /// Under an owner that is being disposed or is gone the child is created
    /// disposed: what is registered under it is released at once.
    pub fn child(self, at: &'static Location<'static>) -> Owner {
        Self::from_key(self.app, self.rt(at).create_owner(Some(self.key), at))
    }

    /// Dispose this owner and its whole subtree: child owners first, then
    /// cleanups (last registered first), effects, memos, sources and
    /// objects. Nothing happens when it is already gone.
    #[track_caller]
    pub fn dispose(self) {
        self.rt(Location::caller()).dispose_owner(self.key);
    }

    /// Whether the owner is alive and its disposal has not started: the
    /// component it owns is still shown. `false` once its App is gone.
    /// Registrations need no such check (see the module documentation).
    pub fn is_alive(self) -> bool {
        try_runtime(self.app).is_some_and(|rt| rt.owner_live(self.key))
    }

    /// Run `f` as component setup of this owner: the setup functions
    /// (`signal`, `memo`, `on_cleanup`, ...) register under it. The setup
    /// frame is popped also on unwind. An owner created disposed accepts the
    /// setup; what it registers is released at once.
    #[track_caller]
    pub fn with_setup<R>(self, f: impl FnOnce() -> R) -> R {
        let rt = self.rt(Location::caller());
        let _setup = rt.setup_frame(self.key);
        f()
    }

    /// A writable signal of this owner holding `value`, created at `at`.
    pub fn signal<T: 'static>(self, value: T, at: &'static Location<'static>) -> Signal<T> {
        let key = self.rt(at).create_source(self.key, value, at);
        Signal::from_key(self.app, key)
    }

    /// A value of this owner that never changes: no handle can write it, so
    /// a property bound to it is written once, as a constant is.
    pub fn constant<T: 'static>(self, value: T, at: &'static Location<'static>) -> Value<T> {
        let key = self.rt(at).create_const(self.key, value, at);
        Value::constant(self.app, key)
    }

    /// A [`Trigger`] of this owner, created at `at`.
    pub fn trigger(self, at: &'static Location<'static>) -> Trigger {
        let key = self.rt(at).create_source(self.key, (), at);
        Trigger::from_key(self.app, key)
    }

    /// An effect of this owner that re-runs `f` when what it read changes.
    /// `Render` and `Internal` effects run once now (their first output is
    /// part of the mount); `User` effects are queued for the end of the
    /// turn. Returns the effect's identity (diagnostics).
    pub fn effect(
        self,
        kind: EffectKind,
        f: impl Fn() + 'static,
        at: &'static Location<'static>,
    ) -> NodeId {
        NodeId(self.rt(at).create_effect(self.key, kind, Box::new(f), at))
    }

    /// Store `value` until this owner is disposed. `name` identifies the
    /// handle in `Disposed` diagnostics; `at` is where it was created.
    pub fn store<T: 'static>(
        self,
        name: &'static str,
        value: Rc<T>,
        at: &'static Location<'static>,
    ) -> Owned<T> {
        let key = self.rt(at).insert_object(self.key, value, None, at);
        Owned::from_key(self.app, key, name)
    }

    /// [`Owner::store`] with `on_dispose`, which runs once, before the value
    /// is dropped, when the object is removed or its owner is disposed. It
    /// runs under the object's owner, with no runtime borrow held.
    pub fn store_with<T: 'static>(
        self,
        name: &'static str,
        value: Rc<T>,
        on_dispose: impl FnOnce(&Runtime) + 'static,
        at: &'static Location<'static>,
    ) -> Owned<T> {
        let rt = self.rt(at);
        let key = rt.insert_object(self.key, value, Some(Box::new(on_dispose)), at);
        Owned::from_key(self.app, key, name)
    }

    /// Provide `value` to this owner and its descendants, replacing a value
    /// of the same type this owner provided. On a gone owner the value is
    /// dropped at once.
    #[track_caller]
    pub fn provide<T: 'static>(self, value: T) {
        self.rt(Location::caller())
            .provide_context(self.key, Rc::new(value));
    }

    /// A clone of the nearest `T` provided to this owner or one of its
    /// ancestors.
    #[track_caller]
    pub fn context<T: Clone + 'static>(self) -> Option<T> {
        self.rt(Location::caller())
            .find_context(self.key, TypeId::of::<T>())
            .and_then(|value| value.downcast_ref::<T>().cloned())
    }

    /// Run `f` when this owner is disposed, after its child owners and
    /// before its effects and values (cleanups run last registered first).
    /// On an owner that is being disposed or is gone, `f` runs at once.
    #[track_caller]
    pub fn on_cleanup(self, f: impl FnOnce() + 'static) {
        self.rt(Location::caller())
            .add_cleanup(self.key, Box::new(f));
    }

    /// Make this owner an error boundary: errors reported to it or to an
    /// owner below it (without a nearer boundary) go to `f`. It runs in a
    /// handler of this owner; a panic in it and the original error both go
    /// to the App's default sink.
    #[track_caller]
    pub fn on_error(self, f: impl Fn(UiError) + 'static) {
        self.rt(Location::caller())
            .set_error_handler(self.key, Rc::new(f));
    }

    /// Route `err` to the nearest error boundary of this owner, or to the
    /// App's default sink (`tracing` and [`Runtime::errors`]).
    #[track_caller]
    pub fn report(self, err: UiError) {
        self.rt(Location::caller()).route(Some(self.key), err);
    }

    /// Run `f` as a handler of this owner: untracked, not pure, not setup;
    /// the owner-relative lookups (`context`, `spawn`) use this owner. The
    /// previous context is restored also on unwind.
    #[track_caller]
    pub fn run_handler<R>(self, f: impl FnOnce() -> R) -> R {
        let rt = self.rt(Location::caller());
        let _handler = rt.handler_frame(Some(self.key));
        f()
    }

    /// Run `f` at a boundary: an `Err` it returns or a panic it raises is
    /// reported to this owner (its error boundary, else the App's sink).
    /// It pushes no frame; combine with [`Owner::run_handler`] to run `f`
    /// as a handler.
    #[track_caller]
    pub fn guarded(self, f: impl FnOnce() -> Result<(), UiError>) {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
            Ok(Ok(())) => {}
            Ok(Err(err)) => self.report(err),
            Err(panic) => self.report(UiError::from_panic(panic, None)),
        }
    }
}

impl<T: 'static> Owned<T> {
    fn from_key(app: AppId, key: Key, name: &'static str) -> Self {
        Self {
            app,
            key,
            name,
            marker: PhantomData,
        }
    }

    /// The stored value. A located `WrongApp` when the App is gone or
    /// another App is running, a located `Disposed` once the object was
    /// released. It still resolves while its owner is being disposed.
    #[track_caller]
    pub fn get(self) -> Rc<T> {
        self.get_at(Location::caller())
    }

    /// [`Owned::get`] with its errors located at `at`: for a layer that
    /// resolves the handle on behalf of a call it recorded earlier (an
    /// effect created for a property).
    pub fn get_at(self, at: &'static Location<'static>) -> Rc<T> {
        runtime(self.app, at).object_or_panic(self.key, self.name, at)
    }

    /// The stored value while its owner is alive: `None` from the start of
    /// the owner's disposal, once the object was released, when the App is
    /// gone or when another App is running. Never panics. Use an
    /// `Owned<()>` stored at setup as a probe of whether a component is
    /// still shown.
    pub fn try_get(self) -> Option<Rc<T>> {
        if !handle_usable_here(self.app) {
            return None;
        }
        let rt = try_runtime(self.app)?;
        let owner = rt.object_owner(self.key)?;
        if !rt.owner_live(owner) {
            return None;
        }
        rt.object::<T>(self.key)
    }

    /// The stored value until it is released, also while its owner is
    /// being disposed (for cleanups); `None` once it was released or when
    /// the App is gone. A located `WrongApp` when another App is running.
    #[track_caller]
    pub fn find(self) -> Option<Rc<T>> {
        check_not_foreign(self.app, Location::caller());
        try_runtime(self.app)?.object::<T>(self.key)
    }

    /// Release the object now: its `on_dispose` runs, then the value is
    /// dropped. Nothing happens when it was released already or its App is
    /// gone. A located `WrongApp` when another App is running.
    #[track_caller]
    pub fn remove(self) {
        check_not_foreign(self.app, Location::caller());
        if let Some(rt) = try_runtime(self.app) {
            rt.remove_object(self.key);
        }
    }
}

impl<T: 'static> Clone for Owned<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Copy for Owned<T> {}

impl<T: 'static> PartialEq for Owned<T> {
    fn eq(&self, other: &Self) -> bool {
        self.app == other.app && self.key == other.key
    }
}

impl<T: 'static> Eq for Owned<T> {}

impl<T: 'static> std::hash::Hash for Owned<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.app.hash(state);
        self.key.hash(state);
    }
}

impl<T: 'static> std::fmt::Debug for Owned<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Owned({}:{})", self.key.slot, self.key.generation)
    }
}
