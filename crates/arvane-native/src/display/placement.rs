//! Role: Chooses window positions on monitors and centers windows in their work areas.

use kivact_types::{Point, Rect, Size, WindowPlacement};

/// Physical desktop bounds of a monitor's usable work area.
pub(super) type WorkArea = (i32, i32, u32, u32);

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
    if !placement_supported(event_loop) {
        return None;
    }
    let monitor = event_loop
        .primary_monitor()
        .or_else(|| event_loop.available_monitors().next())?;
    Some(area_of(monitor))
}

/// The monitor's usable area, in physical pixels, for a monitor.
fn area_of(monitor: winit::monitor::MonitorHandle) -> (winit::monitor::MonitorHandle, WorkArea) {
    let position = monitor.position();
    let size = monitor.size();
    let bounds = (position.x, position.y, size.width, size.height);
    #[cfg(windows)]
    let work = super::windows::work_area(&monitor).unwrap_or(bounds);
    #[cfg(target_os = "linux")]
    let work = super::x11::work_area(bounds).unwrap_or(bounds);
    #[cfg(not(any(windows, target_os = "linux")))]
    let work = bounds;
    (monitor, work)
}

/// The monitor's usable area in physical pixels.
pub(crate) fn work_area_of(monitor: &winit::monitor::MonitorHandle) -> Rect {
    let (_, (x, y, width, height)) = area_of(monitor.clone());
    Rect {
        x: f64::from(x),
        y: f64::from(y),
        width: f64::from(width),
        height: f64::from(height),
    }
}

/// Whether the platform honours explicit window placement (Wayland does not).
pub(crate) fn placement_supported(event_loop: &winit::event_loop::ActiveEventLoop) -> bool {
    #[cfg(target_os = "linux")]
    {
        use winit::platform::wayland::ActiveEventLoopExtWayland;
        !event_loop.is_wayland()
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = event_loop;
        true
    }
}

/// Where a new window opens next to its parent, in physical pixels: centered
/// over it, or to its right, or to its left when the right side has no room.
/// `None` leaves placement to the platform.
pub(crate) fn place_relative(
    parent_outer: Rect,
    size: Size,
    work_area: Rect,
    placement: WindowPlacement,
) -> Option<Point> {
    match placement {
        WindowPlacement::Auto => None,
        WindowPlacement::CenterOnParent => Some(Point {
            x: parent_outer.x + (parent_outer.width - size.width) / 2.0,
            y: parent_outer.y + (parent_outer.height - size.height) / 2.0,
        }),
        WindowPlacement::NextToParent => {
            let right = parent_outer.right();
            if right + size.width <= work_area.right() {
                Some(Point {
                    x: right,
                    y: parent_outer.y,
                })
            } else if parent_outer.x - size.width >= work_area.x {
                Some(Point {
                    x: parent_outer.x - size.width,
                    y: parent_outer.y,
                })
            } else {
                None
            }
        }
    }
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
