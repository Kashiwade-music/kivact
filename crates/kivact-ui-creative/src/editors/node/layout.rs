//! Role: Node geometry and typed-link shape in logical scene coordinates.

use crate::editors::shared::metrics::{SOCKET_START, SOCKET_STEP};
use crate::{NodeData, NodeLink};
use kivact::draw::{Draw, Path};
use kivact::geometry::{Point, Rect};
use kivact::style::Color;

pub(crate) fn rect(node: &NodeData) -> Rect {
    Rect::new(
        node.position.x,
        node.position.y,
        node.size.width,
        node.size.height,
    )
}
pub(crate) fn socket(node: &NodeData, outgoing: bool, index: usize) -> Point {
    let sockets = if outgoing {
        &node.outputs
    } else {
        &node.inputs
    };
    let y = sockets
        .get(index)
        .and_then(|s| s.offset)
        .unwrap_or(SOCKET_START + index as f64 * SOCKET_STEP);
    Point::new(
        node.position.x + if outgoing { node.size.width } else { 0.0 },
        node.position.y + y,
    )
}
pub(crate) fn control(node: &NodeData, index: usize) -> Option<Rect> {
    let control = node.controls.get(index)?;
    Some(Rect::new(
        node.position.x + 10.0,
        node.position.y + control.offset,
        node.size.width - 20.0,
        21.0,
    ))
}
pub(crate) fn endpoints(nodes: &[NodeData], link: NodeLink) -> Option<(Point, Point)> {
    let from = nodes.iter().find(|n| n.id == link.from)?;
    let to = nodes.iter().find(|n| n.id == link.to)?;
    from.outputs.get(link.output)?;
    to.inputs.get(link.input)?;
    Some((
        socket(from, true, link.output),
        socket(to, false, link.input),
    ))
}
pub(crate) fn curve(from: Point, to: Point, t: f64) -> Point {
    let dx = (to.x - from.x).abs().max(80.0) * 0.5;
    let q = 1.0 - t;
    Point::new(
        q * q * q * from.x
            + 3.0 * q * q * t * (from.x + dx)
            + 3.0 * q * t * t * (to.x - dx)
            + t * t * t * to.x,
        q * q * q * from.y + 3.0 * q * q * t * from.y + 3.0 * q * t * t * to.y + t * t * t * to.y,
    )
}
pub(crate) fn draw_link(draw: &mut Draw, from: Point, to: Point, color: Color) {
    let dx = (to.x - from.x).abs().max(80.0) * 0.5;
    draw.stroke_path(
        &Path::new().move_to(from).cubic_to(
            Point::new(from.x + dx, from.y),
            Point::new(to.x - dx, to.y),
            to,
        ),
        &kivact::draw::Stroke::new(2.0),
        color,
    );
}
pub(crate) fn hit_link(nodes: &[NodeData], links: &[NodeLink], point: Point) -> Option<NodeLink> {
    links.iter().copied().find(|link| {
        let Some((from, to)) = endpoints(nodes, *link) else {
            return false;
        };
        let mut previous = from;
        for index in 1..=40 {
            let next = curve(from, to, index as f64 / 40.0);
            let dx = next.x - previous.x;
            let dy = next.y - previous.y;
            let squared = dx * dx + dy * dy;
            let t = if squared > 0.0 {
                ((point.x - previous.x) * dx + (point.y - previous.y) * dy) / squared
            } else {
                0.0
            };
            let t = t.clamp(0.0, 1.0);
            if (point.x - previous.x - dx * t).hypot(point.y - previous.y - dy * t) <= 6.0 {
                return true;
            }
            previous = next;
        }
        false
    })
}

pub(crate) fn bounds(nodes: &[NodeData]) -> Rect {
    let Some(first) = nodes.first() else {
        return Rect::new(0.0, 0.0, 0.0, 0.0);
    };
    let (mut left, mut top, mut right, mut bottom) = (
        first.position.x,
        first.position.y,
        first.position.x + first.size.width,
        first.position.y + first.size.height,
    );
    for node in nodes {
        left = left.min(node.position.x);
        top = top.min(node.position.y);
        right = right.max(node.position.x + node.size.width);
        bottom = bottom.max(node.position.y + node.size.height);
    }
    Rect::new(left, top, right - left, bottom - top)
}
