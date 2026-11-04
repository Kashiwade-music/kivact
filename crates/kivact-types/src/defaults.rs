//! Role: Define the common fallback frame, window and click settings once.

use crate::geometry::Size;

/// Fallback display refresh period in nanoseconds (approximately 60 Hz).
pub const FRAME_PERIOD_NS: u64 = 16_666_667;

/// Initial content size in logical pixels.
pub const DEFAULT_WINDOW_SIZE: Size = Size {
    width: 960.0,
    height: 720.0,
};

/// Fallback double-click timeout in nanoseconds and logical-pixel distance per axis.
///
/// These are Windows' defaults: 500 ms and 4 logical pixels on each axis.
pub const DOUBLE_CLICK: (u64, (f64, f64)) = (500_000_000, (4.0, 4.0));
