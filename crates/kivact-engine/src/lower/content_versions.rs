//! Role: Content versions of composition layers.

/// A content version never given before in this process: the layer is
/// treated as changed. (Annex E's E14 replaces its use in `into_layer`
/// with versions kept while a layer's items stay the same.)
pub(crate) fn fresh_content_version() -> u64 {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}
