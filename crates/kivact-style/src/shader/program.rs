//! Role: Assembling a WGSL source into a shader program, and its errors.

use std::panic::Location;
use std::sync::Arc;

use kivact_runtime::{ErrorKind, UiError};
use kivact_shader::{ShaderError, assemble};
use kivact_types::{ShaderKind, ShaderProgram};

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
pub(crate) fn compile(source: &str, kind: ShaderKind) -> Result<Arc<ShaderProgram>, UiError> {
    assemble(source, kind).map(Arc::new).map_err(shader_error)
}
