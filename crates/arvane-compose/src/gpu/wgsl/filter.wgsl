
// Filter passes (design §8.8): resampling for the blur, the blur itself,
// the drop shadow's colour and composite, and the colour matrix.

struct FilterParams {
    out_size: vec4<f32>,
    src_size: vec4<f32>,
    dir: vec4<f32>,
    color: vec4<f32>,
    offset: vec4<f32>,
    radius: u32,
    mode: u32,
    pad_a: u32,
    pad_b: u32,
    matrix: array<vec4<f32>, 5>,
}

@group(0) @binding(0) var<uniform> fp: FilterParams;
@group(0) @binding(1) var fsrc: texture_2d<f32>;
@group(0) @binding(2) var faux: texture_2d<f32>;
@group(0) @binding(3) var<storage, read> weights: array<f32>;

@fragment
fn fs_filter(@builtin(position) frag: vec4<f32>) -> @location(0) vec4<f32> {
    let x = vec2<i32>(floor(frag.xy));
    let xf = vec2<f32>(x);
    let size = fp.src_size.xy;
    switch fp.mode {
        case 0u: {
            return texel(fsrc, size, x);
        }
        case 1u: {
            let base = x * 2;
            let sum = texel(fsrc, size, base)
                + texel(fsrc, size, base + vec2<i32>(1, 0))
                + texel(fsrc, size, base + vec2<i32>(0, 1))
                + texel(fsrc, size, base + vec2<i32>(1, 1));
            return sum * 0.25;
        }
        case 2u: {
            let idx = (xf + vec2<f32>(0.5)) / 2.0 - vec2<f32>(0.5);
            return bilinear(fsrc, size, idx);
        }
        case 3u: {
            let r = i32(fp.radius);
            let d = vec2<i32>(fp.dir.xy);
            var acc = vec4<f32>(0.0);
            for (var k: i32 = -r; k <= r; k = k + 1) {
                acc = acc + texel(fsrc, size, x + d * k) * weights[u32(k + r)];
            }
            return acc;
        }
        case 4u: {
            return texel(fsrc, size, x).a * fp.color;
        }
        case 5u: {
            // The blurred silhouette sits offset from the pixel centre.
            let shadow = bilinear(faux, size, xf - fp.offset.xy);
            let input = texel(fsrc, size, x);
            return input + shadow * (1.0 - input.a);
        }
        case 6u: {
            let c = texel(fsrc, size, x);
            var rgb = vec3<f32>(0.0);
            if (c.a > 0.0) {
                rgb = c.rgb / c.a;
            }
            let v = vec4<f32>(rgb, c.a);
            let out = clamp(
                vec4<f32>(matrix_row(0u, v), matrix_row(1u, v), matrix_row(2u, v), matrix_row(3u, v)),
                vec4<f32>(0.0),
                vec4<f32>(1.0),
            );
            return vec4<f32>(out.rgb * out.a, out.a);
        }
        default: {
            return vec4<f32>(0.0);
        }
    }
}

// Coefficient j of the row-major 4 x 5 matrix.
fn coef(j: u32) -> f32 {
    return fp.matrix[j / 4u][j % 4u];
}

// Row `row` of the matrix applied to (r, g, b, a, 1).
fn matrix_row(row: u32, v: vec4<f32>) -> f32 {
    let b = row * 5u;
    return coef(b) * v.r + coef(b + 1u) * v.g + coef(b + 2u) * v.b
        + coef(b + 3u) * v.a + coef(b + 4u);
}
