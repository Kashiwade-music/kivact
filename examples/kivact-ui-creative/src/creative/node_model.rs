//! Role: Application node graph constraints and edit reduction.
use super::editor_images as images;
use kivact::geometry::{Point, Size};
use kivact::state::{Signal, signal};
use kivact::style::rgb;
use kivact_ui_creative as ui;
const IMAGE_TYPE: &str = "Image";
const SCALAR_TYPE: &str = "Scalar";
const VECTOR_TYPE: &str = "Vector";
const IMAGE_COLOR: u32 = 0xffd25a;
const SCALAR_COLOR: u32 = 0x9b9ea8;
const VECTOR_COLOR: u32 = 0x8b7cff;
pub(super) fn add(nodes: Signal<Vec<ui::NodeData>>, label: &str) {
    nodes.update(|nodes| {
        let id = nodes.iter().map(|node| node.id).max().unwrap_or(0) + 1;
        nodes.push(editor_node(id, label, 520.0, 30.0, 0x8b7cff));
    });
}
pub(super) fn can_connect(nodes: &[ui::NodeData], link: &ui::NodeLink) -> bool {
    let Some(a) = nodes
        .iter()
        .find(|n| n.id == link.from)
        .and_then(|n| n.outputs.get(link.output))
    else {
        return false;
    };
    let Some(b) = nodes
        .iter()
        .find(|n| n.id == link.to)
        .and_then(|n| n.inputs.get(link.input))
    else {
        return false;
    };
    a.type_name == b.type_name
        || matches!(
            (a.type_name.as_str(), b.type_name.as_str()),
            ("Image", "Color") | ("Color", "Image")
        )
}
pub(super) fn apply(
    nodes: Signal<Vec<ui::NodeData>>,
    links: Signal<Vec<ui::NodeLink>>,
    request: ui::NodeRequest,
) {
    match request {
        ui::NodeRequest::Move(positions) => nodes.update(|nodes| {
            for node in nodes {
                if let Some((_, p)) = positions.iter().find(|(id, _)| *id == node.id) {
                    node.position = *p;
                }
            }
        }),
        ui::NodeRequest::Control { node, index, value } => nodes.update(|nodes| {
            if let Some(control) = nodes
                .iter_mut()
                .find(|n| n.id == node)
                .and_then(|n| n.controls.get_mut(index))
            {
                control.value = value;
            }
        }),
        ui::NodeRequest::Connect(link) => {
            if can_connect(&nodes.get_untracked(), &link) {
                links.update(|links| {
                    if !links.contains(&link) {
                        links.retain(|l| l.to != link.to || l.input != link.input);
                        links.push(link);
                    }
                });
            }
        }
        ui::NodeRequest::Disconnect(link) => links.update(|links| links.retain(|l| *l != link)),
        ui::NodeRequest::Delete {
            nodes: ids,
            links: edges,
        } => {
            nodes.update(|nodes| nodes.retain(|n| !ids.contains(&n.id)));
            links.update(|links| {
                links.retain(|l| {
                    !edges.contains(l) && !ids.contains(&l.from) && !ids.contains(&l.to)
                })
            });
        }
        ui::NodeRequest::ToggleMute(ids) => nodes.update(|nodes| {
            for node in nodes {
                if ids.contains(&node.id) {
                    node.muted = !node.muted;
                }
            }
        }),
    }
}
pub(super) fn edits(
    nodes: Signal<Vec<ui::NodeData>>,
    links: Signal<Vec<ui::NodeLink>>,
) -> impl Fn(ui::EditorEvent<ui::NodeRequest>) + 'static {
    let snapshot = signal(None);
    move |event| match event {
        ui::EditorEvent::Begin => {
            snapshot.set(Some((nodes.get_untracked(), links.get_untracked())))
        }
        ui::EditorEvent::Request(request) => apply(nodes, links, request),
        ui::EditorEvent::Commit => snapshot.set(None),
        ui::EditorEvent::Cancel => {
            if let Some((old_nodes, old_links)) = snapshot.get_untracked() {
                nodes.set(old_nodes);
                links.set(old_links);
            }
            snapshot.set(None);
        }
    }
}

pub(super) fn editor_node(id: u64, label: &str, x: f64, y: f64, color: u32) -> ui::NodeData {
    let socket = ui::NodeSocket::new;
    let model = match label {
        "Render Layers" => ui::NodeData::new(
            id,
            label,
            vec![],
            vec![
                socket("Image", IMAGE_TYPE, rgb(IMAGE_COLOR)),
                socket("Alpha", SCALAR_TYPE, rgb(SCALAR_COLOR)),
                socket("Depth", SCALAR_TYPE, rgb(SCALAR_COLOR)),
                socket("Normal", VECTOR_TYPE, rgb(VECTOR_COLOR)),
            ],
        )
        .parameters(["ViewLayer"]),
        "Blur" => ui::NodeData::new(
            id,
            label,
            vec![
                socket("Image", IMAGE_TYPE, rgb(IMAGE_COLOR)).at(98.0),
                socket("Size", SCALAR_TYPE, rgb(SCALAR_COLOR)).at(122.0),
            ],
            vec![socket("Image", IMAGE_TYPE, rgb(IMAGE_COLOR))],
        )
        .controls(vec![
            ui::NodeControl::choice(
                "Filter",
                60.0,
                ["Gaussian", "Fast Gaussian", "Box", "Tent"],
                0,
            ),
            ui::NodeControl::number("X", 144.0, 10.0, 0.0, 1000.0, 1.0),
        ]),
        "Image" => ui::NodeData::new(
            id,
            label,
            vec![],
            vec![
                socket("Image", IMAGE_TYPE, rgb(IMAGE_COLOR)),
                socket("Alpha", SCALAR_TYPE, rgb(SCALAR_COLOR)),
            ],
        )
        .parameters(["Hero_Plate.exr"])
        .preview(images::gradient(80, 40, 0, false)),
        "Color Balance" => ui::NodeData::new(
            id,
            label,
            vec![
                socket("Fac", SCALAR_TYPE, rgb(SCALAR_COLOR)).at(82.0),
                socket("Image", IMAGE_TYPE, rgb(IMAGE_COLOR)).at(108.0),
            ],
            vec![socket("Image", IMAGE_TYPE, rgb(IMAGE_COLOR))],
        )
        .controls(vec![
            ui::NodeControl::choice(
                "Method",
                58.0,
                ["Lift / Gamma / Gain", "Offset / Power / Slope"],
                0,
            ),
            ui::NodeControl::number("Factor", 132.0, 1.0, 0.0, 1.0, 0.01),
        ]),
        "Keying" => ui::NodeData::new(
            id,
            label,
            vec![socket("Image", IMAGE_TYPE, rgb(IMAGE_COLOR)).at(96.0)],
            vec![
                socket("Image", IMAGE_TYPE, rgb(IMAGE_COLOR)),
                socket("Matte", SCALAR_TYPE, rgb(SCALAR_COLOR)),
            ],
        )
        .parameters(["Screen Color"]),
        "Mix" => ui::NodeData::new(
            id,
            label,
            vec![
                socket("Fac", SCALAR_TYPE, rgb(SCALAR_COLOR)).at(114.0),
                socket("Image A", IMAGE_TYPE, rgb(IMAGE_COLOR)).at(140.0),
                socket("Image B", IMAGE_TYPE, rgb(IMAGE_COLOR)).at(164.0),
            ],
            vec![socket("Image", IMAGE_TYPE, rgb(IMAGE_COLOR))],
        )
        .controls(vec![
            ui::NodeControl::choice("Mode", 58.0, ["Screen", "Mix", "Add", "Multiply"], 0),
            ui::NodeControl::toggle("Use Alpha", 83.0, true),
            ui::NodeControl::number("Factor", 108.0, 0.5, 0.0, 1.0, 0.01),
        ]),
        "Viewer" => ui::NodeData::new(
            id,
            label,
            vec![
                socket("Image", IMAGE_TYPE, rgb(IMAGE_COLOR)).at(142.0),
                socket("Alpha", SCALAR_TYPE, rgb(SCALAR_COLOR)).at(166.0),
            ],
            vec![],
        )
        .preview(images::gradient(80, 40, 0, false)),
        _ => ui::NodeData::new(
            id,
            label,
            vec![
                socket("Image", IMAGE_TYPE, rgb(IMAGE_COLOR)),
                socket("Alpha", SCALAR_TYPE, rgb(SCALAR_COLOR)),
                socket("Z", SCALAR_TYPE, rgb(SCALAR_COLOR)),
            ],
            vec![],
        )
        .controls(vec![ui::NodeControl::number(
            "Z", 126.0, 0.0, 0.0, 1000.0, 0.1,
        )]),
    };
    model
        .position(Point::new(x, y))
        .size(Size::new(
            180.0,
            if label == "Image" {
                215.0
            } else if label == "Viewer" {
                180.0
            } else if label == "Mix" {
                194.0
            } else {
                170.0
            },
        ))
        .color(rgb(color))
}
