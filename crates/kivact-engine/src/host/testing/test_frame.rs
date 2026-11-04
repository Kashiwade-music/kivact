//! Role: A rendered test frame: pixels, hash and PNG output.

use std::path::Path;

/// A frame the host presented: 0x00RRGGBB pixels, row by row.
#[derive(Clone, Debug)]
pub struct TestFrame {
    /// Width in physical pixels.
    pub width: u32,
    /// Height in physical pixels.
    pub height: u32,
    /// `width × height` pixels.
    pub pixels: std::sync::Arc<[u32]>,
}

impl TestFrame {
    /// The pixel at physical `(x, y)`.
    pub fn pixel(&self, x: u32, y: u32) -> Option<u32> {
        (x < self.width && y < self.height).then(|| self.pixels[(y * self.width + x) as usize])
    }

    /// FNV-1a of the pixels: equal frames hash equal.
    pub fn hash(&self) -> u64 {
        self.pixels.iter().fold(0xcbf2_9ce4_8422_2325, |h, p| {
            (h ^ u64::from(*p)).wrapping_mul(0x0000_0100_0000_01b3)
        })
    }

    /// Write the frame as an RGB PNG.
    pub fn save_png(&self, path: &Path) -> std::io::Result<()> {
        let file = std::io::BufWriter::new(std::fs::File::create(path)?);
        let mut encoder = png::Encoder::new(file, self.width, self.height);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let rgb: Vec<u8> = self
            .pixels
            .iter()
            .flat_map(|p| [(p >> 16) as u8, (p >> 8) as u8, *p as u8])
            .collect();
        encoder
            .write_header()
            .and_then(|mut writer| writer.write_image_data(&rgb))
            .map_err(std::io::Error::other)
    }
}
