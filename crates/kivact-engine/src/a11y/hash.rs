//! Role: Fingerprint canonical accessible payloads and their transport geometry.
use kivact_types::render::SemanticsNode;
use std::hash::{DefaultHasher, Hash, Hasher};
pub(super) fn fingerprint(n: &SemanticsNode) -> u64 {
    let mut h = DefaultHasher::new();
    n.id.hash(&mut h);
    n.children.hash(&mut h);
    n.semantics.hash(&mut h);
    n.bounds.is_some().hash(&mut h);
    let bounds = n.bounds.unwrap_or_default();
    [
        bounds.x,
        bounds.y,
        bounds.width,
        bounds.height,
        n.scroll.x,
        n.scroll.y,
        n.transform.a,
        n.transform.b,
        n.transform.c,
        n.transform.d,
        n.transform.e,
        n.transform.f,
    ]
    .map(|v| if v == 0.0 { 0 } else { v.to_bits() })
    .hash(&mut h);
    h.finish()
}
