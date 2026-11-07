//! Role: The fixed WGSL text that surrounds a user shader in an assembled module.

use kivact_types::ShaderKind;

/// The uniform context, the draw block and the first binding of every module.
const HEAD: &str = r#"struct Context { size: vec2<f32>, scale: f32, time: f32 }
struct KivactDraw {
    map: mat3x3<f32>,              // target px → shader space (columns)
    target_size: vec2<f32>,
    content_origin: vec2<f32>,     // filter: shader-space point at content texel (0, 0)
    content_scale: f32,            // filter: content texels per shader unit
    opacity: f32,
}
@group(0) @binding(0) var<uniform> kivact_context: Context;
"#;

/// The `Params` uniform binding, present only when the user declares `Params`.
const PARAMS_BINDING: &str = "@group(0) @binding(1) var<uniform> kivact_params: Params;\n";

/// The remaining bindings and the helper functions every module has.
const TAIL: &str = r#"@group(0) @binding(2) var kivact_sampler: sampler;
@group(0) @binding(3) var kivact_image0: texture_2d<f32>;
@group(0) @binding(4) var kivact_image1: texture_2d<f32>;
@group(0) @binding(5) var kivact_image2: texture_2d<f32>;
@group(0) @binding(6) var kivact_image3: texture_2d<f32>;
@group(0) @binding(7) var kivact_content: texture_2d<f32>;
@group(0) @binding(8) var kivact_coverage: texture_2d<f32>;
@group(0) @binding(9) var<uniform> kivact_draw: KivactDraw;
fn kivact_unpremul(c: vec4<f32>) -> vec4<f32> { if (c.a <= 0.0) { return vec4<f32>(0.0); } return vec4<f32>(c.rgb / c.a, c.a); }
fn image(slot: u32, uv: vec2<f32>) -> vec4<f32> {
    switch slot {
        case 0u: { return kivact_unpremul(textureSampleLevel(kivact_image0, kivact_sampler, uv, 0.0)); }
        case 1u: { return kivact_unpremul(textureSampleLevel(kivact_image1, kivact_sampler, uv, 0.0)); }
        case 2u: { return kivact_unpremul(textureSampleLevel(kivact_image2, kivact_sampler, uv, 0.0)); }
        default: { return kivact_unpremul(textureSampleLevel(kivact_image3, kivact_sampler, uv, 0.0)); }
    }
}
fn content(position: vec2<f32>) -> vec4<f32> {
    let texel = (position - kivact_draw.content_origin) * kivact_draw.content_scale;
    let size = vec2<f32>(textureDimensions(kivact_content));
    if (any(texel < vec2<f32>(0.0)) || any(texel > size)) { return vec4<f32>(0.0); }
    return kivact_unpremul(textureSampleLevel(kivact_content, kivact_sampler, texel / size, 0.0));
}
fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> { return select(pow((c + 0.055) / 1.055, vec3<f32>(2.4)), c / 12.92, c <= vec3<f32>(0.04045)); }
fn linear_to_srgb(c: vec3<f32>) -> vec3<f32> { return select(1.055 * pow(c, vec3<f32>(1.0 / 2.4)) - 0.055, c * 12.92, c <= vec3<f32>(0.0031308)); }
"#;

/// The generated vertex and fragment entry points. `$ENTRY` names the user's
/// function and `$ARGS` adds the `Params` argument when the function takes it.
const ENTRY_POINTS: &str = r#"struct KivactVertex { @builtin(position) clip: vec4<f32>, @location(0) target_px: vec2<f32> }
@vertex fn kivact_vertex(@location(0) target_px: vec2<f32>) -> KivactVertex {
    let ndc = target_px / kivact_draw.target_size * 2.0 - 1.0;
    return KivactVertex(vec4<f32>(ndc.x, -ndc.y, 0.0, 1.0), target_px);
}
@fragment fn kivact_fragment(v: KivactVertex) -> @location(0) vec4<f32> {
    let h = kivact_draw.map * vec3<f32>(v.target_px, 1.0);
    var c = clamp($ENTRY(h.xy / h.z, kivact_context$ARGS), vec4<f32>(0.0), vec4<f32>(1.0));
    let cover = textureSampleLevel(kivact_coverage, kivact_sampler, v.target_px / kivact_draw.target_size, 0.0).a;
    let a = c.a * cover * kivact_draw.opacity;
    return vec4<f32>(c.rgb * a, a);
}
"#;

/// The text placed before the user's source: the prelude, with the `Params`
/// binding only when the user declares `Params`.
pub(crate) fn prefix(params_declared: bool) -> String {
    let binding = if params_declared { PARAMS_BINDING } else { "" };
    format!("{HEAD}{binding}{TAIL}")
}

/// The name of the user function that the entry point calls for `kind`.
pub(crate) fn entry_name(kind: ShaderKind) -> &'static str {
    match kind {
        ShaderKind::Paint => "paint",
        ShaderKind::Filter => "effect",
    }
}

/// The generated entry points that call the user's function for `kind`.
pub(crate) fn entry_text(kind: ShaderKind, takes_params: bool) -> String {
    let args = if takes_params { ", kivact_params" } else { "" };
    ENTRY_POINTS
        .replace("$ENTRY", entry_name(kind))
        .replace("$ARGS", args)
}
