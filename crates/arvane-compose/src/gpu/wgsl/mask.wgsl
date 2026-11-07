
// Mask passes (design §8.9): each mask chunk's value, accumulated from the
// bottom layer upwards into an R32 accumulator.

struct MaskParams {
    mode: u32,
    composite: u32,
    init: u32,
    pad: u32,
}

@group(0) @binding(0) var<uniform> mp: MaskParams;
@group(0) @binding(1) var mask_tex: texture_2d<f32>;
@group(0) @binding(2) var acc_in: texture_2d<f32>;

@fragment
fn fs_mask_step(@builtin(position) frag: vec4<f32>) -> @location(0) vec4<f32> {
    let i = vec2<i32>(floor(frag.xy));
    let t = textureLoad(mask_tex, i, 0);
    var m = t.a;
    if (mp.mode == 1u) {
        m = dot(t.rgb, vec3<f32>(0.2125, 0.7154, 0.0721));
    }
    if (mp.init == 1u) {
        return vec4<f32>(m, 0.0, 0.0, 1.0);
    }
    let acc = textureLoad(acc_in, i, 0).r;
    var r = 0.0;
    switch mp.composite {
        case 0u: { r = m + acc * (1.0 - m); }
        case 1u: { r = m * (1.0 - acc); }
        case 2u: { r = m * acc; }
        default: { r = m * (1.0 - acc) + acc * (1.0 - m); }
    }
    return vec4<f32>(r, 0.0, 0.0, 1.0);
}
