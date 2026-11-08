//! Role: The entry point and `Params` declaration read from a user source.
//!
//! Read from the first parse of the source.

use kivact_types::ShaderKind;
use naga::{Function, Handle, Module, Scalar, ScalarKind, Type, TypeInner, VectorSize};

use crate::error::ShaderError;
use crate::layout::check_members;
use crate::prelude::entry_name;
use crate::source::Placement;

/// What the first parse of a user source declares.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Interface {
    // xtask-allow(state-fields): Independent fact read from the declarations; the entry signature is reported by `takes_params` separately.
    /// Whether the user declares a `Params` struct.
    pub(crate) params_declared: bool,
    // xtask-allow(state-fields): Independent fact read from the entry signature; the struct declaration is reported by `params_declared` separately.
    /// Whether the entry function takes `Params` as its third argument.
    pub(crate) takes_params: bool,
}

/// Checks the entry function of `kind` and reports what the source declares.
pub(crate) fn read_interface(
    module: &Module,
    kind: ShaderKind,
    place: &Placement<'_>,
) -> Result<Interface, ShaderError> {
    let name = entry_name(kind);
    let params_declared = match find_struct(module, "Params") {
        Some(params) => {
            check_members(module, params, place)?;
            true
        }
        None => false,
    };
    let Some((handle, function)) = find_function(module, name) else {
        return Err(place.error(None, format!("the shader must define `fn {name}`")));
    };
    let offset = module
        .functions
        .get_span(handle)
        .to_range()
        .map(|range| range.start);
    let takes_params = check_arguments(module, function, name, params_declared, place, offset)?;
    let returns_color = function
        .result
        .as_ref()
        .is_some_and(|result| is_vec4_f32(module, result.ty));
    if !returns_color {
        return Err(place.error(offset, format!("`fn {name}` must return vec4<f32>")));
    }
    Ok(Interface {
        params_declared,
        takes_params,
    })
}

/// Checks the argument types of the entry function and reports whether it
/// takes the `Params` argument.
fn check_arguments(
    module: &Module,
    function: &Function,
    name: &str,
    params_declared: bool,
    place: &Placement<'_>,
    offset: Option<usize>,
) -> Result<bool, ShaderError> {
    let (position, context, params) = match function.arguments.as_slice() {
        [position, context] => (position, context, None),
        [position, context, params] => (position, context, Some(params)),
        other => {
            return Err(place.error(
                offset,
                format!("`fn {name}` takes 2 or 3 arguments, found {}", other.len()),
            ));
        }
    };
    if !is_vec2_f32(module, position.ty) {
        return Err(place.error(
            offset,
            format!("the first argument of `fn {name}` must be vec2<f32>"),
        ));
    }
    if !is_named_struct(module, context.ty, "Context") {
        return Err(place.error(
            offset,
            format!("the second argument of `fn {name}` must be `Context`"),
        ));
    }
    let Some(params) = params else {
        return Ok(false);
    };
    if !params_declared || !is_named_struct(module, params.ty, "Params") {
        return Err(place.error(
            offset,
            format!(
                "the third argument of `fn {name}` must be `Params`, which the shader must declare"
            ),
        ));
    }
    Ok(true)
}

/// The function of the given name, with its handle.
fn find_function<'m>(module: &'m Module, name: &str) -> Option<(Handle<Function>, &'m Function)> {
    module
        .functions
        .iter()
        .find(|(_, function)| function.name.as_deref() == Some(name))
}

/// The struct of the given name, if the module declares one.
pub(crate) fn find_struct(module: &Module, name: &str) -> Option<Handle<Type>> {
    module
        .types
        .iter()
        .find(|(handle, _)| is_named_struct(module, *handle, name))
        .map(|(handle, _)| handle)
}

fn is_named_struct(module: &Module, ty: Handle<Type>, name: &str) -> bool {
    let ty = &module.types[ty];
    ty.name.as_deref() == Some(name) && matches!(ty.inner, TypeInner::Struct { .. })
}

fn is_vec2_f32(module: &Module, ty: Handle<Type>) -> bool {
    matches!(
        module.types[ty].inner,
        TypeInner::Vector {
            size: VectorSize::Bi,
            scalar: Scalar {
                kind: ScalarKind::Float,
                width: 4
            }
        }
    )
}

fn is_vec4_f32(module: &Module, ty: Handle<Type>) -> bool {
    matches!(
        module.types[ty].inner,
        TypeInner::Vector {
            size: VectorSize::Quad,
            scalar: Scalar {
                kind: ScalarKind::Float,
                width: 4
            }
        }
    )
}
