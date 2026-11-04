//! Role: Scroll snapping policy and the shared nearest-stop calculation.
/// Alignment of a scroll container's direct in-flow children at rest.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollSnap {
    #[default]
    None,
    Start,
    Center,
    End,
}
/// Index of the closest stop. Ties select the first stop in the slice.
pub fn nearest_snap(position: impl Into<f64>, stops: &[f64]) -> Option<usize> {
    let position = position.into();
    stops
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| (*a - position).abs().total_cmp(&(*b - position).abs()))
        .map(|(index, _)| index)
}
