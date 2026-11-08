// The W3C blend modes of the compositor, shared by the draw and graph shaders.

// W3C Compositing 1: co = cs(1 - ab) + cb(1 - as) + as·ab·B(Cb, Cs).
fn blend_over(cs: vec4<f32>, cb: vec4<f32>, mode: u32) -> vec4<f32> {
    let as_ = cs.a;
    let ab = cb.a;
    var big_cs = vec3<f32>(0.0);
    if (as_ > 0.0) {
        big_cs = cs.rgb / as_;
    }
    var big_cb = vec3<f32>(0.0);
    if (ab > 0.0) {
        big_cb = cb.rgb / ab;
    }
    let b = blend_fn(big_cb, big_cs, mode);
    let co = cs.rgb * (1.0 - ab) + cb.rgb * (1.0 - as_) + as_ * ab * b;
    return vec4<f32>(co, as_ + ab * (1.0 - as_));
}

fn screen(cb: vec3<f32>, cs: vec3<f32>) -> vec3<f32> {
    return cb + cs - cb * cs;
}

// HardLight(b, s): the backdrop b under the source s.
fn hard_light(b: vec3<f32>, s: vec3<f32>) -> vec3<f32> {
    let multiply = b * 2.0 * s;
    let other = screen(b, 2.0 * s - vec3<f32>(1.0));
    return select(other, multiply, s <= vec3<f32>(0.5));
}

fn soft_light(cb: vec3<f32>, cs: vec3<f32>) -> vec3<f32> {
    let d = select(sqrt(cb), ((16.0 * cb - vec3<f32>(12.0)) * cb + vec3<f32>(4.0)) * cb, cb <= vec3<f32>(0.25));
    let dark = cb - (vec3<f32>(1.0) - 2.0 * cs) * cb * (vec3<f32>(1.0) - cb);
    let light = cb + (2.0 * cs - vec3<f32>(1.0)) * (d - cb);
    return select(light, dark, cs <= vec3<f32>(0.5));
}

fn color_dodge(cb: vec3<f32>, cs: vec3<f32>) -> vec3<f32> {
    var r = min(vec3<f32>(1.0), cb / (vec3<f32>(1.0) - cs));
    r = select(r, vec3<f32>(1.0), cs >= vec3<f32>(1.0));
    return select(r, vec3<f32>(0.0), cb <= vec3<f32>(0.0));
}

fn color_burn(cb: vec3<f32>, cs: vec3<f32>) -> vec3<f32> {
    var r = vec3<f32>(1.0) - min(vec3<f32>(1.0), (vec3<f32>(1.0) - cb) / cs);
    r = select(r, vec3<f32>(0.0), cs <= vec3<f32>(0.0));
    return select(r, vec3<f32>(1.0), cb >= vec3<f32>(1.0));
}

fn lum(c: vec3<f32>) -> f32 {
    return 0.3 * c.r + 0.59 * c.g + 0.11 * c.b;
}

fn clip_color(c: vec3<f32>) -> vec3<f32> {
    let l = lum(c);
    let n = min(c.r, min(c.g, c.b));
    let x = max(c.r, max(c.g, c.b));
    var r = c;
    if (n < 0.0) {
        r = l + (r - vec3<f32>(l)) * l / (l - n);
    }
    if (x > 1.0) {
        r = l + (r - vec3<f32>(l)) * (1.0 - l) / (x - l);
    }
    return r;
}

fn set_lum(c: vec3<f32>, l: f32) -> vec3<f32> {
    return clip_color(c + vec3<f32>(l - lum(c)));
}

fn sat(c: vec3<f32>) -> f32 {
    return max(c.r, max(c.g, c.b)) - min(c.r, min(c.g, c.b));
}

fn set_sat(c: vec3<f32>, s: f32) -> vec3<f32> {
    let mx = max(c.r, max(c.g, c.b));
    let mn = min(c.r, min(c.g, c.b));
    if (mx > mn) {
        return (c - vec3<f32>(mn)) * s / (mx - mn);
    }
    return vec3<f32>(0.0);
}

// The separable and non-separable blend functions, by BlendMode index.
fn blend_fn(cb: vec3<f32>, cs: vec3<f32>, mode: u32) -> vec3<f32> {
    switch mode {
        case 1u: { return cb * cs; }
        case 2u: { return screen(cb, cs); }
        case 3u: { return hard_light(cs, cb); }
        case 4u: { return min(cb, cs); }
        case 5u: { return max(cb, cs); }
        case 6u: { return color_dodge(cb, cs); }
        case 7u: { return color_burn(cb, cs); }
        case 8u: { return hard_light(cb, cs); }
        case 9u: { return soft_light(cb, cs); }
        case 10u: { return abs(cb - cs); }
        case 11u: { return cb + cs - 2.0 * cb * cs; }
        case 12u: { return set_lum(set_sat(cs, sat(cb)), lum(cb)); }
        case 13u: { return set_lum(set_sat(cb, sat(cs)), lum(cb)); }
        case 14u: { return set_lum(cs, lum(cb)); }
        case 15u: { return set_lum(cb, lum(cs)); }
        default: { return cs; }
    }
}
