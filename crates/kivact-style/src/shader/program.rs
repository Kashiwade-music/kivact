//! Role: Assembling a WGSL source into a shader program, and its errors.

use std::panic::Location;
use std::sync::Arc;

use kivact_runtime::{ErrorKind, UiError};
use kivact_types::{ShaderKind, ShaderProgram};

/// Validate and assemble `source` with naga (GPU builds).
#[cfg(feature = "shader-validation")]
#[track_caller]
pub(crate) fn compile(source: &str, kind: ShaderKind) -> Result<Arc<ShaderProgram>, UiError> {
    kivact_shader::assemble(source, kind)
        .map(Arc::new)
        .map_err(|err| {
            let message = match err.position {
                None => err.message,
                Some((line, column)) => format!("line {line}, column {column}: {}", err.message),
            };
            UiError::new(ErrorKind::Shader, message, Some(Location::caller()))
        })
}

/// Keep `source` as it is (CPU-only builds, which draw the fallback): only
/// an empty source is an error. Parameters are not known, so none is
/// checked (`data::param_problem`).
#[cfg(not(feature = "shader-validation"))]
#[track_caller]
pub(crate) fn compile(source: &str, kind: ShaderKind) -> Result<Arc<ShaderProgram>, UiError> {
    if source.trim().is_empty() {
        return Err(UiError::new(
            ErrorKind::Shader,
            "the shader source is empty",
            Some(Location::caller()),
        ));
    }
    Ok(Arc::new(ShaderProgram {
        kind,
        module: Arc::from(source),
        params: Vec::new(),
        params_size: 16,
        hash: fnv1a64(source),
    }))
}

/// FNV-1a over the source, the identity a program is cached by.
#[cfg(not(feature = "shader-validation"))]
fn fnv1a64(text: &str) -> u64 {
    text.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}
