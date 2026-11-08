//! Role: CSS Color 4 color spaces, conversion, interpolation and gamut mapping.

use crate::color::Rgba;

type Mat3 = [[f64; 3]; 3];

const SRGB_TO_XYZ: Mat3 = [
    [0.41239079926595934, 0.357584339383878, 0.1804807884018343],
    [0.21263900587151027, 0.715168678767756, 0.07219231536073371],
    [0.01933081871559182, 0.11919477979462598, 0.9505321522496607],
];
const XYZ_TO_SRGB: Mat3 = [
    [3.2409699419045226, -1.537383177570094, -0.4986107602930034],
    [-0.9692436362808796, 1.8759675015077202, 0.04155505740717559],
    [
        0.05563007969699366,
        -0.20397695888897652,
        1.0569715142428786,
    ],
];
const P3_TO_XYZ: Mat3 = [
    [0.4865709486482162, 0.26566769316909306, 0.1982172852343625],
    [0.2289745640697488, 0.6917385218365064, 0.079286914093745],
    [0.0, 0.04511338185890264, 1.043944368900976],
];
const XYZ_TO_P3: Mat3 = [
    [2.493496911941425, -0.9313836179191239, -0.40271078445071684],
    [
        -0.8294889695615747,
        1.7626640603183463,
        0.023624685841943577,
    ],
    [
        0.03584583024378447,
        -0.07617238926804182,
        0.9568845240076872,
    ],
];
const A98_TO_XYZ: Mat3 = [
    [0.5766690429101305, 0.1855582379065463, 0.1882286462349947],
    [0.29734497525053605, 0.6273635662554661, 0.07529145849399788],
    [0.02703136138641234, 0.07068885253582723, 0.9913375368376388],
];
const XYZ_TO_A98: Mat3 = [
    [
        2.0415879038107465,
        -0.5650069742788596,
        -0.34473135077832956,
    ],
    [-0.9692436362808795, 1.8759675015077202, 0.04155505740717557],
    [
        0.013444280632031142,
        -0.11836239223101838,
        1.0151749943912054,
    ],
];
const REC2020_TO_XYZ: Mat3 = [
    [0.6369580483012914, 0.14461690358620832, 0.1688809751641721],
    [0.2627002120112671, 0.6779980715188708, 0.05930171646986196],
    [0.0, 0.028072693049087428, 1.060985057710791],
];
const XYZ_TO_REC2020: Mat3 = [
    [1.716651187971268, -0.355670783776392, -0.25336628137366],
    [-0.666684351832489, 1.616481236634939, 0.0157685458139111],
    [0.017639857445311, -0.042770613257809, 0.942103121235474],
];
const PROPHOTO_TO_XYZ_D50: Mat3 = [
    [0.7977666449006423, 0.13518129740053308, 0.0313477341283922],
    [0.2880748288194013, 0.711835234241873, 0.00008993693872564],
    [0.0, 0.0, 0.8251046025104602],
];
const XYZ_D50_TO_PROPHOTO: Mat3 = [
    [
        1.3457868816471583,
        -0.25557208737979464,
        -0.05110186497554526,
    ],
    [-0.5446307051249019, 1.5082477428451468, 0.02052744743642139],
    [0.0, 0.0, 1.2119675456389452],
];
const D65_TO_D50: Mat3 = [
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
const D50_TO_D65: Mat3 = [
    [
        0.9554734527042182,
        -0.023098536874261423,
        0.0632593086610217,
    ],
    [
        -0.028369706963208136,
        1.0099954580058226,
        0.021041398966943008,
    ],
    [
        0.012314001688319899,
        -0.020507696433477912,
        1.3303659366080753,
    ],
];
const XYZ_TO_LMS: Mat3 = [
    [0.819022437996703, 0.3619062600528904, -0.1288737815209879],
    [0.0329836539323885, 0.9292868615863434, 0.0361446663506424],
    [0.0481771893596242, 0.2642395317527308, 0.6335478284694309],
];
const LMS_CBRT_TO_OKLAB: Mat3 = [
    [0.210454268309314, 0.7936177747023054, -0.0040720430116193],
    [1.9779985324311684, -2.42859224204858, 0.450593709617411],
    [0.0259040424655478, 0.7827717124575296, -0.8086757549230774],
];
const OKLAB_TO_LMS_CBRT: Mat3 = [
    [1.0, 0.3963377773761749, 0.2158037573099136],
    [1.0, -0.1055613458156586, -0.0638541728258133],
    [1.0, -0.0894841775298119, -1.2914855480194092],
];
const LMS_TO_XYZ: Mat3 = [
    [1.2268798758459243, -0.5578149944602171, 0.2813910456659647],
    [-0.0405757452148008, 1.112286803280317, -0.0717110580655164],
    [-0.0763729366746601, -0.4214933324022432, 1.5869240198367816],
];

/// The D50 white point used by Lab.
const D50_WHITE: [f64; 3] = [0.9642956764295677, 1.0, 0.8251046025104602];
const KAPPA: f64 = 24389.0 / 27.0;
const EPSILON: f64 = 216.0 / 24389.0;

/// Just noticeable difference in Oklab for gamut mapping.
const JND: f64 = 0.02;
/// Chroma search precision for gamut mapping.
const GAMUT_EPS: f64 = 0.0001;

/// A CSS color space.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum ColorSpace {
    /// Gamma-encoded sRGB.
    #[default]
    Srgb,
    /// Linear-light sRGB.
    SrgbLinear,
    /// Display P3.
    DisplayP3,
    /// Adobe RGB (1998).
    A98Rgb,
    /// ProPhoto RGB (D50).
    ProphotoRgb,
    /// Rec. 2020.
    Rec2020,
    /// CIE XYZ with the D50 white point.
    XyzD50,
    /// CIE XYZ with the D65 white point.
    XyzD65,
    /// CIE Lab (D50).
    Lab,
    /// Polar form of Lab.
    Lch,
    /// Oklab.
    Oklab,
    /// Polar form of Oklab.
    Oklch,
    /// Hue, saturation, lightness over sRGB.
    Hsl,
    /// Hue, whiteness, blackness over sRGB.
    Hwb,
}

/// How hues are interpolated in a polar space.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum HueInterpolation {
    /// Take the shorter arc.
    #[default]
    Shorter,
    /// Take the longer arc.
    Longer,
    /// Always increase the hue.
    Increasing,
    /// Always decrease the hue.
    Decreasing,
}

/// How colors are interpolated (CSS `in <space> <hue>`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ColorInterpolation {
    /// The space the channels are interpolated in.
    pub space: ColorSpace,
    /// The hue rule; ignored in rectangular spaces.
    pub hue: HueInterpolation,
}

impl ColorInterpolation {
    /// CSS's default for gradients, transitions and color-mix: Oklab.
    pub const DEFAULT: ColorInterpolation = ColorInterpolation {
        space: ColorSpace::Oklab,
        hue: HueInterpolation::Shorter,
    };
    /// Canvas 2D gradients: premultiplied sRGB.
    pub const SRGB: ColorInterpolation = ColorInterpolation {
        space: ColorSpace::Srgb,
        hue: HueInterpolation::Shorter,
    };
}

impl Default for ColorInterpolation {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// A color in a space with alpha. Channel ranges are CSS's: sRGB family
/// 0..1, XYZ unbounded, Lab L 0..100, Lch C >= 0 and H in degrees, Oklab L
/// 0..1, Oklch C >= 0 and H in degrees, Hsl H degrees with S and L 0..1,
/// Hwb H degrees with W and B 0..1. Alpha 0..1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpaceColor {
    /// The space `channels` are in.
    pub space: ColorSpace,
    /// The three channels, in the order the space names them.
    pub channels: [f64; 3],
    /// Straight alpha.
    pub alpha: f64,
}

fn mul3(m: &Mat3, v: [f64; 3]) -> [f64; 3] {
    m.map(|row| row[0] * v[0] + row[1] * v[1] + row[2] * v[2])
}

fn sign_pow(v: f64, exp: f64) -> f64 {
    v.signum() * v.abs().powf(exp)
}

fn identity(v: f64) -> f64 {
    v
}

fn srgb_decode(v: f64) -> f64 {
    let a = v.abs();
    if a <= 0.04045 {
        v / 12.92
    } else {
        v.signum() * ((a + 0.055) / 1.055).powf(2.4)
    }
}

fn srgb_encode(v: f64) -> f64 {
    let a = v.abs();
    if a > 0.0031308 {
        v.signum() * (1.055 * a.powf(1.0 / 2.4) - 0.055)
    } else {
        12.92 * v
    }
}

fn a98_decode(v: f64) -> f64 {
    sign_pow(v, 563.0 / 256.0)
}

fn a98_encode(v: f64) -> f64 {
    sign_pow(v, 256.0 / 563.0)
}

fn prophoto_decode(v: f64) -> f64 {
    if v.abs() <= 16.0 / 512.0 {
        v / 16.0
    } else {
        sign_pow(v, 1.8)
    }
}

fn prophoto_encode(v: f64) -> f64 {
    if v.abs() >= 1.0 / 512.0 {
        sign_pow(v, 1.0 / 1.8)
    } else {
        16.0 * v
    }
}

const REC2020_ALPHA: f64 = 1.09929682680944;
const REC2020_BETA: f64 = 0.018053968510807;

fn rec2020_decode(v: f64) -> f64 {
    let a = v.abs();
    if a < REC2020_BETA * 4.5 {
        v / 4.5
    } else {
        v.signum() * ((a + REC2020_ALPHA - 1.0) / REC2020_ALPHA).powf(1.0 / 0.45)
    }
}

fn rec2020_encode(v: f64) -> f64 {
    let a = v.abs();
    if a > REC2020_BETA {
        v.signum() * (REC2020_ALPHA * a.powf(0.45) - (REC2020_ALPHA - 1.0))
    } else {
        4.5 * v
    }
}

/// How an RGB-family space reaches XYZ.
struct RgbSpace {
    to_xyz: &'static Mat3,
    from_xyz: &'static Mat3,
    decode: fn(f64) -> f64,
    encode: fn(f64) -> f64,
    /// Whether the matrices use the D50 white point.
    d50: bool,
}

impl ColorSpace {
    fn rgb_space(self) -> Option<RgbSpace> {
        let rgb = |to_xyz, from_xyz, decode, encode, d50| RgbSpace {
            to_xyz,
            from_xyz,
            decode,
            encode,
            d50,
        };
        Some(match self {
            ColorSpace::Srgb => rgb(&SRGB_TO_XYZ, &XYZ_TO_SRGB, srgb_decode, srgb_encode, false),
            ColorSpace::SrgbLinear => rgb(&SRGB_TO_XYZ, &XYZ_TO_SRGB, identity, identity, false),
            ColorSpace::DisplayP3 => rgb(&P3_TO_XYZ, &XYZ_TO_P3, srgb_decode, srgb_encode, false),
            ColorSpace::A98Rgb => rgb(&A98_TO_XYZ, &XYZ_TO_A98, a98_decode, a98_encode, false),
            ColorSpace::Rec2020 => rgb(
                &REC2020_TO_XYZ,
                &XYZ_TO_REC2020,
                rec2020_decode,
                rec2020_encode,
                false,
            ),
            ColorSpace::ProphotoRgb => rgb(
                &PROPHOTO_TO_XYZ_D50,
                &XYZ_D50_TO_PROPHOTO,
                prophoto_decode,
                prophoto_encode,
                true,
            ),
            _ => return None,
        })
    }

    /// The rectangular space a polar space is built on (itself otherwise).
    fn rectangular_base(self) -> ColorSpace {
        match self {
            ColorSpace::Lch => ColorSpace::Lab,
            ColorSpace::Oklch => ColorSpace::Oklab,
            ColorSpace::Hsl | ColorSpace::Hwb => ColorSpace::Srgb,
            other => other,
        }
    }

    /// Index of the hue channel of a polar space.
    fn hue_index(self) -> Option<usize> {
        match self {
            ColorSpace::Lch | ColorSpace::Oklch => Some(2),
            ColorSpace::Hsl | ColorSpace::Hwb => Some(0),
            _ => None,
        }
    }
}

/// Channels of `space` to XYZ D65; `space` must be rectangular.
fn to_xyz_d65(space: ColorSpace, ch: [f64; 3]) -> [f64; 3] {
    if let Some(rgb) = space.rgb_space() {
        let xyz = mul3(rgb.to_xyz, ch.map(rgb.decode));
        return if rgb.d50 { mul3(&D50_TO_D65, xyz) } else { xyz };
    }
    match space {
        ColorSpace::XyzD50 => mul3(&D50_TO_D65, ch),
        ColorSpace::Lab => mul3(&D50_TO_D65, lab_to_xyz_d50(ch)),
        ColorSpace::Oklab => oklab_to_xyz(ch),
        _ => ch,
    }
}

/// XYZ D65 to the channels of `space`; `space` must be rectangular.
fn from_xyz_d65(space: ColorSpace, xyz: [f64; 3]) -> [f64; 3] {
    if let Some(rgb) = space.rgb_space() {
        let xyz = if rgb.d50 { mul3(&D65_TO_D50, xyz) } else { xyz };
        return mul3(rgb.from_xyz, xyz).map(rgb.encode);
    }
    match space {
        ColorSpace::XyzD50 => mul3(&D65_TO_D50, xyz),
        ColorSpace::Lab => xyz_d50_to_lab(mul3(&D65_TO_D50, xyz)),
        ColorSpace::Oklab => xyz_to_oklab(xyz),
        _ => xyz,
    }
}

fn xyz_d50_to_lab(xyz: [f64; 3]) -> [f64; 3] {
    let f = |v: f64| {
        if v > EPSILON {
            v.cbrt()
        } else {
            (KAPPA * v + 16.0) / 116.0
        }
    };
    let [f0, f1, f2] = [0, 1, 2].map(|i| f(xyz[i] / D50_WHITE[i]));
    [116.0 * f1 - 16.0, 500.0 * (f0 - f1), 200.0 * (f1 - f2)]
}

fn lab_to_xyz_d50(lab: [f64; 3]) -> [f64; 3] {
    let [l, a, b] = lab;
    let f1 = (l + 16.0) / 116.0;
    let f0 = a / 500.0 + f1;
    let f2 = f1 - b / 200.0;
    let cubed_or_linear = |f: f64| {
        let cube = f * f * f;
        if cube > EPSILON {
            cube
        } else {
            (116.0 * f - 16.0) / KAPPA
        }
    };
    let y = if l > KAPPA * EPSILON {
        f1 * f1 * f1
    } else {
        l / KAPPA
    };
    let xyz = [cubed_or_linear(f0), y, cubed_or_linear(f2)];
    [0, 1, 2].map(|i| xyz[i] * D50_WHITE[i])
}

fn xyz_to_oklab(xyz: [f64; 3]) -> [f64; 3] {
    mul3(&LMS_CBRT_TO_OKLAB, mul3(&XYZ_TO_LMS, xyz).map(f64::cbrt))
}

fn oklab_to_xyz(lab: [f64; 3]) -> [f64; 3] {
    let lms = mul3(&OKLAB_TO_LMS_CBRT, lab).map(|v| v * v * v);
    mul3(&LMS_TO_XYZ, lms)
}

/// Hue in degrees normalized to `[0, 360)`.
fn norm_hue(h: f64) -> f64 {
    let n = h.rem_euclid(360.0);
    if n >= 360.0 { 0.0 } else { n }
}

/// (L, a, b) to (L, C, H).
fn rect_to_polar(c: [f64; 3]) -> [f64; 3] {
    [
        c[0],
        c[1].hypot(c[2]),
        norm_hue(c[2].atan2(c[1]).to_degrees()),
    ]
}

/// (L, C, H) to (L, a, b).
fn polar_to_rect(c: [f64; 3]) -> [f64; 3] {
    let (sin, cos) = c[2].to_radians().sin_cos();
    [c[0], c[1] * cos, c[1] * sin]
}

/// CSS Color 4 `rgbToHsl` with S and L in 0..1.
fn rgb_to_hsl(rgb: [f64; 3]) -> [f64; 3] {
    let [r, g, b] = rgb;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let light = (min + max) / 2.0;
    let d = max - min;
    if d == 0.0 {
        return [0.0, 0.0, light];
    }
    let mut sat = if light == 0.0 || light == 1.0 {
        0.0
    } else {
        (max - light) / light.min(1.0 - light)
    };
    let mut hue = if max == r {
        (g - b) / d + if g < b { 6.0 } else { 0.0 }
    } else if max == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    } * 60.0;
    if sat < 0.0 {
        hue += 180.0;
        sat = -sat;
    }
    [norm_hue(hue), sat, light]
}

/// CSS Color 4 `hslToRgb` with S and L in 0..1.
fn hsl_to_rgb(hsl: [f64; 3]) -> [f64; 3] {
    let [h, s, l] = hsl;
    let h = norm_hue(h);
    let a = s * l.min(1.0 - l);
    let f = |n: f64| {
        let k = (n + h / 30.0) % 12.0;
        l - a * (k - 3.0).min(9.0 - k).clamp(-1.0, 1.0)
    };
    [f(0.0), f(8.0), f(4.0)]
}

/// CSS Color 4 `rgbToHwb`.
fn rgb_to_hwb(rgb: [f64; 3]) -> [f64; 3] {
    let hue = rgb_to_hsl(rgb)[0];
    let white = rgb[0].min(rgb[1]).min(rgb[2]);
    let black = 1.0 - rgb[0].max(rgb[1]).max(rgb[2]);
    [hue, white, black]
}

/// CSS Color 4 `hwbToRgb`.
fn hwb_to_rgb(hwb: [f64; 3]) -> [f64; 3] {
    let [h, w, b] = hwb;
    if w + b >= 1.0 {
        let gray = w / (w + b);
        return [gray, gray, gray];
    }
    hsl_to_rgb([h, 1.0, 0.5]).map(|v| v * (1.0 - w - b) + w)
}

impl SpaceColor {
    /// A color from channels in `space` and an alpha.
    pub fn new(space: ColorSpace, channels: [f64; 3], alpha: f64) -> SpaceColor {
        SpaceColor {
            space,
            channels,
            alpha,
        }
    }

    /// The color of `c` in sRGB.
    pub fn from_rgba(c: Rgba) -> SpaceColor {
        SpaceColor::new(ColorSpace::Srgb, [c.r, c.g, c.b], c.a)
    }

    /// The same color in `to`, through XYZ D65 (polar spaces through their
    /// rectangular counterparts). Alpha is kept.
    pub fn convert(self, to: ColorSpace) -> SpaceColor {
        if self.space == to {
            return self;
        }
        let rect = self.to_rectangular();
        let base = to.rectangular_base();
        let in_base = if rect.space == base {
            rect
        } else {
            let xyz = to_xyz_d65(rect.space, rect.channels);
            SpaceColor::new(base, from_xyz_d65(base, xyz), self.alpha)
        };
        in_base.into_space(to)
    }

    /// This color in its rectangular counterpart (itself if rectangular).
    fn to_rectangular(self) -> SpaceColor {
        let space = self.space.rectangular_base();
        let channels = match self.space {
            ColorSpace::Lch | ColorSpace::Oklch => polar_to_rect(self.channels),
            ColorSpace::Hsl => hsl_to_rgb(self.channels),
            ColorSpace::Hwb => hwb_to_rgb(self.channels),
            _ => self.channels,
        };
        SpaceColor::new(space, channels, self.alpha)
    }

    /// `self` (in the rectangular base of `to`) expressed in `to`.
    fn into_space(self, to: ColorSpace) -> SpaceColor {
        let channels = match to {
            ColorSpace::Lch | ColorSpace::Oklch => rect_to_polar(self.channels),
            ColorSpace::Hsl => rgb_to_hsl(self.channels),
            ColorSpace::Hwb => rgb_to_hwb(self.channels),
            _ => self.channels,
        };
        SpaceColor::new(to, channels, self.alpha)
    }

    /// sRGB in gamut by CSS Color 4 §13.2 (OKLCh chroma reduction). The
    /// result is clamped to 0..1, non-finite values become zero.
    pub fn to_srgb(self) -> Rgba {
        let srgb = self.convert(ColorSpace::Srgb);
        let mapped = if in_gamut(srgb.channels) {
            srgb
        } else {
            gamut_map(self)
        };
        let [r, g, b] = mapped.channels;
        Rgba {
            r,
            g,
            b,
            a: self.alpha,
        }
        .sanitize()
    }

    /// Whether the hue channel is powerless: the color is near grey, black or white,
    /// so the hue has no visible effect (the limit differs per space).
    pub fn hue_is_powerless(&self) -> bool {
        let c = self.channels;
        match self.space {
            ColorSpace::Lch => c[1] < 0.02,
            ColorSpace::Oklch => c[1] < 0.00002,
            ColorSpace::Hsl => c[1] < 1e-6 || c[2].abs() < 1e-6 || (c[2] - 1.0).abs() < 1e-6,
            ColorSpace::Hwb => c[1] + c[2] >= 1.0 - 1e-6,
            _ => false,
        }
    }
}

fn in_gamut(c: [f64; 3]) -> bool {
    c.iter().all(|v| (0.0..=1.0).contains(v))
}

/// The sRGB color of `c` with every channel clamped to 0..1.
fn clip(c: SpaceColor) -> SpaceColor {
    let mut out = c.convert(ColorSpace::Srgb);
    out.channels = out.channels.map(|v| v.clamp(0.0, 1.0));
    out
}

/// CSS Color 4 §13.2.2: reduce Oklch chroma until the clipped color is
/// within one JND of the unclipped one. Returns an sRGB color.
fn gamut_map(color: SpaceColor) -> SpaceColor {
    let mut current = color.convert(ColorSpace::Oklch);
    let alpha = color.alpha;
    if current.channels[0] >= 1.0 {
        return SpaceColor::new(ColorSpace::Srgb, [1.0; 3], alpha);
    }
    if current.channels[0] <= 0.0 {
        return SpaceColor::new(ColorSpace::Srgb, [0.0; 3], alpha);
    }
    let mut clipped = clip(current);
    if delta_e_ok(clipped, current) < JND {
        return clipped;
    }
    let (mut min, mut max) = (0.0, current.channels[1]);
    let mut min_in_gamut = true;
    while max - min > GAMUT_EPS {
        let chroma = (min + max) / 2.0;
        current.channels[1] = chroma;
        if min_in_gamut && in_gamut(current.convert(ColorSpace::Srgb).channels) {
            min = chroma;
            continue;
        }
        clipped = clip(current);
        let e = delta_e_ok(clipped, current);
        if e < JND {
            if JND - e < GAMUT_EPS {
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

/// Fix up two hues in [0, 360) for the interpolation rule (CSS Color 4 §12.4).
fn fix_hues(h1: f64, h2: f64, how: HueInterpolation) -> (f64, f64) {
    let (mut h1, mut h2) = (norm_hue(h1), norm_hue(h2));
    let delta = h2 - h1;
    match how {
        HueInterpolation::Shorter => {
            if delta > 180.0 {
                h1 += 360.0;
            } else if delta < -180.0 {
                h2 += 360.0;
            }
        }
        HueInterpolation::Longer => {
            if 0.0 < delta && delta < 180.0 {
                h1 += 360.0;
            } else if -180.0 < delta && delta <= 0.0 {
                h2 += 360.0;
            }
        }
        HueInterpolation::Increasing => {
            if delta < 0.0 {
                h2 += 360.0;
            }
        }
        HueInterpolation::Decreasing => {
            if delta > 0.0 {
                h1 += 360.0;
            }
        }
    }
    (h1, h2)
}

/// Channels multiplied by alpha, except the hue at `hue`.
fn premultiply(c: SpaceColor, hue: Option<usize>) -> [f64; 3] {
    let mut ch = c.channels;
    for (i, v) in ch.iter_mut().enumerate() {
        if Some(i) != hue {
            *v *= c.alpha;
        }
    }
    ch
}

/// Replace a powerless hue by the other color's hue (0 if both are).
fn share_powerless_hue(a: &mut SpaceColor, b: &mut SpaceColor, hue: usize) {
    match (a.hue_is_powerless(), b.hue_is_powerless()) {
        (true, true) => {
            a.channels[hue] = 0.0;
            b.channels[hue] = 0.0;
        }
        (true, false) => a.channels[hue] = b.channels[hue],
        (false, true) => b.channels[hue] = a.channels[hue],
        (false, false) => {}
    }
}

/// CSS Color 4 §12 interpolation at `t` (not clamped).
pub fn interpolate(a: SpaceColor, b: SpaceColor, t: f64, how: ColorInterpolation) -> SpaceColor {
    let space = how.space;
    let (mut ca, mut cb) = (a.convert(space), b.convert(space));
    let hue = space.hue_index();
    if let Some(h) = hue {
        share_powerless_hue(&mut ca, &mut cb, h);
    }
    let (mut xa, mut xb) = (premultiply(ca, hue), premultiply(cb, hue));
    if let Some(h) = hue {
        (xa[h], xb[h]) = fix_hues(ca.channels[h], cb.channels[h], how.hue);
    }
    let lerp = |p: f64, q: f64| p + (q - p) * t;
    let alpha = lerp(ca.alpha, cb.alpha);
    let mut channels = [0, 1, 2].map(|i| lerp(xa[i], xb[i]));
    for (i, v) in channels.iter_mut().enumerate() {
        if Some(i) == hue {
            *v = norm_hue(*v);
        } else if alpha != 0.0 {
            *v /= alpha;
        }
    }
    SpaceColor::new(space, channels, alpha)
}

/// Interpolate two sRGB colors and map the result back to sRGB.
///
/// ```
/// use kivact_types::{ColorInterpolation, Rgba, mix_rgba};
/// let black = Rgba { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
/// let white = Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
/// let mid = mix_rgba(black, white, 0.5, ColorInterpolation::SRGB);
/// assert!((mid.r - 0.5).abs() < 1e-12 && mid.a == 1.0);
/// ```
pub fn mix_rgba(a: Rgba, b: Rgba, t: f64, how: ColorInterpolation) -> Rgba {
    interpolate(SpaceColor::from_rgba(a), SpaceColor::from_rgba(b), t, how).to_srgb()
}

/// Euclidean distance in Oklab.
pub fn delta_e_ok(a: SpaceColor, b: SpaceColor) -> f64 {
    let a = a.convert(ColorSpace::Oklab).channels;
    let b = b.convert(ColorSpace::Oklab).channels;
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}
