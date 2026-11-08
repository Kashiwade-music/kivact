// Shared helpers of the compositor's shaders. Every texture is premultiplied
// RGBA; reads outside a texture give zero (the CPU executor's rule).

// Fullscreen triangle covering the viewport.
@vertex
fn vs_full(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let x = f32((i << 1u) & 2u) * 2.0 - 1.0;
    let y = f32(i & 2u) * 2.0 - 1.0;
    return vec4<f32>(x, y, 0.0, 1.0);
}

// The texel at integer coordinates `i`, zero outside `size`.
fn texel(t: texture_2d<f32>, size: vec2<f32>, i: vec2<i32>) -> vec4<f32> {
    if (i.x < 0 || i.y < 0 || f32(i.x) >= size.x || f32(i.y) >= size.y) {
        return vec4<f32>(0.0);
    }
    return textureLoad(t, i, 0);
}

// Bilinear reads at index-space coordinates `idx` (texel i is centred on i).
fn bilinear(t: texture_2d<f32>, size: vec2<f32>, idx: vec2<f32>) -> vec4<f32> {
    let i0 = floor(idx);
    let f = idx - i0;
    let b = vec2<i32>(i0);
    let top = mix(texel(t, size, b), texel(t, size, b + vec2<i32>(1, 0)), f.x);
    let bottom = mix(texel(t, size, b + vec2<i32>(0, 1)), texel(t, size, b + vec2<i32>(1, 1)), f.x);
    return mix(top, bottom, f.y);
}
