//! Role: These helpers hold the logic the modifier methods share.
//!
//! The modifier methods are written once, as plain default methods:
//! the everyday set (`common`, `draw`, `intrinsic`) on
//! `kivact::view::NodeBuilder` and the advanced set (`advanced`,
//! `draw_over`) on `kivact::view::Node`. `Node`, `Overlay`,
//! `VirtualList` and every component holding a root `Node` implement
//! `NodeBuilder`. `Part` has its own inherent methods for the semantic and
//! input subset, which forward to the same `Common` methods below. Adding a
//! modifier means writing the method on the trait, plus, when it has logic
//! beyond a field write, one `Common` method that `Part` shares.
//!
//! Modifiers transport properties through the builder's `Common`; `with`
//! additionally merges a part's semantic sources, constraints and defaults.
//! Their mounted meaning is decided by `mount::mount_host`.
//!
//! Handler layering (see `handler_layers` for the delivery order):
//! - `on_key`, `on_key_up`, `on_pointer` and `on_wheel` are `Layer::Caller`
//!   entries, the latest registration first: `No` passes the event to the
//!   next handler, `Yes` stops it, and an `Err` stops the rest and is
//!   reported once for the event.
//! - `on_*_default` is the component author's hook: a `Layer::NodeDefault`
//!   entry that runs after the callers the caller registered.
//! - `on_file` keeps one handler; registering it again replaces it.
//! - `on_press` uses the root's one Activate slot (pointer, Enter/Space and
//!   the accessibility Activate); registering it again replaces the caller's
//!   handler in that slot.
//! - `on_a11y` keeps one handler per action and a new registration replaces
//!   it.
//!
//! Semantic modifiers retain ordered patches; mounting folds them once or subscribes
//! one render effect for the complete semantic value.

use crate::view::common::Common;
use crate::view::input::{KeyEvent, PointerEvent, WheelEvent};
use crate::view::semantics_data::{Relation, SemanticInput, SemanticsPatch};
use crate::view::{A11yAction, A11yEvent, A11yRequest, HandlerResult};
use kivact_types::{FocusTarget, Semantics};
use std::panic::Location;
use std::rc::Rc;

/// The shared bodies of the modifiers that are more than a field write.
/// `NodeBuilder`, `Node` and `Part` forward to them.
impl Common {
    #[track_caller]
    pub(crate) fn semantic<T: Clone + 'static>(
        &mut self,
        source: kivact_runtime::Prop<T>,
        apply: impl Fn(&mut Semantics<FocusTarget>, T) + 'static,
    ) {
        let reactive = source.fixed().is_none();
        self.semantics.push(SemanticsPatch {
            part: false,
            input: SemanticInput::None,
            layer: self.semantics.last().map_or(0, |patch| patch.layer),
            at: Location::caller(),
            reactive,
            text: false,
            apply: Rc::new(move |semantics| apply(semantics, source.get())),
            relation: None,
            orientation: None,
        });
    }

    #[track_caller]
    pub(crate) fn semantic_relation(&mut self, relation: Relation, target: crate::view::NodeRef) {
        let at = Location::caller();
        self.semantic(
            kivact_runtime::Prop::computed(Rc::new(move || target.target_value(at))),
            move |s, target| {
                let list = match relation {
                    Relation::DescribedBy => &mut s.described_by,
                    Relation::Controls => &mut s.controls,
                    Relation::LabelledBy => &mut s.labelled_by,
                };
                list.extend(target);
            },
        );
        self.semantics.last_mut().unwrap().relation = Some((relation, target));
    }

    #[track_caller]
    pub(crate) fn semantic_orientation(
        &mut self,
        source: kivact_runtime::Prop<kivact_types::Orientation>,
    ) {
        self.semantic(source.clone(), |s, value| s.orientation = value);
        self.semantics.last_mut().unwrap().orientation = Some(source);
    }

    pub(crate) fn append_semantics(&mut self, patches: &mut Vec<SemanticsPatch>) {
        let next = self.semantics.last().map_or(0, |patch| patch.layer + 1);
        for patch in patches.iter_mut() {
            patch.layer += next;
        }
        self.semantics.append(patches);
    }

    #[track_caller]
    pub(crate) fn set_role(&mut self, value: kivact_types::Role) {
        self.semantic(kivact_runtime::Prop::constant(value), |s, v| {
            s.role = Some(v)
        });
        self.semantics.last_mut().unwrap().input = SemanticInput::Role(value);
    }
    pub(crate) fn role_source(&self) -> Option<kivact_types::Role> {
        self.semantics.iter().rev().find_map(|p| match p.input {
            SemanticInput::Role(v) => Some(v),
            _ => None,
        })
    }
    #[track_caller]
    pub(crate) fn set_name(&mut self, source: kivact_runtime::Prop<String>) {
        self.semantic(source.clone(), |s, v| {
            s.name = Some(v);
        });
        self.semantics.last_mut().unwrap().input = SemanticInput::Name(source);
    }
    #[track_caller]
    pub(crate) fn set_description(&mut self, source: kivact_runtime::Prop<String>) {
        self.semantic(source.clone(), |s, v| {
            s.description = Some(v);
        });
        self.semantics.last_mut().unwrap().input = SemanticInput::Description(source);
    }
    #[track_caller]
    pub(crate) fn set_value(&mut self, source: kivact_runtime::Prop<String>) {
        self.semantic(source.clone(), |s, v| {
            s.value_text = Some(v);
        });
    }
    #[track_caller]
    pub(crate) fn set_selected(&mut self, source: kivact_runtime::Prop<bool>) {
        self.semantic(source.clone(), |s, v| {
            s.selected = Some(v);
        });
        self.semantics.last_mut().unwrap().input = SemanticInput::Selected(source);
    }
    pub(crate) fn selected_source(&self) -> Option<kivact_runtime::Prop<bool>> {
        self.semantics.iter().rev().find_map(|p| match &p.input {
            SemanticInput::Selected(v) => Some(v.clone()),
            _ => None,
        })
    }
    #[track_caller]
    pub(crate) fn set_disabled(&mut self, source: kivact_runtime::Prop<bool>) {
        self.semantic(source.clone(), |s, v| {
            s.disabled = v;
        });
        self.semantics.last_mut().unwrap().input = SemanticInput::Disabled(source);
    }
    pub(crate) fn disabled_source(&self) -> Option<kivact_runtime::Prop<bool>> {
        self.semantics
            .iter()
            .rev()
            .filter(|p| !p.part)
            .find_map(|p| match &p.input {
                SemanticInput::Disabled(v) => Some(v.clone()),
                _ => None,
            })
    }
    #[track_caller]
    pub(crate) fn set_hidden(&mut self, source: kivact_runtime::Prop<bool>) {
        self.semantic(source.clone(), |s, v| {
            s.hidden = v;
        });
        self.semantics.last_mut().unwrap().input = SemanticInput::Hidden(source);
    }
    pub(crate) fn hidden_source(&self) -> Option<kivact_runtime::Prop<bool>> {
        self.semantics
            .iter()
            .rev()
            .filter(|p| !p.part)
            .find_map(|p| match &p.input {
                SemanticInput::Hidden(v) => Some(v.clone()),
                _ => None,
            })
    }
    #[track_caller]
    pub(crate) fn set_invalid(&mut self, source: kivact_runtime::Prop<bool>) {
        self.semantic(source.clone(), |s, v| {
            s.invalid = Some(v);
        });
        self.semantics.last_mut().unwrap().input = SemanticInput::Invalid(source);
    }
    #[track_caller]
    pub(crate) fn set_checked(&mut self, source: kivact_runtime::Prop<bool>) {
        self.semantic(source.clone(), |s, v| {
            s.checked = Some(kivact_types::CheckedState::from_bool(v));
        });
        self.semantics.last_mut().unwrap().input = SemanticInput::Checked(source);
    }
    #[track_caller]
    pub(crate) fn set_indeterminate(&mut self, source: kivact_runtime::Prop<bool>) {
        self.semantic(source.clone(), |s, v| {
            if v {
                s.checked = Some(kivact_types::CheckedState::Indeterminate);
            };
        });
        self.semantics.last_mut().unwrap().input = SemanticInput::Indeterminate(source);
    }
    #[track_caller]
    pub(crate) fn set_expanded(&mut self, source: kivact_runtime::Prop<bool>) {
        self.semantic(source.clone(), |s, v| {
            s.expanded = Some(v);
        });
        self.semantics.last_mut().unwrap().input = SemanticInput::Expanded(source);
    }
    #[track_caller]
    pub(crate) fn set_focusable(&mut self, source: kivact_runtime::Prop<bool>) {
        self.semantic(source.clone(), |s, v| {
            s.focusable = v;
        });
        self.semantics.last_mut().unwrap().input = SemanticInput::Focusable(source);
    }
    pub(crate) fn focusable_source(&self) -> Option<kivact_runtime::Prop<bool>> {
        self.semantics
            .iter()
            .rev()
            .filter(|p| !p.part)
            .find_map(|p| match &p.input {
                SemanticInput::Focusable(v) => Some(v.clone()),
                _ => None,
            })
    }

    /// `on_key`: run before the key handlers already registered.
    pub(crate) fn prepend_key<F, R>(&mut self, f: F)
    where
        F: Fn(KeyEvent) -> R + 'static,
        R: HandlerResult,
    {
        self.handlers.key.caller(move |e| f(e).into_outcome());
    }

    /// `on_key_up`: run before the key release handlers already registered.
    pub(crate) fn prepend_key_up<F, R>(&mut self, f: F)
    where
        F: Fn(KeyEvent) -> R + 'static,
        R: HandlerResult,
    {
        self.handlers.key_up.caller(move |e| f(e).into_outcome());
    }

    /// `on_pointer`: run before the pointer handlers already registered.
    pub(crate) fn prepend_pointer<F, R>(&mut self, f: F)
    where
        F: Fn(PointerEvent) -> R + 'static,
        R: HandlerResult,
    {
        self.handlers.pointer.caller(move |e| f(e).into_outcome());
    }

    /// `on_wheel`: run before the wheel handlers already registered.
    pub(crate) fn prepend_wheel<F, R>(&mut self, f: F)
    where
        F: Fn(WheelEvent) -> R + 'static,
        R: HandlerResult,
    {
        self.handlers.wheel.caller(move |e| f(e).into_outcome());
    }

    /// `on_ime`: run before the IME handlers already registered.
    pub(crate) fn prepend_ime<F, R>(&mut self, f: F)
    where
        F: Fn(kivact_types::ImeInput) -> R + 'static,
        R: HandlerResult,
    {
        self.handlers.ime.caller(move |e| f(e).into_outcome());
    }

    /// `on_press`: the shared pointer, Enter/Space and accessibility
    /// Activate action, replacing the caller's.
    pub(crate) fn set_press<F, R>(&mut self, f: F)
    where
        F: Fn() -> R + 'static,
        R: HandlerResult,
    {
        self.handlers.activate = Some(kivact_runtime::Handler::from_fn(move |_region| f()));
    }

    /// `on_a11y_event`: offer an action with the region and value of the
    /// request. A node has one handler per action; `Activate` shares the
    /// activation slot.
    pub(crate) fn offer_a11y_event<F, R>(&mut self, action: A11yAction, f: F)
    where
        F: Fn(A11yEvent) -> R + 'static,
        R: HandlerResult,
    {
        if action == A11yAction::Activate {
            self.handlers.activate = Some(kivact_runtime::Handler::from_fn(move |region| {
                f(A11yEvent {
                    request: A11yRequest::Activate,
                    region,
                })
                .into_outcome()
                .map(|_| ())
            }));
        } else {
            let h = kivact_runtime::Handler::from_fn(f);
            let a11y = &mut self.handlers.a11y;
            match a11y.iter_mut().find(|(a, _)| *a == action) {
                Some(entry) => entry.1 = h,
                None => a11y.push((action, h)),
            }
        }
    }
}
