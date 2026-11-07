//! Role: Shader paints and filter shaders: declarations and re-exports.

pub(crate) mod data;
mod filter;
mod paint_shader;
mod program;
mod value;

pub use crate::paint::ShaderPaint;
pub use filter::{FilterShader, ShaderFilter};
pub use paint_shader::PaintShader;
pub use value::ShaderValue;
