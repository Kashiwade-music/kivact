//! Role: Passive typed targets for node scene regions.

use crate::{NodeControlValue, NodeData, NodeLink};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum ControlKind {
    Number,
    Choice,
    Toggle,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Target {
    Item(u64),
    Socket(u64, bool, usize),
    Control(u64, usize, ControlKind),
    Link(u64, usize, u64, usize),
}
impl Target {
    pub(crate) fn control(node: &NodeData, index: usize) -> Self {
        let kind = match node.controls[index].value {
            NodeControlValue::Number { .. } => ControlKind::Number,
            NodeControlValue::Choice { .. } => ControlKind::Choice,
            NodeControlValue::Toggle(_) => ControlKind::Toggle,
        };
        Self::Control(node.id, index, kind)
    }
    pub(crate) fn link(link: NodeLink) -> Self {
        Self::Link(link.from, link.output, link.to, link.input)
    }
    pub(crate) fn as_link(self) -> Option<NodeLink> {
        if let Self::Link(a, b, c, d) = self {
            Some(NodeLink::new(a, b, c, d))
        } else {
            None
        }
    }
}
