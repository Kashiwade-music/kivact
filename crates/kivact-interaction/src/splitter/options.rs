//! Role: Store setup-only splitter options and immutable transition snapshots.
use kivact_core::state::Value;

pub(super) struct Input {
    pub scale: f64,
    pub snaps: Vec<f64>,
    pub collapse: Option<f64>,
}
/// What the setters of a [`super::SplitterApi`] store; read where they apply, so they
/// work in any order.
#[derive(Clone, Copy, Default)]
pub(super) struct SplitterOptions {
    pub(super) key_step: Option<Value<f64>>,
    pub(super) shift_step_factor: Option<f64>,
    pub(super) reset: Option<Value<Option<f64>>>,
    pub(super) scale: Option<Value<f64>>,
    pub(super) disabled: Option<Value<bool>>,
    pub(super) collapse: Option<f64>,
    pub(super) snaps: Option<Value<Vec<f64>>>,
}

impl SplitterOptions {
    pub(super) fn snapshot(self) -> Input {
        Input {
            scale: self.scale.map_or(1.0, |scale| scale.get_untracked()),
            snaps: self
                .snaps
                .map(|stops| stops.get_untracked())
                .unwrap_or_default()
                .into_iter()
                .filter(|stop| stop.is_finite() && (0.0..=1.0).contains(stop))
                .collect(),
            collapse: self.collapse,
        }
    }
}
