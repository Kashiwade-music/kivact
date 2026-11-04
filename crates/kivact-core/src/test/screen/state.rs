//! Role: Test state management and persistence.
//!
//! Inspection of a `Screen`'s state: errors, turn trace, counters ([`Stats`])
//! and the semantic tree (`SemanticNode`, `snapshot`, `debug`).

use std::fmt::Write as _;
use std::panic::Location;

use kivact_engine::contract::SceneLayer;
use kivact_types::NodeId;

use super::model::Screen;
use super::results::SemanticNode;
use crate::client::HostState;
use crate::diagnostics::Stats;
use crate::view::state::ViewState;
use kivact_motion::MotionState;
use kivact_reactive::TurnRecord;
use kivact_reactive::UiError;
use kivact_task::TaskState;

impl Screen {
    /// Errors that reached the App's default sink (no `on_error` handled them).
    pub fn errors(&self) -> Vec<UiError> {
        self.rt.errors()
    }

    /// Recent turn records (events, ops, notes).
    pub fn trace(&self) -> Vec<TurnRecord> {
        self.rt.turns()
    }

    /// The App's counters ([`Stats`] says how to read them): `batches` and
    /// `ops` count the revisions the client committed and their writes, and
    /// `core_ns` the time in the core's turns, `on_mount` work included.
    /// Notes recorded during setup go to a record of their own (`Turn::Setup`), so
    /// they add no turn.
    pub fn stats(&self) -> Stats {
        let rec = &self.record;
        let ts = ViewState::tree(&self.rt).engine().stats();
        let host = HostState::of(&self.rt);
        let live = self.rt.live_counts();
        let motion = MotionState::of(&self.rt).stats();
        Stats {
            live_reactive: live.reactive,
            live_owners: live.owners,
            live_objects: live.objects,
            live_constants: live.constants,
            arena_slots: live.slots,
            tree_nodes: ViewState::tree(&self.rt).len(),
            timers: TaskState::of(&self.rt).stats().timers,
            frame_loops: motion.loops,
            animations: motion.animations,
            batches: ts.batches,
            a11y_constructed: ts.a11y_constructed,
            a11y_published: ts.a11y_published,
            ops: ts.ops,
            last_batch_ops: ts.last_ops,
            creates: ts.creates,
            set_props: ts.set_props,
            structural: ts.structural,
            host_touched_last: ts.touched_last,
            turns: host.turns(),
            stale_events: host.stale_events(),
            core_ns: rec.core_ns.get(),
        }
    }

    /// Semantic tree of the whole screen.
    pub fn snapshot(&self) -> SemanticNode {
        let root = ViewState::tree(&self.rt).root;
        self.semantic(root, None)
    }

    /// The semantic node of `id`; `parent_at` is the location of its nearest
    /// located ancestor.
    fn semantic(&self, id: NodeId, parent_at: Option<&'static Location<'static>>) -> SemanticNode {
        let tree = ViewState::tree(&self.rt);
        let focused = self.host.borrow().focused()
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
                        role: Some(r.role),
                        name: r.name.clone(),
                        value: r.value.clone(),
                        test_id: None,
                        focused: self.host.borrow().focused()
                            == Some(kivact_types::FocusTarget {
                                node: id,
                                region: Some(r.id),
                            }),
                        disabled: node.disabled || r.disabled,
                        selected: false,
                        checked: None,
                        at: node.at,
                        children: Vec::new(),
                    });
                }
            }
            let mut stack: Vec<NodeId> = n.children.iter().rev().copied().collect();
            while let Some(c) = stack.pop() {
                match tree.get(c) {
                    Some(cn) if cn.is_host() => kids.push(c),
                    Some(cn) => {
                        for x in cn.children.iter().rev() {
                            stack.push(*x);
                        }
                    }
                    None => {}
                }
            }
        }
        drop(tree);
        for k in kids {
            node.children.push(self.semantic(k, node.at));
        }
        node
    }

    /// Indented tree dump: role, name, value, `#test_id`, `@location` (see
    /// [`SemanticNode::at`]) and focus.
    pub fn debug(&self) -> String {
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
        walk(&mut s, &self.snapshot(), 0);
        s
    }
}
