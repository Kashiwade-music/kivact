// feDiffuseLighting and feSpecularLighting (design §8.8.4; resvg's Sobel normal and light equations).

// Modes of the light source (p0.x).
const LIGHT_DISTANT: f32 = 0.0;
const LIGHT_POINT: f32 = 1.0;
const LIGHT_SPOT: f32 = 2.0;

// The alpha of the surface in 0..=255 at integer position (x, y).
fn surface_byte(x: i32, y: i32) -> f32 {
    return byte_of(texel(ga, gp.a_size.xy, vec2<i32>(x, y)).a);
}

// The Sobel normal of the surface at (x, y) of a w × h region: the unscaled normal
// (-nx, -ny) and the factors of each axis (resvg's kernels for the edges and corners).
// Returns (normal.x, normal.y, factor.x, factor.y).
fn sobel_normal(x: i32, y: i32, w: i32, h: i32) -> vec4<f32> {
    let xl = max(x - 1, 0);
    let xr = min(x + 1, w - 1);
    let yu = max(y - 1, 0);
    let yd = min(y + 1, h - 1);
    // Weights: 2 at the centre row or column, 1 at a neighbour that exists.
    let wu = select(0.0, 1.0, y - 1 >= 0);
    let wd = select(0.0, 1.0, y + 1 <= h - 1);
    let wl = select(0.0, 1.0, x - 1 >= 0);
    let wr = select(0.0, 1.0, x + 1 <= w - 1);
    let row_sum = 2.0 + wu + wd;
    let col_sum = 2.0 + wl + wr;
    let span_x = f32(xr - xl);
    let span_y = f32(yd - yu);
    let nx = wu * (surface_byte(xr, yu) - surface_byte(xl, yu))
        + 2.0 * (surface_byte(xr, y) - surface_byte(xl, y))
        + wd * (surface_byte(xr, yd) - surface_byte(xl, yd));
    let ny = wl * (surface_byte(xl, yd) - surface_byte(xl, yu))
        + 2.0 * (surface_byte(x, yd) - surface_byte(x, yu))
        + wr * (surface_byte(xr, yd) - surface_byte(xr, yu));
    let factor_x = 2.0 / (span_x * row_sum);
    let factor_y = 2.0 / (span_y * col_sum);
    return vec4<f32>(-nx, -ny, factor_x, factor_y);
}

// The surface normal with the surface scale applied (resvg's surface_normal).
fn surface_vector(scale: f32, s: vec4<f32>) -> vec3<f32> {
    let k = scale / 255.0;
    return vec3<f32>(s.x * k * s.z, s.y * k * s.w, 1.0);
}

// A unit vector, or the vector itself when it is (nearly) zero.
fn normalized_or(v: vec3<f32>) -> vec3<f32> {
    let l = length(v);
    if (l > 4.0 * 1.1920929e-7) {
        return v / l;
    }
    return v;
}

// The lighting primitive at pixel (x, y) of the output: the light's colour times the factor.
fn light_at(x: i32, y: i32) -> vec4<f32> {
    let w = i32(gp.out_size.x);
    let h = i32(gp.out_size.y);
    let kind = gp.p0.x;
    let surface_scale = gp.p0.y;
    let constant = gp.p0.z;
    let spec_exponent = gp.p0.w;
    let is_specular = gp.ints.y != 0u;
    let s = sobel_normal(x, y, w, h);
    let flat = abs(s.x) <= 4.0 * 1.1920929e-7 && abs(s.y) <= 4.0 * 1.1920929e-7;

    // The light vector (distant lights are constant; the others point from the surface).
    var light = gp.p1.xyz;
    if (kind != LIGHT_DISTANT) {
        let surface_z = texel(ga, gp.a_size.xy, vec2<i32>(x, y)).a * surface_scale;
        light = normalized_or(gp.p1.xyz - vec3<f32>(f32(x), f32(y), surface_z));
    }

    // The colour the light has at the surface (a spot light's cone).
    var color = gp.p3.xyz;
    if (kind == LIGHT_SPOT) {
        let direction = normalized_or(gp.p2.xyz - gp.p1.xyz);
        let minus_l_dot_s = -dot(light, direction);
        let cone = gp.p3.w;
        if (minus_l_dot_s <= 0.0 || (cone >= -1.0 && minus_l_dot_s < cone)) {
            color = vec3<f32>(0.0);
        } else {
            let factor = pow(minus_l_dot_s, gp.p2.w);
            color = floor(clamp(color * factor, vec3<f32>(0.0), vec3<f32>(255.0)) + 0.5);
        }
    }

    var factor = 0.0;
    if (!is_specular) {
        var k = light.z;
        if (!flat) {
            let n = surface_vector(surface_scale, s);
            k = dot(n, light) / length(n);
        }
        factor = constant * k;
    } else {
        let hv = light + vec3<f32>(0.0, 0.0, 1.0);
        let h_length = length(hv);
        if (h_length > 4.0 * 1.1920929e-7) {
            var n_dot_h = hv.z / h_length;
            if (!flat) {
                let n = surface_vector(surface_scale, s);
                n_dot_h = dot(n, hv) / length(n) / h_length;
            }
            var k = n_dot_h;
            if (abs(spec_exponent - 1.0) > 4.0 * 1.1920929e-7) {
                k = pow(max(n_dot_h, 0.0), spec_exponent);
            }
            factor = constant * k;
        }
    }

    let c = floor(clamp(color * factor, vec3<f32>(0.0), vec3<f32>(255.0)) + 0.5);
    var alpha = 255.0;
    if (is_specular) {
        alpha = max(c.r, max(c.g, c.b));
    }
    return vec4<f32>(c, alpha) / 255.0;
}
