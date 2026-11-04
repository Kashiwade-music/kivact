//! Role: Finite numeric semantics with equality-consistent bit hashing.

use std::hash::{Hash, Hasher};

/// A finite numeric value; NaN and infinities are rejected at construction.
/// Both signed zeros are stored as positive zero so equal values hash equally.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Finite(f64);

impl Eq for Finite {}

impl Hash for Finite {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl Finite {
    /// Accept a finite value, normalizing either signed zero to positive zero.
    pub fn new(value: f64) -> Option<Self> {
        if !value.is_finite() {
            return None;
        }
        Some(Self(if value == 0.0 { 0.0 } else { value }))
    }

    /// The finite value supplied at construction, with signed zero normalized.
    pub fn get(self) -> f64 {
        self.0
    }
}
