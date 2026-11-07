//! Role: Shader paints and filter shaders: WGSL validated by `kivact-shader`,
//! with a `Params` struct, images and a clock, as a paint fill or a filter.

use std::panic::Location;
use std::sync::Arc;

use kivact_runtime::{ErrorKind, FrameRate, UiError};
use kivact_shader::{ShaderError, assemble};
use kivact_types::{ParamType, ShaderClock, ShaderKind, ShaderProgram, ShaderSpec};

use crate::asset::Asset;
use crate::color::{Color, ColorToken};
use crate::geometry::{Point, Size};
use crate::paint::Paint;
use crate::theme::value::Theme;

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

/// Turn a shader error into a located `ErrorKind::Shader` error.
#[track_caller]
fn shader_error(err: ShaderError) -> UiError {
    let message = if err.line == 0 {
        err.message
    } else {
        format!("line {}, column {}: {}", err.line, err.column, err.message)
    };
    UiError::new(ErrorKind::Shader, message, Some(Location::caller()))
}

#[track_caller]
fn compile(source: &str, kind: ShaderKind) -> Result<Arc<ShaderProgram>, UiError> {
    assemble(source, kind).map(Arc::new).map_err(shader_error)
}

/// The parameter type a value must have for a slot of type `ty`, and whether
/// the value fits it.
fn fits(ty: ParamType, value: &ShaderValue) -> bool {
    matches!(
        (ty, value),
        (ParamType::F32, ShaderValue::F32(_))
            | (ParamType::Vec2, ShaderValue::Vec2(_))
            | (ParamType::Vec3, ShaderValue::Vec3(_))
            | (
                ParamType::Vec4,
                ShaderValue::Vec4(_) | ShaderValue::Color(_)
            )
            | (ParamType::I32, ShaderValue::I32(_))
            | (ParamType::U32, ShaderValue::U32(_))
    )
}

/// The problem with setting `name` to `value` in `program`'s `Params`, if any.
fn param_problem(program: &ShaderProgram, name: &str, value: &ShaderValue) -> Option<String> {
    match program.params.iter().find(|slot| slot.name == name) {
        None => Some(format!("shader has no parameter `{name}`")),
        Some(slot) if !fits(slot.ty, value) => {
            Some(format!("parameter `{name}` does not take {value:?}"))
        }
        Some(_) => None,
    }
}

/// The `Params` bytes of `values` against `program`'s layout. Parameters
/// left unset are zero; a value that does not fit is dropped (reported when
/// it was set).
pub(crate) fn pack(
    program: &ShaderProgram,
    values: &[(String, ShaderValue)],
    theme: &Theme,
) -> Arc<[u8]> {
    let mut bytes = vec![0u8; program.params_size.max(16) as usize];
    for (name, value) in values {
        let Some(slot) = program.params.iter().find(|slot| &slot.name == name) else {
            continue;
        };
        if !fits(slot.ty, value) {
            continue;
        }
        let at = slot.offset as usize;
        let mut put = |v: &[f32]| {
            for (i, f) in v.iter().enumerate() {
                let start = at + 4 * i;
                if let Some(dst) = bytes.get_mut(start..start + 4) {
                    dst.copy_from_slice(&f.to_le_bytes());
                }
            }
        };
        match *value {
            ShaderValue::F32(v) => put(&[v as f32]),
            ShaderValue::Vec2(v) => put(&v.map(|x| x as f32)),
            ShaderValue::Vec3(v) => put(&v.map(|x| x as f32)),
            ShaderValue::Vec4(v) => put(&v.map(|x| x as f32)),
            ShaderValue::Color(c) => {
                let c = c.resolve(theme);
                put(&[c.r as f32, c.g as f32, c.b as f32, c.a as f32]);
            }
            ShaderValue::I32(v) => put(&[f32::from_bits(v as u32)]),
            ShaderValue::U32(v) => put(&[f32::from_bits(v)]),
        }
    }
    bytes.into()
}

/// A validated paint shader: `fn paint(position, context[, params])` in WGSL,
/// made by [`PaintShader::wgsl`]. Use [`paint`](Self::paint) to fill with it.
#[derive(Clone, Debug, PartialEq)]
pub struct PaintShader(Arc<ShaderProgram>);

impl PaintShader {
    /// Validate and assemble a paint shader. Errors are `ErrorKind::Shader`
    /// with `line L, column C: message` positions in `source`.
    #[track_caller]
    pub fn wgsl(source: &str) -> Result<PaintShader, UiError> {
        compile(source, ShaderKind::Paint).map(PaintShader)
    }

    /// A paint of this shader, with no parameters, images or clock yet.
    pub fn paint(&self) -> ShaderPaint {
        ShaderPaint {
            program: self.0.clone(),
            params: Vec::new(),
            images: [None, None, None, None],
            clock: ShaderClock::Still,
            essential: false,
            fallback: None,
            problems: Vec::new(),
        }
    }
}

/// A validated filter shader: `fn effect(position, context[, params])` in
/// WGSL, made by [`FilterShader::wgsl`].
#[derive(Clone, Debug, PartialEq)]
pub struct FilterShader(Arc<ShaderProgram>);

impl FilterShader {
    /// Validate and assemble a filter shader (same errors as
    /// [`PaintShader::wgsl`]).
    #[track_caller]
    pub fn wgsl(source: &str) -> Result<FilterShader, UiError> {
        compile(source, ShaderKind::Filter).map(FilterShader)
    }

    /// A filter of this shader, with no parameters, images or clock yet.
    pub fn filter(&self) -> ShaderFilter {
        ShaderFilter {
            program: self.0.clone(),
            params: Vec::new(),
            images: [None, None, None, None],
            clock: ShaderClock::Still,
            essential: false,
            margin: 0.0,
            problems: Vec::new(),
        }
    }
}

// xtask-allow(state-fields): A shader paint independently holds its program, parameters, images, clock, essentiality, fallback and the problems found while setting them.
/// A paint shader with its values: the parameters, the images for `image()`
/// and the clock. A [`Paint`] (`style().background(..)`, `mask(..)`).
#[derive(Clone, Debug, PartialEq)]
pub struct ShaderPaint {
    pub(crate) program: Arc<ShaderProgram>,
    pub(crate) params: Vec<(String, ShaderValue)>,
    pub(crate) images: [Option<Asset>; 4],
    pub(crate) clock: ShaderClock,
    pub(crate) essential: bool,
    pub(crate) fallback: Option<Box<Paint>>,
    pub(crate) problems: Vec<String>,
}

impl ShaderPaint {
    /// Set a field of the WGSL `Params` struct. A name the struct does not
    /// have, or a value of the wrong type, is dropped and reported.
    pub fn param(mut self, name: &str, value: impl Into<ShaderValue>) -> ShaderPaint {
        let value = value.into();
        match param_problem(&self.program, name, &value) {
            Some(problem) => self.problems.push(problem),
            None => {
                self.params.retain(|(n, _)| n != name);
                self.params.push((name.to_string(), value));
            }
        }
        self
    }

    /// Bind an image to `image(slot, uv)`; slot 0..=3.
    pub fn image(mut self, slot: u32, image: impl Into<Asset>) -> ShaderPaint {
        match self.images.get_mut(slot as usize) {
            Some(cell) => *cell = Some(image.into()),
            None => self
                .problems
                .push(format!("image slot {slot} is not in 0..=3")),
        }
        self
    }

    /// Advance `context.time` at `rate`.
    pub fn animate(mut self, rate: FrameRate) -> ShaderPaint {
        self.clock = clock_of(rate);
        self
    }

    /// Keep the clock under reduced motion.
    pub fn essential(mut self) -> ShaderPaint {
        self.essential = true;
        self
    }

    /// What CPU rendering (and a failed GPU pipeline) draws instead; default: nothing.
    pub fn fallback(mut self, paint: impl Into<Paint>) -> ShaderPaint {
        self.fallback = Some(Box::new(paint.into()));
        self
    }

    /// The problems with the values set so far (for reporting).
    pub(crate) fn problems(&self) -> &[String] {
        &self.problems
    }
}

impl From<ShaderPaint> for Paint {
    fn from(s: ShaderPaint) -> Paint {
        Paint::from_shader(s)
    }
}

// xtask-allow(state-fields): A shader filter independently holds its program, parameters, images, clock, essentiality, margin and the problems found while setting them.
/// A filter shader with its values, for `Filter::shader`.
#[derive(Clone, Debug, PartialEq)]
pub struct ShaderFilter {
    program: Arc<ShaderProgram>,
    params: Vec<(String, ShaderValue)>,
    images: [Option<Asset>; 4],
    clock: ShaderClock,
    essential: bool,
    margin: f64,
    problems: Vec<String>,
}

impl ShaderFilter {
    /// Set a field of the WGSL `Params` struct (see [`ShaderPaint::param`]).
    pub fn param(mut self, name: &str, value: impl Into<ShaderValue>) -> ShaderFilter {
        let value = value.into();
        match param_problem(&self.program, name, &value) {
            Some(problem) => self.problems.push(problem),
            None => {
                self.params.retain(|(n, _)| n != name);
                self.params.push((name.to_string(), value));
            }
        }
        self
    }

    /// Bind an image to `image(slot, uv)`; slot 0..=3.
    pub fn image(mut self, slot: u32, image: impl Into<Asset>) -> ShaderFilter {
        match self.images.get_mut(slot as usize) {
            Some(cell) => *cell = Some(image.into()),
            None => self
                .problems
                .push(format!("image slot {slot} is not in 0..=3")),
        }
        self
    }

    /// Advance `context.time` at `rate`.
    pub fn animate(mut self, rate: FrameRate) -> ShaderFilter {
        self.clock = clock_of(rate);
        self
    }

    /// Keep the clock under reduced motion.
    pub fn essential(mut self) -> ShaderFilter {
        self.essential = true;
        self
    }

    /// px the output may extend beyond the element (glows); default 0.
    pub fn margin(mut self, px: impl Into<f64>) -> ShaderFilter {
        let px: f64 = px.into();
        self.margin = px.max(0.0);
        self
    }

    /// The problems with the values set so far (for reporting).
    pub(crate) fn problems(&self) -> &[String] {
        &self.problems
    }

    /// The engine's filter, with the theme's colors resolved.
    pub(crate) fn spec(&self, theme: &Theme) -> ShaderSpec {
        ShaderSpec {
            program: self.program.clone(),
            params: pack(&self.program, &self.params, theme),
            images: self.images.clone(),
            clock: self.clock,
            essential: self.essential,
            fallback: None,
            margin: self.margin,
        }
    }
}

/// The wire clock of a frame rate.
fn clock_of(rate: FrameRate) -> ShaderClock {
    match rate {
        FrameRate::Hz(hz) => ShaderClock::Hz(hz),
        FrameRate::Display => ShaderClock::Display,
    }
}
