//! Role: Application node graph constraints and edit reduction.
use kivact::{Callback, Signal, signal};

use kivact_ui_creative as ui;

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
    observer: Option<Callback<kivact::ui::EditPhase>>,
) -> impl Fn(ui::EditorEvent<ui::NodeRequest>) + 'static {
    let snapshot = signal(None);
    move |event| {
        let phase = match event {
            ui::EditorEvent::Begin => {
                snapshot.set(Some((nodes.get_untracked(), links.get_untracked())));
                Some(kivact::ui::EditPhase::Begin)
            }
            ui::EditorEvent::Request(request) => {
                apply(nodes, links, request);
                None
            }
            ui::EditorEvent::Commit => {
                snapshot.set(None);
                Some(kivact::ui::EditPhase::Commit)
            }
            ui::EditorEvent::Cancel => {
                if let Some((old_nodes, old_links)) = snapshot.get_untracked() {
                    nodes.set(old_nodes);
                    links.set(old_links);
                }
                snapshot.set(None);
                Some(kivact::ui::EditPhase::Cancel)
            }
        };
        if let Some(phase) = phase
            && let Some(observer) = observer
        {
            observer.call(phase);
        }
    }
}
/// Test application adapter: owns the reducer and forwards phase observations.
pub(super) struct AppNodeEditor {
    editor: ui::NodeEditor,
}
pub(super) fn app_node_editor(
    nodes: Signal<Vec<ui::NodeData>>,
    links: Signal<Vec<ui::NodeLink>>,
    state: ui::NodeState,
) -> AppNodeEditor {
    AppNodeEditor {
        editor: ui::node_editor(nodes, links, state)
            .can_connect(move |link| can_connect(&nodes.get_untracked(), link))
            .on_edit(edits(nodes, links, None)),
    }
}
impl AppNodeEditor {
    pub(super) fn snap(mut self, value: impl kivact::view::IntoValue<bool>) -> Self {
        self.editor = self.editor.snap(value);
        self
    }
}
impl kivact::control::NodeBuilder for AppNodeEditor {
    fn node_mut(&mut self) -> &mut kivact::control::Node {
        kivact::control::NodeBuilder::node_mut(&mut self.editor)
    }
}
impl kivact::view::IntoView for AppNodeEditor {
    fn into_view(self) -> kivact::view::View {
        self.editor.into_view()
    }
}
