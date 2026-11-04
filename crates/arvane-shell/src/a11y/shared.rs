//! Role: Stores the accessibility tree shared with native callbacks.

use super::error::AccessibilityError;
use super::{state, validation};
use kivact_types::protocol::{SemanticsTree, SemanticsUpdate};
use kivact_types::{SemanticsId, SemanticsTextRun};
use std::sync::{Arc, Mutex, MutexGuard};

/// What the platform has not been handed yet.
#[derive(Default)]
pub(crate) enum Unsent {
    /// Nothing.
    #[default]
    Nothing,
    /// The focus and these nodes (in the order they changed; a node
    /// changed by several updates repeats), with the text runs they added.
    Changed {
        nodes: Vec<SemanticsId>,
        runs: Vec<Arc<SemanticsTextRun>>,
        /// More than one update's nodes: they may repeat, and a later
        /// update may have dropped a run an earlier one added.
        merged: bool,
    },
    /// The whole tree.
    Tree,
}

/// Whether a platform reader holds the tree.
pub(crate) enum Reader {
    /// No platform reader: changes are not tracked; the next activation takes the whole tree.
    Inactive,
    /// Assistive technology asked for the tree and has not let go of it: meanwhile the application
    /// keeps it up to date; `unsent` is what the reader has not been handed.
    Active { unsent: Unsent },
}

/// The tree and what its window's platform side shares with it.
pub(crate) struct Shared {
    pub(crate) tree: Option<state::AcceptedTree>,
    pub(crate) reader: Reader,
    /// The scale the root maps logical bounds to the window's pixels with.
    pub(crate) scale: f64,
    /// Actions assistive technology asked for, not taken yet.
    pub(crate) actions: Vec<crate::events::ShellEvent>,
}

impl Shared {
    pub(crate) fn new(scale: f64) -> Self {
        Shared {
            tree: None,
            reader: Reader::Inactive,
            scale,
            actions: Vec::new(),
        }
    }

    /// Validates a complete tree and replaces the accepted one with it.
    pub(crate) fn replace(&mut self, tree: &SemanticsTree) -> Result<(), AccessibilityError> {
        let tree = validation::canonicalize(tree)?;
        self.tree = Some(state::AcceptedTree::new(&tree));
        if let Reader::Active { unsent } = &mut self.reader {
            *unsent = Unsent::Tree;
        }
        Ok(())
    }

    /// Validates the nodes that may have changed and applies them; whether
    /// anything changed. Without an accepted tree, or with a new root, the
    /// update must hold the complete tree.
    pub(crate) fn update(&mut self, update: &SemanticsUpdate) -> Result<bool, AccessibilityError> {
        let Some(tree) = self.tree.as_mut().filter(|t| t.root() == update.root) else {
            self.replace(&SemanticsTree {
                root: update.root,
                focus: update.focus,
                nodes: update.nodes.clone(),
            })?;
            return Ok(true);
        };
        let Some(changed) = tree.apply(update)? else {
            return Ok(false);
        };
        let Reader::Active { unsent } = &mut self.reader else {
            // No one reads the tree: activation hands over all of it.
            return Ok(true);
        };
        *unsent = match std::mem::take(unsent) {
            Unsent::Nothing => Unsent::Changed {
                nodes: changed.nodes,
                runs: changed.runs,
                merged: false,
            },
            Unsent::Changed {
                mut nodes,
                mut runs,
                ..
            } => {
                nodes.extend(changed.nodes);
                runs.extend(changed.runs);
                Unsent::Changed {
                    nodes,
                    runs,
                    merged: true,
                }
            }
            Unsent::Tree => Unsent::Tree,
        };
        Ok(true)
    }
}

// Locks contain only validation, assignment, conversion and queue insertion.
// A conversion panic propagates (it ends the App); a poisoned lock is never
// entered, because a panic already happened.
pub(crate) fn lock(shared: &Mutex<Shared>) -> MutexGuard<'_, Shared> {
    shared
        .lock()
        .expect("accessibility tree lock poisoned: a panic already happened")
}

/// A backend's synchronized handle to one window's accepted semantics tree.
///
/// Updates validate transactionally. While a platform reader is active, this
/// handle tracks unpublished changes. Activation converts the complete current
/// tree; deactivation drops pending changes. Inside the adapter's publication
/// callback the adapter decides: if this handle saw no activation (or a
/// deactivation the adapter has not seen), [`Self::pending_with`] hands over the
/// whole tree and the reader becomes active. On Windows, UI Automation never
/// deactivates: the reader stays active until the backend disconnects its
/// provider and calls [`Self::deactivate`]. Conversion closures run under the
/// tree lock and must only convert borrowed data: they must not call this handle,
/// application callbacks, wakers or the platform adapter. Acquire an adapter's
/// publication lock first, then call [`Self::pending_with`] inside its callback.
/// This prevents an older update from following a newer activation snapshot.
///
/// A conversion panic propagates. Only successful conversion consumes
/// unpublished changes.
#[derive(Clone)]
pub struct PlatformTree {
    shared: Arc<Mutex<Shared>>,
}

/// The current semantics selected for one platform update, borrowed under lock.
///
/// Node references come from the current accepted tree, never an earlier update.
/// A merged update includes all text runs on its selected nodes; an unmerged
/// update separately carries only the text runs that changed.
pub struct PlatformUpdate<'a> {
    tree: &'a state::AcceptedTree,
    nodes: Vec<&'a kivact_types::protocol::SemanticsNode>,
    runs: &'a [Arc<SemanticsTextRun>],
    all_runs: bool,
    scale: f64,
}

impl PlatformUpdate<'_> {
    /// The current accepted hierarchy, including root and focus identities.
    pub fn tree(&self) -> &state::AcceptedTree {
        self.tree
    }

    /// The current nodes the platform needs, in delivery order.
    pub fn nodes(&self) -> impl Iterator<Item = &kivact_types::protocol::SemanticsNode> {
        self.nodes.iter().copied()
    }

    /// Whether conversion must include every text run of each selected node.
    pub fn all_runs(&self) -> bool {
        self.all_runs
    }

    /// Additional changed text runs, used when [`Self::all_runs`] is false.
    pub fn changed_runs(&self) -> impl Iterator<Item = &SemanticsTextRun> {
        self.runs.iter().map(AsRef::as_ref)
    }

    /// Logical-to-physical scale, applied after the root's own transform.
    pub fn scale(&self) -> f64 {
        self.scale
    }
}

impl PlatformTree {
    /// An empty, inactive tree whose root bounds map to pixels with `scale`.
    /// The backend supplies the same positive finite scale as its window.
    pub fn new(scale: f64) -> Self {
        Self {
            shared: Arc::new(Mutex::new(Shared::new(scale))),
        }
    }

    /// Validate and replace the complete hierarchy, marking it for publication
    /// when active. A rejected tree leaves all accepted state unchanged.
    /// Publish or notify the window only after this method returns.
    pub fn replace(&self, tree: &SemanticsTree) -> Result<(), AccessibilityError> {
        lock(&self.shared).replace(tree)
    }

    /// Validate and apply a partial update; return whether anything changed.
    /// A new root requires a complete hierarchy. Rejection leaves the accepted
    /// tree unchanged. Publish or notify the window after this method returns.
    pub fn update(&self, update: &SemanticsUpdate) -> Result<bool, AccessibilityError> {
        lock(&self.shared).update(update)
    }

    /// A complete copy of the accepted tree in parent-before-child order.
    pub fn tree(&self) -> Option<SemanticsTree> {
        lock(&self.shared)
            .tree
            .as_ref()
            .map(state::AcceptedTree::tree)
    }

    /// Whether a platform reader has activated the tree and not deactivated it.
    pub fn active(&self) -> bool {
        matches!(lock(&self.shared).reader, Reader::Active { .. })
    }

    /// Stop publication and discard pending changes; the accepted tree remains.
    pub fn deactivate(&self) {
        lock(&self.shared).reader = Reader::Inactive;
    }

    /// Activate a reader and convert its complete current tree under lock.
    /// `None` means no tree has been accepted yet. Wake the application after
    /// return so it can refresh semantics. A conversion panic propagates.
    pub fn activate_with<R>(&self, convert: impl FnOnce(PlatformUpdate<'_>) -> R) -> Option<R> {
        let mut shared = lock(&self.shared);
        if shared.tree.is_none() {
            shared.reader = Reader::Active {
                unsent: Unsent::Tree,
            };
            return None;
        }
        let tree = shared.tree.as_ref()?;
        let result = convert(PlatformUpdate {
            tree,
            nodes: tree.nodes(),
            runs: &[],
            all_runs: true,
            scale: shared.scale,
        });
        shared.reader = Reader::Active {
            unsent: Unsent::Nothing,
        };
        Some(result)
    }

    /// Whether a publication may be needed, optionally forcing the root for a
    /// scale change. This is a hint only: activation can consume changes before
    /// the adapter enters its publication callback. Never hold this lock while
    /// acquiring the adapter's lock; call [`Self::pending_with`] inside it.
    pub fn has_pending(&self, include_root: bool) -> bool {
        let shared = lock(&self.shared);
        shared.tree.is_some()
            && match &shared.reader {
                Reader::Inactive => false,
                Reader::Active { unsent } => include_root || !matches!(unsent, Unsent::Nothing),
            }
    }

    /// Convert what the platform still lacks from the current accepted tree.
    /// Call inside the adapter's publication callback, after its own lock is
    /// acquired. The closure runs under the shell lock; pending state is cleared
    /// only after it returns. If activation already consumed the changes, the
    /// selection may contain only current focus (and the root when requested).
    /// `None` means no accepted tree.
    pub fn pending_with<R>(
        &self,
        include_root: bool,
        convert: impl FnOnce(PlatformUpdate<'_>) -> R,
    ) -> Option<R> {
        let mut shared = lock(&self.shared);
        let tree = shared.tree.as_ref()?;
        let root = tree.root();
        // An inactive handle has tracked nothing: the adapter asked, so it gets the whole tree.
        let whole = Unsent::Tree;
        let unsent = match &shared.reader {
            Reader::Active { unsent } => unsent,
            Reader::Inactive => &whole,
        };
        let (mut nodes, runs, all_runs) = match unsent {
            Unsent::Tree => (tree.nodes(), &[][..], true),
            Unsent::Nothing => (Vec::new(), &[][..], false),
            Unsent::Changed {
                nodes,
                runs,
                merged,
            } => {
                let mut seen = std::collections::HashSet::new();
                let current = nodes
                    .iter()
                    .filter(|id| !*merged || seen.insert(**id))
                    .filter_map(|id| tree.node(*id))
                    .collect();
                (current, if *merged { &[][..] } else { &runs[..] }, *merged)
            }
        };
        if include_root && !nodes.iter().any(|node| node.id == root) {
            nodes.extend(tree.node(root));
        }
        let result = convert(PlatformUpdate {
            tree,
            nodes,
            runs,
            all_runs,
            scale: shared.scale,
        });
        shared.reader = Reader::Active {
            unsent: Unsent::Nothing,
        };
        Some(result)
    }

    /// Map subsequent root bounds with `scale`. An active tree retains its root
    /// as pending until a conversion consumes it.
    pub fn rescale(&self, scale: f64) {
        let mut shared = lock(&self.shared);
        shared.scale = scale;
        let Some(root) = shared.tree.as_ref().map(state::AcceptedTree::root) else {
            return;
        };
        if let Reader::Active { unsent } = &mut shared.reader {
            match unsent {
                Unsent::Nothing => {
                    *unsent = Unsent::Changed {
                        nodes: vec![root],
                        runs: Vec::new(),
                        merged: false,
                    }
                }
                Unsent::Changed { nodes, .. } if !nodes.contains(&root) => nodes.push(root),
                _ => {}
            }
        }
    }

    /// Forget a pooled window's accepted hierarchy and pending publication.
    /// Call only between the backend's publication/activation operations.
    /// Its reader state, scale and queued actions are unchanged.
    pub fn forget(&self) {
        let mut shared = lock(&self.shared);
        shared.tree = None;
        if let Reader::Active { unsent } = &mut shared.reader {
            *unsent = Unsent::Tree;
        }
    }

    /// Check a request against the current hierarchy, without queueing it.
    /// Hidden or disabled targets/ancestors and undeclared actions are rejected.
    /// `Activate`, `Toggle` and `Select` share the default action and are delivered
    /// as `Activate`; a focusable node declares `Focus` even without an explicit
    /// entry in its action list. Other actions require an exact declaration.
    pub fn request_action(
        &self,
        node: u64,
        request: kivact_types::A11yRequest,
    ) -> Option<crate::events::ShellEvent> {
        let shared = lock(&self.shared);
        super::action::accept(shared.tree.as_ref()?, node, request)
    }

    /// Atomically validate and queue a native request using the same contract as
    /// [`Self::request_action`]. Wake the UI thread only after this returns true.
    pub fn queue_action(&self, node: u64, request: kivact_types::A11yRequest) -> bool {
        let mut shared = lock(&self.shared);
        let Some(event) = shared
            .tree
            .as_ref()
            .and_then(|tree| super::action::accept(tree, node, request))
        else {
            return false;
        };
        shared.actions.push(event);
        true
    }

    /// Take all accepted native requests in arrival order for UI-thread delivery.
    pub fn drain_actions(&self) -> Vec<crate::events::ShellEvent> {
        std::mem::take(&mut lock(&self.shared).actions)
    }
}
