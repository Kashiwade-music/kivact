//! Role: CSS Color 4 formulas the color tests compute expected values with.
//! The formulas of CSS Color 4 that the expected values are computed with.
//! The matrices are those of CSS Color 4 §18; nothing here calls the
//! implementation under test.

pub(crate) const SRGB_TO_XYZ: [[f64; 3]; 3] = [
    [0.41239079926595934, 0.357584339383878, 0.1804807884018343],
    [0.21263900587151027, 0.715168678767756, 0.07219231536073371],
    [0.01933081871559182, 0.11919477979462598, 0.9505321522496607],
];
pub(crate) const XYZ_TO_SRGB: [[f64; 3]; 3] = [
    [3.2409699419045226, -1.537383177570094, -0.4986107602930034],
    [-0.9692436362808796, 1.8759675015077202, 0.04155505740717559],
    [
        0.05563007969699366,
        -0.20397695888897652,
        1.0569715142428786,
    ],
];
pub(crate) const P3_TO_XYZ: [[f64; 3]; 3] = [
    [0.4865709486482162, 0.26566769316909306, 0.1982172852343625],
    [0.2289745640697488, 0.6917385218365064, 0.079286914093745],
    [0.0, 0.04511338185890264, 1.043944368900976],
];
pub(crate) const XYZ_TO_LMS: [[f64; 3]; 3] = [
    [
        0.819_022_437_996_703,
        0.3619062600528904,
        -0.1288737815209879,
    ],
    [0.0329836539323885, 0.9292868615863434, 0.0361446663506424],
    [0.0481771893596242, 0.2642395317527308, 0.6335478284694309],
];
pub(crate) const CBRT_LMS_TO_OKLAB: [[f64; 3]; 3] = [
    [
        0.210_454_268_309_314,
        0.7936177747023054,
        -0.0040720430116193,
    ],
    [
        1.9779985324311684,
        -2.428_592_242_048_58,
        0.450_593_709_617_411,
    ],
    [0.0259040424655478, 0.7827717124575296, -0.8086757549230774],
];
pub(crate) const OKLAB_TO_CBRT_LMS: [[f64; 3]; 3] = [
    [1.0, 0.3963377773761749, 0.2158037573099136],
    [1.0, -0.1055613458156586, -0.0638541728258133],
    [1.0, -0.0894841775298119, -1.2914855480194092],
];
pub(crate) const LMS_TO_XYZ: [[f64; 3]; 3] = [
    [1.2268798758459243, -0.5578149944602171, 0.2813910456659647],
    [
        -0.0405757452148008,
        1.112_286_803_280_317,
        -0.0717110580655164,
    ],
    [-0.0763729366746601, -0.4214933324022432, 1.5869240198367816],
];

pub(crate) type V3 = [f64; 3];

pub(crate) fn mul(m: &[[f64; 3]; 3], v: V3) -> V3 {
    let row = |r: &[f64; 3]| r[0] * v[0] + r[1] * v[1] + r[2] * v[2];
    [row(&m[0]), row(&m[1]), row(&m[2])]
}

/// The sRGB transfer function (sign preserving).
pub(crate) fn to_linear(v: f64) -> f64 {
    let a = v.abs();
    let l = if a <= 0.04045 {
        a / 12.92
    } else {
        ((a + 0.055) / 1.055).powf(2.4)
    };
    l.copysign(v)
}

/// The inverse of `to_linear`.
pub(crate) fn to_encoded(v: f64) -> f64 {
    let a = v.abs();
    let e = if a <= 0.0031308 {
        12.92 * a
    } else {
        1.055 * a.powf(1.0 / 2.4) - 0.055
    };
    e.copysign(v)
}

pub(crate) fn xyz_to_oklab(xyz: V3) -> V3 {
    let lms = mul(&XYZ_TO_LMS, xyz).map(f64::cbrt);
    mul(&CBRT_LMS_TO_OKLAB, lms)
}

pub(crate) fn oklab_to_xyz(lab: V3) -> V3 {
    let lms = mul(&OKLAB_TO_CBRT_LMS, lab).map(|v| v * v * v);
    mul(&LMS_TO_XYZ, lms)
}

/// Oklab of sRGB values (encoded, may lie outside 0..1).
pub(crate) fn encoded_to_oklab(enc: V3) -> V3 {
    let xyz = mul(&SRGB_TO_XYZ, enc.map(to_linear));
    xyz_to_oklab(xyz)
}

/// Unclipped sRGB (encoded) of an Oklab color.
pub(crate) fn oklab_to_encoded(lab: V3) -> V3 {
    mul(&XYZ_TO_SRGB, oklab_to_xyz(lab)).map(to_encoded)
}

/// Oklab of an Oklch color (hue in degrees).
pub(crate) fn oklch_to_oklab(l: f64, c: f64, h: f64) -> V3 {
    let (sin, cos) = h.to_radians().sin_cos();
    [l, c * cos, c * sin]
}

pub(crate) const D65_TO_D50: [[f64; 3]; 3] = [
    [
        1.0479298208405488,
        0.022946793341019088,
        -0.05019222954313557,
    ],
    [
        0.029627815688159344,
        0.990434484573249,
        -0.01707382502938514,
    ],
    [
        -0.009243058152591178,
        0.015055144896577895,
        0.7518742899580008,
    ],
];
pub(crate) const D50_WHITE: V3 = [0.9642956764295677, 1.0, 0.8251046025104602];
const LAB_KAPPA: f64 = 24389.0 / 27.0;
const LAB_EPSILON: f64 = 216.0 / 24389.0;

fn lab_f(v: f64) -> f64 {
    if v > LAB_EPSILON {
        v.cbrt()
    } else {
        (LAB_KAPPA * v + 16.0) / 116.0
    }
}

/// CIE Lab (D50, Bradford adapted) of sRGB values (encoded).
pub(crate) fn encoded_to_lab(enc: V3) -> V3 {
    let xyz = mul(&D65_TO_D50, mul(&SRGB_TO_XYZ, enc.map(to_linear)));
    let [f0, f1, f2] = [0, 1, 2].map(|i| lab_f(xyz[i] / D50_WHITE[i]));
    [116.0 * f1 - 16.0, 500.0 * (f0 - f1), 200.0 * (f1 - f2)]
}

pub(crate) fn oklab_to_oklch(lab: V3) -> V3 {
    let c = (lab[1] * lab[1] + lab[2] * lab[2]).sqrt();
    let h = lab[2].atan2(lab[1]).to_degrees().rem_euclid(360.0);
    [lab[0], c, h]
}

pub(crate) fn delta_e(a: V3, b: V3) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// The Display P3 color `enc` (encoded) as unclipped sRGB (encoded).
pub(crate) fn p3_to_srgb(enc: V3) -> V3 {
    let xyz = mul(&P3_TO_XYZ, enc.map(to_linear));
    mul(&XYZ_TO_SRGB, xyz).map(to_encoded)
}

fn in_srgb(enc: V3) -> bool {
    enc.iter().all(|v| (0.0..=1.0).contains(v))
}

fn clip(enc: V3) -> V3 {
    enc.map(|v| v.clamp(0.0, 1.0))
}

/// CSS Color 4 §13.2.2 gamut mapping of an unclipped sRGB color (encoded).
pub(crate) fn gamut_map(enc: V3) -> V3 {
    if in_srgb(enc) {
        return enc;
    }
    const JND: f64 = 0.02;
    const EPS: f64 = 0.0001;
    let current = oklab_to_oklch(encoded_to_oklab(enc));
    let (l, c, h) = (current[0], current[1], current[2]);
    if l >= 1.0 {
        return [1.0; 3];
    }
    if l <= 0.0 {
        return [0.0; 3];
    }
    let at_chroma = |chroma: f64| oklch_to_oklab(l, chroma, h);
    let clip_of = |chroma: f64| {
        let lab = at_chroma(chroma);
        (lab, clip(oklab_to_encoded(lab)))
    };
    let (lab0, mut clipped) = clip_of(c);
    if delta_e(encoded_to_oklab(clipped), lab0) < JND {
        return clipped;
    }
    let (mut min, mut max) = (0.0, c);
    let mut min_in_gamut = true;
    while max - min > EPS {
        let chroma = (min + max) / 2.0;
        let lab = at_chroma(chroma);
        if min_in_gamut && in_srgb(oklab_to_encoded(lab)) {
            min = chroma;
            continue;
        }
        clipped = clip(oklab_to_encoded(lab));
        let e = delta_e(encoded_to_oklab(clipped), lab);
        if e < JND {
            if JND - e < EPS {
                return clipped;
            }
            min_in_gamut = false;
            min = chroma;
        } else {
            max = chroma;
        }
    }
    clipped
}

/// A pixel (0xRRGGBB) as sRGB values in 0..1.
pub(crate) fn pixel_encoded(p: u32) -> V3 {
    [(p >> 16) & 255, (p >> 8) & 255, p & 255].map(|c| f64::from(c) / 255.0)
}

/// The 8-bit value of an encoded channel, rounded.
pub(crate) fn channel(v: f64) -> u32 {
    (v * 255.0).round().clamp(0.0, 255.0) as u32
}

pub(crate) fn packed(enc: V3) -> u32 {
    (channel(enc[0]) << 16) | (channel(enc[1]) << 8) | channel(enc[2])
}
