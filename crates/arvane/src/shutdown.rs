//! Role: Record detached workers and retain embedded code after owned fields drop.
//!
//! Invariants: this leaf knows no handler, presenter or host. Driver's final
//! field is RetainOnDrop; Rust drops its handler (including host asset stores)
//! and presenter first. Only embedded drivers enable retention, including
//! failed startup and destruction deferred beyond the closing callback.

#[cfg(feature = "gpu")]
pub(crate) static DETACHED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Whether a presenter, asset worker or native UI thread was detached.
pub fn detached_threads() -> bool {
    #[cfg(feature = "gpu")]
    if DETACHED.load(std::sync::atomic::Ordering::Acquire) {
        return true;
    }
    arvane_asset::detached_threads() || arvane_native::detached_threads()
}

pub(crate) struct RetainOnDrop(pub(crate) bool);
impl Drop for RetainOnDrop {
    fn drop(&mut self) {
        if self.0 && detached_threads() {
            arvane_native::retain_module();
        }
    }
}
