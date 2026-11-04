//! Role: Raw typed node properties, scenes and direct structural queries.
//!
//! A node owns one typed value per property. Derived layout, measuring, editor
//! and paint state live in Model-owned columns keyed by its stable NodeIndex.
//! Focusability receives actual editor presence from that upper owner; a raw
//! editing property is never substituted for the authoritative editor state.

use std::sync::Arc;

use crate::contract::{NodeKind, NodeProperties, NodeType, SceneLayer};
use kivact_types::{
    A11yAction, DrawList, FocusTarget, LayoutProps, Overflow, OverlayProps, PaintProps, Point,
    Position, Rect, Region, RegionId, Role, Size, StateStyles, TextStyle,
};

/// A stable position in the sole node store, used only by retained links.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NodeIndex(pub(crate) usize);

/// A retained node's primary slot, typed inputs and structural/input metadata.
#[derive(Clone, Debug)]
pub(crate) struct HostNode {
    /// The primary slot it was made for, assigned by `NodeStore::reserve`.
    pub(crate) index: NodeIndex,
    pub(crate) properties: NodeProperties,
    pub(crate) scenes: [Option<Arc<DrawList>>; 2],
    pub(crate) scroll: Point,
    pub(crate) sent_layout: Option<(Rect, Size, bool)>,
    pub(crate) sent_scroll: Option<(Point, Size, Size)>,
    /// Where its paint and layout place it (`model::transform`), read when
    /// its `Paint`, `States` or `Layout` prop is stored.
    pub(crate) place: Place,
    /// How far a sticky box is moved from its laid out position by the
    /// scroll of its scrollport (`model::interact::sticky`; zero otherwise).
    pub(crate) shift: Point,
    /// Whether it shows its enter or exit paint (`model::presence`).
    pub(crate) presence: NodePresence,
}

/// Where a node is between its mount and its removal, for its enter and
/// exit paint (`StateStyles::enter`, `StateStyles::exit`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum NodePresence {
    Arriving,
    #[default]
    Mounted,
    /// Removed by kivact and kept, with its subtree, until its exit paint
    /// is reached: drawn, but neither hit, focused nor exposed.
    Leaving(LeavingRole),
}

/// A leaving subtree root owns the requested completion notification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LeavingRole {
    Root { notify: Notify },
    Descendant,
}

/// Whether physical removal reports exit completion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Notify {
    Silent,
    OnComplete,
}

/// What a node's paint and layout say about where it is: read from its
/// `Paint`, `States` and `Layout` props when they are stored, so the walks
/// over the tree (hit testing, the paint list, the geometry reports) skip
/// the untransformed, unstuck and in-flow nodes and the containers without
/// z-indexed children at no cost.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Place {
    /// Its base paint or a state style transforms it.
    pub(crate) transformed: bool,
    /// Its base paint's z-index.
    pub(crate) z: i32,
    /// A state style sets its z-index.
    pub(crate) z_states: bool,
    /// One of its children has (or had) a z-index or a state style that
    /// sets one: its children are ordered by z-index (`Model::stacked`).
    /// Kept once set.
    pub(crate) stacks: bool,
    /// `position: sticky`: moved by its scrollport's scroll, and drawn and
    /// hit over its in-flow siblings of the same z-index (CSS paints
    /// positioned boxes after in-flow ones).
    pub(crate) sticky: bool,
    /// `position: fixed`: a layer of its own at its window position.
    pub(crate) fixed: bool,
}

impl Place {
    /// It may be ordered among its siblings by a z-index (or by being
    /// sticky).
    pub(crate) fn has_z(&self) -> bool {
        self.z != 0 || self.z_states || self.sticky
    }
}

impl HostNode {
    pub(crate) fn new(kind: NodeKind, index: NodeIndex) -> Self {
        let properties = NodeProperties::new(kind);
        let mut node = HostNode {
            properties,
            index,
            scenes: [None, None],
            scroll: Point::default(),
            sent_layout: None,
            sent_scroll: None,
            place: Place::default(),
            shift: Point::default(),
            presence: NodePresence::Mounted,
        };
        node.update_place();
        node
    }

    pub(crate) fn update_place(&mut self) {
        let base = self.paint();
        let deltas = self.states().into_iter().flat_map(|s| s.deltas());
        let (mut moved, mut z_states) = (false, false);
        for delta in deltas {
            moved |= delta.transform.is_some_and(|t| !t.is_none());
            z_states |= delta.z_index.is_some();
        }
        self.place = Place {
            transformed: moved || base.is_some_and(|p| !p.transform.is_none()),
            z: base.map_or(0, |p| p.z_index),
            z_states,
            ..self.place
        };
        let position = self.layout().map(|l| l.position);
        self.place.sticky = position == Some(Position::Sticky);
        self.place.fixed = position == Some(Position::Fixed);
    }

    pub(crate) fn semantics(&self) -> Option<&kivact_types::Semantics<FocusTarget>> {
        self.properties.semantics.as_deref()
    }
    pub(crate) fn text_data(&self) -> Option<&kivact_types::TextData> {
        match &self.properties.kind {
            NodeKind::Text(data) => Some(data),
            _ => None,
        }
    }
    pub(crate) fn input_data(&self) -> Option<&crate::contract::EditingData> {
        self.properties.editing.as_deref()
    }
    pub(crate) fn image_data(&self) -> Option<&kivact_types::ImageData> {
        match &self.properties.kind {
            NodeKind::Image(data) => Some(data),
            _ => None,
        }
    }
    pub(crate) fn intrinsic(&self) -> Option<&kivact_types::IntrinsicSize> {
        match &self.properties.kind {
            NodeKind::Box(data) => data.intrinsic.as_ref(),
            NodeKind::Image(data) => data.intrinsic.as_ref(),
            _ => None,
        }
    }
    pub(crate) fn measure(&self) -> Option<&crate::contract::LeafMeasure> {
        match &self.properties.kind {
            NodeKind::Box(data) => data.measure.as_ref(),
            _ => None,
        }
    }
    pub(crate) fn image(&self) -> Option<&kivact_types::Asset> {
        self.image_data()?.image.as_ref()
    }

    /// The node kind.
    pub(crate) fn kind(&self) -> NodeType {
        self.properties.kind.node_type()
    }

    /// The scenes of both draw layers (`Draw`, `DrawOver`).
    pub(crate) fn scenes(&self) -> &[Option<Arc<DrawList>>; 2] {
        &self.scenes
    }

    /// The scene of a draw layer.
    pub(crate) fn scene(&self, layer: SceneLayer) -> Option<&Arc<DrawList>> {
        self.scenes[layer_index(layer)].as_ref()
    }

    /// The scroll offset of a scroll container.
    pub(crate) fn scroll(&self) -> Point {
        self.scroll
    }

    pub(crate) fn name(&self) -> Option<&str> {
        self.semantics()?.name.as_deref()
    }
    pub(crate) fn content(&self) -> Option<&str> {
        self.text_data()?.content.as_deref()
    }
    pub(crate) fn spans(&self) -> &[kivact_types::TextSpan] {
        self.text_data()
            .and_then(|d| d.spans.as_deref())
            .unwrap_or(&[])
    }
    pub(crate) fn password(&self) -> bool {
        self.input_data().and_then(|d| d.password).unwrap_or(false)
    }
    /// Whether it reports its editing session when it changes.
    pub(crate) fn reports_session(&self) -> bool {
        self.input_data()
            .and_then(|d| d.report_session)
            .unwrap_or(false)
    }
    /// Whether its edits report its whole text.
    pub(crate) fn reports_text(&self) -> bool {
        self.input_data()
            .and_then(|d| d.report_text)
            .unwrap_or(false)
    }
    pub(crate) fn read_only(&self) -> bool {
        self.input_data().and_then(|d| d.read_only).unwrap_or(false)
    }
    pub(crate) fn max_length(&self) -> Option<usize> {
        self.input_data()?.max_length
    }
    pub(crate) fn placeholder(&self) -> &str {
        self.input_data()
            .and_then(|d| d.placeholder.as_deref())
            .unwrap_or("")
    }
    pub(crate) fn role(&self) -> Option<Role> {
        self.semantics()?.role
    }
    pub(crate) fn actions(&self) -> &[A11yAction] {
        self.semantics()
            .and_then(|d| d.actions.as_deref())
            .unwrap_or(&[])
    }
    pub(crate) fn layout(&self) -> Option<&LayoutProps> {
        self.properties.layout.as_deref()
    }
    pub(crate) fn paint(&self) -> Option<&PaintProps> {
        self.properties.paint.as_deref()
    }
    pub(crate) fn states(&self) -> Option<&StateStyles> {
        self.properties.states.as_deref()
    }
    pub(crate) fn text(&self) -> Option<&TextStyle> {
        self.properties.text.as_deref()
    }
    pub(crate) fn overlay(&self) -> Option<OverlayProps> {
        match &self.properties.kind {
            NodeKind::Overlay(value) => *value,
            _ => None,
        }
    }

    /// It starts a layer of its own (an overlay, or a fixed box): placed in
    /// the window, not in its parent's layout, scroll, clip or transforms.
    pub(crate) fn is_layer(&self) -> bool {
        self.kind() == NodeType::Overlay || self.place.fixed
    }

    /// An overlay that confines input while shown.
    pub(crate) fn is_modal(&self) -> bool {
        self.kind() == NodeType::Overlay && self.overlay().is_some_and(|o| o.modal)
    }

    /// `Disabled` on the node itself.
    pub(crate) fn disabled(&self) -> bool {
        self.properties.input.disabled.unwrap_or(false)
    }

    /// Hidden by its own props (`Hidden`, or a hidden layout).
    /// `HitTestable`: whether the pointer hits the node and its subtree.
    pub(crate) fn hit_testable(&self) -> bool {
        self.properties.input.hit_testable.unwrap_or(true)
    }

    pub(crate) fn hidden(&self) -> bool {
        self.properties.input.hidden.unwrap_or(false) || self.layout().is_some_and(|l| l.hidden)
    }

    /// `Selected`.
    pub(crate) fn selected(&self) -> bool {
        self.semantics().and_then(|s| s.selected).unwrap_or(false)
    }

    /// A multiline text input.
    pub(crate) fn multiline(&self) -> bool {
        self.input_data().and_then(|s| s.multiline).unwrap_or(false)
    }

    pub(crate) fn edit_line_layout(&self) -> kivact_types::EditLineLayout {
        if self.multiline() {
            self.input_data()
                .and_then(|s| s.line_layout)
                .unwrap_or_default()
        } else {
            kivact_types::EditLineLayout::default()
        }
    }

    /// Focusable by its own props: `Focusable` (a text input by default),
    /// not hidden and eligible while disabled. A tab stop must also be shown
    /// ([`crate::model::Model::is_tab_stop`]).
    pub(crate) fn focusable(&self, editor_present: bool) -> bool {
        let focusable = self.properties.input.focusable.unwrap_or(editor_present);
        focusable
            && (!self.disabled() || self.properties.input.focusable_when_disabled == Some(true))
            && !self.hidden()
    }

    /// The declared native pointer focus destination, if currently bound.
    pub(crate) fn focus_on_press(&self) -> Option<kivact_types::FocusTarget> {
        self.properties.input.focus_on_press
    }

    pub(crate) fn roving_focus(&self) -> Option<crate::contract::RovingFocus> {
        self.properties.input.roving_focus
    }

    /// Offers `Activate` and is not disabled: a click activates it.
    pub(crate) fn activatable(&self) -> bool {
        self.actions().contains(&A11yAction::Activate) && !self.disabled()
    }

    /// Listens to pointer events.
    pub(crate) fn listens_pointer(&self) -> bool {
        self.properties.input.listen_pointer.unwrap_or(false)
    }

    /// Listens to wheel events.
    pub(crate) fn listens_wheel(&self) -> bool {
        self.properties.input.listen_wheel.unwrap_or(false)
    }

    /// Which axes scroll (`overflow: Scroll`), `(x, y)`.
    pub(crate) fn scroll_axes(&self) -> (bool, bool) {
        match self.layout() {
            Some(l) => (
                l.overflow_x == Overflow::Scroll,
                l.overflow_y == Overflow::Scroll,
            ),
            None => (false, false),
        }
    }

    /// Clips its children (paint clip or a non-visible overflow).
    pub(crate) fn clips(&self) -> bool {
        self.paint().is_some_and(|p| p.clip)
            || self.layout().is_some_and(|l| {
                l.overflow_x != Overflow::Visible || l.overflow_y != Overflow::Visible
            })
    }

    /// The scene region `id` of either draw layer.
    pub(crate) fn region(&self, id: RegionId) -> Option<&Region> {
        self.scenes
            .iter()
            .flatten()
            .flat_map(|s| s.regions.iter())
            .find(|r| r.id == id)
    }

    /// The topmost scene region under a node-local point.
    pub(crate) fn region_at(&self, local: Point) -> Option<RegionId> {
        self.scenes
            .iter()
            .rev()
            .flatten()
            .flat_map(|s| s.regions.iter().rev())
            .filter(|r| !r.semantics.hidden)
            .find(|r| {
                let Some(inverse) = r.transform.invert() else {
                    return false;
                };
                let point = inverse.apply(local);
                match &r.shape {
                    Some(shape) => crate::lower::scene_shape::shape(shape).contains(point),
                    None => r.local_rect.contains(point),
                }
            })
            .map(|r| r.id)
    }
}

pub(crate) fn layer_index(layer: SceneLayer) -> usize {
    match layer {
        SceneLayer::Draw => 0,
        SceneLayer::DrawOver => 1,
    }
}
