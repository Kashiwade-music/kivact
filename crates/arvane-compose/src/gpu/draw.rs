//! Role: Records the draw operations of a plan: sampling, anti-aliased coverage, blend and composite (design §8.7).

use super::exec::{FrameTextures, size_f, view, with_pass};
use super::pipelines::{SHADER_BLEND, Words, buffer, composite_slot};
use super::state::GpuContext;
use crate::ComposeError;
use crate::cache::ChunkKey;
use crate::plan::{DrawOp, Source};
use kivact_types::geometry::{Point, Rect};
use kivact_types::geometry3d::{Polygon, Projective};
use kivact_types::pixel::BlendMode;

/// The flag of a draw's uniform: nearest sampling.
const FLAG_NEAREST: u32 = 1;
/// The flag of a draw's uniform: a coverage chunk multiplies in.
const FLAG_COVERAGE: u32 = 2;

impl GpuContext {
    /// Record one [`DrawOp`] into its target.
    pub(super) fn record_draw(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        op: &DrawOp<'_>,
        frame: &FrameTextures<'_>,
    ) -> Result<(), ComposeError> {
        if op.polygon.points.len() < 3 {
            return Ok(());
        }
        let target = frame.target(op.target)?;
        let target_size = size_f(target);
        let Some(bbox) = bounding_box(&op.polygon, target_size) else {
            return Ok(());
        };
        let Some((src_view, src_size, map)) = self.source_of(op, frame)? else {
            return Ok(());
        };
        let edges = edge_data(&op.polygon, &op.split_edges);
        let coverage = match op.coverage {
            Some(key) => Some(self.chunk_view(&key)?),
            None => None,
        };
        let blended = op.blend != BlendMode::Normal;
        let copy = if blended {
            let copy = super::targets::target(&self.device, (target.width(), target.height()));
            encoder.copy_texture_to_texture(
                target.as_image_copy(),
                copy.as_image_copy(),
                target.size(),
            );
            Some(copy)
        } else {
            None
        };
        let copy_view = copy.as_ref().map(view);
        let dst = copy_view.as_ref().unwrap_or(&self.pipelines.dummy);
        let cov_view = coverage.as_ref().map(|(v, _)| v);
        let cov_size = coverage.as_ref().map_or((0.0, 0.0), |(_, s)| *s);
        let cov = cov_view.unwrap_or(&self.pipelines.dummy);
        let flags = if op.sampling == crate::plan::Sampling::Nearest {
            FLAG_NEAREST
        } else {
            0
        } | if coverage.is_some() { FLAG_COVERAGE } else { 0 };
        let m = map.m;
        let mut words = Words::default();
        words
            .v4([m[0][0] as f32, m[0][1] as f32, m[0][2] as f32, 0.0])
            .v4([m[1][0] as f32, m[1][1] as f32, m[1][2] as f32, 0.0])
            .v4([m[2][0] as f32, m[2][1] as f32, m[2][2] as f32, 0.0])
            .v4([src_size.0, src_size.1, 0.0, 0.0])
            .v4([cov_size.0, cov_size.1, 0.0, 0.0])
            .v4(bbox)
            .v4([target_size.0, target_size.1, 0.0, 0.0])
            .f(op.opacity as f32)
            .u(edges.len() as u32)
            .u(flags)
            .u(blend_index(op.blend));
        let params = super::exec::uniform(&self.device, &self.queue, words.bytes());
        let edge_bytes: Vec<u8> = edges
            .iter()
            .flat_map(|e| e.iter().flat_map(|v| v.to_le_bytes()))
            .collect();
        let edge_buffer = buffer(
            &self.device,
            &self.queue,
            wgpu::BufferUsages::STORAGE,
            &edge_bytes,
        );
        let group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Arvane draw"),
            layout: &self.pipelines.draw_layout,
            entries: &[
                entry(0, params.as_entire_binding()),
                entry(1, wgpu::BindingResource::TextureView(&src_view)),
                entry(2, wgpu::BindingResource::TextureView(dst)),
                entry(3, wgpu::BindingResource::TextureView(cov)),
                entry(4, edge_buffer.as_entire_binding()),
            ],
        });
        let slot = if blended {
            SHADER_BLEND
        } else {
            composite_slot(op.composite)
        };
        let pipeline = &self.pipelines.draw[slot];
        let target_view = view(target);
        with_pass(encoder, &target_view, wgpu::LoadOp::Load, |pass| {
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &group, &[]);
            pass.draw(0..6, 0..1);
        });
        Ok(())
    }

    /// The source view, its size in px, and the map from the target px into it.
    /// None when an external frame is not drawn this frame.
    fn source_of(
        &self,
        op: &DrawOp<'_>,
        frame: &FrameTextures<'_>,
    ) -> Result<Option<(wgpu::TextureView, (f32, f32), Projective)>, ComposeError> {
        match &op.source {
            Source::Chunk(key) => {
                let (v, size) = self.chunk_view(key)?;
                Ok(Some((v, size, op.map)))
            }
            Source::Target(id) => {
                if *id == op.target {
                    return Err(ComposeError::InvalidComposition(
                        "a draw reads the target it writes".into(),
                    ));
                }
                let texture = frame.target(*id)?;
                Ok(Some((view(texture), size_f(texture), op.map)))
            }
            Source::External {
                frame: ext, source, ..
            } => {
                let Some(bound) = frame.external.bound(ext) else {
                    return Ok(None);
                };
                let (w, h) = (bound.width as f64, bound.height as f64);
                let Some(map) = external_map(op, *source, w, h) else {
                    return Ok(None);
                };
                Ok(Some((
                    bound.view.clone(),
                    (bound.width as f32, bound.height as f32),
                    map,
                )))
            }
            Source::Pixels(_) => Err(ComposeError::InvalidComposition(
                "pixel sources are not drawn by this executor".into(),
            )),
        }
    }

    /// The view and size of the chunk `key`.
    pub(super) fn chunk_view(
        &self,
        key: &ChunkKey,
    ) -> Result<(wgpu::TextureView, (f32, f32)), ComposeError> {
        let texture = self
            .chunks
            .get(key)
            .ok_or_else(|| ComposeError::Backend(format!("chunk {key:?} is missing for a draw")))?;
        Ok((view(texture), size_f(texture)))
    }
}

/// A bind group entry.
fn entry(binding: u32, resource: wgpu::BindingResource<'_>) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry { binding, resource }
}

/// The index of a blend mode in the draw shader (`blend_fn`); PlusLighter is 255.
fn blend_index(mode: BlendMode) -> u32 {
    match mode {
        BlendMode::Normal => 0,
        BlendMode::Multiply => 1,
        BlendMode::Screen => 2,
        BlendMode::Overlay => 3,
        BlendMode::Darken => 4,
        BlendMode::Lighten => 5,
        BlendMode::ColorDodge => 6,
        BlendMode::ColorBurn => 7,
        BlendMode::HardLight => 8,
        BlendMode::SoftLight => 9,
        BlendMode::Difference => 10,
        BlendMode::Exclusion => 11,
        BlendMode::Hue => 12,
        BlendMode::Saturation => 13,
        BlendMode::Color => 14,
        BlendMode::Luminosity => 15,
        BlendMode::PlusLighter => 255,
    }
}

/// The pixel box of a polygon, grown by one px for the anti-aliased edge, clipped to the target.
fn bounding_box(polygon: &Polygon, size: (f32, f32)) -> Option<[f32; 4]> {
    let (mut x0, mut y0) = (f64::INFINITY, f64::INFINITY);
    let (mut x1, mut y1) = (f64::NEG_INFINITY, f64::NEG_INFINITY);
    for p in &polygon.points {
        x0 = x0.min(p.x);
        y0 = y0.min(p.y);
        x1 = x1.max(p.x);
        y1 = y1.max(p.y);
    }
    let x0 = (x0.floor() - 1.0).max(0.0) as f32;
    let y0 = (y0.floor() - 1.0).max(0.0) as f32;
    let x1 = (x1.ceil() + 1.0).min(f64::from(size.0)) as f32;
    let y1 = (y1.ceil() + 1.0).min(f64::from(size.1)) as f32;
    (x1 > x0 && y1 > y0).then_some([x0, y0, x1, y1])
}

/// The polygon's edges as (inward unit normal x, y, offset, kind): kind 0 is an outer
/// edge (its distance anti-aliases), 1 a split edge that includes the pixel on its
/// top or left, 2 a split edge that excludes it. Distance d = n · p + offset.
fn edge_data(polygon: &Polygon, split: &[usize]) -> Vec<[f32; 4]> {
    let points = &polygon.points;
    let n = points.len();
    let area: f64 = (0..n)
        .map(|i| {
            let (a, b) = (points[i], points[(i + 1) % n]);
            a.x * b.y - b.x * a.y
        })
        .sum();
    let sign = if area < 0.0 { -1.0 } else { 1.0 };
    (0..n)
        .filter_map(|i| {
            let (a, b) = (points[i], points[(i + 1) % n]);
            let (dx, dy) = (b.x - a.x, b.y - a.y);
            let len = dx.hypot(dy);
            if len == 0.0 {
                return None;
            }
            let (nx, ny) = (-dy / len * sign, dx / len * sign);
            let offset = -(nx * a.x + ny * a.y);
            let kind = if !split.contains(&i) {
                0.0
            } else if (nx == 0.0 && ny > 0.0) || (ny == 0.0 && nx > 0.0) {
                1.0
            } else {
                2.0
            };
            Some([nx as f32, ny as f32, offset as f32, kind])
        })
        .collect()
}

/// The map from an external frame's texel space: the target px through the op's map (into the
/// item space of the frame's rectangle) and the source rectangle of the frame's pixels.
fn external_map(op: &DrawOp<'_>, source: Option<Rect>, w: f64, h: f64) -> Option<Projective> {
    let mut x0 = f64::INFINITY;
    let mut y0 = f64::INFINITY;
    let mut x1 = f64::NEG_INFINITY;
    let mut y1 = f64::NEG_INFINITY;
    for p in &op.polygon.points {
        let q: Point = op.map.apply(*p)?;
        x0 = x0.min(q.x);
        y0 = y0.min(q.y);
        x1 = x1.max(q.x);
        y1 = y1.max(q.y);
    }
    let (rw, rh) = (x1 - x0, y1 - y0);
    if rw <= 0.0 || rh <= 0.0 {
        return None;
    }
    let src = source.unwrap_or(Rect::new(0.0, 0.0, w, h));
    let (sx, sy) = (src.width / rw, src.height / rh);
    let to_frame = Projective {
        m: [
            [sx, 0.0, src.x - x0 * sx],
            [0.0, sy, src.y - y0 * sy],
            [0.0, 0.0, 1.0],
        ],
    };
    Some(op.map.then(to_frame))
}
