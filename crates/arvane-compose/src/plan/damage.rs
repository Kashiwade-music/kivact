//! Role: Chunk rasters and window damage: which chunks and rectangles change (§8.6).

use std::borrow::Cow;
use std::sync::Arc;

use kivact_types::Matrix4;
use kivact_types::geometry::{Point, Rect};
use kivact_types::render::{PixelRect, RenderScene};

use super::bounds::{mapped_bounds, scene_bounds};
use super::builder::Builder;
use super::frame::{area, ceil_px, floor_px};
use super::hash;
use super::ops::{ChunkTask, Damage, Grid, PlanError, Redraw};
use crate::cache::{ChunkKey, ChunkRecord, LayerRecord, Retained};

/// Changed commands beyond which their bounds merge into one rectangle (§8.6).
const MERGE_AT: usize = 17;
/// How far a chunk's diff searches for a command that moved.
const SEARCH: usize = 64;
/// Damaged rectangles beyond which the whole window redraws (§8.6).
const FULL_AT: usize = 17;

/// The px rectangle of `r` in a grid of px, clipped to `size`; None when nothing is left.
fn clip_px(r: Rect, origin: Point, scale: f64, size: (u32, u32)) -> Option<PixelRect> {
    let (w, h) = (f64::from(size.0), f64::from(size.1));
    let x0 = floor_px((r.x - origin.x) * scale).max(0.0);
    let y0 = floor_px((r.y - origin.y) * scale).max(0.0);
    let x1 = ceil_px((r.right() - origin.x) * scale).min(w);
    let y1 = ceil_px((r.bottom() - origin.y) * scale).min(h);
    if !(x1 > x0 && y1 > y0) {
        return None;
    }
    Some(PixelRect {
        x: x0 as u32,
        y: y0 as u32,
        width: (x1 - x0) as u32,
        height: (y1 - y0) as u32,
    })
}

/// The layer-space rectangle of a chunk px rectangle.
fn layer_rect(r: PixelRect, grid: Grid) -> Rect {
    Rect::new(
        grid.origin.x + f64::from(r.x) / grid.scale,
        grid.origin.y + f64::from(r.y) / grid.scale,
        f64::from(r.width) / grid.scale,
        f64::from(r.height) / grid.scale,
    )
}

/// The smallest px rectangle holding every rectangle of `rects`.
fn union_px(rects: &[PixelRect]) -> Option<PixelRect> {
    let first = rects.first()?;
    let (mut x0, mut y0, mut x1, mut y1) = (
        first.x,
        first.y,
        first.x + first.width,
        first.y + first.height,
    );
    for r in &rects[1..] {
        x0 = x0.min(r.x);
        y0 = y0.min(r.y);
        x1 = x1.max(r.x + r.width);
        y1 = y1.max(r.y + r.height);
    }
    Some(PixelRect {
        x: x0,
        y: y0,
        width: x1 - x0,
        height: y1 - y0,
    })
}

/// The rectangles of a chunk's commands that changed between two indexes (old and new bounds).
fn diff(old: &[(u64, Option<PixelRect>)], new: &[(u64, Option<PixelRect>)]) -> Vec<PixelRect> {
    let head = old.iter().zip(new).take_while(|(a, b)| a == b).count();
    let tail_max = old.len().min(new.len()) - head;
    let tail = old
        .iter()
        .rev()
        .zip(new.iter().rev())
        .take(tail_max)
        .take_while(|(a, b)| a == b)
        .count();
    let (mut i, end_old) = (head, old.len() - tail);
    let (mut j, end_new) = (head, new.len() - tail);
    let mut changed: Vec<PixelRect> = Vec::new();
    while i < end_old || j < end_new {
        if i < end_old && j < end_new {
            if old[i] == new[j] {
                i += 1;
                j += 1;
                continue;
            }
            // A deletion: the old command reappears further on in the new list.
            let deleted = (i + 1..end_old.min(i + SEARCH)).find(|&k| old[k] == new[j]);
            if let Some(k) = deleted {
                changed.extend(old[i..k].iter().filter_map(|c| c.1));
                i = k;
                continue;
            }
            // An insertion: the new command reappears further on in the old list.
            let inserted = (j + 1..end_new.min(j + SEARCH)).find(|&k| new[k] == old[i]);
            if let Some(k) = inserted {
                changed.extend(new[j..k].iter().filter_map(|c| c.1));
                j = k;
                continue;
            }
            changed.extend(old[i].1);
            changed.extend(new[j].1);
            i += 1;
            j += 1;
        } else if i < end_old {
            changed.extend(old[i].1);
            i += 1;
        } else {
            changed.extend(new[j].1);
            j += 1;
        }
    }
    if changed.len() >= MERGE_AT {
        return union_px(&changed).into_iter().collect();
    }
    changed
}

impl<'a> Builder<'a> {
    /// Index a chunk's scene, decide what of it redraws against the cache, and record it.
    ///
    /// `cache_it` is false for chunks of inline layers, which are always redrawn and never cached.
    pub(super) fn chunk(
        &mut self,
        key: ChunkKey,
        scene: Cow<'a, RenderScene>,
        grid: Grid,
        to_win: &Matrix4,
        cache_it: bool,
    ) -> Result<(), PlanError> {
        let cached = cache_it && self.cacheable();
        let bounds = scene_bounds(&scene)?;
        let mut retain: Vec<Retained> = Vec::new();
        let hashes = hash::scene_hashes(&scene, &mut retain);
        let pairs: Vec<(u64, Option<PixelRect>)> = hashes
            .into_iter()
            .zip(bounds)
            .map(|(h, b)| {
                (
                    h,
                    b.and_then(|r| clip_px(r, grid.origin, grid.scale, grid.size)),
                )
            })
            .collect();
        let index: Arc<[(u64, Option<PixelRect>)]> = pairs.into();
        let cache = self.cache;
        let previous = cache.chunks.get(&key).filter(|_| cached);
        let redraw = match previous {
            Some(prev)
                if prev.scale == grid.scale
                    && prev.origin.x == grid.origin.x
                    && prev.origin.y == grid.origin.y
                    && prev.size == grid.size =>
            {
                if prev.index[..] == index[..] {
                    None
                } else {
                    let rects = diff(&prev.index, &index);
                    if rects.is_empty() {
                        None
                    } else {
                        for rect in &rects {
                            self.damage
                                .extend(mapped_bounds(to_win, layer_rect(*rect, grid)));
                        }
                        Some(Redraw::Rects(rects))
                    }
                }
            }
            Some(_) => {
                self.damage.extend(mapped_bounds(to_win, grid.rect()));
                Some(Redraw::All)
            }
            None => Some(Redraw::All),
        };
        if let Some(redraw) = redraw {
            self.chunks.push(ChunkTask {
                key,
                scene,
                origin: grid.origin,
                scale: grid.scale,
                size: grid.size,
                redraw,
            });
        }
        if cached {
            self.chunk_records.push((
                key,
                ChunkRecord {
                    scale: grid.scale,
                    origin: grid.origin,
                    size: grid.size,
                    index,
                    frame: self.frame,
                    retain,
                },
            ));
        }
        Ok(())
    }

    /// The output's damage this frame, from the records and the changes of the frame.
    pub(super) fn finish_damage(&mut self) -> Damage {
        let cache = self.cache;
        // A layer recorded last frame and not this frame vanished: its old place changes.
        for (key, prev) in &cache.layers {
            if !self.seen.contains(key) {
                self.damage.extend(prev.window_bounds);
            }
        }
        // A backdrop redraws when what it covers changes (fixed point: a backdrop's damage can reach another).
        let mut added = vec![false; self.backdrops.len()];
        loop {
            let mut grew = false;
            for (&bounds, done) in self.backdrops.iter().zip(added.iter_mut()) {
                if !*done && self.damage.iter().any(|d| d.intersects(bounds)) {
                    *done = true;
                    self.damage.push(bounds);
                    grew = true;
                }
            }
            if !grew {
                break;
            }
        }
        let window_area = f64::from(self.window.0) * f64::from(self.window.1);
        let rects: Vec<PixelRect> = self
            .damage
            .iter()
            .filter_map(|d| clip_px(*d, Point::new(0.0, 0.0), 1.0, self.window))
            .collect();
        let total: u64 = rects.iter().map(area).sum();
        if rects.len() >= FULL_AT || total as f64 > 0.5 * window_area {
            Damage::Full
        } else {
            Damage::Rects(rects)
        }
    }

    /// Carry the records and chunks of reused surfaces' layers into this frame.
    pub(super) fn carry_reused(&mut self) {
        let cache = self.cache;
        let frame = self.frame;
        for key in &self.reused {
            if let Some(prev) = cache.layers.get(key) {
                let record = LayerRecord {
                    frame,
                    ..prev.clone()
                };
                self.layer_records.push((*key, record));
            }
        }
        for (key, prev) in &cache.chunks {
            if self.reused_chunks.contains(&key.layer) {
                let record = ChunkRecord {
                    frame,
                    ..prev.clone()
                };
                self.chunk_records.push((*key, record));
            }
        }
    }
}
