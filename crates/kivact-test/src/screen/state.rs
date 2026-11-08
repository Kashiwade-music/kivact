//! Role: Test state management and persistence.
//!
//! Inspection of a `Screen`'s state: errors, turn trace, counters ([`Stats`])
//! and the semantic tree (`SemanticNode`, `snapshot`, `debug`).

use std::fmt::Write as _;
use std::panic::Location;

use kivact_engine::contract::SceneLayer;
use kivact_types::NodeId;
use kivact_types::WindowId;
use kivact_types::render::FrameStats;

use super::model::{Screen, TestApp};
use super::results::SemanticNode;
use crate::facts::Facts;
use kivact_core::diagnostics::Stats;
use kivact_runtime::TurnRecord;
use kivact_runtime::UiError;

impl Screen {
    /// Errors that reached the App's default sink (no `on_error` handled them).
    pub fn errors(&self) -> Vec<UiError> {
        self.inspect().errors()
    }

    /// Recent turn records (events, ops, notes).
    pub fn trace(&self) -> Vec<TurnRecord> {
        self.inspect().turns()
    }

    /// The App's counters ([`Stats`] says how to read them): `batches` and
    /// `ops` count the revisions the client committed and their writes, and
    /// `core_ns` the time in the core's turns, `on_mount` work included.
    /// Notes recorded during setup go to a record of their own (`Turn::Setup`), so
    /// they add no turn.
    pub fn stats(&self) -> Stats {
        let mut stats = self.inspect().stats();
        stats.core_ns = self.app.record.core_ns.get();
        stats
    }

    /// The work counts of the last frame the window presented (chunks
    /// rastered, pixels rastered, damage). Panics before any frame is
    /// presented.
    pub fn frame_stats(&self) -> FrameStats {
        let stats = self.host().frame_stats(self.window);
        stats.expect("no frame has been presented yet")
    }

    /// Semantic tree of the whole window.
    pub fn snapshot(&self) -> SemanticNode {
        self.check_open();
        self.app.snapshot(self.window)
    }

    /// Indented tree dump: role, name, value, `#test_id`, `@location` (see
    /// [`SemanticNode::at`]) and focus.
    pub fn debug(&self) -> String {
        self.check_open();
        self.app.debug_tree(self.window)
    }
}

impl TestApp {
    /// Semantic tree of window `window`.
    fn snapshot(&self, window: WindowId) -> SemanticNode {
        let root = Facts::new(self.inspect(), window).root;
        self.semantic(window, root, None)
    }

    /// The semantic node of `id`; `parent_at` is the location of its
    /// nearest located ancestor.
    fn semantic(
        &self,
        window: WindowId,
        id: NodeId,
        parent_at: Option<&'static Location<'static>>,
    ) -> SemanticNode {
        let tree = Facts::new(self.inspect(), window);
        let focused = self.host.borrow().focused(window)
            == Some(kivact_types::FocusTarget {
                node: id,
                region: None,
            });
        let mut node = SemanticNode {
            role: None,
            name: String::new(),
            value: None,
            test_id: None,
            focused,
            disabled: false,
            selected: false,
            checked: None,
            at: None,
            children: Vec::new(),
        };
        let mut kids = Vec::new();
        if let Some(n) = tree.get(id) {
            node.test_id = tree.test_id(id);
            node.at = n.at.or(parent_at);
            node.role = tree.role(id);
            node.name = tree
                .name(id)
                .or_else(|| tree.content(id))
                .unwrap_or_default();
            node.value = tree.input_value(id).or_else(|| tree.value(id));
            node.disabled = tree.disabled(id).unwrap_or(false);
            node.selected = tree.selected(id).unwrap_or(false);
            node.checked = tree.checked(id);
            let scenes = [SceneLayer::Draw, SceneLayer::DrawOver].map(|l| tree.scene(id, l));
            for s in scenes.iter().flatten() {
                for r in &s.regions {
                    node.children.push(SemanticNode {
                        role: r.semantics.role,
                        name: r.semantics.name.clone().unwrap_or_default(),
                        value: r.semantics.value_text.clone(),
                        test_id: None,
                        focused: self.host.borrow().focused(window)
                            == Some(kivact_types::FocusTarget {
                                node: id,
                                region: Some(r.id),
                            }),
                        disabled: node.disabled || r.semantics.disabled,
                        selected: false,
                        checked: None,
                        at: node.at,
                        children: Vec::new(),
                    });
                }
            }
            kids = tree.children(id);
        }
        drop(tree);
        for k in kids {
            node.children.push(self.semantic(window, k, node.at));
        }
        node
    }

    /// Indented dump of window `window`'s retained semantic tree.
    pub(super) fn debug_tree(&self, window: WindowId) -> String {
        fn walk(s: &mut String, n: &SemanticNode, depth: usize) {
            let _ = writeln!(
                s,
                "{}{:?} {:?}{}{}{}{}",
                "  ".repeat(depth),
                n.role,
                n.name,
                n.value
                    .as_ref()
                    .map(|v| format!(" value={v:?}"))
                    .unwrap_or_default(),
                n.test_id
                    .as_ref()
                    .map(|t| format!(" #{t}"))
                    .unwrap_or_default(),
                n.at.map(|at| format!(" @{at}")).unwrap_or_default(),
                if n.focused { " [focused]" } else { "" }
            );
            for c in &n.children {
                walk(s, c, depth + 1);
            }
        }
        let mut s = String::new();
        walk(&mut s, &self.snapshot(window), 0);
        s
    }
}
