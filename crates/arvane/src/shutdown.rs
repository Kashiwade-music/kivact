//! Role: Record detached workers and retain embedded code after owned fields drop.
//!
//! Invariants: this leaf knows no handler, presenter or host. Driver's final
//! field is RetainOnDrop; Rust drops its handler (including host asset stores)
//! and presenter first. Only embedded drivers enable retention, including
//! failed startup and destruction deferred beyond the closing callback.

pub(crate) struct RetainOnDrop(pub(crate) bool);
impl Drop for RetainOnDrop {
    fn drop(&mut self) {
        if self.0 && kivact_types::process::DetachedThreads::any() {
            arvane_native::retain_module();
        }
    }
}
