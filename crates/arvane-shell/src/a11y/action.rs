//! Role: Validates portable accessibility requests against the accepted hierarchy.

use super::state::AcceptedTree;
use crate::events::ShellEvent;
use kivact_types::{A11yAction, A11yRequest, NodeId};

fn normalized(action: A11yAction) -> A11yAction {
    match action {
        A11yAction::Activate | A11yAction::Toggle | A11yAction::Select => A11yAction::Activate,
        other => other,
    }
}

pub(super) fn accept(tree: &AcceptedTree, node: u64, request: A11yRequest) -> Option<ShellEvent> {
    let id = NodeId(node);
    let target = tree.node(id)?;
    let action = normalized(request.action());
    let supported = (action == A11yAction::Focus && target.focusable)
        || target
            .actions
            .iter()
            .any(|offered| normalized(*offered) == action);
    if !supported {
        return None;
    }
    let mut at = Some(id);
    while let Some(id) = at {
        let node = tree.node(id)?;
        if node.hidden || node.disabled {
            return None;
        }
        at = tree.parent(id);
    }
    Some(ShellEvent::Accessibility {
        node,
        request: match request {
            A11yRequest::Toggle | A11yRequest::Select => A11yRequest::Activate,
            other => other,
        },
    })
}
