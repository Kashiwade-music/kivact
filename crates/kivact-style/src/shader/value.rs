//! Role: `ShaderValue`: the values a shader's `Params` struct takes.

use crate::color::{Color, ColorToken};
use crate::geometry::{Point, Size};

/// A value for a field of a shader's `Params` struct.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShaderValue {
    /// An `f32`.
    F32(f64),
    /// A `vec2<f32>`.
    Vec2([f64; 2]),
    /// A `vec3<f32>`.
    Vec3([f64; 3]),
    /// A `vec4<f32>`.
    Vec4([f64; 4]),
    /// A color, for a `vec4<f32>` (straight alpha sRGB).
    Color(Color),
    /// An `i32`.
    I32(i32),
    /// A `u32`.
    U32(u32),
}

impl From<f64> for ShaderValue {
    fn from(v: f64) -> Self {
        ShaderValue::F32(v)
    }
}

impl From<f32> for ShaderValue {
    fn from(v: f32) -> Self {
        ShaderValue::F32(v as f64)
    }
}

impl From<i32> for ShaderValue {
    fn from(v: i32) -> Self {
        ShaderValue::I32(v)
    }
}

impl From<u32> for ShaderValue {
    fn from(v: u32) -> Self {
        ShaderValue::U32(v)
    }
}

impl From<(f64, f64)> for ShaderValue {
    fn from((x, y): (f64, f64)) -> Self {
        ShaderValue::Vec2([x, y])
    }
}

impl From<(f64, f64, f64)> for ShaderValue {
    fn from((x, y, z): (f64, f64, f64)) -> Self {
        ShaderValue::Vec3([x, y, z])
    }
}

impl From<(f64, f64, f64, f64)> for ShaderValue {
    fn from((x, y, z, w): (f64, f64, f64, f64)) -> Self {
        ShaderValue::Vec4([x, y, z, w])
    }
}

impl From<Point> for ShaderValue {
    fn from(p: Point) -> Self {
        ShaderValue::Vec2([p.x, p.y])
    }
}

impl From<Size> for ShaderValue {
    fn from(s: Size) -> Self {
        ShaderValue::Vec2([s.width, s.height])
    }
}

impl From<Color> for ShaderValue {
    fn from(c: Color) -> Self {
        ShaderValue::Color(c)
    }
}

impl From<ColorToken> for ShaderValue {
    fn from(t: ColorToken) -> Self {
        ShaderValue::Color(Color::token(t))
    }
}
