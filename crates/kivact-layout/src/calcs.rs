//! Role: Owns and resolves live calc style handles.
//!
//! A calc length (`pct ± px`) stays a calc through the layout: the style
//! holds a handle taffy resolves against the same basis as a percentage in
//! that place, never a percentage alone. The values sit in one pool shared
//! by every style, so a node's style keeps only small handles.
//!
//! Invariants:
//! - A handle is its pool index plus one, shifted to look like an aligned
//!   pointer (taffy's calc handle); an unknown handle resolves to 0.
//! - The node whose style made a handle owns it (`NodeLayoutState::calcs`)
//!   and frees it when its layout style is rebuilt or the node is removed;
//!   freed entries are reused.

/// The calc values of the live styles (see the module invariants).
#[derive(Clone, Debug, Default)]
pub(crate) struct Calcs {
    /// `(percent, px)` per handle (percent of the basis, 0..=100 for 100%).
    values: Vec<(f64, f64)>,
    pub(crate) free: Vec<u32>,
}

impl Calcs {
    fn add(&mut self, percent: f64, px: f64) -> u32 {
        match self.free.pop() {
            Some(i) => {
                self.values[i as usize] = (percent, px);
                i
            }
            None => {
                self.values.push((percent, px));
                (self.values.len() - 1) as u32
            }
        }
    }

    /// The value of the calc handle `handle` against `basis`.
    pub(crate) fn resolve(&self, handle: *const (), basis: f32) -> f32 {
        let index = (handle.addr() >> 3).wrapping_sub(1);
        self.values.get(index).map_or(0.0, |(percent, px)| {
            (f64::from(basis) * percent / 100.0 + px) as f32
        })
    }
}

/// Makes the calc handles of one style, owned by its slot.
pub(crate) struct CalcAlloc<'a> {
    pub(crate) calcs: &'a mut Calcs,
    pub(crate) owned: Vec<u32>,
}

impl CalcAlloc<'_> {
    /// The handle of `percent`% of the basis plus `px`.
    pub(crate) fn calc(&mut self, percent: f64, px: f64) -> *const () {
        let i = self.calcs.add(percent, px);
        self.owned.push(i);
        std::ptr::without_provenance(((i as usize) + 1) << 3)
    }
}
