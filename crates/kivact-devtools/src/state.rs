//! Role: Every state type of the DevTools app: connection, session and inspected tree.

use std::path::PathBuf;
use std::rc::Rc;

use kivact::prelude::*;
use kivact_devtools_protocol as p;

use crate::source::Source;

/// The window the app inspects when it does not know better.
pub const MAIN_WINDOW: p::WindowId = 1;

/// What the app is connected to (plain data, so it can be a `dynamic` source).
#[derive(Clone, PartialEq)]
pub enum Connection {
    /// Listing the running targets (the first screen without `--target`).
    Searching,
    /// The list to choose from (empty: nothing is running).
    Choosing(Vec<(PathBuf, p::DiscoveryFile)>),
    /// Connecting to the discovery file.
    Connecting(PathBuf),
    /// Connected to a target.
    Connected(Target),
    /// The connection ended or could not be opened.
    Lost {
        /// The discovery file.
        path: PathBuf,
        /// The name of the app.
        app_name: String,
        /// Why the connection is gone.
        reason: String,
    },
}

/// The target of the connection (plain data, so it can live in `Connection`).
#[derive(Clone, PartialEq)]
pub struct Target {
    /// Where the data comes from.
    pub source: Source,
    /// The app.
    pub app: p::AppInfo,
    /// The windows at connect time.
    pub windows: Vec<p::WindowInfo>,
    /// The configuration at connect time.
    pub config: Rc<p::DevtoolsConfig>,
    /// The discovery file; `None` for a test fixture.
    pub path: Option<PathBuf>,
}

/// The handles of one connected session; made once per `Connected` branch.
#[derive(Clone, Copy)]
pub(crate) struct Session {
    /// Written once, never changes while the branch lives.
    pub(crate) target: Var<Target>,
    /// To move to `Lost`.
    pub(crate) connection: Signal<Connection>,
    /// The open windows.
    pub(crate) windows: Signal<Vec<p::WindowInfo>>,
    /// The tree and node the app inspects.
    pub(crate) inspected: Signal<Inspected>,
    /// The recorded turns, newest last.
    pub(crate) turns: Signal<Vec<p::TurnGraph>>,
    /// The recorded frames.
    pub(crate) frames: Signal<Vec<p::FrameInfo>>,
    /// The reported errors.
    pub(crate) errors: Signal<Vec<p::ErrorEntry>>,
    /// The `show_table` results.
    pub(crate) tables: Signal<Vec<p::ShowTableParams>>,
    /// The selected panel.
    pub(crate) panel: Signal<Panel>,
    /// Panels shown in their own window.
    pub(crate) popped: Signal<Vec<Panel>>,
    // xtask-allow(state-fields): independent: the last RPC failure is reported separately from the panel and its pop-outs
    /// The last failure to show in the status bar.
    pub(crate) status: Signal<Option<String>>,
    /// The detail of the selected node.
    pub(crate) detail: Resource<Option<p::NodeDetail>>,
    /// What is being recorded.
    pub(crate) recording: Resource<p::RecordingSettings>,
    /// Every style override that is set.
    pub(crate) overrides: Resource<p::StyleOverrideList>,
    /// The environment emulation.
    pub(crate) emulation: Resource<p::Emulation>,
    /// The debugging overlays (`get_overlays`).
    pub(crate) overlays: Resource<p::OverlaySettings>,
    /// The review of this session.
    pub(crate) review: Resource<p::ReviewDocument>,
}

/// The window and tree the app inspects, and the selected node.
#[derive(PartialEq)]
pub(crate) enum Inspected {
    /// The tree is not loaded yet; `select` is the node to select once it is.
    Loading {
        /// The window.
        window: p::WindowId,
        /// The node to select when the tree arrives.
        select: Option<p::NodeId>,
    },
    /// The loaded tree.
    Loaded {
        /// The window.
        window: p::WindowId,
        /// The tree.
        tree: p::TreeSnapshot,
        /// The selected node.
        selected: Option<p::NodeId>,
    },
}

impl Inspected {
    /// The inspected window.
    pub(crate) fn window(&self) -> p::WindowId {
        match self {
            Inspected::Loading { window, .. } | Inspected::Loaded { window, .. } => *window,
        }
    }

    /// The selected node.
    pub(crate) fn selected(&self) -> Option<p::NodeId> {
        match self {
            Inspected::Loaded { selected, .. } => *selected,
            Inspected::Loading { .. } => None,
        }
    }

    /// Select a node in the loaded tree.
    pub(crate) fn select(&mut self, node: Option<p::NodeId>) {
        if let Inspected::Loaded { selected, .. } = self {
            *selected = node;
        }
    }

    /// Accept a tree answer for the window being loaded; discard every other answer.
    pub(crate) fn finish_load(&mut self, tree: p::TreeSnapshot) {
        let Inspected::Loading { window, select } = self else {
            return;
        };
        let (window, select) = (*window, *select);
        if window != tree.window {
            return;
        }
        let selected = select.filter(|id| tree.nodes.iter().any(|n| n.id == *id));
        *self = Inspected::Loaded {
            window,
            tree,
            selected,
        };
    }

    /// Select a node of `window`; `true` means the caller must load that tree first.
    pub(crate) fn select_in(&mut self, window: p::WindowId, node: p::NodeId) -> bool {
        if let Inspected::Loaded {
            window: loaded,
            tree,
            selected,
        } = self
            && *loaded == window
            && tree.nodes.iter().any(|n| n.id == node)
        {
            *selected = Some(node);
            return false;
        }
        *self = Inspected::Loading {
            window,
            select: Some(node),
        };
        true
    }

    /// Follow a window list: `true` means the inspected window is gone and the main window is loading.
    pub(crate) fn keep_window(&mut self, windows: &[p::WindowInfo]) -> bool {
        if windows.iter().any(|w| w.id == self.window()) {
            return false;
        }
        *self = Inspected::Loading {
            window: MAIN_WINDOW,
            select: None,
        };
        true
    }

    /// Apply the changes of a `tree_changed` notification to the loaded tree.
    pub(crate) fn apply_diff(&mut self, change: &p::TreeChanged) {
        let Inspected::Loaded {
            window,
            tree,
            selected,
        } = self
        else {
            return;
        };
        if *window != change.window {
            return;
        }
        tree.revision = change.revision;
        for upserted in &change.upserted {
            match tree.nodes.iter_mut().find(|node| node.id == upserted.id) {
                Some(existing) => *existing = upserted.clone(),
                None => tree.nodes.push(upserted.clone()),
            }
        }
        tree.nodes.retain(|node| !change.removed.contains(&node.id));
        if let Some(id) = *selected
            && !tree.nodes.iter().any(|node| node.id == id)
        {
            *selected = None;
        }
    }
}

/// Append `item`, dropping the oldest items beyond `max`.
pub(crate) fn push_capped<T>(items: &mut Vec<T>, item: T, max: usize) {
    items.push(item);
    if items.len() > max {
        let excess = items.len() - max;
        items.drain(..excess);
    }
}

/// The greatest number of turns kept.
pub(crate) const MAX_TURNS: usize = 1000;
/// The greatest number of frames kept.
pub(crate) const MAX_FRAMES: usize = 3000;
/// The greatest number of errors kept.
pub(crate) const MAX_ERRORS: usize = 1000;
/// The greatest number of tables kept.
pub(crate) const MAX_TABLES: usize = 50;

/// All overlays off; protocol's `OverlaySettings` has no `Default`.
pub(crate) fn overlays_off() -> p::OverlaySettings {
    p::OverlaySettings {
        paint_flashing: false,
        layout_lint: false,
        update_heatmap: false,
        tab_order: false,
        selected_only: false,
    }
}

/// One panel of the app.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Panel {
    /// The element tree.
    Elements,
    /// Recorded turns.
    Updates,
    /// Frame timings.
    Performance,
    /// The accessibility tree.
    Accessibility,
    /// The input log.
    Input,
    /// Running animations.
    Animations,
    /// Tasks and resources.
    Async,
    /// Theme tokens.
    Theme,
    /// Errors and notes.
    Console,
    /// Environment emulation.
    Emulation,
    /// The review.
    Review,
    /// Scripts and their tables.
    Scripts,
}

impl Panel {
    /// Every panel, in tab order.
    pub(crate) const ALL: [Panel; 12] = [
        Panel::Elements,
        Panel::Updates,
        Panel::Performance,
        Panel::Accessibility,
        Panel::Input,
        Panel::Animations,
        Panel::Async,
        Panel::Theme,
        Panel::Console,
        Panel::Emulation,
        Panel::Review,
        Panel::Scripts,
    ];

    /// The tab title.
    pub(crate) fn title(self) -> &'static str {
        match self {
            Panel::Elements => "Elements",
            Panel::Updates => "Updates",
            Panel::Performance => "Performance",
            Panel::Accessibility => "Accessibility",
            Panel::Input => "Input",
            Panel::Animations => "Animations",
            Panel::Async => "Async",
            Panel::Theme => "Theme",
            Panel::Console => "Console",
            Panel::Emulation => "Emulation",
            Panel::Review => "Review",
            Panel::Scripts => "Scripts",
        }
    }
}

#[cfg(test)]
mod tests {
    //! Role: Tests of the pure state transitions of the app.

    use super::*;

    #[test]
    fn finish_load_discards_another_window() {
        let mut inspected = Inspected::Loading {
            window: MAIN_WINDOW,
            select: None,
        };
        inspected.finish_load(snapshot(2, &[1]));
        assert!(matches!(
            inspected,
            Inspected::Loading {
                window: MAIN_WINDOW,
                select: None
            }
        ));
    }

    #[test]
    fn finish_load_keeps_a_select_that_is_in_the_tree() {
        let mut inspected = Inspected::Loading {
            window: MAIN_WINDOW,
            select: Some(2),
        };
        inspected.finish_load(snapshot(MAIN_WINDOW, &[1, 2]));
        assert_eq!(inspected.selected(), Some(2));

        let mut inspected = Inspected::Loading {
            window: MAIN_WINDOW,
            select: Some(9),
        };
        inspected.finish_load(snapshot(MAIN_WINDOW, &[1, 2]));
        assert_eq!(inspected.selected(), None);
    }

    #[test]
    fn apply_diff_upserts_removes_and_clears_a_removed_selection() {
        let mut inspected = Inspected::Loaded {
            window: MAIN_WINDOW,
            tree: snapshot(MAIN_WINDOW, &[1, 2]),
            selected: Some(2),
        };
        let mut replacement = element(2);
        replacement.update_count = 4;
        inspected.apply_diff(&changed(
            MAIN_WINDOW,
            vec![replacement, element(3)],
            vec![1],
        ));
        assert_eq!(inspected.selected(), Some(2));
        let Inspected::Loaded { tree, .. } = &inspected else {
            panic!("invariant: the tree stays loaded");
        };
        assert_eq!(tree.revision, 2);
        assert_eq!(
            tree.nodes.iter().map(|node| node.id).collect::<Vec<_>>(),
            vec![2, 3]
        );
        assert_eq!(tree.nodes[0].update_count, 4);

        inspected.apply_diff(&changed(MAIN_WINDOW, Vec::new(), vec![2]));
        assert_eq!(inspected.selected(), None);
    }

    #[test]
    fn select_in_another_window_starts_loading_with_the_selection() {
        let mut inspected = Inspected::Loaded {
            window: MAIN_WINDOW,
            tree: snapshot(MAIN_WINDOW, &[2]),
            selected: None,
        };
        assert!(inspected.select_in(2, 2));
        assert!(matches!(
            inspected,
            Inspected::Loading {
                window: 2,
                select: Some(2)
            }
        ));
    }

    #[test]
    fn keep_window_returns_to_main_when_the_window_is_gone() {
        let mut inspected = Inspected::Loaded {
            window: 7,
            tree: snapshot(7, &[]),
            selected: Some(3),
        };
        assert!(inspected.keep_window(&[window_info(MAIN_WINDOW)]));
        assert!(matches!(
            inspected,
            Inspected::Loading {
                window: MAIN_WINDOW,
                select: None
            }
        ));
    }

    fn element(id: p::NodeId) -> p::ElementInfo {
        serde_json::from_value(serde_json::json!({
            "id": id,
            "window": MAIN_WINDOW,
            "kind": "box",
            "hidden": false,
            "leaving": false,
            "update_count": 0
        }))
        .expect("invariant: the test element is valid")
    }

    fn snapshot(window: p::WindowId, nodes: &[p::NodeId]) -> p::TreeSnapshot {
        p::TreeSnapshot {
            window,
            root: 1,
            revision: 1,
            nodes: nodes.iter().map(|id| element(*id)).collect(),
        }
    }

    fn changed(
        window: p::WindowId,
        upserted: Vec<p::ElementInfo>,
        removed: Vec<p::NodeId>,
    ) -> p::TreeChanged {
        p::TreeChanged {
            window,
            revision: 2,
            upserted,
            removed,
        }
    }

    fn window_info(id: p::WindowId) -> p::WindowInfo {
        serde_json::from_value(serde_json::json!({
            "id": id,
            "title": "Main",
            "bounds": {"x": 0.0, "y": 0.0, "width": 800.0, "height": 600.0},
            "scale": 1.0,
            "focused": true,
            "mode": "normal"
        }))
        .expect("invariant: the test window is valid")
    }
}
