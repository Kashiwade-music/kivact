//! Role: Validates portable accessibility requests against the accepted hierarchy.

use super::state::AcceptedTree;
use crate::events::ShellEvent;
use kivact_types::{A11yAction, A11yRequest, SemanticsId};

fn normalized(action: A11yAction) -> A11yAction {
    match action {
        A11yAction::Activate | A11yAction::Toggle | A11yAction::Select => A11yAction::Activate,
        other => other,
    }
}

pub(super) fn accept(tree: &AcceptedTree, node: u64, request: A11yRequest) -> Option<ShellEvent> {
    let id = SemanticsId(node);
    let target = tree.node(id)?;
    let action = normalized(request.action());
    let supported = if action == A11yAction::Focus {
        target.semantics.focusable
    } else {
        target
            .semantics
            .actions
            .iter()
            .flatten()
            .any(|offered| normalized(*offered) == action)
    };
    if !supported {
        return None;
    }
    let mut at = Some(id);
    while let Some(id) = at {
        let node = tree.node(id)?;
        let retained_focus =
            id == target.id && action == A11yAction::Focus && node.semantics.focusable;
        if node.semantics.hidden || (node.semantics.disabled && !retained_focus) {
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
