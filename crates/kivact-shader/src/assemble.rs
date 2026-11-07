//! Role: Assembles the complete shader module of a user source in two parses.

use std::sync::Arc;

use kivact_types::{ShaderKind, ShaderProgram};
use naga::Module;
use naga::valid::{Capabilities, ValidationFlags, Validator};

use crate::entry::{find_struct, read_interface};
use crate::error::ShaderError;
use crate::hash::fnv1a64;
use crate::layout::param_slots;
use crate::prelude::{entry_text, prefix};
use crate::source::{Placement, check_source};

/// Checks a user shader and assembles its complete WGSL module.
///
/// `kind` selects the entry: a paint shader defines `fn paint` and a filter
/// shader defines `fn effect`. The function takes `(position: vec2<f32>,
/// context: Context)`, optionally with a third `params: Params` argument, and
/// returns `vec4<f32>`. Errors carry the line and column in `source`.
///
/// # Examples
///
/// A paint shader with a `Params` uniform:
///
/// ```
/// use kivact_shader::assemble;
/// use kivact_types::ShaderKind;
///
/// let source = r#"
/// struct Params { speed: f32, tint: vec4<f32> }
///
/// fn paint(position: vec2<f32>, context: Context, params: Params) -> vec4<f32> {
///     let uv = position / context.size;
///     let wave = 0.5 + 0.5 * sin(uv.x * 12.0 + context.time * params.speed);
///     return vec4<f32>(params.tint.rgb * wave, 1.0);
/// }
/// "#;
/// let program = assemble(source, ShaderKind::Paint).expect("a valid paint shader");
/// assert_eq!(program.params_size, 32);
/// assert_eq!(program.params[1].offset, 16);
/// ```
///
/// A filter shader that reads the element's own pixels with `content`:
///
/// ```
/// use kivact_shader::assemble;
/// use kivact_types::ShaderKind;
///
/// let source = r#"
/// fn effect(position: vec2<f32>, context: Context) -> vec4<f32> {
///     let c = content(position);
///     let scan = 0.85 + 0.15 * step(0.5, fract(position.y * 0.5));
///     return vec4<f32>(c.rgb * scan, c.a);
/// }
/// "#;
/// let program = assemble(source, ShaderKind::Filter).expect("a valid filter shader");
/// assert!(program.params.is_empty());
/// assert_eq!(program.params_size, 16);
/// ```
///
/// A syntax error is reported at its line and column:
///
/// ```
/// use kivact_shader::assemble;
/// use kivact_types::ShaderKind;
///
/// let source = "fn paint(position: vec2<f32>, context: Context) -> vec4<f32> {\n    return vec4<f32>(1.0, 1.0;\n}\n";
/// let error = assemble(source, ShaderKind::Paint).unwrap_err();
/// assert_eq!((error.line, error.column), (2, 30));
/// ```
///
/// `filter` is a WGSL reserved word, so a filter shader cannot name its entry
/// `fn filter`. The error points at the name in the user's source:
///
/// ```
/// use kivact_shader::assemble;
/// use kivact_types::ShaderKind;
///
/// let source = "fn filter(position: vec2<f32>, context: Context) -> vec4<f32> {\n    return vec4<f32>(0.0);\n}\n";
/// let error = assemble(source, ShaderKind::Filter).unwrap_err();
/// assert_eq!((error.line, error.column), (1, 4));
/// ```
pub fn assemble(source: &str, kind: ShaderKind) -> Result<ShaderProgram, ShaderError> {
    check_source(source)?;
    let probe_prefix = prefix(false);
    let probe_place = Placement::new(source, probe_prefix.len());
    let probe = parse(&format!("{probe_prefix}{source}"), &probe_place)?;
    let interface = read_interface(&probe, kind, &probe_place)?;

    let module_prefix = prefix(interface.params_declared);
    let text = format!(
        "{module_prefix}{source}\n{}",
        entry_text(kind, interface.takes_params)
    );
    let place = Placement::new(source, module_prefix.len());
    let module = validate(&text, &place)?;
    check_generated(&module, &place)?;

    let (params, params_size) = match find_struct(&module, "Params") {
        Some(handle) => param_slots(&module, handle, &place)?,
        None => (Vec::new(), 16),
    };
    Ok(ShaderProgram {
        kind,
        module: Arc::from(text.as_str()),
        params,
        params_size,
        hash: fnv1a64(&text),
    })
}

/// Parses `text`, mapping a failure to a position in the user's source.
fn parse(text: &str, place: &Placement<'_>) -> Result<Module, ShaderError> {
    naga::front::wgsl::parse_str(text).map_err(|error| {
        place.error(
            error
                .location(text)
                .map(|location| location.offset as usize),
            error.to_string(),
        )
    })
}

/// Parses and validates the complete module.
fn validate(text: &str, place: &Placement<'_>) -> Result<Module, ShaderError> {
    let module = parse(text, place)?;
    let mut validator = Validator::new(ValidationFlags::all(), Capabilities::empty());
    validator.validate(&module).map_err(|error| {
        place.error(
            error
                .location(text)
                .map(|location| location.offset as usize),
            error.into_inner().to_string(),
        )
    })?;
    Ok(module)
}

/// Checks what the generated module must contain: exactly the two generated
/// entry points, no overrides, and resource bindings only from the prelude.
fn check_generated(module: &Module, place: &Placement<'_>) -> Result<(), ShaderError> {
    if module.entry_points.len() != 2 {
        return Err(place.internal("the module must have exactly two entry points"));
    }
    if !module.overrides.is_empty() {
        return Err(place.internal("the module must not declare overrides"));
    }
    for (_, global) in module.global_variables.iter() {
        let from_prelude = global
            .name
            .as_deref()
            .is_some_and(|name| name.starts_with("kivact_"));
        if global.binding.is_some() && !from_prelude {
            return Err(place.internal("a user global must not bind a resource"));
        }
    }
    Ok(())
}
