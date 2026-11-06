//! Role: Semantics policy: a node's role, accessible name and assistive actions.
//!
//! The semantics policy: which role a node has, what it is named, and what
//! assistive technology may do with it. The accessibility tree (`a11y`)
//! maps the result onto the window's (AccessKit).
//!
//! Invariants:
//! - A node without a `Role` gets its kind's: `Text`, `TextBox`, `Image`;
//!   other nodes have none (an explicit `Generic` is none too).
//! - An `Image` (by kind or by role) without a name has none either: it is
//!   decorative, as HTML `<img alt="">` (the `kivact-ui` icon).
//!   Unless it declares actions or has scene regions: a clickable
//!   thumbnail or a drawn chart with regions stays an `Image` (unnamed),
//!   so its actions and regions keep reaching assistive technology. A
//!   test id alone does not expose an image.
//! - A `Text` node is named by its content, every other node by `Name`.
//! - A node offers no action, and neither do its scene regions, when it is
//!   disabled, not shown, or outside the shown modal. Otherwise it offers
//!   its declared actions (each once, in order), and a text input also
//!   `SetValue`.
//! - A node is exposed to assistive technology ([`Model::exposed`])
//!   when it has a role: then with that role, its name and its actions. A
//!   node without a role is not exposed (no platform node stands for
//!   it, so neither its actions nor its scene regions reach assistive
//!   technology). The accessibility tree and the test driver take the
//!   exposure from this one function.

use crate::contract::NodeType;
use kivact_types::{A11yAction, NodeId, RegionId, Role};

use crate::model::Model;
use crate::tree::node::HostNode;

/// The role of a node: its `Role`, else its kind's (`None`: no role). An
/// image without a name, actions or scene regions is decorative and has
/// none.
pub(crate) fn semantic_role(node: &HostNode, editor_present: bool) -> Option<Role> {
    let role = match (node.role(), node.kind()) {
        (Some(Role::Generic), _) => None,
        (Some(r), _) => Some(r),
        (None, _) if editor_present => Some(Role::TextBox),
        (None, NodeType::Text) => Some(Role::Text),
        (None, NodeType::Image) => Some(Role::Image),
        (None, _) => None,
    };
    match role {
        Some(Role::Image) if is_decorative(node) => None,
        r => r,
    }
}

/// An image nothing but its pixels would expose: no name, no declared
/// action (whether or not it is actionable now, so disabling it does not
/// change its role) and no scene region.
fn is_decorative(node: &HostNode) -> bool {
    node.name().is_none_or(str::is_empty)
        && node.actions().is_empty()
        && node.scenes().iter().flatten().all(|s| s.regions.is_empty())
}

/// A list-like role whose clipped rows a virtual list mounts with overscan
/// (the accessibility tree hides rows clipped out of such a container).
pub(crate) fn is_list_role(role: Role) -> bool {
    matches!(
        role,
        Role::List | Role::ListBox | Role::Menu | Role::TabList | Role::Table | Role::Tree
    )
}

/// The accessible name: a `Text` node's content, otherwise its `Name`.
pub(crate) fn accessible_name(node: &HostNode, editor_present: bool) -> &str {
    match node.kind() {
        NodeType::Text if !editor_present => node.content().unwrap_or(""),
        _ => node.name().unwrap_or(""),
    }
}

/// What assistive technology is given of a node ([`Model::exposed`]).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Exposed {
    /// The role ([`semantic_role`]; never `Generic`).
    pub(crate) role: Role,
    /// The accessible name (a `Text` node's content, otherwise its `Name`).
    pub(crate) name: String,
    /// The actions it offers now.
    pub(crate) actions: Vec<A11yAction>,
}

impl Model {
    /// How `id` is exposed to assistive technology now: with its role, name
    /// and actions when it has a role, `None` when it has none (or is
    /// unknown). A host's platform tree and the test driver both read
    /// this, so they cannot disagree.
    pub(crate) fn exposed(&self, id: NodeId) -> Option<Exposed> {
        let n = self.tree.get(id)?;
        Some(Exposed {
            role: semantic_role(n, self.editors[n.index.0].is_some())?,
            name: accessible_name(n, self.editors[n.index.0].is_some()).to_string(),
            actions: self.accessible_actions(id),
        })
    }

    /// Whether assistive technology may act on `id` (and its regions).
    fn actionable(&self, id: NodeId) -> bool {
        self.tree.get(id).is_some_and(|n| !n.disabled()) && self.tree.shown(id) && self.in_scope(id)
    }

    /// The actions `id` offers assistive technology now.
    pub(crate) fn accessible_actions(&self, id: NodeId) -> Vec<A11yAction> {
        let Some(n) = self.tree.get(id).filter(|_| self.actionable(id)) else {
            return Vec::new();
        };
        let mut out: Vec<A11yAction> = Vec::new();
        for a in n.actions() {
            if !out.contains(a) {
                out.push(*a);
            }
        }
        if self.editors[n.index.0].is_some()
            && !n.read_only()
            && !out.contains(&A11yAction::SetValue)
        {
            out.push(A11yAction::SetValue);
        }
        out
    }

    /// The actions the scene region `region` of `id` offers now.
    pub(crate) fn region_actions(&self, id: NodeId, region: RegionId) -> Vec<A11yAction> {
        if !self.actionable(id) {
            return Vec::new();
        }
        self.tree
            .get(id)
            .and_then(|n| n.region(region))
            .filter(|r| !r.semantics.disabled && !r.semantics.hidden)
            .map(|r| {
                let mut actions = r.semantics.actions.clone().unwrap_or_default();
                if r.semantics.focusable && !actions.contains(&A11yAction::Focus) {
                    actions.push(A11yAction::Focus);
                }
                actions
            })
            .unwrap_or_default()
    }
}
