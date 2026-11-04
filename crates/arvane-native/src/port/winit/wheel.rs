//! Role: Converts native wheel notches to logical scroll distances.
// Raw notches point left/up. Shift moves a purely vertical wheel sideways;
// a system page setting uses the logical window height on either axis.
pub(crate) fn lines(
    x: f64,
    y: f64,
    shift: bool,
    height: f64,
    wheel_lines: u32,
    wheel_chars: u32,
) -> (f64, f64) {
    let (x, y) = if shift && x == 0.0 { (y, 0.0) } else { (x, y) };
    let step = |count: u32| {
        if count > 100 {
            height
        } else {
            count as f64 * 20.0
        }
    };
    (-x * step(wheel_chars), -y * step(wheel_lines))
}
