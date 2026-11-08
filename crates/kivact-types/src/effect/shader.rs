//! Role: Validated shader programs and their uses as plain values.
use crate::pixel::Pixels;

/// Whether a shader paints a fill or filters a layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ShaderKind {
    /// A paint shader (background, mask or fill).
    Paint,
    /// A filter shader over a layer.
    Filter,
}

/// A complete WGSL module assembled by kivact-shader.
#[derive(Clone, Debug, PartialEq)]
pub struct ShaderProgram {
    /// Paint or filter.
    pub kind: ShaderKind,
    /// The WGSL source of the whole module.
    // xtask-allow(api-shape): Shared immutable payload, cloned without copying its data; one allocation is shared by all clones.
    pub module: std::sync::Arc<str>,
    /// The parameters, in the order of the uniform block.
    pub params: Vec<ParamSlot>,
    /// Size of the params uniform buffer in bytes (a multiple of 16; 16 when there are no params).
    pub params_size: u32,
    /// FNV-1a 64 of `module` (pipeline cache key).
    pub hash: u64,
}

/// One named parameter of a shader program and where it sits in the uniform block.
#[derive(Clone, Debug, PartialEq)]
pub struct ParamSlot {
    /// The parameter's name in the WGSL source.
    pub name: String,
    /// The parameter's type.
    pub ty: ParamType,
    /// Byte offset in the params uniform block.
    pub offset: u32,
}

/// The type of a shader parameter.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ParamType {
    /// A 32-bit float.
    F32,
    /// Two floats.
    Vec2,
    /// Three floats.
    Vec3,
    /// Four floats.
    Vec4,
    /// A signed 32-bit integer.
    I32,
    /// An unsigned 32-bit integer.
    U32,
}

/// A program with its parameter bytes, images and clock for one draw.
#[derive(Clone, Debug, PartialEq)]
pub struct ShaderUse {
    /// The program to run.
    // xtask-allow(api-shape): Shared immutable payload, cloned without copying its data; one allocation is shared by all clones.
    pub program: std::sync::Arc<ShaderProgram>,
    /// Exactly `params_size` bytes, little endian, laid out by `params`.
    // xtask-allow(api-shape): Shared immutable payload, cloned without copying its data; one allocation is shared by all clones.
    pub params: std::sync::Arc<[u8]>,
    /// Slots 0..=3 of the WGSL `image(slot, uv)` helper; None binds a 1×1 transparent texture.
    pub images: [Option<Pixels>; 4],
    /// Seconds of the shader clock.
    pub time: f64,
    /// Filter shaders: px the output may extend beyond the layer bounds.
    pub margin: f64,
}
