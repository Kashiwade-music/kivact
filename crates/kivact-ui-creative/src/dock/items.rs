//! Role: Flattens a dock layout into the keyed, positioned items it renders.

use super::geometry::{Divider, Frame, regions};
use super::model::{DockAxis, DockChild, DockLayout, DockNode, DockTabs};
use std::hash::Hash;

/// One positioned sibling of the dock root.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum Item<K> {
    Chrome {
        tabs: DockTabs<K>,
        frame: Frame,
        // xtask-allow(state-fields): passive record - whether the container is shown in the flattened layout
        shown: bool,
        // xtask-allow(state-fields): passive record - whether the container is maximized in the flattened layout
        maximized: bool,
    },
    Content {
        title: String,
        frame: Frame,
        shown: bool,
    },
    Divider {
        divider: Divider,
        leading: String,
        trailing: String,
        shown: bool,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) enum ItemKey<K> {
    /// A container, named by its first tab.
    Chrome(K),
    Content(K),
    /// The split's path, the boundary's index and the split's axis. A splitter
    /// keeps the axis it was built with, so a split that turns gets a new one.
    Divider(Vec<usize>, usize, DockAxis),
}

impl<K> Item<K> {
    pub(super) fn frame(&self) -> Frame {
        match self {
            Self::Chrome { frame, .. } | Self::Content { frame, .. } => *frame,
            Self::Divider { divider, .. } => divider.span,
        }
    }
    pub(super) fn shown(&self) -> bool {
        match self {
            Self::Chrome { shown, .. }
            | Self::Content { shown, .. }
            | Self::Divider { shown, .. } => *shown,
        }
    }
}

fn first_title<K: Clone + PartialEq>(node: &DockNode<K>) -> String {
    match node {
        DockNode::Tabs(tabs) => tabs
            .active_tab()
            .map(|tab| tab.title.clone())
            .unwrap_or_default(),
        DockNode::Split { children, .. } => children
            .first()
            .map(|child| first_title(&child.node))
            .unwrap_or_default(),
    }
}

fn split_children<'a, K>(root: &'a DockNode<K>, path: &[usize]) -> Option<&'a [DockChild<K>]> {
    let node = path.iter().try_fold(root, |node, index| match node {
        DockNode::Split { children, .. } => children.get(*index).map(|child| &child.node),
        DockNode::Tabs(_) => None,
    })?;
    match node {
        DockNode::Split { children, .. } => Some(children),
        DockNode::Tabs(_) => None,
    }
}

type Entry<K> = (ItemKey<K>, Item<K>);

/// Every positioned sibling of the dock root, keyed.
pub(super) fn build_items<K: Clone + Eq + Hash>(layout: &DockLayout<K>) -> Vec<Entry<K>> {
    let (regions, dividers) = regions(&layout.root);
    let maximized = layout.maximized_container();
    let mut items = Vec::new();
    for region in regions {
        let Some(first) = region.tabs.tabs.first() else {
            continue;
        };
        let this = maximized.is_some_and(|max| std::ptr::eq(max, region.tabs));
        let (frame, shown) = match maximized {
            Some(_) if this => (Frame::FULL, true),
            Some(_) => (region.frame, false),
            None => (region.frame, true),
        };
        items.push((
            ItemKey::Chrome(first.id.clone()),
            Item::Chrome {
                tabs: region.tabs.clone(),
                frame,
                shown,
                maximized: this,
            },
        ));
        for tab in &region.tabs.tabs {
            items.push((
                ItemKey::Content(tab.id.clone()),
                Item::Content {
                    title: tab.title.clone(),
                    frame,
                    shown: shown && region.tabs.active.as_ref() == Some(&tab.id),
                },
            ));
        }
    }
    for divider in dividers {
        let children = split_children(&layout.root, &divider.path);
        let title = |index: usize| {
            children
                .and_then(|children| children.get(index))
                .map(|child| first_title(&child.node))
                .unwrap_or_default()
        };
        items.push((
            ItemKey::Divider(divider.path.clone(), divider.index, divider.axis),
            Item::Divider {
                leading: title(divider.index),
                trailing: title(divider.index + 1),
                divider,
                shown: maximized.is_none(),
            },
        ));
    }
    items
}
