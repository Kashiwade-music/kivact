//! Role: Paint shaders: `PaintShader`, and the builders of a `ShaderPaint`.

use std::sync::Arc;

use kivact_runtime::{FrameRate, UiError};
use kivact_types::{ShaderKind, ShaderProgram};

use crate::asset::Asset;
use crate::paint::{Paint, ShaderPaint};

use super::data::ShaderData;
use super::program::compile;
use super::value::ShaderValue;

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
            data: ShaderData::new(self.0.clone()),
            fallback: None,
        }
    }
}

impl ShaderPaint {
    /// Set a field of the WGSL `Params` struct. A name the struct does not
    /// have, or a value of the wrong type, is dropped and reported.
    pub fn param(mut self, name: &str, value: impl Into<ShaderValue>) -> ShaderPaint {
        self.data.set_param(name, value.into());
        self
    }

    /// Bind an image to `image(slot, uv)`; slot 0..=3.
    pub fn image(mut self, slot: u32, image: impl Into<Asset>) -> ShaderPaint {
        self.data.set_image(slot, image.into());
        self
    }

    /// Advance `context.time` at `rate`.
    pub fn animate(mut self, rate: FrameRate) -> ShaderPaint {
        self.data.set_clock(rate);
        self
    }

    /// Keep the clock under reduced motion.
    pub fn essential(mut self) -> ShaderPaint {
        self.data.essential = true;
        self
    }

    /// What CPU rendering (and a failed GPU pipeline) draws instead; default: nothing.
    pub fn fallback(mut self, paint: impl Into<Paint>) -> ShaderPaint {
        self.fallback = Some(Box::new(paint.into()));
        self
    }

    /// The problems with the values set so far (for reporting).
    pub(crate) fn problems(&self) -> &[String] {
        &self.data.problems
    }
}
