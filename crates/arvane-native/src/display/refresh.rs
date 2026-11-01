//! Role: Turns a monitor's reported refresh rate into a frame period.

/// Common display refresh rates (Hz).
const STANDARD_RATES: [u32; 15] = [
    50, 60, 72, 75, 85, 90, 100, 120, 144, 165, 170, 180, 200, 240, 360,
];

/// The frame period of a display reported at `millihertz`. A whole-hertz
/// report is a rate rounded down (Windows reports a 60 Hz mode as 59 Hz):
/// it stands for the common rate within the hertz above it, if any, so
/// ticks do not run a frame per second slower than the display.
pub(crate) fn period_ns(millihertz: u32) -> u64 {
    let millihertz = if millihertz.is_multiple_of(1000) {
        let hz = millihertz / 1000;
        STANDARD_RATES
            .into_iter()
            .find(|rate| (hz..=hz + 1).contains(rate))
            .map_or(millihertz, |rate| rate * 1000)
    } else {
        millihertz
    };
    1_000_000_000_000 / u64::from(millihertz)
}

/// The refresh rate (mHz) of `monitor` as the platform reports it
/// (Windows reports whole hertz, rounded down: 59 for a 60 Hz mode).
/// `None` when it is unknown or implausible (below 20 Hz or above
/// 1000 Hz: Windows reports 1 Hz for a display's "hardware default"
/// rate).
pub(crate) fn refresh_rate_millihertz(
    monitor: Option<winit::monitor::MonitorHandle>,
) -> Option<u32> {
    monitor?
        .refresh_rate_millihertz()
        .filter(|mhz| (20_000..=1_000_000).contains(mhz))
}
