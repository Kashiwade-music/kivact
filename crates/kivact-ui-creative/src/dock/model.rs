//! Role: Consumer-owned docking layout: a tree of splits with tabbed containers.
//!
//! Consumer-owned docking layout: a tree of splits whose leaves are tabbed containers.
//!
//! The model is plain data with public fields so applications can persist,
//! diff and undo it. Every operation keeps the tree normalized: empty
//! containers disappear, single-child splits collapse into their parent and
//! nested splits on the same axis merge.

use std::collections::HashSet;
use std::hash::Hash;

use kivact::style::Icon;

/// The direction in which a split lays out its children.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DockAxis {
    /// Children side by side, separated by vertical splitters.
    Horizontal,
    /// Children stacked, separated by horizontal splitters.
    Vertical,
}

/// A side of a container (or of the whole dock) that receives a new split.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DockSide {
    Left,
    Right,
    Top,
    Bottom,
}

impl DockSide {
    /// The split axis this side creates.
    pub fn axis(self) -> DockAxis {
        match self {
            Self::Left | Self::Right => DockAxis::Horizontal,
            Self::Top | Self::Bottom => DockAxis::Vertical,
        }
    }
    /// Whether the new region comes before the existing one.
    fn leading(self) -> bool {
        matches!(self, Self::Left | Self::Top)
    }
}

/// One panel tab: a stable consumer key, its visible title and chrome options.
///
/// The key identifies the panel's retained content in [`crate::dock`]. Panel
/// kinds and single-instance policies belong to the consumer; encode them in
/// the key type when needed.
#[derive(Clone, Debug, PartialEq)]
pub struct DockTab<K> {
    pub id: K,
    pub title: String,
    // xtask-allow(state-fields): option: absent means the tab shows no icon
    pub icon: Option<Icon>,
    // xtask-allow(state-fields): independent: whether the application allows closing this tab; either value is valid
    /// Whether the tab shows a close button and the close commands remove it.
    pub closable: bool,
}

impl<K> DockTab<K> {
    pub fn new(id: K, title: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
            icon: None,
            closable: true,
        }
    }
    /// Show an icon before the title: a name in the theme's icon set (such
    /// as `"palette"`) or an image. Double-clicking it toggles maximization.
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }
    /// Keep the tab open: no close button, and close commands skip it. A
    /// constant: a tab is plain data inside the layout value, so a changed
    /// flag is a new layout.
    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }
}

/// A tabbed container: the leaf of a dock layout.
#[derive(Clone, Debug, PartialEq)]
pub struct DockTabs<K> {
    pub tabs: Vec<DockTab<K>>,
    /// The visible tab. Normalization repairs a missing key to the first tab.
    pub active: Option<K>,
}

impl<K: Clone + PartialEq> DockTabs<K> {
    /// A container showing its first tab.
    pub fn new(tabs: Vec<DockTab<K>>) -> Self {
        let active = tabs.first().map(|tab| tab.id.clone());
        Self { tabs, active }
    }
    /// Show this tab instead of the first one.
    pub fn active(mut self, id: K) -> Self {
        self.active = Some(id);
        self
    }
    /// The visible tab, if the active key names one of the tabs.
    pub fn active_tab(&self) -> Option<&DockTab<K>> {
        let active = self.active.as_ref()?;
        self.tabs.iter().find(|tab| &tab.id == active)
    }
    pub fn contains(&self, id: &K) -> bool {
        self.tabs.iter().any(|tab| &tab.id == id)
    }
    fn repair_active(&mut self) {
        if self
            .active
            .as_ref()
            .is_none_or(|active| !self.contains(active))
        {
            self.active = self.tabs.first().map(|tab| tab.id.clone());
        }
    }
}

/// A child of a split with its relative size. Weights of one split need not
/// sum to one; normalization rescales them.
#[derive(Clone, Debug, PartialEq)]
pub struct DockChild<K> {
    pub weight: f64,
    pub node: DockNode<K>,
}

/// A node of the layout tree.
#[derive(Clone, Debug, PartialEq)]
pub enum DockNode<K> {
    Split {
        axis: DockAxis,
        children: Vec<DockChild<K>>,
    },
    Tabs(DockTabs<K>),
}

impl<K: Clone + PartialEq> DockNode<K> {
    /// A container showing its first tab.
    pub fn tabs(tabs: Vec<DockTab<K>>) -> Self {
        Self::Tabs(DockTabs::new(tabs))
    }
    /// A split from `(weight, node)` pairs.
    pub fn split(axis: DockAxis, children: impl IntoIterator<Item = (f64, DockNode<K>)>) -> Self {
        Self::Split {
            axis,
            children: children
                .into_iter()
                .map(|(weight, node)| DockChild { weight, node })
                .collect(),
        }
    }
}

impl<K: Clone + Eq + Hash> DockNode<K> {
    fn path_of(&self, id: &K) -> Option<Vec<usize>> {
        match self {
            Self::Tabs(tabs) => tabs.contains(id).then(Vec::new),
            Self::Split { children, .. } => children.iter().enumerate().find_map(|(i, child)| {
                child.node.path_of(id).map(|mut path| {
                    path.insert(0, i);
                    path
                })
            }),
        }
    }
    fn at_mut(&mut self, path: &[usize]) -> Option<&mut Self> {
        let Some((first, rest)) = path.split_first() else {
            return Some(self);
        };
        match self {
            Self::Split { children, .. } => children.get_mut(*first)?.node.at_mut(rest),
            Self::Tabs(_) => None,
        }
    }
    fn tabs_mut(&mut self, path: &[usize]) -> Option<&mut DockTabs<K>> {
        match self.at_mut(path)? {
            Self::Tabs(tabs) => Some(tabs),
            Self::Split { .. } => None,
        }
    }
    fn visit<'a>(&'a self, out: &mut Vec<&'a DockTabs<K>>) {
        match self {
            Self::Tabs(tabs) => out.push(tabs),
            Self::Split { children, .. } => {
                for child in children {
                    child.node.visit(out);
                }
            }
        }
    }
    fn normalize(&mut self, seen: &mut HashSet<K>) {
        let replacement = match self {
            Self::Tabs(tabs) => {
                tabs.tabs.retain(|tab| seen.insert(tab.id.clone()));
                tabs.repair_active();
                None
            }
            Self::Split { axis, children } => {
                for child in children.iter_mut() {
                    child.node.normalize(seen);
                }
                children.retain(|child| !matches!(&child.node, Self::Tabs(t) if t.tabs.is_empty()));
                repair_weights(children);
                let mut merged = Vec::with_capacity(children.len());
                for child in children.drain(..) {
                    match child.node {
                        Self::Split {
                            axis: inner_axis,
                            children: inner,
                        } if inner_axis == *axis => {
                            let total: f64 = inner.iter().map(|c| c.weight).sum();
                            for c in inner {
                                merged.push(DockChild {
                                    weight: child.weight * c.weight / total,
                                    node: c.node,
                                });
                            }
                        }
                        node => merged.push(DockChild {
                            weight: child.weight,
                            node,
                        }),
                    }
                }
                *children = merged;
                repair_weights(children);
                match children.len() {
                    0 => Some(Self::Tabs(DockTabs {
                        tabs: Vec::new(),
                        active: None,
                    })),
                    1 => children.pop().map(|child| child.node),
                    _ => None,
                }
            }
        };
        if let Some(node) = replacement {
            *self = node;
        }
    }
}

/// Replace invalid weights by the mean of valid ones and rescale to sum 1.
fn repair_weights<K>(children: &mut [DockChild<K>]) {
    let valid = children
        .iter()
        .map(|c| c.weight)
        .filter(|w| w.is_finite() && *w > 0.0)
        .collect::<Vec<_>>();
    let fallback = if valid.is_empty() {
        1.0
    } else {
        valid.iter().sum::<f64>() / valid.len() as f64
    };
    for child in children.iter_mut() {
        if !(child.weight.is_finite() && child.weight > 0.0) {
            child.weight = fallback;
        }
    }
    let total: f64 = children.iter().map(|c| c.weight).sum();
    if total.is_finite() && total > 0.0 {
        for child in children.iter_mut() {
            child.weight /= total;
        }
    }
}

/// Where an inserted or moved tab goes.
///
/// `container` names any tab of the target container, so placements use only
/// consumer keys and stay valid while the tree is restructured.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DockPlacement<K> {
    /// Into the container's tab strip at `index` (counted without the moved
    /// tab), or at its end for `None`. The tab becomes active.
    Tabs { container: K, index: Option<usize> },
    /// Into a new container beside the target container, splitting it in half.
    Split { container: K, side: DockSide },
    /// Into a new container along a whole edge of the dock (a quarter of it).
    Edge(DockSide),
}

/// The complete workspace: the split tree and an optional maximized container.
///
/// Keep it in a consumer `Binding` to persist, diff or undo it. Ids are the
/// consumer's stable tab keys; a container is addressed through any of its tabs.
#[derive(Clone, Debug, PartialEq)]
pub struct DockLayout<K> {
    pub root: DockNode<K>,
    /// A tab whose container fills the dock while the others stay retained.
    pub maximized: Option<K>,
}

impl<K> Default for DockLayout<K> {
    /// An empty layout: one container without tabs.
    fn default() -> Self {
        Self {
            root: DockNode::Tabs(DockTabs {
                tabs: Vec::new(),
                active: None,
            }),
            maximized: None,
        }
    }
}

impl<K: Clone + Eq + Hash> DockLayout<K> {
    /// A normalized layout without a maximized container.
    pub fn new(root: DockNode<K>) -> Self {
        let mut layout = Self {
            root,
            maximized: None,
        };
        layout.normalize();
        layout
    }
    /// Repair data edited or loaded by the consumer: later duplicate keys are
    /// dropped, empty containers and single-child splits collapse, same-axis
    /// nested splits merge, invalid weights become the mean, weights sum to 1,
    /// and missing active or maximized keys are repaired or cleared.
    pub fn normalize(&mut self) {
        let mut seen = HashSet::new();
        self.root.normalize(&mut seen);
        if self.maximized.as_ref().is_some_and(|id| !seen.contains(id)) {
            self.maximized = None;
        }
    }
    pub fn is_empty(&self) -> bool {
        matches!(&self.root, DockNode::Tabs(tabs) if tabs.tabs.is_empty())
    }
    /// All containers in depth-first order.
    pub fn containers(&self) -> Vec<&DockTabs<K>> {
        let mut out = Vec::new();
        self.root.visit(&mut out);
        out
    }
    /// All tab keys in depth-first, strip order.
    pub fn tab_ids(&self) -> Vec<K> {
        self.containers()
            .into_iter()
            .flat_map(|tabs| tabs.tabs.iter().map(|tab| tab.id.clone()))
            .collect()
    }
    pub fn tab(&self, id: &K) -> Option<&DockTab<K>> {
        self.container(id)?.tabs.iter().find(|tab| &tab.id == id)
    }
    /// The container holding `id`.
    pub fn container(&self, id: &K) -> Option<&DockTabs<K>> {
        self.containers().into_iter().find(|tabs| tabs.contains(id))
    }
    /// The container that fills the dock, if any.
    pub fn maximized_container(&self) -> Option<&DockTabs<K>> {
        self.container(self.maximized.as_ref()?)
    }
    /// Show `id` in its container.
    pub fn activate(&mut self, id: &K) -> bool {
        let Some(path) = self.root.path_of(id) else {
            return false;
        };
        let Some(tabs) = self.root.tabs_mut(&path) else {
            return false;
        };
        let changed = tabs.active.as_ref() != Some(id);
        tabs.active = Some(id.clone());
        changed
    }
    /// Replace a tab title. Blank titles are rejected.
    pub fn rename(&mut self, id: &K, title: impl Into<String>) -> bool {
        let title = title.into();
        if title.trim().is_empty() {
            return false;
        }
        let Some(path) = self.root.path_of(id) else {
            return false;
        };
        let Some(tab) = self
            .root
            .tabs_mut(&path)
            .and_then(|tabs| tabs.tabs.iter_mut().find(|tab| &tab.id == id))
        else {
            return false;
        };
        if tab.title == title {
            return false;
        }
        tab.title = title;
        true
    }
    /// Add a new tab. An empty layout accepts any placement; otherwise the
    /// target container must exist and the key must be new.
    pub fn insert(&mut self, tab: DockTab<K>, at: DockPlacement<K>) -> bool {
        if self.tab(&tab.id).is_some() {
            return false;
        }
        if self.is_empty() {
            self.root = DockNode::tabs(vec![tab]);
            self.maximized = None;
            return true;
        }
        if !self.place(tab, &at) {
            return false;
        }
        self.normalize();
        true
    }
    /// Move a tab within its strip, into another container, beside a
    /// container or to a dock edge. Moving the only tab of a container into or
    /// beside that same container is rejected.
    pub fn move_tab(&mut self, id: &K, to: DockPlacement<K>) -> bool {
        let Some(source) = self.container(id) else {
            return false;
        };
        let alone = source.tabs.len() == 1;
        // A placement named by the moved tab itself means its own container;
        // after removal it is addressed through a remaining sibling.
        let sibling = source
            .tabs
            .iter()
            .find(|tab| &tab.id != id)
            .map(|tab| tab.id.clone());
        let same_container = match &to {
            DockPlacement::Tabs { container, .. } | DockPlacement::Split { container, .. } => {
                let Some(target) = self.container(container) else {
                    return false;
                };
                target.contains(id)
            }
            DockPlacement::Edge(_) => false,
        };
        let edge = matches!(to, DockPlacement::Edge(_));
        if alone && (same_container || (edge && self.containers().len() == 1)) {
            return false;
        }
        let anchor = |container: K| {
            if &container == id {
                sibling.clone().unwrap_or(container)
            } else {
                container
            }
        };
        let to = match to {
            DockPlacement::Tabs { container, index } => DockPlacement::Tabs {
                container: anchor(container),
                index,
            },
            DockPlacement::Split { container, side } => DockPlacement::Split {
                container: anchor(container),
                side,
            },
            edge @ DockPlacement::Edge(_) => edge,
        };
        let keep_maximized = same_container && matches!(to, DockPlacement::Tabs { .. });
        let Some(tab) = self.remove(id) else {
            return false;
        };
        if !self.place(tab, &to) {
            return false;
        }
        if !keep_maximized {
            self.maximized = None;
        }
        self.normalize();
        true
    }
    /// Remove a tab. Its container collapses when it was the last tab. A
    /// maximized container stays maximized while it has tabs.
    pub fn close(&mut self, id: &K) -> Option<DockTab<K>> {
        let tab = self.remove(id)?;
        self.normalize();
        Some(tab)
    }
    /// Close the closable tabs of `id`'s container other than `id`.
    pub fn close_others(&mut self, id: &K) -> Vec<DockTab<K>> {
        let Some(tabs) = self.container(id) else {
            return Vec::new();
        };
        let others = tabs
            .tabs
            .iter()
            .filter(|tab| &tab.id != id && tab.closable)
            .map(|tab| tab.id.clone())
            .collect::<Vec<_>>();
        others
            .iter()
            .filter_map(|other| self.close(other))
            .collect()
    }
    /// Close every closable tab of `id`'s container.
    pub fn close_container(&mut self, id: &K) -> Vec<DockTab<K>> {
        let Some(tabs) = self.container(id) else {
            return Vec::new();
        };
        let closing = tabs
            .tabs
            .iter()
            .filter(|tab| tab.closable)
            .map(|tab| tab.id.clone())
            .collect::<Vec<_>>();
        closing.iter().filter_map(|tab| self.close(tab)).collect()
    }
    /// Let `id`'s container fill the dock.
    pub fn maximize(&mut self, id: &K) -> bool {
        if self.tab(id).is_none() || self.maximized.as_ref() == Some(id) {
            return false;
        }
        self.maximized = Some(id.clone());
        true
    }
    /// Show every container again.
    pub fn restore(&mut self) -> bool {
        self.maximized.take().is_some()
    }
    /// Replace the weights of the split at `path` (child indices from the
    /// root; `[]` is a root split). The count must match its children.
    pub fn set_weights(&mut self, path: &[usize], weights: &[f64]) -> bool {
        let Some(DockNode::Split { children, .. }) = self.root.at_mut(path) else {
            return false;
        };
        if children.len() != weights.len() {
            return false;
        }
        for (child, weight) in children.iter_mut().zip(weights) {
            child.weight = *weight;
        }
        self.normalize();
        true
    }

    /// Take a tab out without collapsing, so other paths stay valid.
    fn remove(&mut self, id: &K) -> Option<DockTab<K>> {
        let path = self.root.path_of(id)?;
        let tabs = self.root.tabs_mut(&path)?;
        let index = tabs.tabs.iter().position(|tab| &tab.id == id)?;
        let tab = tabs.tabs.remove(index);
        if tabs.active.as_ref() == Some(id) {
            let next = index.min(tabs.tabs.len().saturating_sub(1));
            tabs.active = tabs.tabs.get(next).map(|tab| tab.id.clone());
        }
        if self.maximized.as_ref() == Some(id) {
            self.maximized = tabs.active.clone();
        }
        Some(tab)
    }

    fn place(&mut self, tab: DockTab<K>, at: &DockPlacement<K>) -> bool {
        match at {
            DockPlacement::Tabs { container, index } => {
                let Some(path) = self.root.path_of(container) else {
                    return false;
                };
                let Some(tabs) = self.root.tabs_mut(&path) else {
                    return false;
                };
                let index = index.unwrap_or(tabs.tabs.len()).min(tabs.tabs.len());
                tabs.active = Some(tab.id.clone());
                tabs.tabs.insert(index, tab);
                true
            }
            DockPlacement::Split { container, side } => {
                let Some(path) = self.root.path_of(container) else {
                    return false;
                };
                let node = DockNode::tabs(vec![tab]);
                split_at(&mut self.root, &path, *side, node);
                true
            }
            DockPlacement::Edge(side) => {
                let node = DockNode::tabs(vec![tab]);
                let axis = side.axis();
                let root = std::mem::replace(
                    &mut self.root,
                    DockNode::Tabs(DockTabs {
                        tabs: Vec::new(),
                        active: None,
                    }),
                );
                self.root = match root {
                    DockNode::Split {
                        axis: root_axis,
                        mut children,
                    } if root_axis == axis => {
                        let total: f64 = children.iter().map(|c| c.weight).sum();
                        let child = DockChild {
                            weight: total / 3.0,
                            node,
                        };
                        if side.leading() {
                            children.insert(0, child);
                        } else {
                            children.push(child);
                        }
                        DockNode::Split { axis, children }
                    }
                    root if side.leading() => DockNode::split(axis, [(0.25, node), (0.75, root)]),
                    root => DockNode::split(axis, [(0.75, root), (0.25, node)]),
                };
                true
            }
        }
    }
}

/// Split the node at `path`: join its parent split on the same axis, or
/// replace it by a new two-child split.
fn split_at<K: Clone + PartialEq>(
    root: &mut DockNode<K>,
    path: &[usize],
    side: DockSide,
    node: DockNode<K>,
) {
    let axis = side.axis();
    if let Some((index, parent_path)) = path.split_last() {
        let parent = parent_path
            .iter()
            .try_fold(&mut *root, |node, i| match node {
                DockNode::Split { children, .. } => children.get_mut(*i).map(|c| &mut c.node),
                DockNode::Tabs(_) => None,
            });
        if let Some(DockNode::Split {
            axis: parent_axis,
            children,
        }) = parent
            && *parent_axis == axis
            && *index < children.len()
        {
            let weight = children[*index].weight / 2.0;
            children[*index].weight = weight;
            let at = if side.leading() { *index } else { index + 1 };
            children.insert(at, DockChild { weight, node });
            return;
        }
    }
    let mut target = root;
    for i in path {
        let DockNode::Split { children, .. } = target else {
            return;
        };
        let Some(child) = children.get_mut(*i) else {
            return;
        };
        target = &mut child.node;
    }
    let existing = std::mem::replace(
        target,
        DockNode::Tabs(DockTabs {
            tabs: Vec::new(),
            active: None,
        }),
    );
    *target = if side.leading() {
        DockNode::split(axis, [(0.5, node), (0.5, existing)])
    } else {
        DockNode::split(axis, [(0.5, existing), (0.5, node)])
    };
}
