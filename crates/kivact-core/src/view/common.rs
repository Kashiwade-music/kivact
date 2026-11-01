//! Role: Mounted data shared by host mounting, parts and canvas regions.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::panic::Location;
use std::rc::{Rc, Weak};

use kivact_reactive::Prop as InputProp;
use kivact_reactive::{Owner, Runtime};
use kivact_wire::{IntrinsicSize, NodeId};

use crate::view::descriptor::Mcx;
use crate::view::part_data::PartBinding;
use crate::view::tree::Handlers;
use crate::{Signal, Value};
use kivact_style::Style;
use kivact_style::asset::ImageFit;
use kivact_style::draw::{Draw, RegionId, Size};
use kivact_wire::Role;

pub(crate) type DrawFn = Rc<dyn Fn(&mut Draw, Size)>;

/// Modifiers shared by every element builder.
#[derive(Default)]
pub(crate) struct Common {
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
    /// Expanded state (disclosures).
    pub expanded: Option<InputProp<bool>>,
    /// Focusability (a builder sets its own before the user's modifiers).
    pub focusable: Option<bool>,
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

pub(super) struct Target {
    pub(super) generation: u64,
    pub(super) region: Option<RegionId>,
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

/// One canvas identity owns this lookup for its entire mounted lifetime.
pub(crate) struct Registry {
    pub(super) node: NodeId,
    pub(super) owner: Owner,
    pub(super) runtime: Weak<Runtime>,
    pub(super) parent: Constraints,
    pub(super) at: &'static Location<'static>,
    pub(super) state: RefCell<State>,
    pub(super) revision: Signal<u64>,
    pub(super) changed: Signal<u64>,
    pub(super) ready: Cell<bool>,
    pub(super) cancel: RefCell<Option<Rc<dyn Fn(u64)>>>,
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
    pub(crate) focusable: Option<InputProp<bool>>,
    pub(crate) indeterminate: Option<InputProp<bool>>,
    pub(crate) value_min: Option<InputProp<f64>>,
    pub(crate) value_max: Option<InputProp<f64>>,
    pub(crate) value_now: Option<InputProp<f64>>,
    pub(crate) live_region: Option<InputProp<kivact_wire::LiveRegion>>,
    pub(crate) sort: Option<InputProp<kivact_wire::SortDirection>>,
    pub(crate) orientation: Option<InputProp<kivact_wire::Orientation>>,
    pub(crate) active_descendant: Option<InputProp<Option<crate::view::NodeRef>>>,
    pub(crate) level: Option<InputProp<u32>>,
    pub(crate) set_size: Option<InputProp<u32>>,
    pub(crate) pos_in_set: Option<InputProp<u32>>,
    pub(crate) multiselectable: Option<InputProp<bool>>,
    pub(crate) described_by: Vec<crate::view::NodeRef>,
    pub(crate) controls: Vec<crate::view::NodeRef>,
    pub(crate) labelled_by: Vec<crate::view::NodeRef>,
    /// The node's own text runs for assistive technology.
    pub(crate) text: Option<InputProp<kivact_wire::SemanticsText>>,
}
