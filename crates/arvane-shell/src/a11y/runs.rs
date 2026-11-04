//! Role: The accepted tree's text runs: their owners, and what an update changes of them.
//!
//! An update of a text node compares its runs with the accepted ones by
//! identity (`Arc::ptr_eq`): the runs kept at both ends are the same
//! runs, already checked; only the runs between them are new. So an edit
//! of one line of a long text checks, records and hands over that line's
//! run, not the text's. How a run reaches the platform (CR LF as one
//! character, its direction, no word boundaries) is `conversion.rs`'s.
//!
//! Invariants:
//! - `owners` names the text node of every run of the accepted tree, and
//!   `lengths` the length in scalars of every text node's text.
//! - A run identity is unique among the nodes and the runs of the tree.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use kivact_types::protocol::SemanticsNode;
use kivact_types::{SemanticsId, SemanticsText, SemanticsTextRun};

use super::error::AccessibilityError;
use super::validation::{check_run, check_selection, check_text_node};

fn text(node: &SemanticsNode) -> Option<&SemanticsText> {
    node.properties.as_ref()?.text.as_ref()
}

/// The runs of a node's text (none without one).
fn runs_of(node: Option<&SemanticsNode>) -> &[Arc<SemanticsTextRun>] {
    node.and_then(text).map_or(&[], |t| &t.runs[..])
}

/// The runs of `old` that `new` drops, and those it adds: the runs
/// between the ends both share.
fn diff<'a>(
    old: &'a [Arc<SemanticsTextRun>],
    new: &'a [Arc<SemanticsTextRun>],
) -> (&'a [Arc<SemanticsTextRun>], &'a [Arc<SemanticsTextRun>]) {
    if std::ptr::eq(old, new) {
        return (&[], &[]);
    }
    let same = |(a, b): (&Arc<SemanticsTextRun>, &Arc<SemanticsTextRun>)| Arc::ptr_eq(a, b);
    let head = old.iter().zip(new).take_while(|&p| same(p)).count();
    let tail = old[head..]
        .iter()
        .rev()
        .zip(new[head..].iter().rev())
        .take_while(|&p| same(p))
        .count();
    (&old[head..old.len() - tail], &new[head..new.len() - tail])
}

/// The runs of the accepted tree.
#[derive(Default)]
pub(super) struct Runs {
    // Platform nodes and runs share one numeric identity space.
    owners: HashMap<SemanticsId, SemanticsId>,
    lengths: HashMap<SemanticsId, u32>,
}

/// What an update changes of the runs.
#[derive(Default)]
pub(super) struct RunEdits {
    removed: Vec<SemanticsId>,
    /// The new runs and their nodes, in order.
    pub(super) added: Vec<(SemanticsId, Arc<SemanticsTextRun>)>,
    /// The new length of each changed text (`None`: it has none any more).
    lengths: Vec<(SemanticsId, Option<u32>)>,
}

impl Runs {
    /// The runs of a checked complete tree.
    pub(super) fn new<'a>(nodes: impl Iterator<Item = &'a SemanticsNode>) -> Self {
        let mut runs = Runs::default();
        for node in nodes {
            let Some(text) = text(node) else { continue };
            let mut length = 0u32;
            for run in text.runs.iter() {
                runs.owners.insert(SemanticsId(run.id.0), node.id);
                length = length.saturating_add(run.value.chars().count() as u32);
            }
            runs.lengths.insert(node.id, length);
        }
        runs
    }

    /// Check the texts of `changed` against the accepted ones (`old`):
    /// each new run's values and identity (not a node after the update,
    /// `is_node`, nor another run), each text's own values and its
    /// selection inside it; and that no new node takes a run's identity.
    pub(super) fn check<'a>(
        &self,
        changed: &[&'a SemanticsNode],
        old: impl Fn(SemanticsId) -> Option<&'a SemanticsNode>,
        is_node: impl Fn(SemanticsId) -> bool,
    ) -> Result<RunEdits, AccessibilityError> {
        let diffs: Vec<_> = changed
            .iter()
            .map(|node| {
                let (gone, new) = diff(runs_of(old(node.id)), runs_of(Some(node)));
                (*node, gone, new)
            })
            .collect();
        let removed: HashSet<SemanticsId> = diffs
            .iter()
            .flat_map(|(_, gone, _)| gone.iter().map(|run| SemanticsId(run.id.0)))
            .collect();
        let taken = |id: SemanticsId| self.owners.contains_key(&id) && !removed.contains(&id);
        let mut edits = RunEdits::default();
        let mut added = HashSet::new();
        for (node, gone, new) in diffs {
            if old(node.id).is_none() && taken(node.id) {
                return Err(AccessibilityError::DuplicateNode(node.id));
            }
            edits
                .removed
                .extend(gone.iter().map(|run| SemanticsId(run.id.0)));
            let Some(text) = text(node) else {
                if self.lengths.contains_key(&node.id) {
                    edits.lengths.push((node.id, None));
                }
                continue;
            };
            check_text_node(node)?;
            let kept = gone.iter().fold(
                self.lengths.get(&node.id).copied().unwrap_or(0),
                |length, run| length.saturating_sub(run.value.chars().count() as u32),
            );
            let mut length = kept;
            for run in new {
                if is_node(SemanticsId(run.id.0))
                    || taken(SemanticsId(run.id.0))
                    || !added.insert(SemanticsId(run.id.0))
                {
                    return Err(AccessibilityError::DuplicateNode(SemanticsId(run.id.0)));
                }
                length = length
                    .checked_add(check_run(node.id, run)?)
                    .ok_or(AccessibilityError::InvalidProperties(node.id))?;
                edits.added.push((node.id, run.clone()));
            }
            check_selection(node.id, text, length)?;
            edits.lengths.push((node.id, Some(length)));
        }
        Ok(edits)
    }

    /// Take the checked `edits`.
    pub(super) fn commit(&mut self, edits: &RunEdits) {
        for id in &edits.removed {
            self.owners.remove(id);
        }
        for (owner, run) in &edits.added {
            self.owners.insert(SemanticsId(run.id.0), *owner);
        }
        for (node, length) in &edits.lengths {
            match length {
                Some(length) => self.lengths.insert(*node, *length),
                None => self.lengths.remove(node),
            };
        }
    }

    /// `node` left the tree: so did its runs.
    pub(super) fn forget(&mut self, node: &SemanticsNode) {
        for run in runs_of(Some(node)) {
            self.owners.remove(&SemanticsId(run.id.0));
        }
        self.lengths.remove(&node.id);
    }
}
