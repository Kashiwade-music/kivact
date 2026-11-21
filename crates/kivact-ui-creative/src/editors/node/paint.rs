//! Role: Paint node bodies, sockets and controls using typed scene targets.

use crate::editors::node::gesture::Target;
use crate::editors::node::gesture::{DragMode, State};
use crate::editors::node::layout as geometry;
use crate::editors::node::layout::{control, rect, socket};
use crate::editors::node::model::{NodeControlValue, NodeData};
use crate::editors::shared::targets::Scene;
use crate::editors::shared::text::label;
use kivact::draw::Draw;
use kivact::draw::Layer;
use kivact::geometry::Rect;
use kivact::geometry::Size;
use kivact::style::Transform;

fn item(
    draw: &mut Draw,
    node: &NodeData,
    selected: bool,
    scene: Option<&Scene<Target>>,
    texts: crate::editors::shared::texts::Texts,
) {
    let r = rect(node);
    draw.rounded_rect(r, 10.0, kivact::style::color::RAISED);
    draw.rounded_rect(
        Rect::new(r.x, r.y, r.width, 26.0),
        8.0,
        node.color.with_alpha(0.18),
    );
    if selected {
        draw.stroke_rect(r, 1.5, crate::color::ACCENT);
    }
    label(
        draw,
        &node.label,
        r.x + 10.0,
        r.y + 8.0,
        10.5,
        crate::color::TEXT_HIGH,
    );
    if let Some(id) = scene.and_then(|s| s.id(&Target::Item(node.id))) {
        draw.region(
            id,
            Rect::new(
                r.x,
                r.y,
                r.width,
                crate::editors::shared::metrics::NODE_HEADER,
            ),
        );
    }
    sockets(draw, node, r, scene);
    parameters(draw, node, r);
    controls(draw, node, scene, &texts);
    preview(draw, node, r);
    if node.muted {
        draw.line(
            kivact::geometry::Point::new(r.x, r.y + r.height),
            kivact::geometry::Point::new(r.x + r.width, r.y),
            2.0,
            crate::color::X,
        );
    }
}

/// The sockets of a node, on its left (inputs) and right (outputs) edges.
fn sockets(draw: &mut Draw, node: &NodeData, r: Rect, scene: Option<&Scene<Target>>) {
    for (outgoing, sockets) in [(false, &node.inputs), (true, &node.outputs)] {
        for (index, entry) in sockets.iter().enumerate() {
            let point = socket(node, outgoing, index);
            draw.circle(point, 4.5, entry.color);
            let x = if outgoing {
                r.x + r.width - 10.0 - entry.label.len() as f64 * 4.8
            } else {
                r.x + 11.0
            };
            label(
                draw,
                &entry.label,
                x,
                point.y - 5.0,
                9.5,
                crate::color::MUTED,
            );
            if let Some(id) = scene.and_then(|s| s.id(&Target::Socket(node.id, outgoing, index))) {
                draw.region(id, Rect::new(point.x - 9.0, point.y - 9.0, 18.0, 18.0));
            }
        }
    }
}

/// The parameter rows that fit under the sockets.
fn parameters(draw: &mut Draw, node: &NodeData, r: Rect) {
    let start = 58.0
        + node.inputs.len().max(node.outputs.len()) as f64
            * crate::editors::shared::metrics::SOCKET_STEP;
    for (index, parameter) in node.parameters.iter().enumerate() {
        let y = r.y + start + index as f64 * 27.0;
        if y + 22.0 > r.y + r.height {
            break;
        }
        draw.rounded_rect(
            Rect::new(r.x + 10.0, y, r.width - 20.0, 21.0),
            5.0,
            crate::color::FIELD,
        );
        label(
            draw,
            parameter,
            r.x + 16.0,
            y + 5.0,
            10.0,
            crate::color::TEXT,
        );
    }
}

/// The controls of a node: each shows its value and is a region of its own.
fn controls(
    draw: &mut Draw,
    node: &NodeData,
    scene: Option<&Scene<Target>>,
    texts: &crate::editors::shared::texts::Texts,
) {
    for (index, entry) in node.controls.iter().enumerate() {
        let r = control(node, index).expect("enumerated control");
        draw.rounded_rect(r, 5.0, crate::color::FIELD);
        let value = match &entry.value {
            NodeControlValue::Number { value, .. } => format!("{value:.3}"),
            NodeControlValue::Choice { options, selected } => {
                options.get(*selected).cloned().unwrap_or_default()
            }
            NodeControlValue::Toggle(value) => texts.toggle(*value),
        };
        label(
            draw,
            &format!("{}  {}", entry.label, value),
            r.x + 6.0,
            r.y + 4.0,
            10.0,
            crate::color::TEXT,
        );
        if let Some(id) = scene.and_then(|s| s.id(&Target::control(node, index))) {
            draw.region(id, r);
        }
    }
}

pub(crate) fn editor(draw: &mut kivact::draw::Draw, size: Size, state: State) {
    for x in (0..size.width.max(0.0) as usize).step_by(20) {
        for y in (0..size.height.max(0.0) as usize).step_by(20) {
            draw.rounded_rect(
                Rect::new(x as f64, y as f64, 2.0, 2.0),
                1.0,
                crate::color::HAIR,
            );
        }
    }
    let transform = state.transform.get();
    let layer = Layer::new().transform(
        Transform::new()
            .translate(transform.offset.x, transform.offset.y)
            .scale(transform.scale),
    );
    let nodes = state.data().get();
    let links = state.links.get();
    let scene = state.scene.get().ok();
    draw.layer(layer, |draw| {
        for link in links {
            if state.rewired() == Some(link) {
                continue;
            }
            if let Some((from, to)) = geometry::endpoints(&nodes, link) {
                geometry::draw_link(
                    draw,
                    from,
                    to,
                    if state.selected_link.get() == Some(link) {
                        crate::color::ACCENT
                    } else {
                        crate::color::DIM
                    },
                );
                if let Some(id) = scene.as_deref().and_then(|s| s.id(&Target::link(link))) {
                    let point = geometry::curve(from, to, 0.5);
                    draw.region(id, Rect::new(point.x - 6.0, point.y - 6.0, 12.0, 12.0));
                }
            }
        }
        for node in &nodes {
            item(
                draw,
                node,
                state.selected.get().contains(&node.id),
                scene.as_deref(),
                state.texts,
            );
        }
        if let Some(DragMode::Link { source, to, .. }) = state.drag_state()
            && let (id, index) = source.output()
            && let Some(node) = nodes.iter().find(|n| n.id == id)
        {
            geometry::draw_link(
                draw,
                geometry::socket(node, true, index),
                to,
                crate::color::ACCENT,
            );
            for candidate in &nodes {
                for input in 0..candidate.inputs.len() {
                    let point = geometry::socket(candidate, false, input);
                    if (point.x - to.x).abs() < 15.0
                        && (point.y - to.y).abs() < 15.0
                        && state.valid_link(crate::editors::node::model::NodeLink::new(
                            id,
                            index,
                            candidate.id,
                            input,
                        ))
                    {
                        draw.circle(point, 7.0, crate::color::ACCENT);
                    }
                }
            }
        }
        if let Some(DragMode::Marquee { from, to }) = state.drag_state() {
            draw.stroke_rect(Rect::from_points(from, to), 1.0, crate::color::ACCENT);
        }
    });
}

fn preview(draw: &mut Draw, node: &NodeData, r: Rect) {
    if let Some(image) = &node.preview {
        let preview = Rect::new(r.x + 10.0, r.y + r.height - 65.0, r.width - 20.0, 55.0);
        let scale = (preview.width / image.width as f64).min(preview.height / image.height as f64);
        let width = image.width as f64 * scale;
        let height = image.height as f64 * scale;
        draw.pixels(
            image,
            Rect::new(
                preview.x + (preview.width - width) / 2.0,
                preview.y + (preview.height - height) / 2.0,
                width,
                height,
            ),
            kivact::draw::ImageOptions::default(),
        );
    }
}
