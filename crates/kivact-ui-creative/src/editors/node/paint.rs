//! Role: Paint node bodies, sockets and controls using typed scene targets.

use crate::editors::node::gesture::Target;
use crate::editors::node::gesture::{DragMode, State};
use crate::editors::node::layout as geometry;
use crate::editors::node::layout::{control, rect, socket};
use crate::editors::node::model::{NodeControlValue, NodeData};
use crate::editors::shared::targets::Scene;
use crate::editors::shared::text::label;
use kivact::draw::{Draw, Rect};
use kivact::draw::{Layer, Size, Transform};
use kivact::style::ColorToken;

fn item(
    draw: &mut Draw,
    node: &NodeData,
    selected: bool,
    scene: Option<&Scene<Target>>,
    texts: crate::editors::shared::texts::Texts,
) {
    let p = draw.theme().get::<crate::tokens::Tokens>();
    let r = rect(node);
    draw.rounded_rect(r, 10.0, ColorToken::Raised);
    draw.rounded_rect(
        Rect::new(r.x, r.y, r.width, 26.0),
        8.0,
        node.color.with_alpha(0.18),
    );
    if selected {
        draw.stroke_rect(r, 1.5, p.accent);
    }
    label(draw, &node.label, r.x + 10.0, r.y + 8.0, 10.5, p.text_high);
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
    for (outgoing, sockets) in [(false, &node.inputs), (true, &node.outputs)] {
        for (index, entry) in sockets.iter().enumerate() {
            let point = socket(node, outgoing, index);
            draw.circle(point, 4.5, entry.color);
            let x = if outgoing {
                r.x + r.width - 10.0 - entry.label.len() as f32 * 4.8
            } else {
                r.x + 11.0
            };
            label(draw, &entry.label, x, point.y - 5.0, 9.5, p.muted);
            if let Some(id) = scene.and_then(|s| s.id(&Target::Socket(node.id, outgoing, index))) {
                draw.region(id, Rect::new(point.x - 9.0, point.y - 9.0, 18.0, 18.0));
            }
        }
    }
    let start = 58.0
        + node.inputs.len().max(node.outputs.len()) as f32
            * crate::editors::shared::metrics::SOCKET_STEP;
    for (index, parameter) in node.parameters.iter().enumerate() {
        let y = r.y + start + index as f32 * 27.0;
        if y + 22.0 > r.y + r.height {
            break;
        }
        draw.rounded_rect(Rect::new(r.x + 10.0, y, r.width - 20.0, 21.0), 5.0, p.field);
        label(draw, parameter, r.x + 16.0, y + 5.0, 10.0, p.text);
    }
    for (index, entry) in node.controls.iter().enumerate() {
        let r = control(node, index).expect("enumerated control");
        draw.rounded_rect(r, 5.0, p.field);
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
            p.text,
        );
        if let Some(id) = scene.and_then(|s| s.id(&Target::control(node, index))) {
            draw.region(id, r);
        }
    }
    if let Some(image) = &node.preview {
        let preview = Rect::new(r.x + 10.0, r.y + r.height - 65.0, r.width - 20.0, 55.0);
        let scale = (preview.width / image.width as f32).min(preview.height / image.height as f32);
        let width = image.width as f32 * scale;
        let height = image.height as f32 * scale;
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
    if node.muted {
        draw.line(
            kivact::draw::Point::new(r.x, r.y + r.height),
            kivact::draw::Point::new(r.x + r.width, r.y),
            2.0,
            p.x,
        );
    }
}

pub(crate) fn editor(draw: &mut kivact::draw::Draw, size: Size, state: State) {
    let p = draw.theme().get::<crate::tokens::Tokens>();
    for x in (0..size.width.max(0.0) as usize).step_by(20) {
        for y in (0..size.height.max(0.0) as usize).step_by(20) {
            draw.rounded_rect(Rect::new(x as f32, y as f32, 2.0, 2.0), 1.0, p.hair);
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
                        p.accent
                    } else {
                        p.dim
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
        if let Some(DragMode::Link {
            from: id,
            output: index,
            to,
            ..
        }) = state.drag_state()
            && let Some(node) = nodes.iter().find(|n| n.id == id)
        {
            geometry::draw_link(draw, geometry::socket(node, true, index), to, p.accent);
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
                        draw.circle(point, 7.0, p.accent);
                    }
                }
            }
        }
        if let Some(DragMode::Marquee { from, to }) = state.drag_state() {
            draw.stroke_rect(Rect::from_points(from, to), 1.0, p.accent);
        }
    });
}
