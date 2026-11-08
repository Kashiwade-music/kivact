//! Role: The byte layout of the `Params` uniform block.

use kivact_types::{ParamSlot, ParamType};
use naga::proc::Layouter;
use naga::{Handle, Module, Scalar, ScalarKind, Type, TypeInner, VectorSize};

use crate::error::ShaderError;
use crate::source::Placement;

/// Checks that every member of the `Params` struct has an allowed type.
pub(crate) fn check_members(
    module: &Module,
    params: Handle<Type>,
    place: &Placement<'_>,
) -> Result<(), ShaderError> {
    let TypeInner::Struct { members, .. } = &module.types[params].inner else {
        return Err(place.internal("`Params` is not a struct"));
    };
    let offset = module
        .types
        .get_span(params)
        .to_range()
        .map(|range| range.start);
    for member in members {
        if param_type(module, member.ty).is_none() {
            return Err(place.error(
                offset,
                "a `Params` member must be f32, vec2<f32>, vec3<f32>, vec4<f32>, i32 or u32",
            ));
        }
    }
    Ok(())
}

/// The parameters of the `Params` struct with their offsets, and the size of
/// the uniform block in bytes (a multiple of 16). The offsets are the ones
/// naga gave the members, so `@align` and `@size` are honoured. The members
/// must already have passed `check_members`.
pub(crate) fn param_slots(
    module: &Module,
    params: Handle<Type>,
    place: &Placement<'_>,
) -> Result<(Vec<ParamSlot>, u32), ShaderError> {
    let TypeInner::Struct { members, .. } = &module.types[params].inner else {
        return Err(place.internal("`Params` is not a struct"));
    };
    let mut layouter = Layouter::default();
    layouter.update(module.to_ctx()).map_err(|error| {
        place.internal(format!("the layout of the shader types failed: {error}"))
    })?;
    let mut slots = Vec::with_capacity(members.len());
    for member in members {
        let ty = param_type(module, member.ty)
            .ok_or_else(|| place.internal("a `Params` member type was not checked"))?;
        slots.push(ParamSlot {
            name: member.name.clone().unwrap_or_default(),
            ty,
            offset: member.offset,
        });
    }
    let size = layouter[params].size.div_ceil(16) * 16;
    Ok((slots, size))
}

/// The parameter type of a `Params` member, if it is one that is allowed.
fn param_type(module: &Module, ty: Handle<Type>) -> Option<ParamType> {
    match module.types[ty].inner {
        TypeInner::Scalar(Scalar {
            kind: ScalarKind::Float,
            width: 4,
        }) => Some(ParamType::F32),
        TypeInner::Scalar(Scalar {
            kind: ScalarKind::Sint,
            width: 4,
        }) => Some(ParamType::I32),
        TypeInner::Scalar(Scalar {
            kind: ScalarKind::Uint,
            width: 4,
        }) => Some(ParamType::U32),
        TypeInner::Vector {
            size,
            scalar:
                Scalar {
                    kind: ScalarKind::Float,
                    width: 4,
                },
        } => match size {
            VectorSize::Bi => Some(ParamType::Vec2),
            VectorSize::Tri => Some(ParamType::Vec3),
            VectorSize::Quad => Some(ParamType::Vec4),
        },
        _ => None,
    }
}
