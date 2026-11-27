//! Role: Premultiplied RGBA8 pixel buffers of the CPU compositor and sampling.

use arvane_protocol::PixelRect;
use kivact_types::geometry::Point;

/// A premultiplied color with components in 0..=1 (red, green, blue, alpha).
pub(crate) type Premul = [f32; 4];

/// Anything whose texels can be read at integer coordinates.
pub(crate) trait Texels {
    /// The premultiplied texel at (`x`, `y`); transparent outside the texture.
    fn texel(&self, x: i64, y: i64) -> Premul;
}

/// A premultiplied RGBA8 pixel buffer, row-major from the top left.
#[derive(Clone, Debug, Default)]
pub(crate) struct Pixmap {
    /// Width in px.
    width: u32,
    /// Height in px.
    height: u32,
    /// `width * height * 4` premultiplied bytes.
    pub(crate) data: Vec<u8>,
}

/// Quantizes a 0..=1 component to a byte as `(v * 255).round()`.
pub(crate) fn quantize(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}

impl Pixmap {
    /// A transparent pixmap of `width` × `height` px.
    pub(crate) fn new(width: u32, height: u32) -> Self {
        Pixmap {
            width,
            height,
            data: vec![0; width as usize * height as usize * 4],
        }
    }

    /// The size in px.
    pub(crate) fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// The premultiplied color at (`x`, `y`), which must be inside.
    pub(crate) fn get(&self, x: u32, y: u32) -> Premul {
        let i = (y as usize * self.width as usize + x as usize) * 4;
        let p = &self.data[i..i + 4];
        [
            f32::from(p[0]) / 255.0,
            f32::from(p[1]) / 255.0,
            f32::from(p[2]) / 255.0,
            f32::from(p[3]) / 255.0,
        ]
    }

    /// Store the premultiplied color `c` at (`x`, `y`), which must be inside.
    pub(crate) fn set(&mut self, x: u32, y: u32, c: Premul) {
        let i = (y as usize * self.width as usize + x as usize) * 4;
        let p = &mut self.data[i..i + 4];
        for (dst, v) in p.iter_mut().zip(c) {
            *dst = quantize(v);
        }
    }

    /// Copies the pixels of `src` at each rectangle of `rects`, moved by `offset` (target px to
    /// chunk px), over the rectangle. Where `src` has no pixel the target becomes transparent,
    /// as a draw leaves it.
    pub(crate) fn copy_rects_from(
        &mut self,
        src: &Pixmap,
        offset: (i64, i64),
        rects: &[PixelRect],
    ) {
        let (w, h) = (i64::from(self.width), i64::from(self.height));
        let (sw, sh) = (i64::from(src.width), i64::from(src.height));
        let (ox, oy) = offset;
        let stride = self.width as usize * 4;
        for r in rects {
            let x0 = i64::from(r.x).min(w);
            let x1 = (i64::from(r.x) + i64::from(r.width)).min(w);
            let y_end = (i64::from(r.y) + i64::from(r.height)).min(h);
            for y in i64::from(r.y).min(h)..y_end {
                let row = y as usize * stride;
                let sy = y + oy;
                // The columns whose source pixel lies inside the chunk: [vx0, vx1).
                let (vx0, vx1) = (x0.max(-ox), x1.min(sw - ox));
                if sy < 0 || sy >= sh || vx0 >= vx1 {
                    self.data[row + x0 as usize * 4..row + x1 as usize * 4].fill(0);
                    continue;
                }
                let d0 = row + vx0 as usize * 4;
                let d1 = row + vx1 as usize * 4;
                let s0 = sy as usize * src.width as usize * 4 + (vx0 + ox) as usize * 4;
                self.data[d0..d1].copy_from_slice(&src.data[s0..s0 + d1 - d0]);
                self.data[row + x0 as usize * 4..d0].fill(0);
                self.data[d1..row + x1 as usize * 4].fill(0);
            }
        }
    }

    /// Clear `rect` (or all of it when None) to transparent.
    pub(crate) fn clear(&mut self, rect: Option<PixelRect>) {
        let Some(r) = rect else {
            self.data.fill(0);
            return;
        };
        let x_end = r.x.saturating_add(r.width).min(self.width);
        let y_end = r.y.saturating_add(r.height).min(self.height);
        if r.x >= x_end {
            return;
        }
        let row = self.width as usize * 4;
        for y in r.y..y_end {
            let start = y as usize * row + r.x as usize * 4;
            let end = y as usize * row + x_end as usize * 4;
            self.data[start..end].fill(0);
        }
    }

    /// Take the pixel buffer's storage out, leaving an empty pixmap.
    pub(crate) fn take(&mut self) -> Pixmap {
        std::mem::take(self)
    }
}

impl Texels for Pixmap {
    fn texel(&self, x: i64, y: i64) -> Premul {
        if x < 0 || y < 0 || x >= i64::from(self.width) || y >= i64::from(self.height) {
            return [0.0; 4];
        }
        self.get(x as u32, y as u32)
    }
}

/// Bilinear sampling at `p`, where texel (i, j) is centred on (i + 0.5, j + 0.5).
/// Out-of-range texels count as transparent.
pub(crate) fn sample_bilinear<T: Texels + ?Sized>(t: &T, p: Point) -> Premul {
    let fx = p.x - 0.5;
    let fy = p.y - 0.5;
    let (x0, y0) = (fx.floor(), fy.floor());
    let tx = (fx - x0) as f32;
    let ty = (fy - y0) as f32;
    let (x0, y0) = (x0 as i64, y0 as i64);
    let a = t.texel(x0, y0);
    let b = t.texel(x0 + 1, y0);
    let c = t.texel(x0, y0 + 1);
    let d = t.texel(x0 + 1, y0 + 1);
    let mut out = [0.0_f32; 4];
    for (i, o) in out.iter_mut().enumerate() {
        *o = (1.0 - tx) * (1.0 - ty) * a[i]
            + tx * (1.0 - ty) * b[i]
            + (1.0 - tx) * ty * c[i]
            + tx * ty * d[i];
    }
    out
}

/// Nearest sampling at `p`: the texel holding it, transparent outside.
pub(crate) fn sample_nearest<T: Texels + ?Sized>(t: &T, p: Point) -> Premul {
    t.texel(p.x.floor() as i64, p.y.floor() as i64)
}
