//! Role: Chunk rasters and window damage: which chunks and rectangles change.

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;

use arvane_protocol::{LayerKey, PixelRect, RenderCmd, RenderScene};
use kivact_types::Matrix4;
use kivact_types::geometry::{Point, Rect};

use super::bounds::{mapped_bounds, scene_bounds};
use super::builder::Builder;
use super::frame::{area, ceil_px, floor_px};
use super::hash;
use super::ops::{ChunkTask, Damage, Grid, PlanError, Redraw};
use crate::cache::{ChunkKey, ChunkRecord, ChunkRole, Indexed, LayerRecord, Reach};

/// How far a chunk's diff searches for a command that moved.
const SEARCH: usize = 64;
/// The most clusters damage or a chunk's drawn area is kept as.
const MAX_CLUSTERS: usize = 16;
/// The cell, in px, a content chunk's grid snaps out to.
const CHUNK_CELL_PX: u32 = 64;

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

/// Whether two px rectangles are one change: their union is at most 1.5 times their two areas.
fn near(a: PixelRect, b: PixelRect) -> bool {
    let u = union_px(&[a, b]).expect("two rects");
    2 * area(&u) <= 3 * (area(&a) + area(&b))
}

/// The px rectangle of `x0..x1` by `y0..y1`; None when it is empty.
fn span(x0: u32, x1: u32, y0: u32, y1: u32) -> Option<PixelRect> {
    (x1 > x0 && y1 > y0).then(|| PixelRect {
        x: x0,
        y: y0,
        width: x1 - x0,
        height: y1 - y0,
    })
}

/// The parts of `r` outside `c`: the band above `c`, the band below it (both the full width of
/// `r`), then the parts left and right of `c` in the rows `c` covers. `r` itself when they do not
/// overlap. Empty parts are dropped.
fn minus(r: PixelRect, c: PixelRect) -> Vec<PixelRect> {
    let (r_right, r_bottom) = (r.x + r.width, r.y + r.height);
    let (c_right, c_bottom) = (c.x + c.width, c.y + c.height);
    if !(r.x < c_right && c.x < r_right && r.y < c_bottom && c.y < r_bottom) {
        return vec![r];
    }
    let (top, bottom) = (r.y.max(c.y), r_bottom.min(c_bottom));
    [
        span(r.x, r_right, r.y, c.y),
        span(r.x, r_right, c_bottom, r_bottom),
        span(r.x, c.x, top, bottom),
        span(c_right, r_right, top, bottom),
    ]
    .into_iter()
    .flatten()
    .collect()
}

/// The parts of `rects` outside every rect of `cut` (pairwise disjoint when `rects` are).
fn subtract_all(rects: Vec<PixelRect>, cut: &[PixelRect]) -> Vec<PixelRect> {
    cut.iter().fold(rects, |rects, c| {
        rects.into_iter().flat_map(|r| minus(r, *c)).collect()
    })
}

/// Replace two near clusters by their union until no two clusters are near.
fn merge_near(clusters: &mut Vec<PixelRect>) {
    'again: loop {
        for i in 0..clusters.len() {
            for j in i + 1..clusters.len() {
                if near(clusters[i], clusters[j]) {
                    clusters[i] = union_px(&[clusters[i], clusters[j]]).expect("two rects");
                    clusters.swap_remove(j);
                    continue 'again;
                }
            }
        }
        break;
    }
}

/// Rectangles grouped into clusters of nearby changes: two merge when their union is at most 1.5
/// times their two areas; overlapping clusters that do not merge are cut into disjoint pieces;
/// more than `MAX_CLUSTERS` become their union.
fn cluster(rects: Vec<PixelRect>) -> Vec<PixelRect> {
    let mut clusters: Vec<PixelRect> = Vec::new();
    for rect in &rects {
        match clusters.iter().position(|c| near(*c, *rect)) {
            Some(i) => clusters[i] = union_px(&[clusters[i], *rect]).expect("two rects"),
            None => clusters.push(*rect),
        }
        if clusters.len() > MAX_CLUSTERS {
            // Near clusters are one cluster: merge them before giving up.
            merge_near(&mut clusters);
            if clusters.len() > MAX_CLUSTERS {
                return union_px(&rects).into_iter().collect();
            }
        }
    }
    merge_near(&mut clusters);
    let mut pieces: Vec<PixelRect> = Vec::new();
    for c in clusters {
        pieces.extend(subtract_all(vec![c], &pieces));
        if pieces.len() > MAX_CLUSTERS {
            return union_px(&rects).into_iter().collect();
        }
    }
    pieces
}

/// The rectangles of the commands of `index` that draw, in chunk px.
fn draws(index: &[Indexed]) -> Vec<PixelRect> {
    index
        .iter()
        .filter_map(|entry| match entry.reach {
            Reach::Draws(rect) => Some(rect),
            Reach::Opens(_) | Reach::Nothing => None,
        })
        .collect()
}

/// The cell of a chunk grid of `size` px that `draws` cover: their union, its left and top edges rounded down and its right and bottom edges rounded up to `CHUNK_CELL_PX`, kept inside the grid; `None` when nothing draws.
fn content_cell(draws: &[PixelRect], size: (u32, u32)) -> Option<PixelRect> {
    let union = union_px(draws)?;
    let cell = CHUNK_CELL_PX;
    let x0 = union.x / cell * cell;
    let y0 = union.y / cell * cell;
    let x1 = ((union.x + union.width).div_ceil(cell) * cell).min(size.0);
    let y1 = ((union.y + union.height).div_ceil(cell) * cell).min(size.1);
    Some(PixelRect {
        x: x0,
        y: y0,
        width: x1 - x0,
        height: y1 - y0,
    })
}

/// The part of `r` (layer grid px) inside `cell` (layer grid px), in the cell's px; `None` when none is left.
fn cut(r: PixelRect, cell: PixelRect) -> Option<PixelRect> {
    let x0 = r.x.max(cell.x);
    let y0 = r.y.max(cell.y);
    let x1 = (r.x + r.width).min(cell.x + cell.width);
    let y1 = (r.y + r.height).min(cell.y + cell.height);
    if x1 > x0 && y1 > y0 {
        Some(PixelRect {
            x: x0 - cell.x,
            y: y0 - cell.y,
            width: x1 - x0,
            height: y1 - y0,
        })
    } else {
        None
    }
}

/// The parts of `drawn` (layer grid px) inside `cell`, in the cell's px.
fn parts_of(drawn: &[PixelRect], cell: PixelRect) -> Vec<PixelRect> {
    drawn.iter().filter_map(|r| cut(*r, cell)).collect()
}

/// The redraw of a whole chunk with `cell`: the clusters it draws when it has them (a content
/// chunk, whose `drawn` is not empty), else the whole cell.
fn whole_redraw(drawn: &[PixelRect], cell: PixelRect) -> Redraw {
    if drawn.is_empty() {
        Redraw::All
    } else {
        Redraw::Fresh(parts_of(drawn, cell))
    }
}

/// The cell a chunk covers and the clusters it draws, in its grid's px. A content chunk covers the
/// cell its commands reach and draws the clusters of them; another chunk covers its whole grid and
/// draws no clusters.
fn coverage(index: &[Indexed], grid: Grid, content: bool) -> (Option<PixelRect>, Vec<PixelRect>) {
    if !content {
        let whole = PixelRect {
            x: 0,
            y: 0,
            width: grid.size.0,
            height: grid.size.1,
        };
        return (Some(whole), Vec::new());
    }
    let rects = draws(index);
    (content_cell(&rects, grid.size), cluster(rects))
}

/// What a chunk of an unchanged grid redraws when `changed` (layer grid px) changed: nothing when its
/// cell draws nothing, the whole cell when the cell moved, else the changed parts of the cell.
fn redraw_of(
    changed: &[PixelRect],
    cell: Option<PixelRect>,
    prev_cell: Option<PixelRect>,
    drawn: &[PixelRect],
) -> Option<Redraw> {
    let cell = cell?;
    if changed.is_empty() {
        None
    } else if prev_cell != Some(cell) {
        Some(whole_redraw(drawn, cell))
    } else {
        let parts: Vec<PixelRect> = changed.iter().filter_map(|r| cut(*r, cell)).collect();
        (!parts.is_empty()).then_some(Redraw::Rects(parts))
    }
}

/// The index entry of a command with its chunk px bounds.
fn indexed(command: &RenderCmd, hash: u64, clipped: Option<PixelRect>) -> Indexed {
    let reach = match (command, clipped) {
        (_, None) => Reach::Nothing,
        (RenderCmd::PushClip(_) | RenderCmd::PushTransform(_), Some(rect)) => Reach::Opens(rect),
        (RenderCmd::PushGroup(g), Some(rect)) if g.blur.is_none() => Reach::Opens(rect),
        (_, Some(rect)) => Reach::Draws(rect),
    };
    Indexed { hash, reach }
}

/// The rectangles of a chunk's commands that changed between two indexes (old and new bounds).
fn diff(old: &[Indexed], new: &[Indexed]) -> Vec<PixelRect> {
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
                changed.extend(old[i..k].iter().filter_map(Indexed::damage));
                i = k;
                continue;
            }
            // An insertion: the new command reappears further on in the old list.
            let inserted = (j + 1..end_new.min(j + SEARCH)).find(|&k| new[k] == old[i]);
            if let Some(k) = inserted {
                changed.extend(new[j..k].iter().filter_map(Indexed::damage));
                j = k;
                continue;
            }
            changed.extend(old[i].damage());
            changed.extend(new[j].damage());
            i += 1;
            j += 1;
        } else if i < end_old {
            changed.extend(old[i].damage());
            i += 1;
        } else {
            changed.extend(new[j].damage());
            j += 1;
        }
    }
    cluster(changed)
}

/// Debug builds: a kept content version must mean the same commands.
fn check_version(
    scene: &RenderScene,
    prev: &ChunkRecord,
    grid: Grid,
    key: ChunkKey,
    version: u64,
) -> Result<(), PlanError> {
    let bounds = scene_bounds(scene)?;
    let mut retain = Vec::new();
    let index: Vec<Indexed> = hash::scene_hashes(scene, &mut retain)
        .into_iter()
        .zip(bounds)
        .zip(scene.commands.iter())
        .map(|((h, b), command)| {
            indexed(
                command,
                h,
                b.and_then(|r| clip_px(r, grid.origin, grid.scale, grid.size)),
            )
        })
        .collect();
    assert!(
        prev.index[..] == index[..],
        "content_version {version} of {key:?} was kept though the layer's commands changed",
    );
    Ok(())
}

impl<'a> Builder<'a> {
    /// Index a chunk's scene, decide what of it redraws against the cache, and record it.
    ///
    /// `cache_it` is false for chunks of inline layers, which are always redrawn and never cached.
    /// `version` is the layer's content version for its content chunks, None otherwise.
    /// Returns the chunk's cell in the layer grid's px; None when it draws nothing.
    pub(super) fn chunk(
        &mut self,
        key: ChunkKey,
        scene: Cow<'a, RenderScene>,
        grid: Grid,
        to_win: &Matrix4,
        cache_it: bool,
        version: Option<u64>,
    ) -> Result<Option<PixelRect>, PlanError> {
        let cached = cache_it && self.cacheable();
        if let Some(prev) = self.keep_chunk(key, &scene, grid, cached, version)? {
            let cell = prev.cell;
            self.chunk_records.push((
                key,
                ChunkRecord {
                    frame: self.frame,
                    ..prev
                },
            ));
            return Ok(cell);
        }
        let bounds = scene_bounds(&scene)?;
        let hashed = match &scene {
            Cow::Borrowed(borrowed) => self.hashed(*borrowed),
            Cow::Owned(owned) => Rc::new(hash::scene_hashed(owned)),
        };
        let index: Arc<[Indexed]> = hashed
            .hashes
            .iter()
            .copied()
            .zip(bounds)
            .zip(scene.commands.iter())
            .map(|((h, b), command)| {
                let clipped = b.and_then(|r| clip_px(r, grid.origin, grid.scale, grid.size));
                indexed(command, h, clipped)
            })
            .collect();
        let content = version.is_some();
        let (cell, drawn) = coverage(&index, grid, content);
        let cache = self.cache;
        let prev = cache.chunk(&key).filter(|_| cached);
        // A content chunk new to the cache damages the places its clusters are drawn.
        if cached && content && prev.is_none() && cache.layer(&key.layer).is_some() {
            for rect in &drawn {
                self.damage
                    .extend(mapped_bounds(to_win, layer_rect(*rect, grid)));
            }
        }
        let redraw = self.redraw_chunk(prev, grid, to_win, &index, cell, &drawn);
        if let (Some(cell), Some(redraw)) = (cell, redraw) {
            let sub = grid.cell(cell);
            self.chunks.push(ChunkTask {
                key,
                scene,
                origin: sub.origin,
                scale: sub.scale,
                size: sub.size,
                offset: (cell.x, cell.y),
                redraw,
                index: Arc::clone(&index),
            });
        }
        if let Some(cell) = cell.filter(|_| content) {
            self.drawn_px.insert(key, parts_of(&drawn, cell));
        }
        if cached {
            self.chunk_records.push((
                key,
                ChunkRecord {
                    scale: grid.scale,
                    origin: grid.origin,
                    size: grid.size,
                    cell,
                    drawn,
                    index,
                    version,
                    frame: self.frame,
                    retain: hashed.retain.clone(),
                },
            ));
        } else {
            self.forgets.push(key);
        }
        Ok(cell)
    }

    /// What a chunk with `index`, `cell` and `drawn` redraws against its previous record `prev` (none
    /// when the cache keeps none), damaging the window where its pixels change.
    fn redraw_chunk(
        &mut self,
        prev: Option<&ChunkRecord>,
        grid: Grid,
        to_win: &Matrix4,
        index: &[Indexed],
        cell: Option<PixelRect>,
        drawn: &[PixelRect],
    ) -> Option<Redraw> {
        match prev {
            Some(prev)
                if prev.scale == grid.scale
                    && prev.origin.x == grid.origin.x
                    && prev.origin.y == grid.origin.y
                    && prev.size == grid.size =>
            {
                if prev.index[..] == *index {
                    None
                } else {
                    let changed = diff(&prev.index, index);
                    for rect in &changed {
                        self.damage
                            .extend(mapped_bounds(to_win, layer_rect(*rect, grid)));
                    }
                    redraw_of(&changed, cell, prev.cell, drawn)
                }
            }
            Some(_) => {
                self.damage.extend(mapped_bounds(to_win, grid.rect()));
                cell.map(|c| whole_redraw(drawn, c))
            }
            None => cell.map(|c| whole_redraw(drawn, c)),
        }
    }

    /// The previous record of a content chunk whose layer has the same content
    /// version and grid, when it is kept: nothing is hashed, diffed or drawn.
    fn keep_chunk(
        &mut self,
        key: ChunkKey,
        scene: &RenderScene,
        grid: Grid,
        cached: bool,
        version: Option<u64>,
    ) -> Result<Option<ChunkRecord>, PlanError> {
        let cache = self.cache;
        let Some(version) = version else {
            return Ok(None);
        };
        let Some(prev) = cache.chunk(&key).filter(|_| cached) else {
            return Ok(None);
        };
        if prev.version != Some(version)
            || prev.scale != grid.scale
            || prev.origin.x != grid.origin.x
            || prev.origin.y != grid.origin.y
            || prev.size != grid.size
        {
            return Ok(None);
        }
        if cfg!(debug_assertions) {
            check_version(scene, prev, grid, key, version)?;
        }
        if let Some(cell) = prev.cell {
            self.drawn_px.insert(key, parts_of(&prev.drawn, cell));
        }
        Ok(Some(prev.clone()))
    }

    /// The output's damage this frame, from the records and the changes of the frame.
    pub(super) fn finish_damage(&mut self) -> Damage {
        let cache = self.cache;
        // After a resize, `invalidate()` or a failed frame, the whole output is redrawn.
        let crate::cache::Records::Kept { chunks, layers } = &cache.records else {
            return Damage::Full;
        };
        // A layer recorded last frame and not this frame vanished: its old place changes.
        for (key, prev) in layers {
            if !self.seen.contains(key) {
                self.damage.extend(prev.window_bounds);
            }
        }
        // A content chunk drawn last frame and not this frame vanished: its clusters change where its layer is placed now.
        let now: HashSet<ChunkKey> = self.chunk_records.iter().map(|(k, _)| *k).collect();
        let placed: HashMap<LayerKey, Matrix4> = self
            .layer_records
            .iter()
            .map(|(k, r)| (*k, r.to_win))
            .collect();
        for (key, prev) in chunks {
            let vanished = prev.frame + 1 == self.frame
                && matches!(key.role, ChunkRole::Content(_))
                && !self.reused_chunks.contains(&key.layer)
                && !now.contains(key);
            if let Some(map) = placed.get(&key.layer).filter(|_| vanished) {
                let grid = Grid {
                    origin: prev.origin,
                    scale: prev.scale,
                    size: prev.size,
                };
                for rect in &prev.drawn {
                    self.damage
                        .extend(mapped_bounds(map, layer_rect(*rect, grid)));
                }
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
        let rects = cluster(rects);
        let total: u64 = rects.iter().map(area).sum();
        if total as f64 > 0.5 * window_area {
            Damage::Full
        } else {
            Damage::Rects(rects)
        }
    }

    /// Carry the records and chunks of reused surfaces' layers into this frame.
    pub(super) fn carry_reused(&mut self) {
        let cache = self.cache;
        let frame = self.frame;
        let crate::cache::Records::Kept { chunks, layers } = &cache.records else {
            return;
        };
        for key in &self.reused {
            if let Some(prev) = layers.get(key) {
                let record = LayerRecord {
                    frame,
                    ..prev.clone()
                };
                self.layer_records.push((*key, record));
            }
        }
        for (key, prev) in chunks {
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

#[cfg(test)]
mod tests {
    //! Role: Tests of the clusters that window damage and chunk draws are cut into.
    use super::*;

    /// A rect of `x, y, width, height`.
    fn px(x: u32, y: u32, width: u32, height: u32) -> PixelRect {
        PixelRect {
            x,
            y,
            width,
            height,
        }
    }

    /// Whether no two rects share a pixel.
    fn disjoint(rects: &[PixelRect]) -> bool {
        rects.iter().enumerate().all(|(i, a)| {
            rects[i + 1..].iter().all(|b| {
                a.x + a.width <= b.x
                    || b.x + b.width <= a.x
                    || a.y + a.height <= b.y
                    || b.y + b.height <= a.y
            })
        })
    }

    /// Two rects that overlap by a few px and are not near stay apart, as disjoint pieces of their union.
    #[test]
    fn overlapping_far_rects_become_disjoint_pieces() {
        let pieces = cluster(vec![px(600, 0, 200, 580), px(0, 575, 800, 25)]);
        assert_eq!(pieces.len(), 3, "the panel and the bar merged: {pieces:?}");
        assert!(disjoint(&pieces), "the pieces overlap: {pieces:?}");
        assert_eq!(
            pieces.iter().map(area).sum::<u64>(),
            135_000,
            "the pieces do not cover the union exactly"
        );
    }

    /// Crossing lines that cut each other into more than 16 pieces become their union.
    #[test]
    fn too_many_pieces_become_their_union() {
        let mut rects: Vec<PixelRect> = (1..=4).map(|i| px(0, i * 100, 800, 1)).collect();
        rects.extend((0..4).map(|i| px(100 + i * 200, 0, 1, 600)));
        assert_eq!(
            cluster(rects),
            vec![px(0, 0, 800, 600)],
            "the pieces were not capped"
        );
    }
}
