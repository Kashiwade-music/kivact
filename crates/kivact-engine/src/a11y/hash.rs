//! Role: Fingerprint canonical accessible payloads and their transport geometry.
use kivact_types::render::SemanticsNode;
use std::hash::{DefaultHasher, Hash, Hasher};
pub(super) fn fingerprint(n: &SemanticsNode) -> u64 {
    let mut h = DefaultHasher::new();
    n.id.hash(&mut h);
    n.children.hash(&mut h);
    n.semantics.hash(&mut h);
    [
        n.bounds.x,
        n.bounds.y,
        n.bounds.width,
        n.bounds.height,
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
