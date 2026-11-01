//! Role: Normalized dock geometry, absolute region styles and pointer drop targets.
//!
//! Regions are placed with percentage insets of the dock root, so every
//! container and its retained panel content follow model changes and window
//! resizes in the same layout pass, without measuring nested boxes.

use std::hash::Hash;

use kivact::draw::{Point, Rect};
use kivact::style::{Length, Position, Style, pct, style};

use super::model::{DockAxis, DockLayout, DockNode, DockPlacement, DockSide, DockTabs};

/// Splitter thickness in logical pixels.
use crate::dock_style::SPLITTER_THICKNESS as GAP;
/// Tab strip height in logical pixels.
pub(super) const STRIP: f32 = 30.0;
/// Width of the dock-edge drop band.
const EDGE: f32 = 16.0;
/// Smallest pixel extent a splitter drag leaves to either neighbor.
pub(super) const MIN_EXTENT: f32 = 48.0;
const EPSILON: f64 = 1e-6;

/// A normalized rectangle of the dock root, `0..=1` on both axes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Frame {
    pub(super) x0: f64,
    pub(super) y0: f64,
    pub(super) x1: f64,
    pub(super) y1: f64,
}

impl Frame {
    pub(super) const FULL: Self = Self {
        x0: 0.0,
        y0: 0.0,
        x1: 1.0,
        y1: 1.0,
    };

    /// Pixel inset of each edge: half a splitter on interior edges.
    fn gaps(self) -> [f32; 4] {
        let half = GAP / 2.0;
        [
            if self.x0 > EPSILON { half } else { 0.0 },
            if self.y0 > EPSILON { half } else { 0.0 },
            if self.x1 < 1.0 - EPSILON { half } else { 0.0 },
            if self.y1 < 1.0 - EPSILON { half } else { 0.0 },
        ]
    }

    /// Absolute insets for this region, moved down by `top` pixels and
    /// inset by `inset` pixels on every side.
    pub(super) fn absolute(self, top: f32, inset: f32) -> Style {
        let [left, upper, right, lower] = self.gaps();
        let edge = |fraction: f64, gap: f32| -> Length { pct(fraction * 100.0) + (gap + inset) };
        style()
            .position(Position::Absolute)
            .left(edge(self.x0, left))
            .top(edge(self.y0, upper) + top)
            .right(edge(1.0 - self.x1, right))
            .bottom(edge(1.0 - self.y1, lower))
    }

    /// The window rectangle of this region inside `root`.
    pub(super) fn rect(self, root: Rect) -> Rect {
        let [left, upper, right, lower] = self.gaps();
        let x = root.x + self.x0 as f32 * root.width + left;
        let y = root.y + self.y0 as f32 * root.height + upper;
        let x1 = root.x + self.x1 as f32 * root.width - right;
        let y1 = root.y + self.y1 as f32 * root.height - lower;
        Rect::new(x, y, (x1 - x).max(0.0), (y1 - y).max(0.0))
    }
}

/// A container and the region it occupies.
pub(super) struct Region<'a, K> {
    pub(super) frame: Frame,
    pub(super) tabs: &'a DockTabs<K>,
}

/// The boundary between children `index` and `index + 1` of a split.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Divider {
    pub(super) path: Vec<usize>,
    pub(super) index: usize,
    pub(super) axis: DockAxis,
    /// The whole split's region.
    pub(super) span: Frame,
    /// Boundary position along the axis, normalized to the dock root.
    pub(super) at: f64,
    /// Normalized weights of every child of the split.
    pub(super) weights: Vec<f64>,
}

impl Divider {
    pub(super) fn style(&self) -> Style {
        let [left, upper, right, lower] = self.span.gaps();
        let half = GAP / 2.0;
        let base = style().position(Position::Absolute);
        match self.axis {
            DockAxis::Horizontal => base
                .left(pct(self.at * 100.0) - half)
                .width(GAP)
                .top(pct(self.span.y0 * 100.0) + upper)
                .bottom(pct((1.0 - self.span.y1) * 100.0) + lower),
            DockAxis::Vertical => base
                .top(pct(self.at * 100.0) - half)
                .height(GAP)
                .left(pct(self.span.x0 * 100.0) + left)
                .right(pct((1.0 - self.span.x1) * 100.0) + right),
        }
    }
    /// Pixel extent of the whole split along its axis.
    pub(super) fn extent(&self, root: Rect) -> f32 {
        let fraction = match self.axis {
            DockAxis::Horizontal => self.span.x1 - self.span.x0,
            DockAxis::Vertical => self.span.y1 - self.span.y0,
        };
        let size = match self.axis {
            DockAxis::Horizontal => root.width,
            DockAxis::Vertical => root.height,
        };
        (fraction as f32 * size).max(1.0)
    }
}

/// Containers in depth-first order and the dividers of every split, each
/// split's dividers following its children.
pub(super) fn regions<K>(root: &DockNode<K>) -> (Vec<Region<'_, K>>, Vec<Divider>) {
    let mut out = (Vec::new(), Vec::new());
    visit(root, Frame::FULL, &mut Vec::new(), &mut out);
    out
}

fn visit<'a, K>(
    node: &'a DockNode<K>,
    frame: Frame,
    path: &mut Vec<usize>,
    out: &mut (Vec<Region<'a, K>>, Vec<Divider>),
) {
    match node {
        DockNode::Tabs(tabs) => out.0.push(Region { frame, tabs }),
        DockNode::Split { axis, children } => {
            let total: f64 = children.iter().map(|c| c.weight).sum::<f64>().max(EPSILON);
            let weights = children
                .iter()
                .map(|c| c.weight / total)
                .collect::<Vec<_>>();
            let (start, extent) = match axis {
                DockAxis::Horizontal => (frame.x0, frame.x1 - frame.x0),
                DockAxis::Vertical => (frame.y0, frame.y1 - frame.y0),
            };
            let mut cursor = start;
            let mut boundaries = Vec::new();
            for (index, child) in children.iter().enumerate() {
                let next = if index + 1 == children.len() {
                    start + extent
                } else {
                    cursor + weights[index] * extent
                };
                let child_frame = match axis {
                    DockAxis::Horizontal => Frame {
                        x0: cursor,
                        x1: next,
                        ..frame
                    },
                    DockAxis::Vertical => Frame {
                        y0: cursor,
                        y1: next,
                        ..frame
                    },
                };
                path.push(index);
                visit(&child.node, child_frame, path, out);
                path.pop();
                if index + 1 < children.len() {
                    boundaries.push(next);
                }
                cursor = next;
            }
            for (index, at) in boundaries.into_iter().enumerate() {
                out.1.push(Divider {
                    path: path.clone(),
                    index,
                    axis: *axis,
                    span: frame,
                    at,
                    weights: weights.clone(),
                });
            }
        }
    }
}

/// A validated drop and its preview rectangle relative to the dock root.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct DropTarget<K> {
    pub(super) placement: DockPlacement<K>,
    pub(super) preview: Rect,
    /// A strip insertion shows a bar instead of a zone.
    pub(super) bar: bool,
}

/// The placement under a window-space pointer while dragging `source`.
/// Targets that would not change the layout are rejected.
pub(super) fn drop_target<K: Clone + Eq + Hash>(
    layout: &DockLayout<K>,
    root: Rect,
    pos: Point,
    source: &K,
    tab_rect: impl Fn(&K) -> Option<Rect>,
) -> Option<DropTarget<K>> {
    if !root.contains(pos) {
        return None;
    }
    let local = |rect: Rect| Rect::new(rect.x - root.x, rect.y - root.y, rect.width, rect.height);
    let maximized = layout.maximized_container();
    let (regions, _) = regions(&layout.root);
    let region = regions.into_iter().find_map(|region| {
        let frame = match maximized {
            Some(max) if std::ptr::eq(max, region.tabs) => Frame::FULL,
            Some(_) => return None,
            None => region.frame,
        };
        let rect = frame.rect(root);
        rect.contains(pos).then_some((region.tabs, rect))
    });
    // Tab strips take precedence over the dock-edge bands they overlap.
    if let Some((tabs, rect)) = region
        && pos.y < rect.y + STRIP
    {
        let container = tabs.tabs.first()?.id.clone();
        let target = strip_target(tabs, rect, pos, source, container, &tab_rect, local);
        return validated(layout, source, target);
    }
    if maximized.is_none() {
        let band = |side: DockSide| -> Rect {
            let (w, h) = (root.width / 4.0, root.height / 4.0);
            match side {
                DockSide::Left => Rect::new(root.x, root.y, w, root.height),
                DockSide::Right => Rect::new(root.x + root.width - w, root.y, w, root.height),
                DockSide::Top => Rect::new(root.x, root.y, root.width, h),
                DockSide::Bottom => Rect::new(root.x, root.y + root.height - h, root.width, h),
            }
        };
        let edges = [
            (pos.x - root.x, DockSide::Left),
            (root.x + root.width - pos.x, DockSide::Right),
            (pos.y - root.y, DockSide::Top),
            (root.y + root.height - pos.y, DockSide::Bottom),
        ];
        if let Some((_, side)) = edges
            .into_iter()
            .filter(|(distance, _)| *distance < EDGE)
            .min_by(|a, b| a.0.total_cmp(&b.0))
        {
            let target = DropTarget {
                placement: DockPlacement::Edge(side),
                preview: local(band(side)),
                bar: false,
            };
            return validated(layout, source, target);
        }
    }
    let (tabs, rect) = region?;
    let container = tabs.tabs.first()?.id.clone();
    let zone = zone_target(rect, pos, container, local);
    // Dropping a tab on its own container's center is not a move.
    if tabs.contains(source) && matches!(zone.placement, DockPlacement::Tabs { .. }) {
        return None;
    }
    validated(layout, source, zone)
}

/// Keep only targets that change the layout.
fn validated<K: Clone + Eq + Hash>(
    layout: &DockLayout<K>,
    source: &K,
    target: DropTarget<K>,
) -> Option<DropTarget<K>> {
    let mut probe = layout.clone();
    (probe.move_tab(source, target.placement.clone()) && probe != *layout).then_some(target)
}

fn strip_target<K: Clone + Eq>(
    tabs: &DockTabs<K>,
    rect: Rect,
    pos: Point,
    source: &K,
    container: K,
    tab_rect: &impl Fn(&K) -> Option<Rect>,
    local: impl Fn(Rect) -> Rect,
) -> DropTarget<K> {
    let others = tabs
        .tabs
        .iter()
        .filter(|tab| &tab.id != source)
        .filter_map(|tab| tab_rect(&tab.id))
        .collect::<Vec<_>>();
    let index = others.iter().filter(|r| r.center().x < pos.x).count();
    let x = match (others.get(index), others.last()) {
        (Some(next), _) => next.x - 2.0,
        (None, Some(last)) => last.x + last.width + 1.0,
        (None, None) => rect.x + 4.0,
    };
    DropTarget {
        placement: DockPlacement::Tabs {
            container,
            index: Some(index),
        },
        preview: local(Rect::new(x - 1.0, rect.y + 5.0, 3.0, STRIP - 10.0)),
        bar: true,
    }
}

fn zone_target<K>(
    rect: Rect,
    pos: Point,
    container: K,
    local: impl Fn(Rect) -> Rect,
) -> DropTarget<K> {
    let body = Rect::new(
        rect.x,
        rect.y + STRIP,
        rect.width,
        (rect.height - STRIP).max(1.0),
    );
    let u = (pos.x - body.x) / body.width.max(1.0);
    let v = (pos.y - body.y) / body.height.max(1.0);
    let side = [
        (u, DockSide::Left),
        (1.0 - u, DockSide::Right),
        (v, DockSide::Top),
        (1.0 - v, DockSide::Bottom),
    ]
    .into_iter()
    .filter(|(distance, _)| *distance < 0.25)
    .min_by(|a, b| a.0.total_cmp(&b.0))
    .map(|(_, side)| side);
    let (w, h) = (body.width / 2.0, body.height / 2.0);
    match side {
        None => DropTarget {
            placement: DockPlacement::Tabs {
                container,
                index: None,
            },
            preview: local(body),
            bar: false,
        },
        Some(side) => DropTarget {
            placement: DockPlacement::Split { container, side },
            preview: local(match side {
                DockSide::Left => Rect::new(body.x, body.y, w, body.height),
                DockSide::Right => Rect::new(body.x + w, body.y, w, body.height),
                DockSide::Top => Rect::new(body.x, body.y, body.width, h),
                DockSide::Bottom => Rect::new(body.x, body.y + h, body.width, h),
            }),
            bar: false,
        },
    }
}
