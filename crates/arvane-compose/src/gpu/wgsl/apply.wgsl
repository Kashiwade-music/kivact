
// Full-target passes that take a value from a texture and write it: the
// multiplications of masks and clips (blended as dst * src), the copy of
// the output onto the swapchain (over white), and the clear of a rectangle.

@group(0) @binding(0) var apply_tex: texture_2d<f32>;

@fragment
fn fs_apply_acc(@builtin(position) frag: vec4<f32>) -> @location(0) vec4<f32> {
    let i = vec2<i32>(floor(frag.xy));
    let size = vec2<f32>(textureDimensions(apply_tex));
    let a = texel(apply_tex, size, i).r;
    return vec4<f32>(a);
}

@fragment
fn fs_apply_alpha(@builtin(position) frag: vec4<f32>) -> @location(0) vec4<f32> {
    let i = vec2<i32>(floor(frag.xy));
    let size = vec2<f32>(textureDimensions(apply_tex));
    let a = texel(apply_tex, size, i).a;
    return vec4<f32>(a);
}

@fragment
fn fs_present(@builtin(position) frag: vec4<f32>) -> @location(0) vec4<f32> {
    let i = vec2<i32>(floor(frag.xy));
    let size = vec2<f32>(textureDimensions(apply_tex));
    return texel(apply_tex, size, i);
}
