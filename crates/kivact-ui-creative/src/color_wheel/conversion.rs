//! Role: Public HSV color conversions for color controls.
use kivact::style::Color;
/// Convert an sRGB color to hue (degrees), saturation and value.
pub fn color_to_hsv(color: Color) -> (f64, f64, f64) {
    let [r, g, b, _] = color.components();
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;
    let hue = if d < 0.00001 {
        0.0
    } else if max == r {
        (g - b) / d
    } else if max == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    (
        (hue * 60.0).rem_euclid(360.0),
        if max > 0.0 { d / max } else { 0.0 },
        max,
    )
}
/// Build an sRGB color from HSV and alpha, with finite bounded channels.
pub fn color_from_hsv(hue: f64, saturation: f64, value: f64, alpha: f64) -> Color {
    let finite = |v: f64, default: f64| if v.is_finite() { v } else { default };
    let h = finite(hue, 0.0).rem_euclid(360.0) / 60.0;
    let s = finite(saturation, 0.0).clamp(0.0, 1.0);
    let v = finite(value, 0.0).clamp(0.0, 1.0);
    let c = v * s;
    let x = c * (1.0 - (h.rem_euclid(2.0) - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match h as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    Color::from_rgba_f64(
        (r + m),
        (g + m),
        (b + m),
        if alpha.is_finite() {
            alpha.clamp(0.0, 1.0)
        } else {
            1.0
        },
    )
}
