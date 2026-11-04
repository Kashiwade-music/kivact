//! Role: Mounted data shared by host mounting, parts and canvas regions.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::panic::Location;
use std::rc::{Rc, Weak};

use kivact_runtime::Prop as InputProp;
use kivact_runtime::{Owner, Runtime};
use kivact_types::{IntrinsicSize, NodeId};

use crate::view::descriptor::Mcx;
use crate::view::part_data::PartBinding;
use crate::view::tree::Handlers;
use crate::view::{MouseButton, PointerEvent, PointerId};
use crate::{Signal, Value};
use kivact_style::Style;
use kivact_style::asset::ImageFit;
use kivact_style::draw::{Draw, RegionId, Size};
use kivact_types::Role;

pub(crate) type DrawFn = Rc<dyn Fn(&mut Draw, Size)>;

/// Modifiers shared by every element builder.
#[derive(Default)]
pub(crate) struct Common {
    pub(crate) exit_layout: super::exit_layout::ExitLayout,
    pub(crate) exit_complete: Option<kivact_runtime::Handler<()>>,
    /// Test identifier.
    pub test_id: Option<Rc<str>>,
    /// NodeRefs bound at mount (each once, in order).
    pub node_ref: Vec<crate::view::node_ref::NodeRef>,
    /// Style layers, merged in order (a later layer wins per property).
    pub style: Vec<StyleLayer>,
    /// How many of the first layers of `style` are base layers
    /// (`NodeExt::base_style`).
    pub base_styles: usize,
    /// Role (a builder sets its own before the user's modifiers).
    pub role: Option<Role>,
    /// Accessible name.
    pub name: Option<InputProp<String>>,
    /// Accessible description.
    pub description: Option<InputProp<String>>,
    /// Accessible value text.
    pub value: Option<InputProp<String>>,
    /// Selected state.
    pub selected: Option<InputProp<bool>>,
    /// Disabled state.
    pub disabled: Option<InputProp<bool>>,
    /// Hidden subtree.
    pub hidden: Option<InputProp<bool>>,
    pub invalid: Option<InputProp<bool>>,
    pub semantics: Option<Box<SemanticSources>>,
    /// Checked state (toggles).
    pub checked: Option<InputProp<bool>>,
    /// Mixed checked state (HTML `indeterminate`).
    pub indeterminate: Option<InputProp<bool>>,
    /// Expanded state (disclosures).
    pub expanded: Option<InputProp<bool>>,
    /// Focusability, constant or reactive (a builder sets its own before the
    /// user's modifiers).
    pub focusable: Option<InputProp<bool>>,
    pub focusable_when_disabled: bool,
    pub roving_focus: Option<kivact_engine::contract::RovingFocus>,
    /// `Some(false)`: the pointer passes through the subtree.
    pub hit_testable: Option<bool>,
    /// Native pointer focus destination, resolved through the reference binding.
    pub(crate) focus_on_press: Option<crate::view::node_ref::NodeRef>,
    /// Event handlers.
    pub handlers: Handlers,
    pub(crate) text_editing: Option<Box<crate::view::editing_data::TextEditing>>,
    /// The IME session of a node that edits its own text.
    pub(crate) ime: Option<crate::view::ime_session::ImeSession>,
    /// Behavior requirements cannot be erased by later caller modifiers.
    pub(crate) part_constraints: crate::view::part_data::PartConstraints,
    /// The activation, text and accessibility-action defaults of attached
    /// parts, which remain defaults even after later customizations. Their
    /// key, pointer and wheel handlers are entries of `handlers`.
    pub(crate) part_actions: crate::view::tree::PartActions,
    /// This part opts its own defaults into unhandled canvas-region input.
    pub(crate) part_region_events: bool,
    /// Mount hooks and references of attached parts, in attachment order.
    pub(crate) part_bindings: Vec<crate::view::part_data::PartBinding>,
    /// An attached behavior requires the existing Overlay modal focus scope.
    pub(crate) part_modal: bool,
    /// Setup declarations; drawing supplies only geometry for these identities.
    pub(crate) regions: Vec<Declaration>,
    /// Draw layer behind the children.
    pub draw: Option<DrawFn>,
    /// Draw layer over the children.
    pub draw_over: Option<DrawFn>,
    /// Size hints for layout.
    pub intrinsic: Option<IntrinsicSize>,
    /// How an image fills its box (image nodes only).
    pub image_fit: Option<InputProp<ImageFit>>,
    /// Paint an image in the text color (image nodes only).
    pub monochrome: Option<InputProp<bool>>,
    /// Creation location set by `.at(..)`; replaces the builder's call site
    /// for every node the builder creates.
    pub at: Option<&'static Location<'static>>,
}

/// One style layer: a constant or reactive style (`.style(..)`). Signals
/// read inside, the theme from `current_theme()` included, are tracked by
/// the node's style effect.
pub(crate) type StyleLayer = Rc<dyn Fn() -> Style>;

impl Common {
    /// Insert a style layer after the base layers already inserted and before
    /// every other one: a builder's base style that a caller's `.style(..)`
    /// overrides whatever the order of the calls. Base layers merge in the
    /// order they were inserted.
    pub(crate) fn prepend_style(&mut self, s: InputProp<Style>) {
        self.style.insert(self.base_styles, s.into_fn());
        self.base_styles += 1;
    }
    /// Append a style layer (`.style(..)`).
    pub(crate) fn push_style(&mut self, s: InputProp<Style>) {
        self.style.push(s.into_fn());
    }
}

pub(super) type Collection = Box<dyn FnOnce(&Mcx<'_>, Rc<Registry>, usize)>;

/// Setup declarations keep builder order across static and keyed collections.
pub(crate) enum Declaration {
    Static(RegionId, Box<Common>),
    Collection(Collection),
}

/// Which target of a node attached-part input goes to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TargetId {
    /// The node itself.
    Root,
    /// One adoption of a canvas region (a re-adopted region is a new target).
    Region { region: RegionId, generation: u64 },
}

pub(super) struct Target {
    pub(super) id: TargetId,
    pub(super) owner: Owner,
    pub(super) runtime: Weak<Runtime>,
    /// The lists hold the target's observers and part defaults (the node's
    /// region fallback too, for the node's own target).
    pub(super) handlers: Handlers,
    pub(super) actions: crate::view::tree::PartActions,
    pub(super) bindings: Vec<PartBinding>,
    pub(super) constraints: Constraints,
}

impl Target {
    /// The canvas region this target is, if it is not the node itself.
    pub(super) fn region(&self) -> Option<RegionId> {
        match self.id {
            TargetId::Root => None,
            TargetId::Region { region, .. } => Some(region),
        }
    }

    pub(super) fn live(&self) -> bool {
        self.runtime.upgrade().is_some() && self.owner.is_alive()
    }
    pub(super) fn enabled(&self) -> bool {
        self.live()
            && !self.constraints.disabled.get_untracked()
            && !self.constraints.hidden.get_untracked()
    }
}

pub(super) struct Adopted {
    pub(super) owner: Owner,
    pub(super) common: Common,
    pub(super) target: Rc<Target>,
    pub(super) bound: Cell<bool>,
}

pub(super) struct Entry {
    pub(super) id: RegionId,
    pub(super) generation: u64,
    pub(super) item_owner: Owner,
    pub(super) template: Common,
    /// `None` while the entry is not adopted: before its first adoption, and
    /// while an earlier duplicate shadows it (it is adopted again when the
    /// duplicate goes).
    pub(super) adopted: RefCell<Option<Rc<Adopted>>>,
}

#[derive(Default)]
pub(super) struct State {
    pub(super) groups: Vec<Vec<Rc<Entry>>>,
    pub(super) active: Vec<Rc<Entry>>,
    pub(super) by_id: HashMap<RegionId, Rc<Target>>,
    pub(super) duplicates: HashSet<RegionId>,
    pub(super) duplicate_keys: HashSet<u64>,
    pub(super) next: u64,
    pub(super) retired: Vec<Owner>,
}

/// Whether the node of a registry has finished mounting.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum RegistryPhase {
    /// Adopted regions wait for `Registry::bind` to bind their references and hooks.
    Mounting,
    /// A region adopted from now on binds at once.
    Mounted,
}

/// One canvas identity owns this lookup for its entire mounted lifetime.
pub(crate) struct Registry {
    pub(super) node: NodeId,
    /// The node's own target; its owner, runtime and final constraints are the registry's.
    pub(super) root: Rc<Target>,
    pub(super) at: &'static Location<'static>,
    pub(super) state: RefCell<State>,
    pub(super) revision: Signal<u64>,
    pub(super) changed: Signal<u64>,
    pub(super) phase: Cell<RegistryPhase>,
    /// The pointer gestures on the node and its regions (`mount::part_pointer`).
    pub(super) gestures: RefCell<Gestures>,
}

/// The pointer gestures of a node with parts (`mount::part_pointer`).
#[derive(Default)]
pub(super) struct Gestures {
    pub(super) hover: PrimaryHover,
    /// The capture of each pointer with a button held, in press order.
    pub(super) captures: Vec<Capture>,
    /// The last event of a primary pointer.
    pub(super) last: Option<PointerEvent>,
}

/// Where the primary pointer is.
#[derive(Default)]
pub(super) enum PrimaryHover {
    #[default]
    Outside,
    /// Inside the node, over this enabled target (or none).
    Inside(Option<Rc<Target>>),
}

pub(super) struct Capture {
    pub(super) pointer: PointerId,
    /// `None`: the gesture has no target until its release (pressed over no
    /// enabled target, or cancelled).
    pub(super) target: Option<Rc<Target>>,
    pub(super) buttons: Vec<MouseButton>,
    /// The pointer's last event, for a `Cancel` the part sends itself.
    pub(super) last: PointerEvent,
    /// The button whose press the node's region fallback consumed: the
    /// fallback's gesture ends with its release or a cancel.
    pub(super) fallback: Option<MouseButton>,
}

/// Final values are shared by publication, input gating, and every bind hook.
#[derive(Clone, Copy)]
pub(super) struct Constraints {
    pub(super) disabled: Value<bool>,
    pub(super) hidden: Value<bool>,
    pub(super) focusable: Value<bool>,
    pub(super) eligible: Value<bool>,
    pub(super) caller_disabled: Value<bool>,
    pub(super) caller_hidden: Value<bool>,
}

#[derive(Clone, Default)]
pub(crate) struct SemanticSources {
    pub(crate) value_min: Option<InputProp<f64>>,
    pub(crate) value_max: Option<InputProp<f64>>,
    pub(crate) value_now: Option<InputProp<f64>>,
    pub(crate) live_region: Option<InputProp<kivact_types::LiveRegion>>,
    pub(crate) sort: Option<InputProp<kivact_types::SortDirection>>,
    pub(crate) orientation: Option<InputProp<kivact_types::Orientation>>,
    pub(crate) active_descendant: Option<InputProp<Option<crate::view::NodeRef>>>,
    pub(crate) level: Option<InputProp<u32>>,
    pub(crate) set_size: Option<InputProp<u32>>,
    pub(crate) pos_in_set: Option<InputProp<u32>>,
    pub(crate) multiselectable: Option<InputProp<bool>>,
    pub(crate) busy: Option<InputProp<bool>>,
    pub(crate) required: Option<InputProp<bool>>,
    pub(crate) read_only: Option<InputProp<bool>>,
    pub(crate) has_popup: Option<InputProp<Option<kivact_types::PopupKind>>>,
    pub(crate) current_item: Option<InputProp<Option<kivact_types::CurrentItem>>>,
    pub(crate) described_by: Vec<crate::view::NodeRef>,
    pub(crate) controls: Vec<crate::view::NodeRef>,
    pub(crate) labelled_by: Vec<crate::view::NodeRef>,
    /// The node's own text runs for assistive technology.
    pub(crate) text: Option<InputProp<kivact_types::SemanticsText>>,
}
