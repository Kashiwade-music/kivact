//! Role: The component author's owner: scoped resources without graph access.
//!
//! [`Owner`] wraps the runtime's owner handle and exposes only what a
//! component needs to keep resources for the lifetime of the part it
//! builds: store an object, register a cleanup, provide and read contexts,
//! report through the error boundary, and find the App's root and focus.
//! [`use_app_state`] keeps a component library's App-wide state at the root.
//! Creating signals or effects under an owner, or running setup in one, is
//! the core's job (the setup functions and the flows) and is not offered
//! here.
//!
//! Invariants:
//! - Every method raises a located `WrongApp` for an owner of an App that
//!   is gone or not the one running, and a located `Disposed` for an owner
//!   that is gone (an owner being disposed still accepts registrations,
//!   which the runtime releases with it).
//! - The registering methods (`store`, `store_with`, `provide`,
//!   `on_cleanup`, `node_ref`) are rejected in memos and render effects
//!   (`WriteInPureContext`).

use std::panic::Location;
use std::rc::Rc;

use super::NodeRef;
use super::state::ViewState;
use kivact_runtime::{ErrorKind, Owned, Runtime, UiError, raise, require_effectful};

/// The identity of a component resource owner, without retaining its App.
/// Equality compares identities: every owner of an App has the same
/// [`Owner::root`], and roots of different Apps differ.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Owner(pub(crate) kivact_runtime::Owner);

impl Owner {
    /// Capture the current setup owner. A located `SetupOnly` naming `api`
    /// outside component setup.
    #[track_caller]
    pub fn current() -> Self {
        Self(kivact_runtime::Owner::current(
            "Owner::current",
            Location::caller(),
        ))
    }

    /// Capture the live root of the running App. A located `WrongApp`
    /// outside every App or once its root is gone.
    #[track_caller]
    pub fn root() -> Self {
        let at = Location::caller();
        let rt = Runtime::current()
            .unwrap_or_else(|| raise(ErrorKind::WrongApp, "`Owner::root` needs a running App", at));
        let owner = Self(ViewState::tree(&rt).root_owner());
        owner.runtime();
        owner
    }

    /// The runtime of this owner's App, checked as the module documentation
    /// says.
    #[track_caller]
    pub(crate) fn runtime(self) -> Rc<Runtime> {
        self.0.runtime()
    }

    #[track_caller]
    fn registration(self, api: &'static str) {
        self.runtime();
        require_effectful(api);
    }

    /// Store the supplied shared object until this owner is disposed.
    /// `name` identifies the component handle in disposed-use diagnostics.
    #[track_caller]
    pub fn store<T: 'static>(self, name: &'static str, value: T) -> Owned<T> {
        self.registration("Owner::store");
        self.0.store(name, value, Location::caller())
    }

    /// [`Owner::store`] with `on_dispose`, which runs once, before the value
    /// is dropped, when the owner is disposed or the object is removed
    /// ([`Owned::remove`]).
    #[track_caller]
    pub fn store_with<T: 'static>(
        self,
        name: &'static str,
        value: T,
        on_dispose: impl FnOnce() + 'static,
    ) -> Owned<T> {
        self.registration("Owner::store_with");
        self.0
            .store_with(name, value, move |_| on_dispose(), Location::caller())
    }

    /// A clone of the nearest inherited context starting at this owner.
    #[track_caller]
    pub fn context<T: Clone + 'static>(self) -> Option<T> {
        self.runtime();
        self.0.context::<T>()
    }

    /// A new [`NodeRef`] owned by this owner (disposed with it). Unlike
    /// [`node_ref`](crate::view::node_ref), it can be called outside setup, e.g. from a handler
    /// or effect that binds rows lazily.
    #[track_caller]
    pub fn node_ref(self) -> NodeRef {
        self.registration("Owner::node_ref");
        NodeRef::owned_by(self.0, Location::caller())
    }

    /// Install or replace this owner's context of the supplied type.
    #[track_caller]
    pub fn provide<T: 'static>(self, value: T) {
        self.registration("Owner::provide");
        self.0.provide(value);
    }

    /// Deliver a diagnostic through this owner's error boundary.
    #[track_caller]
    pub fn report(self, kind: ErrorKind, message: impl Into<String>) {
        self.runtime();
        self.0
            .report(UiError::new(kind, message, Some(Location::caller())));
    }

    /// Register cleanup under this owner. While the owner is being
    /// disposed, `f` runs at once.
    #[track_caller]
    pub fn on_cleanup(self, f: impl FnOnce() + 'static) {
        self.registration("Owner::on_cleanup");
        self.0.on_cleanup(f);
    }
}

/// State that belongs to the running App rather than to a component.
///
/// The first call for a type `T` runs `init` as setup of the App's root
/// owner (so `signal`, `memo` and `effect` inside `init` are owned by the
/// root and live as long as the App) and provides the result at the root.
/// Later calls return a clone of that value. It works in setup, handlers,
/// tasks and effects. Outside a running App it raises a located `WrongApp`.
#[track_caller]
pub fn use_app_state<T: Clone + 'static>(init: impl FnOnce() -> T) -> T {
    let root = Owner::root();
    if let Some(value) = root.context::<T>() {
        return value;
    }
    let value = root.0.with_setup(init);
    root.provide(value.clone());
    value
}
