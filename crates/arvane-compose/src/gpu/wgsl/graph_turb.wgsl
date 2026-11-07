// feTurbulence (design §8.8.4): the Perlin gradient noise of the SVG reference algorithm.
// The lattice and gradient tables are built by the CPU (cpu/graph/turbulence.rs).

// Gradient noise of channel `ch` at (x, y) with the stitch tile of the frequencies (wrap, width, height).
fn turb_noise2(ch: u32, x: f32, y: f32, stitch_on: bool, wrap: vec4<f32>, size: vec2<f32>) -> f32 {
    let tt_x = x + TURB_PERLIN_N;
    var bx0 = i32(tt_x);
    var bx1 = bx0 + 1;
    let rx0 = tt_x - f32(bx0);
    let rx1 = rx0 - 1.0;
    let tt_y = y + TURB_PERLIN_N;
    var by0 = i32(tt_y);
    var by1 = by0 + 1;
    let ry0 = tt_y - f32(by0);
    let ry1 = ry0 - 1.0;
    if (stitch_on) {
        let wrap_x = i32(wrap.x);
        let width = i32(wrap.z);
        let wrap_y = i32(wrap.y);
        let height = i32(wrap.w);
        if (bx0 >= wrap_x) {
            bx0 = bx0 - width;
        }
        if (bx1 >= wrap_x) {
            bx1 = bx1 - width;
        }
        if (by0 >= wrap_y) {
            by0 = by0 - height;
        }
        if (by1 >= wrap_y) {
            by1 = by1 - height;
        }
    }
    bx0 = bx0 & 255;
    bx1 = bx1 & 255;
    by0 = by0 & 255;
    by1 = by1 & 255;
    let i = turb_lattice[u32(bx0)];
    let j = turb_lattice[u32(bx1)];
    let b00 = turb_lattice[i + u32(by0)];
    let b10 = turb_lattice[j + u32(by0)];
    let b01 = turb_lattice[i + u32(by1)];
    let b11 = turb_lattice[j + u32(by1)];
    let sx = rx0 * rx0 * (3.0 - 2.0 * rx0);
    let sy = ry0 * ry0 * (3.0 - 2.0 * ry0);
    let base = ch * TURB_LATTICE;
    var q = turb_gradient[base + b00];
    let u0 = rx0 * q.x + ry0 * q.y;
    q = turb_gradient[base + b10];
    let v0 = rx1 * q.x + ry0 * q.y;
    let a = u0 + sx * (v0 - u0);
    q = turb_gradient[base + b01];
    let u1 = rx0 * q.x + ry1 * q.y;
    q = turb_gradient[base + b11];
    let v1 = rx1 * q.x + ry1 * q.y;
    let b = u1 + sx * (v1 - u1);
    return a + sy * (b - a);
}

// One channel of the turbulence at noise point (x, y) for the tile at `tile` of `size` (resvg's turbulence).
fn turb_channel(ch: u32, x: f32, y: f32, tile: vec2<f32>, size: vec2<f32>) -> f32 {
    var base = gp.p0.zw;
    let stitch_on = gp.ints.z != 0u;
    var wrap = vec4<f32>(0.0);
    if (stitch_on) {
        if (base.x != 0.0) {
            let lo = floor(size.x * base.x) / size.x;
            let hi = ceil(size.x * base.x) / size.x;
            base.x = select(hi, lo, base.x / lo < hi / base.x);
        }
        if (base.y != 0.0) {
            let lo = floor(size.y * base.y) / size.y;
            let hi = ceil(size.y * base.y) / size.y;
            base.y = select(hi, lo, base.y / lo < hi / base.y);
        }
        let width = floor(size.x * base.x + 0.5);
        let height = floor(size.y * base.y + 0.5);
        // The wrap points, as the CPU keeps them (i32 truncation of the sums).
        wrap = vec4<f32>(
            floor(tile.x * base.x + TURB_PERLIN_N + width),
            floor(tile.y * base.y + TURB_PERLIN_N + height),
            width,
            height,
        );
    }
    let is_fractal = gp.ints.w == 0u;
    var sum = 0.0;
    var px = x * base.x;
    var py = y * base.y;
    var ratio = 1.0;
    for (var octave: u32 = 0u; octave < gp.ints.y; octave = octave + 1u) {
        let n = turb_noise2(ch, px, py, stitch_on, wrap, size);
        var term = abs(n);
        if (is_fractal) {
            term = n;
        }
        sum = sum + term / ratio;
        px = px * 2.0;
        py = py * 2.0;
        ratio = ratio * 2.0;
        if (stitch_on) {
            wrap = vec4<f32>(2.0 * wrap.x - TURB_PERLIN_N, 2.0 * wrap.y - TURB_PERLIN_N, wrap.z * 2.0, wrap.w * 2.0);
        }
    }
    return sum;
}

// The turbulence pixel at (x, y): four noise channels, premultiplied by alpha.
fn turb_pixel(x: f32, y: f32) -> vec4<f32> {
    let size = gp.out_size.xy;
    let tile = vec2<f32>(x, y);
    var c = array<f32, 4>(0.0, 0.0, 0.0, 0.0);
    for (var ch: u32 = 0u; ch < 4u; ch = ch + 1u) {
        let n = turb_channel(ch, x + gp.p0.x, y + gp.p0.y, tile, size);
        var v = n * 255.0;
        if (gp.ints.w == 0u) {
            v = (n * 255.0 + 255.0) / 2.0;
        }
        c[ch] = clamp(clamp(v, 0.0, 255.0) / 255.0, 0.0, 1.0);
    }
    let alpha = c[3];
    return vec4<f32>(c[0] * alpha, c[1] * alpha, c[2] * alpha, alpha);
}
