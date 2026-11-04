//! Role: Query and element selection API.
//!
//! Queries (`get_by_*`, `query_by_*`, `find_by_*`, `all_by_*`), the
//! failure report, and subtree-scoped `Within` queries.
//!
//! Queries by role match the accessibility tree the host published (what
//! assistive technology sees: the host's role and name policy, scene
//! regions); queries by text and test id match the retained tree. Either
//! way, only nodes outside hidden subtrees match; rows a list scrolled out
//! of view still do (as a Playwright locator finds off-screen elements).
//! Props and test ids are read from the engine's tree.
//!
//! An `Element`'s name, role, value, states and `a11y_actions` are read
//! from the published tree (a node assistive technology does not see has
//! the role `None`). `Element::rect` is the bounding box of the drawn box
//! (`NodeRef::rect`'s); `Element::accessibility_bounds` is the published
//! bounds mapped by scrolling and transforms as the platform maps them;
//! `children()` projects the retained tree through structural Contents boxes.

use std::fmt::Write as _;
use std::panic::Location;

use kivact_engine::contract::NodeType;
use kivact_types::NodeId;

use super::model::{Element, FIND_TIMEOUT, Query, Screen, Within};
use kivact_core::view::Role;
use kivact_runtime::TurnRecord;
use kivact_style::draw::RegionId;

impl Screen {
    fn host_nodes(&self, scope: Option<NodeId>) -> Vec<NodeId> {
        let tree = self.facts();
        let mut out = Vec::new();
        let start = scope.unwrap_or(tree.root);
        let mut stack: Vec<NodeId> = vec![start];
        while let Some(id) = stack.pop() {
            let Some(n) = tree.get(id) else { continue };
            if tree.hidden(id) == Some(true) {
                continue;
            }
            out.push(id);
            stack.extend(n.children.iter().rev().copied());
        }
        out
    }

    pub(super) fn name_of(&self, id: NodeId) -> String {
        let tree = self.facts();
        match (tree.name(id), tree.content(id)) {
            (Some(s), _) | (None, Some(s)) => s,
            _ => String::new(),
        }
    }

    pub(super) fn role_of(&self, id: NodeId) -> Option<Role> {
        self.facts().role(id)
    }

    fn matches(&self, q: &Query, scope: Option<NodeId>) -> Vec<(NodeId, Option<RegionId>)> {
        let visible = self.host_nodes(scope);
        if let Query::Role(role, name) = q {
            return self.role_matches(*role, name, &visible);
        }
        let mut out = Vec::new();
        let tree = self.facts();
        for id in visible {
            let Some(n) = tree.get(id) else { continue };
            match q {
                Query::Text(t) => {
                    if n.kind == Some(NodeType::Text) && tree.content(id).as_ref() == Some(t) {
                        out.push((id, None));
                    }
                }
                Query::TestId(t) => {
                    if tree.test_id(id).as_ref() == Some(t) {
                        out.push((id, None));
                    }
                }
                Query::Role(..) => {}
            }
        }
        out
    }

    /// The nodes and regions of `visible` the published accessibility tree
    /// exposes with `role` and `name`, in tree order.
    fn role_matches(
        &self,
        role: Role,
        name: &str,
        visible: &[NodeId],
    ) -> Vec<(NodeId, Option<RegionId>)> {
        let visible: std::collections::HashSet<NodeId> = visible.iter().copied().collect();
        let wire = role;
        let host = self.host.borrow();
        let Some(tree) = host.semantics() else {
            return Vec::new();
        };
        let mut out = Vec::new();
        let mut stack = vec![tree.root];
        while let Some(id) = stack.pop() {
            let Some(n) = tree.nodes.iter().find(|node| node.id == id) else {
                continue;
            };
            stack.extend(n.children.iter().rev().copied());
            // A node without a role is published as a bare structural node.
            if n.semantics.name.clone().unwrap_or_default() != name
                || n.semantics.role.unwrap_or(kivact_types::Role::Generic) != wire
            {
                continue;
            }
            if let Some((node, region)) = host.semantics_target(n.id.0)
                && visible.contains(&node)
            {
                out.push((node, region));
            }
        }
        out
    }

    #[track_caller]
    pub(super) fn get(&self, q: Query, scope: Option<NodeId>) -> Element<'_> {
        let found = self.matches(&q, scope);
        match found.as_slice() {
            [(node, region)] => Element {
                screen: self,
                node: *node,
                region: *region,
            },
            [] => self.fail(&self.report(&format!("no element matches {q}"), scope, &found)),
            _ => self.fail(&self.report(
                &format!("{} elements match {q} (ambiguous)", found.len()),
                scope,
                &found,
            )),
        }
    }

    fn query(&self, q: Query, scope: Option<NodeId>) -> Option<Element<'_>> {
        let found = self.matches(&q, scope);
        (found.len() == 1).then(|| Element {
            screen: self,
            node: found[0].0,
            region: found[0].1,
        })
    }

    /// Wait (advancing virtual time a frame at a time, at most 5 s) until
    /// exactly one element matches.
    #[track_caller]
    fn find(&self, q: Query, scope: Option<NodeId>) -> Element<'_> {
        let start = self.now_ns();
        let limit = FIND_TIMEOUT.as_nanos() as u64;
        loop {
            self.run_until_idle();
            let found = self.matches(&q, scope);
            if found.len() == 1 {
                return Element {
                    screen: self,
                    node: found[0].0,
                    region: found[0].1,
                };
            }
            if self.now_ns().saturating_sub(start) >= limit {
                self.fail(&self.report(
                    &format!(
                        "find: {q} not found within {FIND_TIMEOUT:?} (virtual); {} candidate(s)",
                        found.len()
                    ),
                    scope,
                    &found,
                ));
            }
            self.step();
        }
    }

    pub(super) fn all(&self, q: Query, scope: Option<NodeId>) -> Vec<Element<'_>> {
        self.matches(&q, scope)
            .into_iter()
            .map(|(node, region)| Element {
                screen: self,
                node,
                region,
            })
            .collect()
    }

    /// `node <id> <role> <name> @<location>` (see `SemanticNode::at`).
    pub(super) fn describe(&self, id: NodeId) -> String {
        let at = self
            .location_of(id)
            .map(|at| format!(" @{at}"))
            .unwrap_or_default();
        format!(
            "node {} {:?} {:?}{at}",
            id.0,
            self.role_of(id),
            self.name_of(id)
        )
    }

    /// Where `id` was created, or its nearest located ancestor.
    fn location_of(&self, id: NodeId) -> Option<&'static Location<'static>> {
        let tree = self.facts();
        let mut cur = Some(id);
        while let Some(n) = cur.and_then(|c| tree.get(c)) {
            if n.at.is_some() {
                return n.at;
            }
            cur = n.parent;
        }
        None
    }

    /// The failure report: query, scope, candidates, focus, recent
    /// actions and updates, errors and the semantic tree.
    pub(super) fn report(
        &self,
        what: &str,
        scope: Option<NodeId>,
        candidates: &[(NodeId, Option<RegionId>)],
    ) -> String {
        let mut s = format!("kivact test: {what}\n");
        let scope = scope.map_or("whole screen".into(), |n| self.describe(n));
        let _ = writeln!(s, "scope: {scope}");
        let _ = writeln!(s, "candidates: {}", candidates.len());
        for (id, r) in candidates {
            let region = r.map(|r| format!(" region {}", r.0)).unwrap_or_default();
            let _ = writeln!(s, "  {}{region}", self.describe(*id));
        }
        let focus = self.host.borrow().focused();
        let focus = focus.map_or("none".into(), |f| {
            format!(
                "{} region {:?}",
                self.describe(f.node),
                f.region.map(|r| r.0)
            )
        });
        let _ = writeln!(s, "focus: {focus}");
        let _ = writeln!(s, "recent actions: {:?}", self.actions.borrow());
        let _ = writeln!(s, "recent updates:");
        let turns = self.trace();
        let busy = |t: &&TurnRecord| {
            !(t.causes.is_empty() && t.writes.is_empty() && t.ops == 0 && t.notes.is_empty())
        };
        let recent: Vec<&TurnRecord> = turns.iter().rev().filter(busy).take(3).collect();
        for t in recent.into_iter().rev() {
            let _ = writeln!(s, "  {}", t.to_string().replace('\n', "\n  "));
        }
        let errs: Vec<String> = self.errors().iter().map(|e| e.to_string()).collect();
        let _ = writeln!(s, "errors: {errs:?}");
        let _ = write!(
            s,
            "accessibility tree (published by the host):\n{}semantic tree:\n{}",
            self.accessibility_tree(),
            self.debug()
        );
        s
    }

    /// Queries limited to `element`'s subtree.
    pub fn within<'a>(&'a self, element: Element<'a>) -> Within<'a> {
        Within {
            screen: self,
            scope: element.node,
        }
    }

    #[track_caller]
    pub fn get_by_role(&self, role: Role, name: &str) -> Element<'_> {
        self.get(Query::Role(role, name.into()), None)
    }
    #[track_caller]
    pub fn get_by_text(&self, text: &str) -> Element<'_> {
        self.get(Query::Text(text.into()), None)
    }
    #[track_caller]
    pub fn get_by_test_id(&self, id: &str) -> Element<'_> {
        self.get(Query::TestId(id.into()), None)
    }
    pub fn query_by_role(&self, role: Role, name: &str) -> Option<Element<'_>> {
        self.query(Query::Role(role, name.into()), None)
    }
    pub fn query_by_text(&self, text: &str) -> Option<Element<'_>> {
        self.query(Query::Text(text.into()), None)
    }
    pub fn query_by_test_id(&self, id: &str) -> Option<Element<'_>> {
        self.query(Query::TestId(id.into()), None)
    }
    #[track_caller]
    pub fn find_by_role(&self, role: Role, name: &str) -> Element<'_> {
        self.find(Query::Role(role, name.into()), None)
    }
    #[track_caller]
    pub fn find_by_text(&self, text: &str) -> Element<'_> {
        self.find(Query::Text(text.into()), None)
    }
    #[track_caller]
    pub fn find_by_test_id(&self, id: &str) -> Element<'_> {
        self.find(Query::TestId(id.into()), None)
    }
    pub fn all_by_role(&self, role: Role, name: &str) -> Vec<Element<'_>> {
        self.all(Query::Role(role, name.into()), None)
    }
    pub fn all_by_text(&self, text: &str) -> Vec<Element<'_>> {
        self.all(Query::Text(text.into()), None)
    }
    pub fn all_by_test_id(&self, id: &str) -> Vec<Element<'_>> {
        self.all(Query::TestId(id.into()), None)
    }
}

impl<'a> Within<'a> {
    #[track_caller]
    pub fn get_by_role(&self, role: Role, name: &str) -> Element<'a> {
        self.screen
            .get(Query::Role(role, name.into()), Some(self.scope))
    }
    #[track_caller]
    pub fn get_by_text(&self, text: &str) -> Element<'a> {
        self.screen.get(Query::Text(text.into()), Some(self.scope))
    }
    #[track_caller]
    pub fn get_by_test_id(&self, id: &str) -> Element<'a> {
        self.screen.get(Query::TestId(id.into()), Some(self.scope))
    }
    pub fn query_by_role(&self, role: Role, name: &str) -> Option<Element<'a>> {
        self.screen
            .query(Query::Role(role, name.into()), Some(self.scope))
    }
    pub fn query_by_text(&self, text: &str) -> Option<Element<'a>> {
        self.screen
            .query(Query::Text(text.into()), Some(self.scope))
    }
    pub fn query_by_test_id(&self, id: &str) -> Option<Element<'a>> {
        self.screen
            .query(Query::TestId(id.into()), Some(self.scope))
    }
    #[track_caller]
    pub fn find_by_role(&self, role: Role, name: &str) -> Element<'a> {
        self.screen
            .find(Query::Role(role, name.into()), Some(self.scope))
    }
    #[track_caller]
    pub fn find_by_text(&self, text: &str) -> Element<'a> {
        self.screen.find(Query::Text(text.into()), Some(self.scope))
    }
    #[track_caller]
    pub fn find_by_test_id(&self, id: &str) -> Element<'a> {
        self.screen.find(Query::TestId(id.into()), Some(self.scope))
    }
    pub fn all_by_role(&self, role: Role, name: &str) -> Vec<Element<'a>> {
        self.screen
            .all(Query::Role(role, name.into()), Some(self.scope))
    }
    pub fn all_by_text(&self, text: &str) -> Vec<Element<'a>> {
        self.screen.all(Query::Text(text.into()), Some(self.scope))
    }
    pub fn all_by_test_id(&self, id: &str) -> Vec<Element<'a>> {
        self.screen.all(Query::TestId(id.into()), Some(self.scope))
    }
}
