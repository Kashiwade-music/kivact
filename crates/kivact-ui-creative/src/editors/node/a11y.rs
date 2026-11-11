//! Role: Node region targets and accessible semantic descriptions.
use super::gesture::Target;
use super::gesture::{Facet, Info};
use super::model::{NodeControlValue, NodeData, NodeLink};
use crate::editors::shared::targets::Scene;
use kivact::UiError;
use kivact::view::Role;
use std::collections::HashSet;
pub(crate) fn targets(nodes: &[NodeData], links: &[NodeLink]) -> Result<Vec<Target>, UiError> {
    let mut ids = HashSet::new();
    let mut targets = Vec::new();
    for node in nodes {
        if !ids.insert(node.id) {
            return Err(UiError::msg(format!("duplicate node item id {}", node.id)));
        }
        targets.push(Target::Item(node.id));
        for (outgoing, count) in [(false, node.inputs.len()), (true, node.outputs.len())] {
            targets.extend((0..count).map(|index| Target::Socket(node.id, outgoing, index)));
        }
        targets.extend((0..node.controls.len()).map(|index| Target::control(node, index)));
    }
    let mut edges = HashSet::new();
    for link in links {
        if ids.contains(&link.from) && ids.contains(&link.to) {
            let target = Target::link(*link);
            if !edges.insert(target) {
                return Err(UiError::msg("duplicate node link"));
            }
            targets.push(target);
        }
    }
    Ok(targets)
}
pub(crate) fn semantic(
    nodes: Vec<NodeData>,
    links: Vec<NodeLink>,
    selection: Vec<u64>,
    selected_link: Option<NodeLink>,
    scene: Option<&Scene<Target>>,
    texts: crate::editors::shared::texts::Texts,
) -> Vec<Info> {
    let mut result = Vec::new();
    let Some(scene) = scene else { return result };
    let info = |target, name, value, role, facet| {
        scene.id(&target).map(|id| Info {
            id,
            target,
            name,
            value,
            role,
            facet,
        })
    };
    for node in &nodes {
        result.extend(info(
            Target::Item(node.id),
            node.label.clone(),
            texts.selection(selection.contains(&node.id)),
            Role::Button,
            Facet::Plain,
        ));
        for (outgoing, sockets) in [(false, &node.inputs), (true, &node.outputs)] {
            for (index, socket) in sockets.iter().enumerate() {
                result.extend(info(
                    Target::Socket(node.id, outgoing, index),
                    texts.socket(outgoing, &node.label, &socket.label),
                    socket.type_name.clone(),
                    Role::Button,
                    Facet::Plain,
                ));
            }
        }
        for (index, control) in node.controls.iter().enumerate() {
            let (role, value, facet) = match &control.value {
                NodeControlValue::Number {
                    value, min, max, ..
                } => (
                    Role::SpinButton,
                    format!("{value:.3}"),
                    Facet::Range {
                        min: *min,
                        max: *max,
                        now: *value,
                    },
                ),
                NodeControlValue::Choice { options, selected } => (
                    Role::Button,
                    options.get(*selected).cloned().unwrap_or_default(),
                    Facet::Plain,
                ),
                NodeControlValue::Toggle(value) => (
                    Role::Button,
                    texts.toggle(*value),
                    Facet::Toggle { checked: *value },
                ),
            };
            result.extend(info(
                Target::control(node, index),
                format!("{} {}", node.label, control.label),
                value,
                role,
                facet,
            ));
        }
    }
    result.extend(link_info(&nodes, links, selected_link, scene, texts));
    result
}

fn link_info(
    nodes: &[NodeData],
    links: Vec<NodeLink>,
    selected_link: Option<NodeLink>,
    scene: &Scene<Target>,
    texts: crate::editors::shared::texts::Texts,
) -> Vec<Info> {
    let mut result = Vec::new();
    let info = |target, name, value, role, facet| {
        scene.id(&target).map(|id| Info {
            id,
            target,
            name,
            value,
            role,
            facet,
        })
    };
    for link in links {
        let Some(from) = nodes.iter().find(|n| n.id == link.from) else {
            continue;
        };
        let Some(to) = nodes.iter().find(|n| n.id == link.to) else {
            continue;
        };
        result.extend(info(
            Target::link(link),
            texts.port_link(&from.label, link.output, &to.label, link.input),
            String::new(),
            Role::Button,
            Facet::Link {
                selected: selected_link == Some(link),
            },
        ));
    }
    result
}
