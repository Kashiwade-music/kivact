// The primitives of a filter graph, one mode each. The values are
// premultiplied; a conversion or a table read follows resvg's 8-bit arithmetic.

const M_COPY: u32 = 0u;
const M_SOURCE_ALPHA: u32 = 1u;
const M_CONVERT: u32 = 2u;
const M_BLEND: u32 = 3u;
const M_COMPOSITE: u32 = 4u;
const M_ARITHMETIC: u32 = 5u;
const M_FLOOD: u32 = 6u;
const M_IMAGE: u32 = 7u;
const M_OFFSET: u32 = 8u;
const M_TILE: u32 = 9u;
const M_TRANSFER: u32 = 10u;
const M_DEMULTIPLY: u32 = 11u;
const M_CONVOLVE: u32 = 12u;
const M_LIGHT: u32 = 13u;
const M_DISPLACE: u32 = 14u;
const M_MORPH: u32 = 15u;
const M_TURBULENCE: u32 = 16u;
const M_CLEAR: u32 = 17u;

// The sRGB or linearRGB conversion of a premultiplied texel: demultiply, the table, premultiply.
// `offset` selects the table (LUT_TO_LINEAR or LUT_TO_SRGB).
fn convert_texel(t: vec4<f32>, offset: u32) -> vec4<f32> {
    let a8 = byte_of(t.a);
    if (a8 <= 0.0) {
        return vec4<f32>(0.0);
    }
    let c8 = byte_rgb(t.rgb);
    let s = clamp(floor(c8 * 255.0 / a8 + 0.5), vec3<f32>(0.0), vec3<f32>(255.0));
    let l = vec3<f32>(tables[offset + u32(s.r)], tables[offset + u32(s.g)], tables[offset + u32(s.b)]);
    let p = floor(l * a8 / 255.0 + 0.5);
    return vec4<f32>(p / 255.0, a8 / 255.0);
}

// The straight colour of a premultiplied texel, rounded to bytes (resvg's demultiply).
fn demultiply_texel(t: vec4<f32>) -> vec4<f32> {
    let a8 = byte_of(t.a);
    if (a8 <= 0.0) {
        return vec4<f32>(0.0, 0.0, 0.0, t.a);
    }
    let c8 = byte_rgb(t.rgb);
    let s = clamp(floor(c8 * 255.0 / a8 + 0.5), vec3<f32>(0.0), vec3<f32>(255.0));
    return vec4<f32>(s / 255.0, t.a);
}

// Channel record at `base` through its transfer function, before the clamp.
fn transfer_raw(base: u32, v: f32) -> f32 {
    let kind = u32(tables[base]);
    let p1 = tables[base + 1u];
    let p2 = tables[base + 2u];
    let p3 = tables[base + 3u];
    let count = u32(tables[base + 4u]);
    let off = u32(tables[base + 5u]);
    if (kind == 1u) {
        // Table: piecewise linear over count values.
        let n = f32(count) - 1.0;
        let k = clamp(floor(v * n), 0.0, n);
        let vk = tables[off + u32(k)];
        if (k == n) {
            return vk;
        }
        let vk1 = tables[off + u32(k) + 1u];
        return vk + (v - k / n) * n * (vk1 - vk);
    }
    if (kind == 2u) {
        // Discrete: stepped over count values.
        let n = f32(count);
        let k = clamp(floor(v * n), 0.0, n - 1.0);
        return tables[off + u32(k)];
    }
    if (kind == 3u) {
        return p1 * v + p2;
    }
    return p1 * pow(max(v, 0.0), p2) + p3;
}

// Channel `ch` (0 r, 1 g, 2 b, 3 a) through its transfer function: identity, or clamped.
fn transfer_value(ch: u32, v: f32) -> f32 {
    let base = TRANSFER_INFO + ch * 8u;
    let kind = u32(tables[base]);
    let count = u32(tables[base + 4u]);
    if (kind == 0u || ((kind == 1u || kind == 2u) && count == 0u)) {
        return v;
    }
    return clamp(transfer_raw(base, v), 0.0, 1.0);
}

// feComposite's Porter-Duff operator with `src` over `dst` (ints.y: over, in, out, atop, xor, lighter).
fn composite_texels(src: vec4<f32>, dst: vec4<f32>, op: u32) -> vec4<f32> {
    var fa = 1.0;
    var fb = 1.0 - src.a;
    switch op {
        case 1u: {
            fa = dst.a;
            fb = 0.0;
        }
        case 2u: {
            fa = 1.0 - dst.a;
            fb = 0.0;
        }
        case 3u: {
            fa = dst.a;
            fb = 1.0 - src.a;
        }
        case 4u: {
            fa = 1.0 - dst.a;
            fb = 1.0 - src.a;
        }
        case 5u: {
            fa = 1.0;
            fb = 1.0;
        }
        default: {}
    }
    let out = fa * src + fb * dst;
    if (op == 5u) {
        return min(out, vec4<f32>(1.0));
    }
    return out;
}

// feComposite's arithmetic operator: k1·i1·i2 + k2·i1 + k3·i2 + k4, the colour bounded by the alpha.
fn arithmetic_texels(src: vec4<f32>, dst: vec4<f32>) -> vec4<f32> {
    let k = gp.p0;
    let a = clamp(k.x * src.a * dst.a + k.y * src.a + k.z * dst.a + k.w, 0.0, 1.0);
    if (a <= 1.1920929e-7) {
        return vec4<f32>(0.0);
    }
    let i1 = src.rgb;
    let i2 = dst.rgb;
    let c = clamp(k.x * i1 * i2 + k.y * i1 + k.z * i2 + vec3<f32>(k.w), vec3<f32>(0.0), vec3<f32>(a));
    return vec4<f32>(c, a);
}

// feConvolveMatrix at (x, y) of the input (straight colour when preserve_alpha, else premultiplied).
fn convolve_at(x: vec2<i32>) -> vec4<f32> {
    let cols = i32(gp.ints.y);
    let rows = i32(gp.ints.z);
    let target_x = i32(gp.ints.w);
    let target_y = i32(gp.ints2.x);
    let edge = gp.ints2.y;
    let preserve = gp.ints2.z != 0u;
    let w = i32(gp.out_size.x);
    let h = i32(gp.out_size.y);
    let divisor = gp.p0.x;
    let bias = gp.p0.y;
    var acc = vec4<f32>(0.0);
    for (var oy: i32 = 0; oy < rows; oy = oy + 1) {
        for (var ox: i32 = 0; ox < cols; ox = ox + 1) {
            var tx = x.x - target_x + ox;
            var ty = x.y - target_y + oy;
            if (edge == 0u) {
                if (tx < 0 || tx >= w || ty < 0 || ty >= h) {
                    continue;
                }
            } else if (edge == 1u) {
                tx = clamp(tx, 0, w - 1);
                ty = clamp(ty, 0, h - 1);
            } else {
                tx = ((tx % w) + w) % w;
                ty = ((ty % h) + h) % h;
            }
            let k = tables[KERNEL + u32((rows - oy - 1) * cols + (cols - ox - 1))];
            let p = texel(ga, gp.a_size.xy, vec2<i32>(tx, ty));
            acc = vec4<f32>(acc.rgb + p.rgb * k, acc.a + select(p.a * k, 0.0, preserve));
        }
    }
    let in_alpha = texel(ga, gp.a_size.xy, x).a;
    var new_a = acc.a / divisor + bias;
    if (preserve) {
        new_a = in_alpha;
    }
    let bounded_a = clamp(new_a, 0.0, 1.0);
    let rgb = vec3<f32>(
        conv_channel(acc.r, divisor, bias, new_a, bounded_a, preserve),
        conv_channel(acc.g, divisor, bias, new_a, bounded_a, preserve),
        conv_channel(acc.b, divisor, bias, new_a, bounded_a, preserve),
    );
    return vec4<f32>(rgb, bounded_a);
}

// One colour channel of the convolution: bounded by the new alpha (premultiplied) or by one (preserved alpha).
fn conv_channel(sum: f32, divisor: f32, bias: f32, new_a: f32, bounded_a: f32, preserve: bool) -> f32 {
    let v = sum / divisor + bias * new_a;
    if (preserve) {
        return clamp(v, 0.0, 1.0) * bounded_a;
    }
    return clamp(v, 0.0, bounded_a);
}

// Round half away from zero, as Rust's f32::round.
fn round_away(v: f32) -> f32 {
    return select(-floor(-v + 0.5), floor(v + 0.5), v >= 0.0);
}

// feDisplacementMap at (x, y): the source pixel that the map moves this pixel to.
fn displace_at(x: vec2<i32>) -> vec4<f32> {
    let m = texel(gb, gp.b_size.xy, x);
    let dx = m[gp.ints.y] - 0.5;
    let dy = m[gp.ints.z] - 0.5;
    let ox = round_away(f32(x.x) + dx * gp.p0.x);
    let oy = round_away(f32(x.y) + dy * gp.p0.y);
    return texel(ga, gp.a_size.xy, vec2<i32>(i32(ox), i32(oy)));
}

// feMorphology at (x, y): the minimum (erode) or maximum (dilate) over the window of the radii p0.
fn morph_at(x: vec2<i32>) -> vec4<f32> {
    let w = gp.out_size.x;
    let h = gp.out_size.y;
    let cols = min(2.0 * ceil(gp.p0.x), w);
    let rows = min(2.0 * ceil(gp.p0.y), h);
    let target_x = floor(cols / 2.0);
    let target_y = floor(rows / 2.0);
    let erode = gp.ints.y == 0u;
    var best = vec4<f32>(0.0);
    if (erode) {
        best = vec4<f32>(1.0);
    }
    for (var oy: i32 = 0; oy < i32(rows); oy = oy + 1) {
        for (var ox: i32 = 0; ox < i32(cols); ox = ox + 1) {
            let tx = x.x - i32(target_x) + ox;
            let ty = x.y - i32(target_y) + oy;
            if (tx < 0 || tx >= i32(w) || ty < 0 || ty >= i32(h)) {
                continue;
            }
            let p = texel(ga, gp.a_size.xy, vec2<i32>(tx, ty));
            if (erode) {
                best = min(best, p);
            } else {
                best = max(best, p);
            }
        }
    }
    return best;
}

// feOffset at (x, y): the source moved by the whole pixels p0.
fn offset_at(x: vec2<i32>) -> vec4<f32> {
    let d = vec2<i32>(i32(gp.p0.x), i32(gp.p0.y));
    return texel(ga, gp.a_size.xy, x - d);
}

// feTile at (x, y): the tile rectangle p0 (px) repeated, sampled at the nearest texel.
fn tile_at(xf: vec2<f32>) -> vec4<f32> {
    let tw = gp.p0.z;
    let th = gp.p0.w;
    if (tw <= 0.0 || th <= 0.0) {
        return vec4<f32>(0.0);
    }
    let origin = gp.p0.xy;
    let rel = xf + vec2<f32>(0.5) - origin;
    let uv = vec2<f32>(rel.x - tw * floor(rel.x / tw), rel.y - th * floor(rel.y / th));
    return texel(ga, gp.a_size.xy, vec2<i32>(floor(origin + uv)));
}

// feImage at (x, y): the image (texture b) stretched over the rectangle p0 (px), bilinear.
fn image_at(xf: vec2<f32>) -> vec4<f32> {
    let rx = gp.p0.x;
    let ry = gp.p0.y;
    let rw = gp.p0.z;
    let rh = gp.p0.w;
    if (rw <= 0.0 || rh <= 0.0) {
        return vec4<f32>(0.0);
    }
    let p = xf + vec2<f32>(0.5);
    if (p.x < rx || p.y < ry || p.x >= rx + rw || p.y >= ry + rh) {
        return vec4<f32>(0.0);
    }
    let uv = vec2<f32>((p.x - rx) / rw * gp.p1.x, (p.y - ry) / rh * gp.p1.y);
    return bilinear(gb, gp.b_size.xy, uv - vec2<f32>(0.5));
}

@fragment
fn fs_graph(@builtin(position) frag: vec4<f32>) -> @location(0) vec4<f32> {
    let x = vec2<i32>(floor(frag.xy));
    let xf = vec2<f32>(x);
    let src = texel(ga, gp.a_size.xy, x);
    let dst = texel(gb, gp.b_size.xy, x);
    switch gp.ints.x {
        case 0u: {
            return src;
        }
        case 1u: {
            return vec4<f32>(0.0, 0.0, 0.0, src.a);
        }
        case 2u: {
            return convert_texel(src, gp.ints.y);
        }
        case 3u: {
            if (gp.ints.y == 255u) {
                return min(vec4<f32>(1.0), src + dst);
            }
            return blend_over(src, dst, gp.ints.y);
        }
        case 4u: {
            return composite_texels(src, dst, gp.ints.y);
        }
        case 5u: {
            return arithmetic_texels(src, dst);
        }
        case 6u: {
            return gp.p0;
        }
        case 7u: {
            return image_at(xf);
        }
        case 8u: {
            return offset_at(x);
        }
        case 9u: {
            return tile_at(xf);
        }
        case 10u: {
            var straight = vec3<f32>(0.0);
            if (src.a > 0.0) {
                straight = src.rgb / src.a;
            }
            let rgb = vec3<f32>(
                transfer_value(0u, straight.r),
                transfer_value(1u, straight.g),
                transfer_value(2u, straight.b),
            );
            let alpha = transfer_value(3u, src.a);
            return vec4<f32>(rgb * alpha, alpha);
        }
        case 11u: {
            return demultiply_texel(src);
        }
        case 12u: {
            return convolve_at(x);
        }
        case 13u: {
            return light_at(x.x, x.y);
        }
        case 14u: {
            return displace_at(x);
        }
        case 15u: {
            return morph_at(x);
        }
        case 16u: {
            return turb_pixel(xf.x, xf.y);
        }
        default: {
            return vec4<f32>(0.0);
        }
    }
}
