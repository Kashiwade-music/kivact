//! Role: 3D contexts: planes sharing a context, drawn in depth order (§8.4).

use kivact_types::Matrix4;
use kivact_types::geometry::{Point, Rect};
use kivact_types::geometry3d::{Polygon, Projective};
use kivact_types::pixel::BlendMode;
use kivact_types::render::{Composite, LayerItem, MAX_CONTEXT_PLANES, Plane as LayerPlane};
use kivact_types::scene::Shape;

use super::bounds::mapped_bounds;
use super::builder::Builder;
use super::frame::{Dest, ItemCtx, px_map};
use super::ops::{DrawOp, Op, PlanError, Sampling, Scissor, Source};
use super::polygon::{Plane, Poly, Split, plane_of, project_rect, split};

/// One participant of a context: a run of the root's own items, or a flat child layer.
#[derive(Clone, Copy, Debug)]
enum PartKind {
    /// Items `start..end` of `layer` (a plane of the context root).
    Fragment {
        layer: u32,
        start: usize,
        end: usize,
    },
    /// A flat child layer.
    Flat { layer: u32 },
}

/// A participant and the maps that place it.
#[derive(Clone, Debug)]
struct Part {
    /// What it draws.
    kind: PartKind,
    /// Layer space → target px, not flattened.
    to_t: Matrix4,
    /// Layer space → window px, not flattened.
    to_win: Matrix4,
    /// The painting key: the item positions that lead to it.
    key: Vec<u32>,
}

/// How a participant's surface is drawn once its polygon is placed.
struct Draw<'a> {
    /// The surface.
    source: Source<'a>,
    /// Target px → surface px.
    map: Projective,
    /// Alpha multiplier.
    opacity: f64,
    /// Blend mode.
    blend: BlendMode,
    /// Porter–Duff operator.
    composite: Composite,
    /// Disable smoothing.
    nearest: bool,
}

/// A participant's planned surface: its draw, its polygon in target px (3D), and its window bounds.
type PlannedPart<'a> = (Draw<'a>, Vec<[f64; 3]>, Option<Rect>);

/// The binary space partition of the participants' polygons.
enum Bsp {
    /// Nothing left.
    Empty,
    /// A splitting plane with the polygons in it, and the two sides.
    Split {
        /// The splitting plane.
        plane: Plane,
        /// The polygons on its positive side.
        front: Box<Bsp>,
        /// The polygons on its negative side.
        back: Box<Bsp>,
        /// The polygons in the plane.
        coplanar: Vec<Poly>,
    },
}

impl<'a> Builder<'a> {
    /// Plan the 3D context rooted at `index`: its participants' surfaces, then their polygons in depth order into `t`.
    pub(super) fn emit_context(
        &mut self,
        index: u32,
        dest: Dest,
        to_win: Matrix4,
        clips: &'a [Shape],
    ) -> Result<(), PlanError> {
        let Dest {
            target: t,
            to: to_t,
            size: tsize,
            cover,
        } = dest;
        let root = self.layer(index)?;
        let mut parts = Vec::new();
        self.collect_parts(index, &to_t, &to_win, &mut Vec::new(), &mut parts)?;
        let flat_overlay = parts.len() > MAX_CONTEXT_PLANES;

        let mut draws: Vec<Draw<'a>> = Vec::new();
        let mut polys: Vec<Poly> = Vec::new();
        let mut bounds: Option<Rect> = None;
        for part in parts {
            let planned = self.plan_part(&part, tsize)?;
            let Some((draw, pts, win)) = planned else {
                continue;
            };
            bounds = match (bounds, win) {
                (Some(a), Some(b)) => Some(a.union(b)),
                (None, b) => b,
                (a, None) => a,
            };
            let n = pts.len();
            polys.push(Poly {
                pts,
                leave: vec![false; n],
                key: part.key,
                draw: draws.len(),
            });
            draws.push(draw);
        }

        let sig = self.signature(index, &to_t, tsize)?;
        self.record(root, bounds, sig, &to_win, clips, true);

        let ordered = if flat_overlay {
            polys.sort_by(|a, b| a.key.cmp(&b.key));
            polys
        } else {
            let mut out = Vec::with_capacity(polys.len());
            order(build_bsp(polys), &mut out);
            out
        };
        for poly in ordered {
            let draw = &draws[poly.draw];
            let split_edges = poly
                .leave
                .iter()
                .enumerate()
                .filter_map(|(i, leave)| leave.then_some(i))
                .collect();
            let points = poly.pts.iter().map(|p| Point::new(p[0], p[1])).collect();
            self.ops.push(Op::Draw(DrawOp {
                target: t,
                source: draw.source.clone(),
                polygon: Polygon { points },
                split_edges,
                map: draw.map,
                opacity: draw.opacity,
                blend: draw.blend,
                composite: draw.composite,
                sampling: Sampling::nearest(draw.nearest),
                coverage: cover,
                scissor: Scissor::Off,
            }));
        }
        Ok(())
    }

    /// Plan one participant's surface. Returns its draw, its projected polygon in target px (3D), and its window bounds.
    fn plan_part(
        &mut self,
        part: &Part,
        tsize: (u32, u32),
    ) -> Result<Option<PlannedPart<'a>>, PlanError> {
        match part.kind {
            PartKind::Fragment { layer, start, end } => {
                self.fragment_part(part, layer, start, end, tsize)
            }
            PartKind::Flat { layer } => self.flat_part(part, layer, tsize),
        }
    }

    /// Plan a run of the root's own items as one surface.
    fn fragment_part(
        &mut self,
        part: &Part,
        index: u32,
        start: usize,
        end: usize,
        tsize: (u32, u32),
    ) -> Result<Option<PlannedPart<'a>>, PlanError> {
        let owner = self.layer(index)?;
        if owner.backface == kivact_types::paint::BackfaceVisibility::Hidden
            && part.to_t.shows_back()
        {
            return Ok(None);
        }
        let Some(geo) = self.geometry(owner, &part.to_t, tsize) else {
            return Ok(None);
        };
        let surface = self.new_temp(geo.size);
        let cx = ItemCtx {
            layer: owner,
            first: start,
            items: &owner.items[start..end],
            grid: geo.grid(),
            dest: Dest {
                target: surface,
                to: geo.ms,
                size: geo.size,
                cover: None,
            },
            to_win: part.to_win,
            cache_chunks: true,
        };
        self.emit_items(&cx)?;
        let (Some(pts), Some(map)) = (
            project_rect(&part.to_t, geo.rect),
            px_map(&part.to_t, &geo.ms),
        ) else {
            return Ok(None);
        };
        let win = mapped_bounds(&part.to_win, geo.rect);
        let draw = Draw {
            source: Source::Target(surface),
            map,
            opacity: 1.0,
            blend: BlendMode::Normal,
            composite: Composite::SourceOver,
            nearest: false,
        };
        Ok(Some((draw, pts, win)))
    }

    /// Plan a flat child layer as its own surface.
    fn flat_part(
        &mut self,
        part: &Part,
        index: u32,
        tsize: (u32, u32),
    ) -> Result<Option<PlannedPart<'a>>, PlanError> {
        let owner = self.layer(index)?;
        if owner.backface == kivact_types::paint::BackfaceVisibility::Hidden
            && part.to_t.shows_back()
        {
            return Ok(None);
        }
        let Some(geo) = self.geometry(owner, &part.to_t, tsize) else {
            return Ok(None);
        };
        let sig = self.signature(index, &part.to_t, tsize)?;
        let mark = self.damage.len();
        let source = self.build_surface(index, owner, &geo, &part.to_win, sig)?;
        self.expand_effects(owner, &geo, mark, &part.to_win);
        let win = mapped_bounds(&part.to_win, geo.rect);
        self.record(owner, win, sig, &part.to_win, &[], false);
        let (Some(pts), Some(map)) = (
            project_rect(&part.to_t, geo.rect),
            px_map(&part.to_t, &geo.ms),
        ) else {
            return Ok(None);
        };
        let draw = Draw {
            source,
            map,
            opacity: owner.effects.opacity,
            blend: owner.effects.blend,
            composite: owner.effects.composite,
            nearest: geo.nearest,
        };
        Ok(Some((draw, pts, win)))
    }

    /// Collect the participants of the context rooted at `index`, depth first in item order.
    fn collect_parts(
        &self,
        index: u32,
        to_t: &Matrix4,
        to_win: &Matrix4,
        path: &mut Vec<u32>,
        out: &mut Vec<Part>,
    ) -> Result<(), PlanError> {
        let layer = self.layer(index)?;
        let mut start = 0usize;
        for (pos, item) in layer.items.iter().enumerate() {
            // A child with clips is not a participant (design §6.4, §8.4): it stays in the
            // fragment, drawn flat with its clips as a coverage on that fragment's surface.
            let LayerItem::Layer {
                index: child,
                clips,
            } = item
            else {
                continue;
            };
            if !clips.is_empty() {
                continue;
            }
            if start < pos {
                let mut key = path.clone();
                key.push(start as u32);
                out.push(Part {
                    kind: PartKind::Fragment {
                        layer: index,
                        start,
                        end: pos,
                    },
                    to_t: *to_t,
                    to_win: *to_win,
                    key,
                });
            }
            let child_layer = self.layer(*child)?;
            let child_to = child_layer.transform.then(*to_t);
            let child_win = child_layer.transform.then(*to_win);
            path.push(pos as u32);
            if child_layer.plane == LayerPlane::Preserve3d {
                self.collect_parts(*child, &child_to, &child_win, path, out)?;
            } else {
                out.push(Part {
                    kind: PartKind::Flat { layer: *child },
                    to_t: child_to,
                    to_win: child_win,
                    key: path.clone(),
                });
            }
            path.pop();
            start = pos + 1;
        }
        if start < layer.items.len() {
            let mut key = path.clone();
            key.push(start as u32);
            out.push(Part {
                kind: PartKind::Fragment {
                    layer: index,
                    start,
                    end: layer.items.len(),
                },
                to_t: *to_t,
                to_win: *to_win,
                key,
            });
        }
        Ok(())
    }
}

/// Build the BSP of polygons: the first polygon's plane splits the rest.
fn build_bsp(mut polys: Vec<Poly>) -> Bsp {
    while !polys.is_empty() {
        let first = polys.remove(0);
        let Some(plane) = plane_of(&first.pts) else {
            continue;
        };
        let mut front = Vec::new();
        let mut back = Vec::new();
        let mut coplanar = vec![first];
        for poly in polys {
            let Split {
                front: f,
                back: b,
                coplanar: c,
            } = split(poly, &plane);
            front.extend(f);
            back.extend(b);
            coplanar.extend(c);
        }
        return Bsp::Split {
            plane,
            front: Box::new(build_bsp(front)),
            back: Box::new(build_bsp(back)),
            coplanar,
        };
    }
    Bsp::Empty
}

/// Paint order from the back: the far side first, then the plane's polygons in key order, then the near side.
/// The viewer looks down −z from +∞, so the side the normal points to is the near side when its z is positive.
fn order(node: Bsp, out: &mut Vec<Poly>) {
    if let Bsp::Split {
        plane,
        front,
        back,
        mut coplanar,
    } = node
    {
        coplanar.sort_by(|a, b| a.key.cmp(&b.key));
        if plane.n[2] >= 0.0 {
            order(*back, out);
            out.append(&mut coplanar);
            order(*front, out);
        } else {
            order(*front, out);
            out.append(&mut coplanar);
            order(*back, out);
        }
    }
}
