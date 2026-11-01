//! Role: Validates portable accessibility requests against the accepted hierarchy.

use super::state::AcceptedTree;
use crate::events::ShellEvent;
use ui_types::{A11yAction, NodeId};

fn normalized(action: A11yAction) -> A11yAction {
    match action {
        A11yAction::Activate | A11yAction::Toggle | A11yAction::Select => A11yAction::Activate,
        other => other,
    }
}

pub(super) fn accept(
    tree: &AcceptedTree,
    node: u64,
    action: A11yAction,
    value: Option<&str>,
) -> Option<ShellEvent> {
    let id = NodeId(node);
    let target = tree.node(id)?;
    let action = normalized(action);
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
        action,
        value: if action == A11yAction::Dismiss {
            None
        } else {
            value.map(str::to_owned)
        },
    })
}
