//! Role: Tick positions and the label format of plot axes and readouts.

use super::model::TickFormat;

/// The most ticks one axis makes, whatever the target.
const MAX_TICKS: usize = 1000;

/// The tick values of `lo..hi` at about `target` ticks: a step of
/// `10^k × {1, 2, 5}` (or `10^(k+1)`), the smallest that keeps the count at
/// most `target`. Empty for a range of zero width or a non-finite range.
pub(super) fn linear_ticks(lo: f64, hi: f64, target: usize) -> Vec<f64> {
    let (lo, hi) = if lo <= hi { (lo, hi) } else { (hi, lo) };
    let range = hi - lo;
    if target == 0 || !range.is_finite() || range <= 0.0 {
        return Vec::new();
    }
    let base = 10f64.powf((range / target as f64).log10().floor());
    let step = [1.0, 2.0, 5.0, 10.0]
        .into_iter()
        .map(|m| base * m)
        .find(|step| count(lo, hi, *step) <= target)
        .unwrap_or(base * 10.0);
    let first = (lo / step).ceil() as i64;
    let last = (hi / step).floor() as i64;
    (first..=last)
        .take(MAX_TICKS)
        .map(|k| k as f64 * step)
        .collect()
}

/// The number of multiples of `step` in `lo..=hi`.
fn count(lo: f64, hi: f64, step: f64) -> usize {
    let first = (lo / step).ceil() as i64;
    let last = (hi / step).floor() as i64;
    (last - first + 1).max(0) as usize
}

/// A tick of a log axis: its value and whether it is labeled (a major tick).
#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) struct LogTick {
    /// The value; the axis places it at `log10(value)`.
    pub(super) value: f64,
    /// Whether it has a label and a strong grid line.
    pub(super) major: bool,
}

/// The ticks of a log axis over the values `lo..hi` (both positive).
///
/// A range of a decade or more has a major tick at each power of ten, and
/// minor ticks at 2 to 9 times each power when it spans at most six decades.
/// A narrower range uses the linear ticks of its values.
pub(super) fn log_ticks(lo: f64, hi: f64, target: usize) -> Vec<LogTick> {
    if !(lo > 0.0 && hi > lo && hi.is_finite()) {
        return Vec::new();
    }
    let decades = hi.log10() - lo.log10();
    if decades < 1.0 {
        return linear_ticks(lo, hi, target)
            .into_iter()
            .filter(|value| *value > 0.0)
            .map(|value| LogTick { value, major: true })
            .collect();
    }
    let first = lo.log10().floor() as i32;
    let last = hi.log10().ceil() as i32;
    let mut ticks = Vec::new();
    for power in first..=last {
        let base = 10f64.powi(power);
        if decades <= 6.0 {
            for multiple in 2..=9 {
                let value = base * f64::from(multiple);
                if value >= lo && value <= hi {
                    ticks.push(LogTick {
                        value,
                        major: false,
                    });
                }
            }
        }
        if base >= lo && base <= hi {
            ticks.push(LogTick {
                value: base,
                major: true,
            });
        }
    }
    ticks.sort_by(|a, b| a.value.total_cmp(&b.value));
    ticks
}

/// The label of `value` in `format`. `step` is the tick spacing (zero for a
/// readout), which decides how many decimals a decibel label shows.
pub(super) fn format_value(value: f64, format: &TickFormat, step: f64) -> String {
    match format {
        TickFormat::Plain => plain(value),
        TickFormat::Si => si(value),
        TickFormat::Decibel => {
            if step >= 1.0 {
                format!("{value:.0}")
            } else {
                format!("{value:.1}")
            }
        }
        TickFormat::Degrees => format!("{}°", plain(value)),
    }
}

/// Four significant digits, with trailing zeros dropped.
fn plain(value: f64) -> String {
    if !value.is_finite() {
        return format!("{value}");
    }
    if value == 0.0 {
        return "0".to_owned();
    }
    let magnitude = value.abs().log10().floor() as i32;
    let decimals = (3 - magnitude).clamp(0, 12) as usize;
    trim(format!("{value:.decimals$}"))
}

/// Remove the trailing zeros and the point of a fixed-point number.
fn trim(text: String) -> String {
    if !text.contains('.') {
        return text;
    }
    let trimmed = text.trim_end_matches('0').trim_end_matches('.');
    if trimmed == "-0" {
        "0".to_owned()
    } else {
        trimmed.to_owned()
    }
}

/// SI prefixes for magnitudes of 10³ and up, and of 10⁻³ and down to 10⁻⁶.
fn si(value: f64) -> String {
    let magnitude = value.abs();
    let (factor, suffix) = if magnitude >= 1e9 {
        (1e9, "G")
    } else if magnitude >= 1e6 {
        (1e6, "M")
    } else if magnitude >= 1e3 {
        (1e3, "k")
    } else if magnitude >= 1.0 || magnitude == 0.0 || !magnitude.is_finite() {
        (1.0, "")
    } else if magnitude >= 1e-3 {
        (1e-3, "m")
    } else if magnitude >= 1e-6 {
        (1e-6, "µ")
    } else {
        (1.0, "")
    };
    format!("{}{suffix}", plain(value / factor))
}

#[cfg(test)]
mod tests {
    //! Role: Tests for tick positions and label formats.

    use super::*;

    #[test]
    fn linear_ticks_use_one_two_or_five_steps() {
        assert_eq!(linear_ticks(0.0, 10.0, 5), vec![0.0, 5.0, 10.0]);
        assert_eq!(linear_ticks(0.0, 1.0, 4), vec![0.0, 0.5, 1.0]);
        let ticks = linear_ticks(0.3, 9.7, 6);
        assert!(ticks.len() <= 6 && ticks.iter().all(|t| *t >= 0.3 && *t <= 9.7));
    }

    #[test]
    fn a_degenerate_range_has_no_ticks() {
        assert!(linear_ticks(1.0, 1.0, 5).is_empty());
        assert!(linear_ticks(0.0, f64::INFINITY, 5).is_empty());
        assert!(linear_ticks(0.0, 1.0, 0).is_empty());
    }

    #[test]
    fn a_wide_log_range_labels_each_decade_and_adds_minor_ticks() {
        let ticks = log_ticks(1.0, 1000.0, 5);
        let majors: Vec<f64> = ticks.iter().filter(|t| t.major).map(|t| t.value).collect();
        assert_eq!(majors, vec![1.0, 10.0, 100.0, 1000.0]);
        assert!(
            ticks
                .iter()
                .any(|t| !t.major && (t.value - 20.0).abs() < 1e-9)
        );
    }

    #[test]
    fn a_narrow_log_range_uses_linear_ticks_of_its_values() {
        let ticks = log_ticks(2.0, 6.0, 4);
        assert!(!ticks.is_empty() && ticks.iter().all(|t| t.major));
        assert!(ticks.iter().all(|t| t.value >= 2.0 && t.value <= 6.0));
    }

    #[test]
    fn plain_labels_have_four_significant_digits() {
        assert_eq!(format_value(1500.0, &TickFormat::Plain, 0.0), "1500");
        assert_eq!(format_value(186.0, &TickFormat::Plain, 0.0), "186");
        assert_eq!(format_value(1.23456, &TickFormat::Plain, 0.0), "1.235");
        assert_eq!(
            format_value(0.00123456, &TickFormat::Plain, 0.0),
            "0.001235"
        );
        assert_eq!(format_value(-0.0, &TickFormat::Plain, 0.0), "0");
    }

    #[test]
    fn si_labels_use_prefixes() {
        assert_eq!(format_value(1500.0, &TickFormat::Si, 0.0), "1.5k");
        assert_eq!(format_value(2e6, &TickFormat::Si, 0.0), "2M");
        assert_eq!(format_value(0.01, &TickFormat::Si, 0.0), "10m");
        assert_eq!(format_value(3e-6, &TickFormat::Si, 0.0), "3µ");
        assert_eq!(format_value(42.0, &TickFormat::Si, 0.0), "42");
    }

    #[test]
    fn decibel_and_degree_labels() {
        assert_eq!(format_value(-6.0, &TickFormat::Decibel, 10.0), "-6");
        assert_eq!(format_value(-0.5, &TickFormat::Decibel, 0.5), "-0.5");
        assert_eq!(format_value(90.0, &TickFormat::Degrees, 0.0), "90°");
    }
}
