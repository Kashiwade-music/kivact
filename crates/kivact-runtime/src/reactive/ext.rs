//! Role: Per-app extension state for subsystems.
//!
//! Per-App extensions: state that subsystems above the reactive core keep per
//! App, installed once when the App's runtime is created and reachable from
//! any `&Runtime` by type.
//!
//! The set never changes after creation, so `&T` can be handed out for the
//! runtime's lifetime; values use interior mutability like the runtime's own
//! fields. Entries drop in installation order when the runtime drops.

use crate::reactive::data::Runtime;

impl Runtime {
    /// The extension of type `T` installed at creation. A missing extension
    /// is a composition bug of the App, not a user error: it panics.
    #[inline]
    #[track_caller]
    pub fn ext<T: 'static>(&self) -> &T {
        let at = std::panic::Location::caller();
        let extensions = self.ext.get().unwrap_or_else(|| {
            panic!("kivact misuse: invariant: extension read while the App is being constructed at {at}")
        });
        match extensions.get::<T>() {
            Some(t) => t,
            None => missing_extension::<T>(),
        }
    }
}

#[cold]
#[inline(never)]
fn missing_extension<T>() -> ! {
    panic!(
        "invariant: extension {} was not installed when the App was created",
        std::any::type_name::<T>()
    )
}
