//! Role: Test element queries and inspection.
//!
//! Reads on an `Element`: scoped queries, its accessibility (as the host
//! published it, falling back to the retained tree for nodes without a
//! role), its text and state.

use kivact_engine::testing::SemanticsNode;
use kivact_wire::{self as p, SceneLayer};

use super::model::{Element, Query};
use crate::diagnostics::Why;
use crate::view::state::ViewState;
use crate::view::{A11yAction, Role};
use kivact_style::draw::RegionId;

impl<'a> Element<'a> {
    /// Why this element's props have their values (effects, reads, writes).
    pub fn why(&self) -> Why {
        crate::diagnostics::why::explain(&self.screen.rt, self.node)
    }

    /// Search within this element's subtree.
    #[track_caller]
    pub fn get_by_role(&self, role: Role, name: &str) -> Element<'a> {
        self.screen
            .get(Query::Role(role, name.into()), Some(self.node))
    }
    #[track_caller]
    pub fn get_by_text(&self, text: &str) -> Element<'a> {
        self.screen.get(Query::Text(text.into()), Some(self.node))
    }
    #[track_caller]
    pub fn get_by_test_id(&self, id: &str) -> Element<'a> {
        self.screen.get(Query::TestId(id.into()), Some(self.node))
    }
    pub fn all_by_test_id(&self, id: &str) -> Vec<Element<'a>> {
        self.screen.all(Query::TestId(id.into()), Some(self.node))
    }

    /// The element's child nodes on the host, in order (structure, not
    /// semantics: nodes assistive technology does not see are included).
    pub fn children(&self) -> Vec<Element<'a>> {
        let host = self.screen.host.borrow();
        host.children(self.node)
            .unwrap_or_default()
            .iter()
            .map(|c| Element {
                screen: self.screen,
                node: *c,
                region: None,
            })
            .collect()
    }

    /// The node the host published for this element, structural or not.
    pub(super) fn published_any(&self) -> Option<SemanticsNode> {
        let host = self.screen.host.borrow();
        let id = host.semantics_id(self.node, self.region)?;
        host.semantics()?.nodes.into_iter().find(|n| n.id.0 == id)
    }

    /// The node the host published for this element, if it is exposed to
    /// assistive technology (it has a role).
    fn published(&self) -> Option<SemanticsNode> {
        self.published_any()
            .filter(|n| self.region.is_some() || n.role != p::Role::Generic)
    }

    /// Where assistive technology sees the element, in logical window
    /// coordinates: the bounds the host published, mapped by the
    /// published scrolls and transforms of its ancestors and its own, as
    /// platforms (AccessKit) map them. `None` when it was not published.
    pub fn accessibility_bounds(&self) -> Option<crate::draw::Rect> {
        let host = self.screen.host.borrow();
        let id = host.semantics_id(self.node, self.region)?;
        let tree = host.semantics()?;
        let node = tree.nodes.iter().find(|n| n.id.0 == id)?.id;
        let r = tree.window_bounds(node)?;
        Some(r)
    }

    /// Accessible name (or region name), as the host published it.
    pub fn name(&self) -> String {
        if let Some(n) = self.published() {
            return n.label;
        }
        match self.region {
            Some(r) => self.region_info(r).map(|x| x.name).unwrap_or_default(),
            None => self.screen.name_of(self.node),
        }
    }

    /// Accessible description, as the host published it (empty when the
    /// element is not exposed or has none).
    pub fn description(&self) -> String {
        self.published().map(|n| n.description).unwrap_or_default()
    }

    /// The elements the host published as describing this one
    /// (`.described_by(..)`), in order. Empty when this element is not
    /// exposed or has none.
    pub fn described_by(&self) -> Vec<Element<'a>> {
        self.related(|p| p.described_by.iter().map(|id| id.0).collect())
    }

    /// The elements the host published as controlled by this one
    /// (`.controls(..)`), in order.
    pub fn controls(&self) -> Vec<Element<'a>> {
        self.related(|p| p.controls.iter().map(|id| id.0).collect())
    }

    /// The elements the host published as labelling this one
    /// (`.labelled_by(..)`), in order.
    pub fn labelled_by(&self) -> Vec<Element<'a>> {
        self.related(|p| p.labelled_by.iter().map(|id| id.0).collect())
    }

    /// The elements of one relation of the published node, in order.
    fn related(
        &self,
        pick: impl FnOnce(&kivact_engine::testing::SemanticsProperties) -> Vec<u64>,
    ) -> Vec<Element<'a>> {
        let ids = self
            .published()
            .and_then(|n| n.properties)
            .map(|p| pick(&p))
            .unwrap_or_default();
        let host = self.screen.host.borrow();
        ids.into_iter()
            .filter_map(|id| host.semantics_target(id))
            .map(|(node, region)| Element {
                screen: self.screen,
                node,
                region,
            })
            .collect()
    }

    fn region_info(&self, r: RegionId) -> Option<p::Region> {
        let tree = ViewState::tree(&self.screen.rt);
        [SceneLayer::Draw, SceneLayer::DrawOver]
            .into_iter()
            .filter_map(|layer| tree.scene(self.node, layer))
            .find_map(|s| s.regions.iter().find(|x| x.id.0 == r.0).cloned())
    }

    /// Accessible role, as the host published it: `None` when the host
    /// does not expose the node to assistive technology (it has no role, or
    /// is an unnamed image without actions or regions).
    pub fn role(&self) -> Option<Role> {
        match self.published_any() {
            Some(n) if self.region.is_none() && n.role == p::Role::Generic => None,
            Some(n) => Some(n.role),
            None => match self.region {
                Some(r) => self.region_info(r).map(|x| x.role),
                None => self.screen.role_of(self.node),
            },
        }
    }

    /// The actions the host offers assistive technology on the element
    /// now (none when it is not exposed, disabled, hidden or outside the
    /// open modal).
    pub fn a11y_actions(&self) -> Vec<A11yAction> {
        self.published_any().map(|n| n.actions).unwrap_or_default()
    }

    /// Text content of a text node.
    pub fn text(&self) -> String {
        ViewState::tree(&self.screen.rt)
            .content(self.node)
            .unwrap_or_default()
    }

    /// The published editor's `(anchor, caret)` in Unicode scalar offsets
    /// of its whole text, cut to the lines published (those around the
    /// caret and those shown). Returns `None` when no editing snapshot is
    /// published, including passwords.
    pub fn text_selection(&self) -> Option<(u32, u32)> {
        Some(self.published()?.properties?.text?.selection)
    }

    /// The text runs the element publishes to assistive technology, one
    /// string per run (a native editor's lines around its caret, or the
    /// runs a node gave with `NodeExt::semantics_text`), in reading order.
    /// `None` when it publishes none, including passwords.
    pub fn text_runs(&self) -> Option<Vec<String>> {
        let text = self.published()?.properties?.text?;
        Some(text.runs.iter().map(|run| run.value.clone()).collect())
    }

    /// The published editing caret in logical window coordinates, after the
    /// published scrolls and transforms. Protected text has no public snapshot.
    pub fn text_caret(&self) -> Option<crate::draw::Rect> {
        let node = self.published()?;
        let caret = node.properties.as_ref()?.text.as_ref()?.caret;
        self.screen
            .host
            .borrow()
            .semantics()?
            .window_rect(node.id, caret)
    }

    /// Accessible value (a text input's value as the host edits it). A
    /// published editor gives assistive technology its text as runs of the
    /// lines around its caret, not as a value: its value here is the text
    /// it edits (`None` for a password).
    pub fn value(&self) -> Option<String> {
        if let Some(n) = self.published() {
            let editor = self.region.is_none()
                && n.properties
                    .as_ref()
                    .is_some_and(|p| p.text.is_some() || p.protected);
            if editor {
                return ViewState::tree(&self.screen.rt)
                    .reader()?
                    .edit_value(self.node);
            }
            return n.value;
        }
        if let Some(r) = self.region {
            return self.region_info(r).and_then(|x| x.value);
        }
        match (
            ViewState::tree(&self.screen.rt).input_value(self.node),
            ViewState::tree(&self.screen.rt).value(self.node),
        ) {
            (Some(v), _) | (None, Some(v)) => Some(v),
            _ => None,
        }
    }

    pub fn is_disabled(&self) -> bool {
        match self.published() {
            Some(n) => n.disabled,
            None => matches!(
                ViewState::tree(&self.screen.rt).disabled(self.node),
                Some(true)
            ),
        }
    }

    pub fn is_selected(&self) -> bool {
        match self.published() {
            Some(n) => n.selected,
            _ => matches!(
                ViewState::tree(&self.screen.rt).selected(self.node),
                Some(true)
            ),
        }
    }

    pub fn is_checked(&self) -> Option<bool> {
        match self.published() {
            Some(n) => n.checked,
            _ => ViewState::tree(&self.screen.rt)
                .checked(self.node)
                .flatten(),
        }
    }

    /// Whether the host publishes a partially checked control to AT.
    /// Its binary value is still available through `is_checked`.
    pub fn is_indeterminate(&self) -> bool {
        self.published()
            .and_then(|n| n.properties)
            .is_some_and(|p| p.indeterminate)
    }

    pub fn is_expanded(&self) -> Option<bool> {
        match self.published() {
            Some(n) => n.expanded,
            _ => ViewState::tree(&self.screen.rt)
                .expanded(self.node)
                .flatten(),
        }
    }

    /// The depth the host publishes for this element (`.level(..)`),
    /// counted from 1. `None` when not exposed or not set.
    pub fn level(&self) -> Option<u32> {
        self.published()
            .and_then(|n| n.properties)
            .and_then(|p| p.level)
    }

    /// The set size the host publishes for this element (`.set_size(..)`).
    pub fn set_size(&self) -> Option<u32> {
        self.published()
            .and_then(|n| n.properties)
            .and_then(|p| p.set_size)
    }

    /// The position in its set the host publishes for this element
    /// (`.pos_in_set(..)`), counted from 1.
    pub fn pos_in_set(&self) -> Option<u32> {
        self.published()
            .and_then(|n| n.properties)
            .and_then(|p| p.pos_in_set)
    }

    /// Whether the host publishes this element as allowing more than one
    /// selected item (`.multiselectable(true)`).
    pub fn is_multiselectable(&self) -> bool {
        self.published()
            .and_then(|n| n.properties)
            .is_some_and(|p| p.multiselectable == Some(true))
    }

    /// Whether the element has the keyboard focus.
    pub fn is_focused(&self) -> bool {
        self.screen.host.borrow().focused()
            == Some(p::FocusTarget {
                node: self.node,
                region: self.region,
            })
    }

    /// Whether the element is still mounted.
    pub fn exists(&self) -> bool {
        ViewState::tree(&self.screen.rt).get(self.node).is_some()
            && self.region.is_none_or(|r| self.region_info(r).is_some())
    }

    /// Whether the element is shown: mounted, not hidden, with a size.
    pub fn is_visible(&self) -> bool {
        self.screen.host.borrow().shown(self.node)
            && self
                .host_rect()
                .is_some_and(|r| r.width > 0.0 && r.height > 0.0)
    }

    /// Number of draw commands in the node's scene (canvas/draw layers).
    pub fn scene_len(&self) -> usize {
        let tree = ViewState::tree(&self.screen.rt);
        [SceneLayer::Draw, SceneLayer::DrawOver]
            .into_iter()
            .filter_map(|layer| tree.scene(self.node, layer))
            .map(|s| s.cmds.len())
            .sum()
    }
}

impl Element<'_> {
    /// Numeric minimum, maximum and current value, as published to AT.
    pub fn value_range(&self) -> (Option<f64>, Option<f64>, Option<f64>) {
        self.published()
            .and_then(|n| n.properties)
            .map(|p| (p.value_min, p.value_max, p.value_now))
            .unwrap_or_default()
    }
    /// Whether AT sees an invalid value.
    pub fn invalid(&self) -> bool {
        self.published()
            .and_then(|n| n.properties)
            .is_some_and(|p| p.invalid)
    }
    /// Whether AT sees read-only text.
    pub fn read_only(&self) -> bool {
        self.published()
            .and_then(|n| n.properties)
            .is_some_and(|p| p.read_only)
    }
    /// Whether AT sees protected text (the value is omitted).
    pub fn password(&self) -> bool {
        self.published()
            .and_then(|n| n.properties)
            .is_some_and(|p| p.protected)
    }
    /// Live announcement policy, as published to AT.
    pub fn live_region(&self) -> crate::view::LiveRegion {
        self.published()
            .and_then(|n| n.properties)
            .and_then(|p| p.live_region)
            .unwrap_or_default()
    }
    /// Sort direction, as published to AT.
    pub fn sort(&self) -> crate::view::SortDirection {
        self.published()
            .and_then(|n| n.properties)
            .map(|p| p.sort)
            .unwrap_or_default()
    }
    /// Orientation, as published to AT.
    pub fn orientation(&self) -> crate::view::Orientation {
        self.published()
            .and_then(|n| n.properties)
            .map(|p| p.orientation)
            .unwrap_or_default()
    }
    /// Whether AT identifies this other element as the active descendant.
    pub fn has_active_descendant(&self, other: &Element<'_>) -> bool {
        let target = other.published().map(|n| n.id);
        target.is_some()
            && self
                .published()
                .and_then(|n| n.properties)
                .and_then(|p| p.active_descendant)
                == target
    }
}
