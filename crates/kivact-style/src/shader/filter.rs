//! Role: Filter shaders: `FilterShader`, and `ShaderFilter` with its builders.

use std::sync::Arc;

use kivact_runtime::{FrameRate, UiError};
use kivact_types::{ShaderKind, ShaderProgram, ShaderSpec};

use crate::asset::Asset;
use crate::theme::value::Theme;

use super::data::{ShaderData, pack};
use super::program::compile;
use super::value::ShaderValue;

/// A validated filter shader: `fn effect(position, context[, params])` in
/// WGSL, made by [`FilterShader::wgsl`].
#[derive(Clone, Debug, PartialEq)]
pub struct FilterShader(Arc<ShaderProgram>);

impl FilterShader {
    /// Validate and assemble a filter shader (same errors as
    /// `PaintShader::wgsl`).
    #[track_caller]
    pub fn wgsl(source: &str) -> Result<FilterShader, UiError> {
        compile(source, ShaderKind::Filter).map(FilterShader)
    }

    /// A filter of this shader, with no parameters, images or clock yet.
    pub fn filter(&self) -> ShaderFilter {
        ShaderFilter {
            data: ShaderData::new(self.0.clone()),
            margin: 0.0,
        }
    }
}

/// A filter shader with its values, for `Filter::shader`.
#[derive(Clone, Debug, PartialEq)]
pub struct ShaderFilter {
    pub(crate) data: ShaderData,
    pub(crate) margin: f64,
}

impl ShaderFilter {
    /// Set a field of the WGSL `Params` struct (see `ShaderPaint::param`).
    pub fn param(mut self, name: &str, value: impl Into<ShaderValue>) -> ShaderFilter {
        self.data.set_param(name, value.into());
        self
    }

    /// Bind an image to `image(slot, uv)`; slot 0..=3.
    pub fn image(mut self, slot: u32, image: impl Into<Asset>) -> ShaderFilter {
        self.data.set_image(slot, image.into());
        self
    }

    /// Advance `context.time` at `rate`.
    pub fn animate(mut self, rate: FrameRate) -> ShaderFilter {
        self.data.set_clock(rate);
        self
    }

    /// Keep the clock under reduced motion.
    pub fn essential(mut self) -> ShaderFilter {
        self.data.essential = true;
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
        &self.data.problems
    }

    /// The engine's filter, with the theme's colors resolved.
    pub(crate) fn spec(&self, theme: &Theme) -> ShaderSpec {
        ShaderSpec {
            program: self.data.program.clone(),
            params: pack(&self.data.program, &self.data.params, theme),
            images: self.data.images.clone(),
            clock: self.data.clock,
            essential: self.data.essential,
            fallback: None,
            margin: self.margin,
        }
    }
}
