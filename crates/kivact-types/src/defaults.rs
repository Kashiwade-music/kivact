//! Role: Define the common fallback frame, window and click settings once.

/// Fallback display refresh period in nanoseconds (approximately 60 Hz).
pub const FRAME_PERIOD_NS: u64 = 16_666_667;

/// Initial content size in logical pixels, as (width, height).
pub const DEFAULT_WINDOW_SIZE: (f64, f64) = (960.0, 720.0);

/// Fallback double-click timeout in nanoseconds and logical-pixel distance per axis.
///
/// These are Windows' defaults: 500 ms and 4 logical pixels on each axis.
pub const DOUBLE_CLICK: (u64, (f64, f64)) = (500_000_000, (4.0, 4.0));
