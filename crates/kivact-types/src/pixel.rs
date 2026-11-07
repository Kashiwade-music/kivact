//! Role: Pixel buffers, color matrices and blend modes shared by paint, scene, effect and composition.
//! Plain data usable on any thread.

use crate::color::Rgba;

/// A 4×5 color matrix over straight-alpha colors with components in
/// `0.0..=1.0`: each output component is a row times `(r, g, b, a, 1)`,
/// rows in the order r, g, b, a, then clamped to `0.0..=1.0` (SVG
/// `feColorMatrix`).
///
/// The constructors are CSS Filter Effects' filter functions, applied to
/// sRGB components as browsers apply them; [`ColorMatrix::then`] chains
/// them as a CSS `filter` list does.
///
/// Arvane applies one to the pixels of an image and to the colors of a
/// filter graph; a host applies one to the colors of what it draws
/// ([`ColorMatrix::apply`]).
///
/// ```
/// use kivact_types::Rgba;
/// use kivact_types::ColorMatrix;
/// let red = Rgba { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
/// let gray = ColorMatrix::grayscale(1.0).apply(red);
/// assert!((gray.r - 0.2126).abs() < 1e-4 && gray.r == gray.g && gray.g == gray.b);
/// // brightness(0.5) then invert(1): applied in that order.
/// let m = ColorMatrix::brightness(0.5).then(ColorMatrix::invert(1.0));
/// assert!((m.apply(red).r - 0.5).abs() < 1e-4);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorMatrix(pub [f64; 20]);

impl Default for ColorMatrix {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl ColorMatrix {
    /// The matrix that changes nothing.
    pub const IDENTITY: ColorMatrix = ColorMatrix([
        1.0, 0.0, 0.0, 0.0, 0.0, //
        0.0, 1.0, 0.0, 0.0, 0.0, //
        0.0, 0.0, 1.0, 0.0, 0.0, //
        0.0, 0.0, 0.0, 1.0, 0.0,
    ]);

    /// The 3×3 color part `m` (row-major), alpha unchanged, no offsets.
    fn rgb(m: [f64; 9]) -> Self {
        ColorMatrix([
            m[0], m[1], m[2], 0.0, 0.0, //
            m[3], m[4], m[5], 0.0, 0.0, //
            m[6], m[7], m[8], 0.0, 0.0, //
            0.0, 0.0, 0.0, 1.0, 0.0,
        ])
    }

    /// Each color component scaled by `slope` and moved by `intercept`,
    /// alpha unchanged (`feComponentTransfer` linear functions).
    fn linear(slope: f64, intercept: f64) -> Self {
        ColorMatrix([
            slope, 0.0, 0.0, 0.0, intercept, //
            0.0, slope, 0.0, 0.0, intercept, //
            0.0, 0.0, slope, 0.0, intercept, //
            0.0, 0.0, 0.0, 1.0, 0.0,
        ])
    }

    /// CSS `brightness(amount)`: 0 black, 1 unchanged, more is brighter.
    pub fn brightness(amount: f64) -> Self {
        Self::linear(amount.max(0.0), 0.0)
    }

    /// CSS `contrast(amount)`: 0 gray, 1 unchanged, more is harsher.
    pub fn contrast(amount: f64) -> Self {
        let a = amount.max(0.0);
        Self::linear(a, 0.5 - 0.5 * a)
    }

    /// CSS `saturate(amount)`: 0 gray, 1 unchanged, more is more vivid.
    pub fn saturate(amount: f64) -> Self {
        let s = amount.max(0.0);
        Self::rgb([
            0.213 + 0.787 * s,
            0.715 - 0.715 * s,
            0.072 - 0.072 * s,
            0.213 - 0.213 * s,
            0.715 + 0.285 * s,
            0.072 - 0.072 * s,
            0.213 - 0.213 * s,
            0.715 - 0.715 * s,
            0.072 + 0.928 * s,
        ])
    }

    /// CSS `grayscale(amount)`: 0 unchanged, 1 fully gray (clamped to
    /// `0..=1`).
    pub fn grayscale(amount: f64) -> Self {
        let k = 1.0 - amount.clamp(0.0, 1.0);
        Self::rgb([
            0.2126 + 0.7874 * k,
            0.7152 - 0.7152 * k,
            0.0722 - 0.0722 * k,
            0.2126 - 0.2126 * k,
            0.7152 + 0.2848 * k,
            0.0722 - 0.0722 * k,
            0.2126 - 0.2126 * k,
            0.7152 - 0.7152 * k,
            0.0722 + 0.9278 * k,
        ])
    }

    /// CSS `sepia(amount)`: 0 unchanged, 1 fully sepia (clamped to
    /// `0..=1`).
    pub fn sepia(amount: f64) -> Self {
        let k = 1.0 - amount.clamp(0.0, 1.0);
        Self::rgb([
            0.393 + 0.607 * k,
            0.769 - 0.769 * k,
            0.189 - 0.189 * k,
            0.349 - 0.349 * k,
            0.686 + 0.314 * k,
            0.168 - 0.168 * k,
            0.272 - 0.272 * k,
            0.534 - 0.534 * k,
            0.131 + 0.869 * k,
        ])
    }

    /// CSS `hue-rotate(deg)`: hues turned by `deg` degrees.
    pub fn hue_rotate(deg: f64) -> Self {
        let (s, c) = deg.to_radians().sin_cos();
        Self::rgb([
            0.213 + c * 0.787 - s * 0.213,
            0.715 - c * 0.715 - s * 0.715,
            0.072 - c * 0.072 + s * 0.928,
            0.213 - c * 0.213 + s * 0.143,
            0.715 + c * 0.285 + s * 0.140,
            0.072 - c * 0.072 - s * 0.283,
            0.213 - c * 0.213 - s * 0.787,
            0.715 - c * 0.715 + s * 0.715,
            0.072 + c * 0.928 + s * 0.072,
        ])
    }

    /// CSS `invert(amount)`: 0 unchanged, 1 fully inverted (clamped to
    /// `0..=1`).
    pub fn invert(amount: f64) -> Self {
        let a = amount.clamp(0.0, 1.0);
        Self::linear(1.0 - 2.0 * a, a)
    }

    /// CSS `opacity(amount)`: alpha scaled by `amount` (clamped to `0..=1`).
    pub fn opacity(amount: f64) -> Self {
        let mut m = Self::IDENTITY;
        m.0[18] = amount.clamp(0.0, 1.0);
        m
    }

    /// `self`, then `next` (the matrix of a CSS filter list `self next`).
    pub fn then(self, next: ColorMatrix) -> ColorMatrix {
        let (a, b) = (&next.0, &self.0);
        let mut out = [0.0; 20];
        for row in 0..4 {
            for col in 0..5 {
                let mut v: f64 = (0..4).map(|k| a[row * 5 + k] * b[k * 5 + col]).sum();
                if col == 4 {
                    v += a[row * 5 + 4];
                }
                out[row * 5 + col] = v;
            }
        }
        ColorMatrix(out)
    }

    /// Whether it is exactly the identity.
    pub fn is_identity(&self) -> bool {
        *self == Self::IDENTITY
    }

    /// `color` through the matrix, clamped to `0.0..=1.0`.
    pub fn apply(&self, color: Rgba) -> Rgba {
        let [r, g, b, a] = self.apply_array([color.r, color.g, color.b, color.a]);
        Rgba { r, g, b, a }
    }

    /// Straight-alpha components `[r, g, b, a]` through the matrix, clamped
    /// to `0.0..=1.0`.
    pub fn apply_array(&self, c: [f64; 4]) -> [f64; 4] {
        let m = &self.0;
        let row = |i: usize| {
            (m[i * 5] * c[0]
                + m[i * 5 + 1] * c[1]
                + m[i * 5 + 2] * c[2]
                + m[i * 5 + 3] * c[3]
                + m[i * 5 + 4])
                .clamp(0.0, 1.0)
        };
        [row(0), row(1), row(2), row(3)]
    }
}

/// Straight-alpha RGBA8 pixels a scene draws as they are (no decode, no
/// asset store). Compared by size and `Arc` identity, never by content, so
/// a new `Arc` every frame costs one comparison.
#[derive(Clone, Debug)]
pub struct Pixels {
    /// Pixel width of the buffer.
    pub width: u32,
    /// Pixel height of the buffer.
    pub height: u32,
    /// Tightly packed straight-alpha RGBA8, row-major from the top left,
    /// width times height times four bytes with no row padding.
    /// Keep the same allocation for unchanged pixels; renderers cache by Arc identity.
    pub rgba: crate::AssetBytes,
}

impl Pixels {
    /// Construct an image without copying its shared pixel allocation.
    pub fn new(width: u32, height: u32, rgba: impl Into<crate::AssetBytes>) -> Self {
        Self {
            width,
            height,
            rgba: rgba.into(),
        }
    }
}

impl PartialEq for Pixels {
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width
            && self.height == other.height
            && self.rgba.same_allocation(&other.rgba)
    }
}

/// How a layer's pixels combine with what is beneath them (CSS
/// `mix-blend-mode`; [`BlendMode::PlusLighter`] is canvas `lighter`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BlendMode {
    /// Source over.
    #[default]
    Normal,
    /// Multiply.
    Multiply,
    /// Screen.
    Screen,
    /// Overlay.
    Overlay,
    /// Darken.
    Darken,
    /// Lighten.
    Lighten,
    /// Color dodge.
    ColorDodge,
    /// Color burn.
    ColorBurn,
    /// Hard light.
    HardLight,
    /// Soft light.
    SoftLight,
    /// Difference.
    Difference,
    /// Exclusion.
    Exclusion,
    /// Hue.
    Hue,
    /// Saturation.
    Saturation,
    /// Color.
    Color,
    /// Luminosity.
    Luminosity,
    /// Additive (CSS `plus-lighter`, canvas `lighter`).
    PlusLighter,
}
