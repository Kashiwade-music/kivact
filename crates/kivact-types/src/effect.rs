//! Role: Effect values of a scene: SVG filter graphs and validated shader programs.
mod filter_graph;
mod shader;

pub use filter_graph::{
    Channel, CompositeOp, EdgeMode, FilterColorSpace, FilterGraph, FilterInput, Light,
    MorphologyOp, Primitive, PrimitiveOp, TransferFn, TurbulenceKind,
};
pub use shader::{ParamSlot, ParamType, ShaderKind, ShaderProgram, ShaderUse};
