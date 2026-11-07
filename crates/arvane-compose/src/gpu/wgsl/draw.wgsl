
// One DrawOp: a source sampled through the map, clipped to the polygon's
// edges (anti-aliased outside split edges, top-left rule on them), scaled by
// opacity and the coverage chunk, then composited (design §8.7).

struct Params {
    m0: vec4<f32>,
    m1: vec4<f32>,
    m2: vec4<f32>,
    src_size: vec4<f32>,
    cov_size: vec4<f32>,
    bbox: vec4<f32>,
    target_size: vec4<f32>,
    opacity: f32,
    edge_count: u32,
    flags: u32,
    blend: u32,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var src: texture_2d<f32>;
@group(0) @binding(2) var dst: texture_2d<f32>;
@group(0) @binding(3) var cov: texture_2d<f32>;
@group(0) @binding(4) var<storage, read> edges: array<vec4<f32>>;

// Bounding box of the polygon, two triangles.
@vertex
fn vs_draw(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let b = params.bbox;
    let corners = array<vec2<f32>, 6>(
        vec2<f32>(b.x, b.y), vec2<f32>(b.z, b.y), vec2<f32>(b.x, b.w),
        vec2<f32>(b.x, b.w), vec2<f32>(b.z, b.y), vec2<f32>(b.z, b.w),
    );
    let p = corners[i];
    let t = params.target_size.xy;
    return vec4<f32>(p.x / t.x * 2.0 - 1.0, 1.0 - p.y / t.y * 2.0, 0.0, 1.0);
}

// The map from target px to source px, with the projective divide.
fn mapped(p: vec2<f32>) -> vec2<f32> {
    let x = params.m0.x * p.x + params.m0.y * p.y + params.m0.z;
    let y = params.m1.x * p.x + params.m1.y * p.y + params.m1.z;
    let w = params.m2.x * p.x + params.m2.y * p.y + params.m2.z;
    return vec2<f32>(x, y) / w;
}

// The source at source px `s`: nearest, or bilinear with texel centres at i + 0.5.
fn sample_src(s: vec2<f32>) -> vec4<f32> {
    if ((params.flags & 1u) != 0u) {
        return texel(src, params.src_size.xy, vec2<i32>(floor(s)));
    }
    return bilinear(src, params.src_size.xy, s - vec2<f32>(0.5));
}

// The anti-aliased coverage at pixel centre `p`; zero outside the polygon.
fn coverage_of(p: vec2<f32>) -> f32 {
    var best = 1.0e9;
    for (var k: u32 = 0u; k < params.edge_count; k = k + 1u) {
        let e = edges[k];
        let d = dot(e.xy, p) + e.z;
        if (e.w == 0.0) {
            best = min(best, d);
        } else if (!(d > 0.0 || (d == 0.0 && e.w == 1.0))) {
            return 0.0;
        }
    }
    return clamp(0.5 + best, 0.0, 1.0);
}

// The coverage chunk's alpha at `p` (target px).
fn chunk_alpha(p: vec2<f32>) -> f32 {
    return texel(cov, params.cov_size.xy, vec2<i32>(floor(p))).a;
}

// The premultiplied source colour scaled by opacity, coverage and the chunk.
fn source_at(p: vec2<f32>, cov_value: f32) -> vec4<f32> {
    var k = params.opacity * cov_value;
    if ((params.flags & 2u) != 0u) {
        k = k * chunk_alpha(p);
    }
    return sample_src(mapped(p)) * k;
}

@fragment
fn fs_draw(@builtin(position) frag: vec4<f32>) -> @location(0) vec4<f32> {
    let cov_value = coverage_of(frag.xy);
    if (cov_value <= 0.0) {
        discard;
    }
    return source_at(frag.xy, cov_value);
}

// A non-Normal blend or PlusLighter: the destination is a copy of the target.
@fragment
fn fs_blend(@builtin(position) frag: vec4<f32>) -> @location(0) vec4<f32> {
    let cov_value = coverage_of(frag.xy);
    if (cov_value <= 0.0) {
        discard;
    }
    let cs = source_at(frag.xy, cov_value);
    let cb = texel(dst, params.target_size.xy, vec2<i32>(floor(frag.xy)));
    if (params.blend == 255u) {
        return min(vec4<f32>(1.0), cs + cb);
    }
    return blend_over(cs, cb, params.blend);
}

// Clears the scissored rectangle to transparent.
@fragment
fn fs_clear(@builtin(position) frag: vec4<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0);
}

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
