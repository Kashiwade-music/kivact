//! Role: Content versions of composition layers.

/// A content version never given before in this process: the layer is
/// treated as changed.
pub(crate) fn fresh_content_version() -> u64 {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

/// Give every layer of `composition` a fresh version: a composition
/// whose layers are not compared with a last frame (an export, a
/// snapshot, a drawable's subtree).
pub(crate) fn stamp_fresh(composition: &mut arvane_protocol::Composition) {
    for layer in &mut composition.layers {
        layer.content_version = fresh_content_version();
    }
}
