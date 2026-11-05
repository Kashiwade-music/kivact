//! Role: Accessibility validation and checking.
//!
//! Checks semantics before replacing snapshots; ownership and focus are valid together.
//!
//! A complete tree is accepted when its ids are unique, every child it
//! names exists and has one parent, every node is reachable from the root,
//! the focus names a node that is shown and focusable (or the
//! enabled root), bounds, scroll offsets and transforms are finite (bounds of no
//! negative size), numeric values are finite with `min <= max`, and text
//! runs are well formed. Relations (`described_by`, `controls`,
//! `labelled_by`) must name nodes of the tree (`MissingNode`), and an
//! `active_descendant` must be a node of the tree (`MissingNode`) inside
//! the node's subtree or the subtree of a node it `controls` (that node
//! included; ARIA's combobox: the input and its sibling list), else
//! `InvalidProperties`. An update is checked by the same rules against
//! the tree it changes (`state.rs`).

use std::collections::{HashMap, HashSet};

use kivact_types::render::{SemanticsNode, SemanticsTree};

use super::error::AccessibilityError;

pub(crate) fn canonicalize(tree: &SemanticsTree) -> Result<SemanticsTree, AccessibilityError> {
    let mut lookup = HashMap::new();
    for node in &tree.nodes {
        if lookup.insert(node.id, node).is_some() {
            return Err(AccessibilityError::DuplicateNode(node.id));
        }
        check_values(node)?;
    }
    check_text(tree.nodes.iter(), |id| lookup.contains_key(&id))?;
    check_relations(tree, &lookup)?;
    let mut nodes = Vec::with_capacity(tree.nodes.len());
    let mut visited = HashSet::new();
    let mut hidden_nodes = HashSet::new();
    let mut pending = vec![(tree.root, false)];
    while let Some((id, ancestor_hidden)) = pending.pop() {
        let node = lookup.get(&id).ok_or(AccessibilityError::MissingNode(id))?;
        if !visited.insert(id) {
            return Err(AccessibilityError::InvalidTree(id));
        }
        let hidden = ancestor_hidden || node.semantics.hidden;
        if hidden {
            hidden_nodes.insert(id);
        }
        nodes.push((*node).clone());
        pending.extend(node.children.iter().rev().map(|child| (*child, hidden)));
    }
    if let Some(node) = tree.nodes.iter().find(|node| !visited.contains(&node.id)) {
        return Err(AccessibilityError::UnreachableNode(node.id));
    }
    if let Some(focus) = tree.focus {
        let node = lookup
            .get(&focus)
            .ok_or(AccessibilityError::InvalidFocus(focus))?;
        if hidden_nodes.contains(&focus)
            || (!node.semantics.focusable && (node.semantics.disabled || focus != tree.root))
        {
            return Err(AccessibilityError::InvalidFocus(focus));
        }
    }
    Ok(SemanticsTree {
        root: tree.root,
        focus: tree.focus,
        nodes,
    })
}

/// Validate active descendants and node references before traversal.
fn check_relations(
    tree: &SemanticsTree,
    lookup: &HashMap<kivact_types::SemanticsId, &SemanticsNode>,
) -> Result<(), AccessibilityError> {
    for node in &tree.nodes {
        if let Some(active) = node.semantics.active_descendant {
            if !lookup.contains_key(&active) {
                return Err(AccessibilityError::MissingNode(active));
            }
            // The node's own subtree, or the subtree of a node it controls.
            let mut pending = node.children.clone();
            pending.extend(node.semantics.controls.iter().copied());
            let mut seen = HashSet::new();
            let mut found = false;
            while let Some(id) = pending.pop() {
                if !seen.insert(id) {
                    continue;
                }
                if id == active {
                    found = true;
                    break;
                }
                if let Some(child) = lookup.get(&id) {
                    pending.extend(&child.children);
                }
            }
            if !found {
                return Err(AccessibilityError::InvalidProperties(node.id));
            }
        }
    }
    for node in &tree.nodes {
        {
            let props = &node.semantics;
            let targets = [&props.described_by, &props.controls, &props.labelled_by];
            if let Some(missing) = targets
                .into_iter()
                .flatten()
                .find(|id| !lookup.contains_key(id))
            {
                return Err(AccessibilityError::MissingNode(*missing));
            }
        }
    }
    Ok(())
}

/// Bounds are finite with a non-negative size, and the scroll offset and
/// the transform are finite.
pub(crate) fn check_values(node: &SemanticsNode) -> Result<(), AccessibilityError> {
    let props = &node.semantics;
    if matches!((props.value_min, props.value_max), (Some(min), Some(max)) if min.get() > max.get())
    {
        return Err(AccessibilityError::InvalidProperties(node.id));
    }
    let b = node.bounds;
    let t = node.transform;
    let finite = [
        b.x,
        b.y,
        b.width,
        b.height,
        b.x + b.width,
        b.y + b.height,
        node.scroll.x,
        node.scroll.y,
        t.a,
        t.b,
        t.c,
        t.d,
        t.e,
        t.f,
    ]
    .into_iter()
    .all(f64::is_finite);
    if !finite || b.width < 0.0 || b.height < 0.0 {
        return Err(AccessibilityError::InvalidBounds(node.id));
    }
    Ok(())
}

/// A complete tree's text snapshots: each node's own values
/// ([`check_text_node`]), each run's ([`check_run`]), run identities
/// unique among all nodes and runs, and every selection inside its text.
pub(super) fn check_text<'a>(
    nodes: impl Iterator<Item = &'a SemanticsNode>,
    is_node: impl Fn(kivact_types::SemanticsId) -> bool,
) -> Result<(), AccessibilityError> {
    let mut seen_runs = HashSet::new();
    for node in nodes {
        let Some(text) = node.semantics.text.as_ref() else {
            continue;
        };
        check_text_node(node)?;
        let mut end = 0u32;
        for run in text.runs.iter() {
            // A run must not reuse a platform node identity.
            if is_node(kivact_types::SemanticsId(run.id.0))
                || !seen_runs.insert(kivact_types::SemanticsId(run.id.0))
            {
                return Err(AccessibilityError::DuplicateNode(
                    kivact_types::SemanticsId(run.id.0),
                ));
            }
            end = end
                .checked_add(check_run(node.id, run)?)
                .ok_or(AccessibilityError::InvalidProperties(node.id))?;
        }
        check_selection(node.id, text, end)?;
    }
    Ok(())
}

/// A text snapshot's own values: shown (not protected), with at least one
/// run and a finite caret.
pub(super) fn check_text_node(node: &SemanticsNode) -> Result<(), AccessibilityError> {
    let props = &node.semantics;
    let Some(text) = &props.text else {
        return Ok(());
    };
    if props.protected || text.runs.is_empty() {
        return Err(AccessibilityError::InvalidProperties(node.id));
    }
    if !text_bounds(text.caret) {
        return Err(AccessibilityError::InvalidBounds(node.id));
    }
    Ok(())
}

/// A run of `owner`'s text: finite bounds and carets, the carets inside
/// it. Returns its length in scalars.
pub(super) fn check_run(
    owner: kivact_types::SemanticsId,
    run: &kivact_types::SemanticsTextRun,
) -> Result<u32, AccessibilityError> {
    let count = u32::try_from(run.value.chars().count())
        .map_err(|_| AccessibilityError::InvalidProperties(owner))?;
    if !text_bounds(run.bounds) || run.carets.iter().any(|c| !text_bounds(c.rect)) {
        return Err(AccessibilityError::InvalidBounds(
            kivact_types::SemanticsId(run.id.0),
        ));
    }
    if run.carets.iter().any(|c| c.offset > count) {
        return Err(AccessibilityError::InvalidProperties(owner));
    }
    Ok(count)
}

/// The selection of `owner`'s text, whose runs are `len` scalars long from
/// its start, lies in the runs.
pub(super) fn check_selection(
    owner: kivact_types::SemanticsId,
    text: &kivact_types::SemanticsText,
    len: u32,
) -> Result<(), AccessibilityError> {
    let end = text
        .start
        .checked_add(len)
        .ok_or(AccessibilityError::InvalidProperties(owner))?;
    let inside = |at: u32| (text.start..=end).contains(&at);
    if !inside(text.selection.0) || !inside(text.selection.1) {
        return Err(AccessibilityError::InvalidProperties(owner));
    }
    Ok(())
}

fn text_bounds(rect: kivact_types::Rect) -> bool {
    [
        rect.x,
        rect.y,
        rect.width,
        rect.height,
        rect.right(),
        rect.bottom(),
    ]
    .into_iter()
    .all(f64::is_finite)
        && rect.width >= 0.0
        && rect.height >= 0.0
}
