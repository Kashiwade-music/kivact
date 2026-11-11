//! Role: Per-node layout columns: results, measure records, cursors and geometry changes.

use crate::LayoutTree;
use crate::calcs::{CalcAlloc, Calcs};
use crate::pass_state::Epoch;
use crate::style::node::NodeStyle;
use kivact_types::cell::Recent;
use kivact_types::{Baseline, Insets, LeafSize, MeasureRequest, Orientation, Point, Rect, Size};
use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

/// The computed layout of one node.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NodeLayout {
    /// Border-box origin relative to the parent's border box (unscrolled).
    pub location: Point,
    /// Border-box size.
    pub size: Size,
    /// Size of the content (what a scroll container scrolls over): the
    /// border box grown by how far it can scroll (taffy's scrollable
    /// overflow), never smaller than the box.
    pub content_size: Size,
    /// The content box (inside border and padding), relative to the node.
    pub content_box: Rect,
    /// Padding between the border and the content box.
    pub padding: Insets,
    /// Border widths around the padding box.
    pub border: Insets,
    /// Margins outside the border box.
    pub margin: Insets,
}

/// A measure request as the layout pass asks it: taffy's f32 values as
/// bits (known width, known height, available width, available height).
/// An absent known size and the content-based available spaces are bit
/// patterns no f32 arithmetic produces (NaNs with every payload bit set).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Ask([u32; 4]);

const UNKNOWN: u32 = u32::MAX;
const MIN_CONTENT: u32 = u32::MAX - 1;
const MAX_CONTENT: u32 = u32::MAX;

impl Ask {
    pub(crate) fn new(
        known: taffy::Size<Option<f32>>,
        available: taffy::Size<taffy::AvailableSpace>,
    ) -> Self {
        let known_bits = |v: Option<f32>| v.map_or(UNKNOWN, f32::to_bits);
        let space_bits = |s: taffy::AvailableSpace| match s {
            taffy::AvailableSpace::Definite(v) => v.to_bits(),
            taffy::AvailableSpace::MinContent => MIN_CONTENT,
            taffy::AvailableSpace::MaxContent => MAX_CONTENT,
        };
        Self([
            known_bits(known.width),
            known_bits(known.height),
            space_bits(available.width),
            space_bits(available.height),
        ])
    }

    /// The request the leaf measurer is asked (the f32 values widened to f64).
    pub(crate) fn request(self) -> MeasureRequest {
        let known = |b: u32| (b != UNKNOWN).then(|| f64::from(f32::from_bits(b)));
        let space = |b: u32| match b {
            MIN_CONTENT => kivact_types::AvailableSpace::MinContent,
            MAX_CONTENT => kivact_types::AvailableSpace::MaxContent,
            b => kivact_types::AvailableSpace::Definite(f64::from(f32::from_bits(b))),
        };
        MeasureRequest {
            known_width: known(self.0[0]),
            known_height: known(self.0[1]),
            available_width: space(self.0[2]),
            available_height: space(self.0[3]),
        }
    }
}

/// What a leaf measured to, as the layout pass reads it (f32).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Measured {
    width: f32,
    height: f32,
    baseline: f32,
    from: BaselineFrom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BaselineFrom {
    None,
    Top,
    Center,
}

impl Measured {
    pub(crate) fn of(leaf: &LeafSize) -> Self {
        let (from, baseline) = match leaf.baseline {
            None => (BaselineFrom::None, 0.0),
            Some(Baseline::FromTop(y)) => (BaselineFrom::Top, y as f32),
            Some(Baseline::FromCenter(y)) => (BaselineFrom::Center, y as f32),
        };
        Self {
            width: leaf.size.width as f32,
            height: leaf.size.height as f32,
            baseline,
            from,
        }
    }

    /// The leaf size these values are (widened back exactly).
    pub(crate) fn leaf(self) -> LeafSize {
        let y = f64::from(self.baseline);
        LeafSize {
            size: Size::new(f64::from(self.width), f64::from(self.height)),
            baseline: match self.from {
                BaselineFrom::None => None,
                BaselineFrom::Top => Some(Baseline::FromTop(y)),
                BaselineFrom::Center => Some(Baseline::FromCenter(y)),
            },
        }
    }
}

/// What a leaf measured to, by request: the last 16 requests (`sizes`), in
/// the f32 the layout reads; `incomplete` once an older one was dropped, so the record no longer
/// covers every request the last computation made (see
/// [`NodeLayoutColumns::remeasures_same`]). An edit that invalidates the leaf's layout
/// clears it; nothing else measures a leaf again, frame after frame.
#[derive(Clone, Debug, Default)]
pub(crate) struct Record {
    pub(crate) sizes: Vec<(Ask, Measured)>,
    pub(crate) state: RecordState,
}

/// Whether retained measurements cover the last computation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum RecordState {
    #[default]
    Complete,
    Incomplete,
}

/// A disposable filtered index cursor (`None`: never counted); never owns links or child IDs.
#[derive(Clone, Debug, Default)]
pub(crate) struct ChildCursor(Cell<Option<Counted>>);

/// Which children of a counted node are in flow, as counted at its epoch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Filtering {
    /// Every child is in flow: a flow index is the raw index.
    Unfiltered,
    /// Some child is out of flow; `at` is the last (filtered, raw)
    /// position looked up (`None`: none yet at this epoch).
    Filtered { at: Option<(usize, usize)> },
}

/// A cursor as counted at one epoch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Counted {
    pub(crate) epoch: Epoch,
    /// The children in flow (the viewport's: its attached fixed boxes).
    pub(crate) count: usize,
    /// Whether some child is filtered out, and where the last lookup was.
    pub(crate) filtering: Filtering,
}

impl Counted {
    /// Whether no child is filtered out.
    #[inline(always)]
    pub(crate) fn unfiltered(self) -> bool {
        matches!(self.filtering, Filtering::Unfiltered)
    }
}

impl ChildCursor {
    /// The cursor counted at `epoch`; `count` (the count and whether nothing is filtered)
    /// counts it again when it was counted at another epoch or never.
    #[inline(always)]
    pub(crate) fn at_epoch(&self, epoch: Epoch, count: impl FnOnce() -> (usize, bool)) -> Counted {
        if let Some(counted) = self.0.get()
            && counted.epoch == epoch
        {
            return counted;
        }
        let (count, unfiltered) = count();
        let filtering = if unfiltered {
            Filtering::Unfiltered
        } else {
            Filtering::Filtered { at: None }
        };
        let counted = Counted {
            epoch,
            count,
            filtering,
        };
        self.0.set(Some(counted));
        counted
    }

    /// Keep `counted` (with a new position).
    pub(crate) fn store(&self, counted: Counted) {
        self.0.set(Some(counted));
    }

    /// Forget the count: the next read counts again.
    pub(crate) fn forget(&self) {
        self.0.set(None);
    }
}

/// Derived per-slot measurement records and calc ownership; never authoritative props.
#[derive(Clone, Debug, Default)]
pub(crate) struct NodeLayoutState {
    pub(crate) calcs: Vec<u32>,
    pub(crate) measured: Record,
}

/// Values of a few slots, by layout position (`slot + 1`).
pub(crate) type SlotMap<V> =
    std::collections::HashMap<u64, V, std::hash::BuildHasherDefault<kivact_types::IdHasher>>;

/// Style, caches and optional geometry borrowed for one root computation.
#[derive(Clone, Debug)]
pub(crate) struct NodeLayoutValues {
    /// The style of each slot; slots with equal styles share one `Rc` (`recent_styles`).
    pub(crate) styles: Vec<Rc<NodeStyle>>,
    /// The default style every new or reset slot shares.
    pub(crate) default_style: Rc<NodeStyle>,
    /// The last distinct styles built from layout props: a rebuilt style equal to one of them shares its `Rc`.
    pub(crate) recent_styles: Recent<NodeStyle>,
    /// Whether its own box resolves a length against its parent's size: a
    /// percentage or calc in its size, min/max size, flex basis, margin, padding,
    /// border or inset. Only then do its cache keys keep the parent's size.
    /// Written with `styles` only: `push`, `reset`, `rebuild_style`.
    pub(crate) relative: Vec<bool>,
    pub(crate) presence_scale: Vec<Option<(Orientation, f64)>>,
    pub(crate) natural_size: Vec<Option<taffy::Size<f32>>>,
    pub(crate) caches: Vec<taffy::Cache>,
    pub(crate) unrounded: Vec<taffy::Layout>,
    pub(crate) final_layout: Vec<Option<taffy::Layout>>,
    /// Rounded geometry changes awaiting consumption by the integration.
    pub(crate) geometry_changed: Vec<GeometryChange>,
    /// The inputs of a parent's last full layout (taffy's `PerformLayout`),
    /// which lay a parent of an absolute box out again on its own (`tree`).
    /// Only a node with children keeps them (a sparse table: most nodes are leaves).
    pub(crate) performed: SlotMap<taffy::LayoutInput>,
    /// The pass number of the last `set_unrounded_layout` that changed this
    /// slot's value (taffy sets every child again when it lays out the parent).
    pub(crate) touched: Vec<u32>,
    /// The pass number of the last change to this slot or one of its
    /// descendants (the path `round_changed` visits).
    pub(crate) below: Vec<u32>,
    /// The parent's cumulative origin when this slot was last rounded;
    /// `None` until it is rounded (or after `reset`).
    pub(crate) origin: Vec<Option<(f32, f32)>>,
    /// The slots (`position + 1`) whose presence scale is set: the
    /// sparse index `scale_item_styles` walks instead of every slot.
    pub(crate) scaled: BTreeSet<usize>,
    /// The live slots (`position + 1`) that take part in layout and that no pass
    /// has rounded since they were allocated or reset: a subtree taffy skips under
    /// a cached ancestor still needs a final layout (`finish_rounding` rounds it
    /// from its nearest rounded ancestor).
    pub(crate) never_rounded: BTreeSet<usize>,
}

impl Default for NodeLayoutValues {
    fn default() -> Self {
        let default_style = Rc::new(NodeStyle::default());
        Self {
            styles: vec![Rc::clone(&default_style)],
            default_style,
            recent_styles: Recent::default(),
            relative: vec![false],
            presence_scale: vec![None],
            natural_size: vec![None],
            caches: vec![taffy::Cache::new()],
            unrounded: vec![taffy::Layout::new()],
            final_layout: vec![None],
            geometry_changed: Vec::new(),
            performed: SlotMap::default(),
            touched: vec![0],
            below: vec![0],
            origin: vec![None],
            scaled: BTreeSet::new(),
            never_rounded: BTreeSet::new(),
        }
    }
}

/// Layout components of the primary node store, sharing its stable positions.
/// Position zero is the viewport; no identity map or topology is stored here.
///
/// One column per component: the style derived from the layout props, the
/// taffy cache, the unrounded geometry, the rounded layout (`None` until
/// laid out), the last full layout's inputs, and the child traversal
/// cursor. A node's entry is at its store
/// position plus one; position zero is the shared viewport the fixed boxes
/// are laid out in. There is no layout-only id table, no copy of the
/// links and no replay of layout operations.
///
/// Each instance belongs to one structural slot-allocation domain. Allocate a
/// column slot when the matching tree slot is created; reset it before removal,
/// replacement or reuse. All tree-taking operations require that same domain:
/// a live identity must resolve to the slot whose derived state is held here.
/// Clone columns together with their tree/state to obtain an independent domain
/// with the same mapping; unrelated trees or reordered slot allocations are not
/// interchangeable. Slots in this API are zero-based, excluding the viewport.
#[derive(Clone, Debug)]
pub(crate) struct NodeLayoutColumns {
    pub(crate) cursors: Vec<ChildCursor>,
    pub(crate) states: Vec<NodeLayoutState>,
    pub(crate) calcs: Calcs,
    pub(crate) values: NodeLayoutValues,
    /// The number of the current pass, marked into `touched` and `below`
    /// (`0` never marks: the first pass advances it to `1`).
    pub(crate) stamp: u32,
}

impl Default for NodeLayoutColumns {
    fn default() -> Self {
        Self {
            cursors: vec![ChildCursor::default()],
            states: vec![NodeLayoutState::default()],
            calcs: Calcs::default(),
            values: NodeLayoutValues::default(),
            stamp: 0,
        }
    }
}

impl NodeLayoutColumns {
    /// Advance and return the pass number, never `0` (the unmarked value).
    pub(crate) fn next_stamp(&mut self) -> u32 {
        self.stamp = self.stamp.wrapping_add(1);
        if self.stamp == 0 {
            // Wrapped: forget every old mark so none equals a new pass number.
            self.values.touched.fill(0);
            self.values.below.fill(0);
            self.stamp = 1;
        }
        self.stamp
    }
}

impl NodeLayoutColumns {
    /// Append one fresh structural node slot to every derived column.
    /// Default columns already contain the synthetic viewport at slot zero.
    /// The new slot must match the next newly allocated slot of this instance's
    /// tree domain; reusing an existing reset slot does not append columns.
    pub(crate) fn push(&mut self) {
        let index = self.states.len();
        self.cursors.push(ChildCursor::default());
        self.states.push(NodeLayoutState::default());
        let values = &mut self.values;
        values.styles.push(Rc::clone(&values.default_style));
        values.relative.push(false);
        values.presence_scale.push(None);
        values.natural_size.push(None);
        values.caches.push(taffy::Cache::new());
        values.unrounded.push(taffy::Layout::new());
        values.final_layout.push(None);
        values.touched.push(0);
        values.below.push(0);
        values.origin.push(None);
        values.never_rounded.insert(index);
    }

    /// Release calcs and reset every derived component before a slot is reused.
    /// `slot` must be a zero-based allocated slot in this instance's domain,
    /// still owned by the node being removed or replaced. Reset before publishing
    /// its availability for reuse; reset does not change authoritative links.
    pub(crate) fn reset(&mut self, slot: usize) {
        let index = slot + 1;
        self.calcs
            .free
            .extend(std::mem::take(&mut self.states[index].calcs));
        self.states[index] = NodeLayoutState::default();
        self.cursors[index] = ChildCursor::default();
        let values = &mut self.values;
        values.styles[index] = Rc::clone(&values.default_style);
        values.relative[index] = false;
        values.presence_scale[index] = None;
        values.natural_size[index] = None;
        values.caches[index] = taffy::Cache::new();
        values.unrounded[index] = taffy::Layout::new();
        values.final_layout[index] = None;
        values.performed.remove(&(index as u64));
        values.touched[index] = 0;
        values.below[index] = 0;
        values.origin[index] = None;
        values.scaled.remove(&index);
        values.never_rounded.insert(index);
    }
}

impl NodeLayoutColumns {
    /// Drain neutral changes once; dropping the drain discards unread changes.
    /// Consume after computation and before the next compute/reset/reuse of the
    /// paired tree domain.
    pub(crate) fn drain_geometry_changes(&mut self) -> GeometryChanges<'_> {
        GeometryChanges {
            values: &mut self.values,
            at: 0,
        }
    }
}

/// Whether rounded local geometry changed or only the parent's placement changed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GeometryChangeKind {
    /// Initial layout or a change to any rounded non-location geometry field.
    Local,
    /// Only rounded location changed; every local geometry field is unchanged.
    PositionOnly,
}

/// A rounded geometry observation; integrations decide paint/accessibility effects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GeometryChange {
    /// The logical identity, never a reusable slot.
    pub id: kivact_types::NodeId,
    /// Local includes initial layout; order bookkeeping is excluded.
    pub kind: GeometryChangeKind,
}

/// An allocation-free drain retaining the columns' event-buffer capacity.
pub struct GeometryChanges<'a> {
    values: &'a mut NodeLayoutValues,
    at: usize,
}

impl Iterator for GeometryChanges<'_> {
    type Item = GeometryChange;
    fn next(&mut self) -> Option<Self::Item> {
        let value = self.values.geometry_changed.get(self.at).copied()?;
        self.at += 1;
        Some(value)
    }
}

impl Drop for GeometryChanges<'_> {
    fn drop(&mut self) {
        self.values.geometry_changed.clear();
    }
}

/// The calc value `handle` scaled by `scale`: a calc is linear in its
/// basis, so it is read back as `percent% + px` and both parts are scaled.
fn scaled_calc(
    handle: *const (),
    scale: f64,
    calcs: &Calcs,
    alloc: &mut CalcAlloc<'_>,
) -> *const () {
    let px = f64::from(calcs.resolve(handle, 0.0));
    let percent = f64::from(calcs.resolve(handle, 100.0)) - px;
    alloc.calc(percent * scale, px * scale)
}

fn scaled_dimension(
    value: taffy::Dimension,
    scale: f64,
    calcs: &Calcs,
    alloc: &mut CalcAlloc<'_>,
) -> taffy::Dimension {
    use taffy::style::CompactLength;
    match value.tag() {
        CompactLength::LENGTH_TAG => taffy::Dimension::length(value.value() * scale as f32),
        CompactLength::PERCENT_TAG => taffy::Dimension::percent(value.value() * scale as f32),
        CompactLength::CALC_TAG => taffy::Dimension::calc(scaled_calc(
            value.into_raw().calc_value(),
            scale,
            calcs,
            alloc,
        )),
        _ => value,
    }
}

fn scaled_length(
    value: taffy::LengthPercentageAuto,
    scale: f64,
    calcs: &Calcs,
    alloc: &mut CalcAlloc<'_>,
) -> taffy::LengthPercentageAuto {
    use taffy::style::CompactLength;
    match value.into_raw().tag() {
        CompactLength::LENGTH_TAG => {
            taffy::LengthPercentageAuto::length(value.into_raw().value() * scale as f32)
        }
        CompactLength::PERCENT_TAG => {
            taffy::LengthPercentageAuto::percent(value.into_raw().value() * scale as f32)
        }
        CompactLength::CALC_TAG => taffy::LengthPercentageAuto::calc(scaled_calc(
            value.into_raw().calc_value(),
            scale,
            calcs,
            alloc,
        )),
        _ => value,
    }
}

pub(crate) fn scale_item_styles<Tree: LayoutTree + ?Sized>(
    tree: &Tree,
    styles: &[Rc<NodeStyle>],
    presence_scale: &[Option<(Orientation, f64)>],
    scaled: &BTreeSet<usize>,
    calcs: &Calcs,
    pass_calcs: &mut std::borrow::Cow<'_, Calcs>,
) -> BTreeMap<usize, NodeStyle> {
    let mut item_styles = BTreeMap::new();
    for &index in scaled {
        let Some((axis, scale)) = presence_scale[index] else {
            continue;
        };
        if axis == Orientation::Unspecified {
            continue;
        }
        let style = item_styles
            .entry(index)
            .or_insert_with(|| (*styles[index]).clone());
        let mut alloc = CalcAlloc {
            calcs: pass_calcs.to_mut(),
            owned: Vec::new(),
        };
        let (size, min, max) = match axis {
            Orientation::Horizontal => (
                &mut style.size.width,
                &mut style.min_size.width,
                &mut style.max_size.width,
            ),
            Orientation::Vertical => (
                &mut style.size.height,
                &mut style.min_size.height,
                &mut style.max_size.height,
            ),
            Orientation::Unspecified => continue,
        };
        *size = scaled_dimension(*size, scale, calcs, &mut alloc);
        *min = scaled_length(*min, scale, calcs, &mut alloc);
        *max = scaled_length(*max, scale, calcs, &mut alloc);
        if axis == Orientation::Horizontal {
            style.min_width_floor *= scale as f32;
        }
        if let Some(parent) = index.checked_sub(1).and_then(|slot| tree.parent(slot)) {
            let row = matches!(
                styles[parent + 1].flex_direction,
                taffy::FlexDirection::Row | taffy::FlexDirection::RowReverse
            );
            if row == (axis == Orientation::Horizontal) {
                style.flex_basis = scaled_dimension(style.flex_basis, scale, calcs, &mut alloc);
            }
        }
    }
    item_styles
}
