//! Role: Initial window placement and display frame pacing.
//!
//! Invariant: the period is read from the monitor's reported refresh rate
//! when the window is created and when it moves to another monitor; an
//! unknown or implausible report keeps the period the window had (60 Hz
//! before any report).
//!
//! The refresh rate is read again only when `Moved` or a scale change
//! finds the window on a monitor of another identity, before the
//! `EnvironmentChanged` that announces it (`runtime.rs`). Windows reports
//! whole hertz, rounded down: such a report stands for the common rate
//! (60, 120, 144 ..) within the hertz above it ([`period_ns`]). Below 20
//! Hz or above 1000 Hz a report is implausible (Windows reports 1 Hz for a
//! display's "hardware default" rate).
//!
//! Limits: a change of the same monitor's refresh rate is not noticed
//! (nothing reads it every frame).

#[cfg(windows)]
mod windows;
#[cfg(target_os = "linux")]
mod x11;

mod types;
use types::WorkArea;

/// The monitor and work area selected before creating the first window.
pub(crate) struct InitialPlacement {
    monitor: winit::monitor::MonitorHandle,
    work: WorkArea,
}

/// Center the complete window in physical desktop coordinates.
fn centered_window_origin(work: WorkArea, logical: (f64, f64), scale: f64) -> (i32, i32) {
    let (x, y, width, height) = work;
    let offset = |available: u32, requested: f64| {
        ((f64::from(available) - (requested * scale).round()).max(0.0) / 2.0).floor() as i64
    };
    let axis = |origin: i32, offset: i64| {
        (i64::from(origin) + offset).clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
    };
    (
        axis(x, offset(width, logical.0)),
        axis(y, offset(height, logical.1)),
    )
}

fn target_monitor_area(
    event_loop: &winit::event_loop::ActiveEventLoop,
) -> Option<(winit::monitor::MonitorHandle, WorkArea)> {
    #[cfg(target_os = "linux")]
    {
        use winit::platform::wayland::ActiveEventLoopExtWayland;
        if event_loop.is_wayland() {
            return None;
        }
    }
    let monitor = event_loop
        .primary_monitor()
        .or_else(|| event_loop.available_monitors().next())?;
    let position = monitor.position();
    let size = monitor.size();
    let bounds = (position.x, position.y, size.width, size.height);
    #[cfg(windows)]
    let work = windows::work_area(&monitor).unwrap_or(bounds);
    #[cfg(target_os = "linux")]
    let work = x11::work_area(bounds).unwrap_or(bounds);
    #[cfg(not(any(windows, target_os = "linux")))]
    let work = bounds;
    Some((monitor, work))
}

/// Choose a creation position on the target monitor so its DPI resolves the size.
/// `center_window` then corrects decoration extents on that same monitor.
pub(crate) fn initial_window_position(
    event_loop: &winit::event_loop::ActiveEventLoop,
    logical: (f64, f64),
) -> Option<(winit::dpi::PhysicalPosition<i32>, InitialPlacement)> {
    let (monitor, work) = target_monitor_area(event_loop)?;
    let (x, y) = centered_window_origin(work, logical, monitor.scale_factor());
    Some((
        winit::dpi::PhysicalPosition::new(x, y),
        InitialPlacement { monitor, work },
    ))
}

/// Correct the first hidden window's outer bounds before it is shown.
/// Creation must use `initial_window_position` so its size already uses the
/// target monitor's DPI. Wayland owns placement and receives no position.
pub(crate) fn center_window(
    placement: &InitialPlacement,
    window: &winit::window::Window,
    logical: (f64, f64),
) {
    let scale = placement.monitor.scale_factor();
    let outer = window.outer_size();
    let logical = if outer.width > 0 && outer.height > 0 {
        (
            f64::from(outer.width) / scale,
            f64::from(outer.height) / scale,
        )
    } else {
        logical
    };
    let (x, y) = centered_window_origin(placement.work, logical, scale);
    window.set_outer_position(winit::dpi::PhysicalPosition::new(x, y));
}

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

#[cfg(test)]
mod tests {
    //! Role: Tests for startup placement in physical monitor work areas.
    use super::centered_window_origin;

    #[test]
    fn scale_and_negative_origins_use_physical_coordinates() {
        for (scale, expected) in [
            (1.0, (-1280, 320)),
            (1.5, (-1440, 230)),
            (2.0, (-1600, 140)),
        ] {
            assert_eq!(
                centered_window_origin((-1920, -40, 1920, 1080), (640.0, 360.0), scale),
                expected
            );
        }
    }

    #[test]
    fn oversized_windows_start_at_the_work_area_edge_per_axis() {
        assert_eq!(
            centered_window_origin((20, 40, 300, 200), (400.0, 100.0), 1.0),
            (20, 90)
        );
        assert_eq!(
            centered_window_origin((20, 40, 300, 200), (400.0, 300.0), 1.0),
            (20, 40)
        );
    }

    #[test]
    fn monitor_bounds_fallback_centers_without_reserved_space() {
        assert_eq!(
            centered_window_origin((0, 0, 1920, 1080), (800.0, 600.0), 1.0),
            (560, 240)
        );
    }
}
