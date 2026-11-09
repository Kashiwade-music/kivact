//! Role: A shader program and the values set on it: parameters, images, clock.

use std::sync::Arc;

use kivact_runtime::{ErrorKind, FrameRate, UiError};
use kivact_types::{ParamType, ShaderClock, ShaderProgram};

use crate::asset::Asset;
use crate::theme::value::Theme;

use super::value::ShaderValue;

/// The program of a shader and the values set on it. Paint and filter
/// shaders both hold one.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ShaderData {
    pub(crate) program: Arc<ShaderProgram>,
    pub(crate) params: Vec<(String, ShaderValue)>,
    pub(crate) images: [Option<Asset>; 4],
    pub(crate) clock: ShaderClock,
    pub(crate) essential: bool,
    pub(crate) problems: Vec<UiError>,
}

impl ShaderData {
    /// A program with no parameters, images or clock yet.
    pub(crate) fn new(program: Arc<ShaderProgram>) -> ShaderData {
        ShaderData {
            program,
            params: Vec::new(),
            images: [None, None, None, None],
            clock: ShaderClock::Still,
            essential: false,
            problems: Vec::new(),
        }
    }

    /// Set a field of the WGSL `Params` struct. A name the struct does not
    /// have, or a value of the wrong type, is dropped and reported.
    pub(crate) fn set_param(&mut self, name: &str, value: ShaderValue) {
        match param_problem(&self.program, name, &value) {
            Some(problem) => self.problems.push(shader_problem(problem)),
            None => {
                self.params.retain(|(n, _)| n != name);
                self.params.push((name.to_string(), value));
            }
        }
    }

    /// Bind an image to `image(slot, uv)`; slot 0..=3.
    pub(crate) fn set_image(&mut self, slot: u32, image: Asset) {
        match self.images.get_mut(slot as usize) {
            Some(cell) => *cell = Some(image),
            None => self
                .problems
                .push(shader_problem(format!("image slot {slot} is not in 0..=3"))),
        }
    }

    /// Advance `context.time` at `rate`.
    pub(crate) fn set_clock(&mut self, rate: FrameRate) {
        self.clock = clock_of(rate);
    }
}

/// A problem with the values set on a shader, reported as a shader error
/// (its position is the node's, given when the style is reported).
fn shader_problem(message: String) -> UiError {
    UiError::new(ErrorKind::Shader, message, None)
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
/// The wire clock of a frame rate.
fn clock_of(rate: FrameRate) -> ShaderClock {
    match rate {
        FrameRate::Hz(hz) => ShaderClock::Hz(hz),
        FrameRate::Display => ShaderClock::Display,
    }
}
