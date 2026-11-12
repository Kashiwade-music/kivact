//! Role: The round tick step shared by charts and plots.

/// The step of round tick values over `lo..=hi` at about `target` ticks: a
/// step of `10^k × {1, 2, 5, 10}`, the smallest one whose multiples inside the
/// range number at most `target`.
///
/// `lo` must not be above `hi`. `None` for a zero `target`, a range of zero
/// width, a reversed range and a range that is not finite (or whose width
/// overflows).
pub(crate) fn round_step(lo: f64, hi: f64, target: usize) -> Option<f64> {
    let range = hi - lo;
    if target == 0 || !range.is_finite() || range <= 0.0 {
        return None;
    }
    let base = 10f64.powf((range / target as f64).log10().floor());
    [1.0, 2.0, 5.0, 10.0]
        .into_iter()
        .map(|m| base * m)
        .find(|step| count(lo, hi, *step) <= target)
        .or(Some(base * 10.0))
}

/// The number of multiples of `step` in `lo..=hi`.
fn count(lo: f64, hi: f64, step: f64) -> usize {
    let first = (lo / step).ceil() as i64;
    let last = (hi / step).floor() as i64;
    (last - first + 1).max(0) as usize
}

#[cfg(test)]
mod tests {
    //! Role: Tests for the round tick step.

    use super::*;

    #[test]
    fn the_step_is_one_two_or_five_times_a_power_of_ten() {
        assert_eq!(round_step(0.0, 10.0, 5), Some(5.0));
        assert_eq!(round_step(0.0, 1.0, 4), Some(0.5));
        assert_eq!(round_step(0.0, 305.0, 5), Some(100.0));
        assert_eq!(round_step(-3.0, 9.0, 5), Some(5.0));
        assert_eq!(round_step(0.0, 9.0, 5), Some(2.0));
    }

    #[test]
    fn the_multiples_inside_the_range_stay_within_the_target() {
        for (lo, hi, target) in [(0.3, 9.7, 6), (-7.5, 42.0, 4), (0.0, 1e6, 10)] {
            let step = round_step(lo, hi, target).expect("a step");
            assert!(count(lo, hi, step) <= target, "{lo}..{hi} at {target}");
        }
    }

    #[test]
    fn a_range_without_a_step_has_none() {
        assert_eq!(round_step(1.0, 1.0, 5), None);
        assert_eq!(round_step(1.0, 0.0, 5), None);
        assert_eq!(round_step(0.0, f64::INFINITY, 5), None);
        assert_eq!(round_step(-f64::MAX, f64::MAX, 5), None);
        assert_eq!(round_step(0.0, 1.0, 0), None);
    }
}
