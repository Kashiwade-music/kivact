# Connect nodes in a node graph

[日本語](../../../jp/creative/how-to-guide/node-graph.md)

Goal: show nodes with typed sockets and links, let the user move nodes, drag out and re-route connections, change a node's parameters and delete things, while your application keeps the graph and decides which edits are valid.

As with the other editors of the kit, `ui::node_editor(nodes, links, state)` is a view over data you own. It reports requests; you apply or refuse them. This guide assumes you finished the [tutorial](../getting-started/README.md) and read how requests work in [Edit a timeline](timeline-and-arrangement.md#how-the-contract-works).

## A working graph editor

```rust,no_run
use kivact::draw::{Point, Size};
use kivact::prelude::*;
use kivact::style::{ThemeMode, rgb};
use kivact_ui_creative as ui;

const IMAGE: &str = "Image";
const NUMBER: &str = "Number";

fn sockets_of_type(nodes: &[ui::NodeData], link: &ui::NodeLink) -> Option<(String, String)> {
    let from = nodes.iter().find(|n| n.id == link.from)?.outputs.get(link.output)?;
    let to = nodes.iter().find(|n| n.id == link.to)?.inputs.get(link.input)?;
    Some((from.type_name.clone(), to.type_name.clone()))
}

/// The app's rule: only sockets of the same type connect.
fn allowed(nodes: &[ui::NodeData], link: &ui::NodeLink) -> bool {
    sockets_of_type(nodes, link).is_some_and(|(from, to)| from == to)
}

fn initial_nodes() -> Vec<ui::NodeData> {
    let image = |label: &str| ui::NodeSocket::new(label, IMAGE, rgb(0xffd25a));
    let number = |label: &str| ui::NodeSocket::new(label, NUMBER, rgb(0x9b9ea8));
    vec![
        ui::NodeData::new(1, "Source", vec![], vec![image("Image")])
            .position(Point::new(30.0, 40.0))
            .size(Size::new(160.0, 90.0)),
        ui::NodeData::new(2, "Blur", vec![image("Image"), number("Size")], vec![image("Image")])
            .position(Point::new(280.0, 40.0))
            .size(Size::new(180.0, 140.0))
            .controls(vec![ui::NodeControl::number("Radius", 100.0, 8.0, 0.0, 64.0, 1.0)]),
        ui::NodeData::new(3, "Output", vec![image("Image")], vec![])
            .position(Point::new(560.0, 40.0))
            .size(Size::new(160.0, 90.0)),
    ]
}

fn apply(
    nodes: &mut Vec<ui::NodeData>,
    links: &mut Vec<ui::NodeLink>,
    request: ui::NodeRequest,
) {
    match request {
        ui::NodeRequest::Move(positions) => {
            for (id, position) in positions {
                if let Some(node) = nodes.iter_mut().find(|n| n.id == id) {
                    node.position = position;
                }
            }
        }
        ui::NodeRequest::Connect(link) => {
            // Nothing is checked for you: apply your own rules.
            if allowed(nodes, &link) && !links.contains(&link) {
                // An input takes one link: the new one replaces the old one.
                links.retain(|l| !(l.to == link.to && l.input == link.input));
                links.push(link);
            }
        }
        ui::NodeRequest::Disconnect(link) => links.retain(|l| *l != link),
        ui::NodeRequest::Delete { nodes: ids, links: gone } => {
            nodes.retain(|n| !ids.contains(&n.id));
            links.retain(|l| !gone.contains(l) && !ids.contains(&l.from) && !ids.contains(&l.to));
        }
        ui::NodeRequest::ToggleMute(ids) => {
            for node in nodes.iter_mut().filter(|n| ids.contains(&n.id)) {
                node.muted = !node.muted;
            }
        }
        ui::NodeRequest::Control { node, index, value } => {
            if let Some(control) = nodes
                .iter_mut()
                .find(|n| n.id == node)
                .and_then(|n| n.controls.get_mut(index))
            {
                control.value = value;
            }
        }
        // Variants added later fall back to being ignored.
        _ => {}
    }
}

fn graph_panel() -> impl IntoView {
    let nodes = signal(initial_nodes());
    let links = signal(vec![ui::NodeLink::new(1, 0, 2, 0), ui::NodeLink::new(2, 0, 3, 0)]);
    let state = ui::NodeState::new();

    // Nodes and links change together, so they share one snapshot.
    let snapshot = signal(None::<(Vec<ui::NodeData>, Vec<ui::NodeLink>)>);
    let on_edit = move |event: ui::EditorEvent<ui::NodeRequest>| match event {
        ui::EditorEvent::Begin => {
            snapshot.set(Some((nodes.get_untracked(), links.get_untracked())));
        }
        ui::EditorEvent::Request(request) => {
            nodes.update(|n| links.update(|l| apply(n, l, request)));
        }
        ui::EditorEvent::Commit => snapshot.set(None),
        ui::EditorEvent::Cancel => {
            if let Some((old_nodes, old_links)) = snapshot.update(|saved| saved.take()) {
                nodes.set(old_nodes);
                links.set(old_links);
            }
        }
        // Variants added later fall back to being ignored.
        _ => {}
    };

    ui::node_editor(nodes, links, state)
        .can_connect(move |link| nodes.with_untracked(|all| allowed(all, link)))
        .snap(true)
        .on_edit(on_edit)
        .name("Node graph")
        .style(style().height(420))
}

fn main() -> kivact::Result<()> {
    App::new("Nodes")
        .theme(ui::theme(ThemeMode::Dark))
        .run(graph_panel)
}
```

Run it and drag a node. Drag from an output socket (right edge) to a compatible input socket (left edge) to connect, and from a connected input socket away to detach or re-route a link. Select a node or link and press `Delete`. Turn the mouse wheel to zoom, and pan with the middle button or with `Space` and a drag.

## What the editor decides, and what you decide

| The editor | You |
|------------|-----|
| Draws nodes, sockets, links and the in-progress link | Store nodes and links, give each node a unique `id` |
| Highlights targets with `can_connect` and reports `Connect` only if it returns `true` | Check the rules *again* in `apply`; `Connect` reaches you without duplicate or input-replacement handling |
| Reports `Move`, `Connect`, `Disconnect`, `Delete`, `ToggleMute` and `Control` requests | Apply, constrain or ignore each one |
| Keeps selection (`NodeState::selection`, `NodeState::link`) and pan/zoom (`NodeState::transform`) as bindings | Replace those bindings with your own signals to read or persist them |

A few details that matter in practice:

- **Sockets are identified by index.** A `NodeLink::new(from, output, to, input)` names a node id and a socket index on each end. Parallel links between the same two nodes are separate links, and re-routing one does not disturb the others. Keep the socket order stable if you save graphs.
- **Types and colors are yours.** `NodeSocket::new(label, type_name, color)` carries the type as a plain string and the color of the socket and its links. The editor does not interpret the type; `can_connect` and your `apply` do.
- **A re-route is two requests inside one gesture.** Dragging a link end from one input to another sends `Disconnect` and `Connect` between a single `Begin` and `Commit`, so one undo step covers it.
- **Self-connections are never reported.**
- **`Move` positions are absolute** and computed from the positions at `Begin`, as in the timeline editors.
- **`snap(true)`** rounds how far a drag moves the nodes to multiples of 10 units on both axes. It also accepts a signal.

## Add parameters to a node

`NodeData::controls` takes a list of `NodeControl`s that the editor draws inside the node at the vertical offset you give:

```rust,no_run
use kivact_ui_creative as ui;

let controls = vec![
    ui::NodeControl::number("Radius", 100.0, 8.0, 0.0, 64.0, 1.0),
    ui::NodeControl::choice("Filter", 130.0, ["Gaussian", "Box"], 0),
    ui::NodeControl::toggle("Premultiply", 160.0, true),
];
# let _ = controls;
```

A change arrives as `NodeRequest::Control { node, index, value }` with an absolute `NodeControlValue` (`Number { value, min, max, step }`, `Choice { options, selected }` or `Toggle(bool)`). Store it back into `node.controls[index].value`, as the `apply` above does. The control values live in your node data, so they save and load with it. A number control also opens a direct-entry popover for typed values.

For a node that is only labeled text, `NodeData::parameters(["ViewLayer"])` lists display strings.

## Show a preview image in a node

`NodeData::preview(pixels)` draws an image you rendered into the node's preview area. Rendering is yours, so cache the `Pixels` and update it only when the node's inputs change; see [Show frames your application renders](show-app-rendered-images.md) for building and updating `Pixels`.

## Keep the view state

`NodeState` is a `Copy` struct of three bindings: `selection` (node ids), `link` (the selected link) and `transform` (a `ViewTransform` with the pan offset and scale). To restore the view when a document opens, create your own signals and build the struct with them:

```rust,no_run
use kivact::interaction::ViewTransform;
use kivact::prelude::*;
use kivact_ui_creative as ui;

let transform = signal(ViewTransform::default());
let selection = signal(Vec::<u64>::new());
let mut state = ui::NodeState::new();
state.transform = transform.into();
state.selection = selection.into();
# let _ = state;
```

## See also

- [Edit a timeline, dope sheet, graph or arrangement](timeline-and-arrangement.md): the same `EditorEvent` contract and snapshot pattern.
- [Show frames your application renders](show-app-rendered-images.md)
- Full API: `cargo doc -p kivact-ui-creative --open`
- The catalog's node chapter: `cargo run -p kivact-ui-creative-catalog`
