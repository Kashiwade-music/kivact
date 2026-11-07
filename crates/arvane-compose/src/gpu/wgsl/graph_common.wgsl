// The filter graph primitives (design §8.8.4). One render pass computes one
// primitive: `mode` selects the operation, `a` and `b` are its input textures
// (premultiplied RGBA8, the region's size), and the tables hold the
// conversions and the transfer, kernel and turbulence data the CPU built.
//
// The byte layout of GraphParams is shared with gpu/graph.rs (144 bytes).

struct GraphParams {
    // xy: the output size in px.
    out_size: vec4<f32>,
    // xy: the size of texture a; zw unused.
    a_size: vec4<f32>,
    // xy: the size of texture b; zw unused.
    b_size: vec4<f32>,
    // Real parameters; their meaning is per mode (see graph_ops.wgsl).
    p0: vec4<f32>,
    p1: vec4<f32>,
    p2: vec4<f32>,
    p3: vec4<f32>,
    // ints.x: the mode; ints.yzw: integer parameters.
    ints: vec4<u32>,
    // ints2: further integer parameters.
    ints2: vec4<u32>,
}

// Offsets into `tables` (f32 values), shared with gpu/graph.rs.
const LUT_TO_LINEAR: u32 = 0u;
const LUT_TO_SRGB: u32 = 256u;
const KERNEL: u32 = 512u;
// Eight values per channel (r, g, b, a): kind, p1, p2, p3, count, table offset, 0, 0.
const TRANSFER_INFO: u32 = 768u;
const TRANSFER_TABLE: u32 = 800u;

// Lattice and gradient sizes of the turbulence tables (the CPU's B_LEN).
const TURB_LATTICE: u32 = 514u;
const TURB_PERLIN_N: f32 = 4096.0;

@group(0) @binding(0) var<uniform> gp: GraphParams;
@group(0) @binding(1) var ga: texture_2d<f32>;
@group(0) @binding(2) var gb: texture_2d<f32>;
@group(0) @binding(3) var<storage, read> tables: array<f32>;
@group(0) @binding(4) var<storage, read> turb_lattice: array<u32>;
@group(0) @binding(5) var<storage, read> turb_gradient: array<vec2<f32>>;

// The 8-bit value of a channel in 0..=255 (textures hold exact multiples of 1/255).
fn byte_of(v: f32) -> f32 {
    return round(v * 255.0);
}

// The 8-bit values of the colour channels.
fn byte_rgb(v: vec3<f32>) -> vec3<f32> {
    return round(v * 255.0);
}
