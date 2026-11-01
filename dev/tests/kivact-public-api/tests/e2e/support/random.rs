//! Role: Deterministic random driver for probe tests.
//!
//! The deterministic random driver of the random probe tests
//! (`async_tests`, `lifecycle_tests`), the live counts of `Screen::stats`
//! and the texts on the screen.

use kivact::prelude::*;
use kivact::test::{Screen, SemanticNode};

// ---------------------------------------------------------------------------
// deterministic random driver

pub(crate) struct Rng(u64);

impl Rng {
    pub(crate) fn new(seed: u64) -> Self {
        Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    pub(crate) fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            (self.next() % n as u64) as usize
        }
    }
    pub(crate) fn chance(&mut self, pct: u64) -> bool {
        self.next() % 100 < pct
    }
}

pub(crate) fn seeds() -> std::ops::Range<u64> {
    0..40
}

// ---------------------------------------------------------------------------
// observations

/// What is still alive: the live counts `Screen::stats` reports.
#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct Residue {
    reactive: usize,
    owners: usize,
    objects: usize,
    tree: usize,
    timers: usize,
    loops: usize,
    anims: usize,
}

pub(crate) fn residue(s: &Screen) -> Residue {
    let st = s.stats();
    Residue {
        reactive: st.live_reactive,
        owners: st.live_owners,
        objects: st.live_objects,
        tree: st.tree_nodes,
        timers: st.timers,
        loops: st.frame_loops,
        anims: st.animations,
    }
}

/// The contents of the text nodes on the screen, in tree order.
pub(crate) fn texts(s: &Screen) -> Vec<String> {
    fn walk(n: &SemanticNode, out: &mut Vec<String>) {
        if n.role == Some(Role::Text) {
            out.push(n.name.clone());
        }
        for c in &n.children {
            walk(c, out);
        }
    }
    let mut out = Vec::new();
    walk(&s.snapshot(), &mut out);
    out
}
