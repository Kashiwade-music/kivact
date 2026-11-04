//! Role: Dependency chain explanation for debugging.
//!
//! `why`: the chain explaining why a node's props have their values — the
//! effect that set each dynamic prop, what it read (memos recursively) and
//! where those signals were created and last written.

use std::fmt;
use std::panic::Location;

use kivact_runtime::Turn;
use kivact_types::NodeId;

use crate::view::NodeRef;
use crate::view::state::ViewState;
use kivact_runtime::Kind;
use kivact_runtime::{NodeId as GraphNode, Runtime};

const DEPTH: usize = 8;

/// Why a node looks the way it does.
#[derive(Clone, Debug, PartialEq)]
pub struct Why {
    /// Host node id of the explained node.
    pub node: u64,
    /// Role and name of the node, e.g. `Text "count: 2"`.
    pub label: String,
    /// Dynamic props, in the order they were first set.
    pub props: Vec<WhyProp>,
}

/// A dynamic prop and the effect that last set it.
#[derive(Clone, Debug, PartialEq)]
pub struct WhyProp {
    /// name (`Content`, `Paint`, `Disabled`, ...).
    pub prop: String,
    /// Where the binding (render effect) was created.
    pub effect_at: &'static Location<'static>,
    /// What that effect read during its last run.
    pub reads: Vec<WhyDep>,
}

/// A dependency: a signal (with its last write) or a memo (with its reads).
#[derive(Clone, Debug, PartialEq)]
pub struct WhyDep {
    /// `"signal"` or `"memo"`.
    pub kind: &'static str,
    /// Value type name.
    pub ty: &'static str,
    /// Where the signal or memo was created.
    pub created_at: &'static Location<'static>,
    /// Last value-changing write: its location and turn (`Turn::Setup` while
    /// the App was set up).
    pub last_write: Option<(&'static Location<'static>, Turn)>,
    /// What a memo read during its last run (empty for signals).
    pub reads: Vec<WhyDep>,
}

/// Explain the props of the node bound to `node` (`None` when unmounted).
pub fn why(node: &NodeRef) -> Option<Why> {
    let id = node.node()?;
    let rt = Runtime::get(node.app)?;
    Some(explain(&rt, id))
}

fn dep(rt: &Runtime, key: GraphNode, depth: usize, seen: &mut Vec<GraphNode>) -> Option<WhyDep> {
    let n = rt.node_info(key)?;
    let kind = match n.kind {
        Kind::Source => "signal",
        Kind::Memo => "memo",
        Kind::Effect(_) => return None,
    };
    let mut reads = Vec::new();
    if depth < DEPTH && !seen.contains(&key) {
        seen.push(key);
        reads = n
            .deps
            .into_iter()
            .filter_map(|d| dep(rt, d, depth + 1, seen))
            .collect();
    }
    Some(WhyDep {
        kind,
        ty: n.ty,
        created_at: n.at,
        last_write: n.written,
        reads,
    })
}

/// The [`Why`] of host node `id` in `rt` (also for removed nodes).
pub(crate) fn explain(rt: &Runtime, id: NodeId) -> Why {
    let (label, origins) = {
        let tree = ViewState::tree(rt);
        match tree.get(id) {
            Some(n) => {
                let role = match tree.role(id) {
                    Some(r) => format!("{r:?}"),
                    _ => format!("{:?}", n.kind),
                };
                let name = match (tree.name(id), tree.content(id)) {
                    (Some(s), _) | (None, Some(s)) => s,
                    _ => String::new(),
                };
                (format!("{role} {name:?}"), n.origins.clone())
            }
            None => ("(removed)".to_string(), Vec::new()),
        }
    };
    let mut props = Vec::new();
    for (kind, effect) in origins {
        let Some(n) = rt.node_info(effect) else {
            continue;
        };
        let mut seen = Vec::new();
        props.push(WhyProp {
            prop: kind.name().to_string(),
            effect_at: n.at,
            reads: n
                .deps
                .into_iter()
                .filter_map(|d| dep(rt, d, 1, &mut seen))
                .collect(),
        });
    }
    Why {
        node: id.0,
        label,
        props,
    }
}

impl fmt::Display for Why {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn deps(f: &mut fmt::Formatter<'_>, d: &[WhyDep], depth: usize) -> fmt::Result {
            for x in d {
                write!(
                    f,
                    "\n{}reads {} {} created at {}",
                    "  ".repeat(depth),
                    x.kind,
                    x.ty,
                    x.created_at
                )?;
                match x.last_write {
                    Some((at, Turn::Setup)) => write!(f, ", last written at {at} (mount)")?,
                    Some((at, Turn::Number(n))) => write!(f, ", last written at {at} (turn {n})")?,
                    None if x.kind == "signal" => write!(f, ", never written")?,
                    None => {}
                }
                deps(f, &x.reads, depth + 1)?;
            }
            Ok(())
        }
        write!(f, "node {} {}", self.node, self.label)?;
        if self.props.is_empty() {
            write!(f, "\n  (all props are static)")?;
        }
        for p in &self.props {
            write!(
                f,
                "\n  {} set by the effect created at {}",
                p.prop, p.effect_at
            )?;
            deps(f, &p.reads, 2)?;
        }
        Ok(())
    }
}
