//! Role: The typed property table: one row per prop generates its setter and its `Prop`.
//!
//! Every typed property kivact writes is one row of `props!`: its
//! [`Prop`] variant, its [`Writer`] setter and value type, where the value
//! is stored in the node's props, and what a change outdates (its
//! `Mutation`). Adding a property is adding a row; a row without
//! `mut = ..` does not compile, and a `Mutation` has no default.
//!
//! Invariants:
//! - A setter compares before it stores: an equal value changes nothing,
//!   records nothing and returns `false` (a boxed prop is compared before
//!   its box is allocated).
//! - A change is counted once (`Writes::field`) and recorded by
//!   `Model::set_prop`; `then` runs after a change, and `custom` writes a
//!   prop whose change also moves engine-owned state.
//! - The compare, the store and the record of what a change outdates are
//!   one path (`Model::set_prop`), its `Mutation` choosing the extra steps:
//!   a read-only change commits the preedit while the old policy still
//!   applies, a paint or state style change prepares the state motion
//!   before the value lands. A `Mutation` constant names a combination of
//!   the existing flags.
//! - `Prop` names exactly the rows, and the engine-owned values kivact
//!   writes beside them (`EditState`, `EditSession`); `PropValue` has one
//!   variant per `Prop`, and `Writer::set` writes it through the setter.
//! - `RESTACK` (the paint list is built again) is for props that may
//!   show, hide, move, reorder or reclip nodes without a layout pass;
//!   semantics, listeners and editing limits never do. `Paint`,
//!   `TextStyle` and `States` do not name it: they restack when the
//!   stacking facts they changed differ (`Model::set_prop`, `changed`
//!   and `sync` compare the node's `Place` and paint `Look`), so a color
//!   alone only redraws.

use super::Writer;
use crate::model::Model;
use crate::mutation::Mutation as Mut;
use kivact_wire::{
    A11yAction, Asset, CheckedState, EditingData, FocusTarget, ImageData, ImageFit, ImeSession,
    IntrinsicSize, Key, LayoutProps, LeafMeasure, NodeId, NodeKind, NodeProperties, NodeSemantics,
    OverlayProps, PaintProps, Role, SemanticsProperties, StateStyles, TextData, TextProps,
    TextSpan,
};

fn semantics(p: &mut NodeProperties) -> &mut NodeSemantics {
    p.semantics.get_or_insert_default()
}
fn text(p: &mut NodeProperties) -> Option<&mut TextData> {
    match &mut p.kind {
        NodeKind::Text(data) => Some(data),
        _ => None,
    }
}
fn image(p: &mut NodeProperties) -> Option<&mut ImageData> {
    match &mut p.kind {
        NodeKind::Image(data) => Some(data),
        _ => None,
    }
}
fn editing(p: &mut NodeProperties) -> Option<&mut EditingData> {
    p.editing.as_deref_mut()
}
fn read_only(value: Option<&EditingData>) -> bool {
    value.is_some_and(|data| data.read_only == Some(true))
}

/// The property table: `Variant: setter(value: Type) => store, mut = ..;`
/// with `store` one of `field(value, place)`, `boxed(value, place)` (the
/// place holds a box) or `custom(method)`, and an optional `then =
/// method` run after a change. Engine-owned values follow in `@owned`.
macro_rules! props {
    (
        $(
            $(#[$doc:meta])*
            $prop:ident: $setter:ident($v:ident: $ty:ty) => $store:ident($($how:tt)*),
                mut = $mutation:expr $(, then = $then:ident)?;
        )*
        @owned { $($(#[$odoc:meta])* $owned:ident($oty:ty) => $osetter:ident),* $(,)? }
    ) => {
        /// A typed property kivact writes to a node: one per [`Writer`]
        /// setter, and the engine-owned values written beside them.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum Prop {
            $(
                #[doc = concat!("Written by [`Writer::", stringify!($setter), "`].")]
                $prop,
            )*
            $($(#[$odoc])* $owned,)*
        }

        impl Prop {
            /// The property's name.
            pub fn name(self) -> &'static str {
                match self {
                    $(Prop::$prop => stringify!($prop),)*
                    $(Prop::$owned => stringify!($owned),)*
                }
            }
        }

        /// One typed property value, as written by a `Writer` setter.
        #[allow(
            clippy::large_enum_variant,
            reason = "a value is built and moved straight into `Writer::set`; boxing the large ones would allocate on every write"
        )]
        pub enum PropValue {
            $(
                $(#[$doc])*
                $prop($ty),
            )*
            $(
                $(#[$odoc])*
                $owned($oty),
            )*
        }

        impl PropValue {
            /// The property this value is for.
            pub fn prop(&self) -> Prop {
                match self {
                    $(PropValue::$prop(_) => Prop::$prop,)*
                    $(PropValue::$owned(_) => Prop::$owned,)*
                }
            }
        }

        impl Writer {
            /// Write `value` to `id` through its typed setter: compared before it is stored,
            /// recorded with the row's `Mutation`; `false` when equal or the node has no such
            /// property (same contract as the setter).
            pub fn set(&self, id: NodeId, value: PropValue) -> bool {
                match value {
                    $(PropValue::$prop(v) => self.$setter(id, v),)*
                    $(PropValue::$owned(v) => self.$osetter(id, v),)*
                }
            }

            $(
                $(#[$doc])*
                pub fn $setter(&self, id: NodeId, $v: $ty) -> bool {
                    let mutation: Mut = $mutation;
                    let changed = props!(@store self, id, $v, mutation, $store($($how)*));
                    $(
                        if changed {
                            self.$then(id);
                        }
                    )?
                    changed
                }
            )*
        }
    };
    (@store $w:ident, $id:ident, $v:ident, $m:ident, field($value:expr, $place:expr)) => {
        $w.record($id, |model| model.set_prop($id, $value, $place, $m))
    };
    (@store $w:ident, $id:ident, $v:ident, $m:ident, boxed($value:expr, $place:expr)) => {
        $w.record($id, |model| model.set_boxed_prop($id, $value, $place, $m))
    };
    (@store $w:ident, $id:ident, $v:ident, $m:ident, custom($method:ident)) => {
        $w.$method($id, $v, $m)
    };
}

props! {
    /// Replace the node's layout when its value differs.
    Layout: set_layout(v: LayoutProps)
        => boxed(Some(v), |p| Some(&mut p.layout)),
        mut = Mut::STYLE.and(Mut::RESTACK);
    /// Replace the node's paint when its value differs.
    Paint: set_paint(v: PaintProps)
        => boxed(Some(v), |p| Some(&mut p.paint)),
        mut = Mut::PAINT;
    /// Replace the node's text style when its value differs.
    TextStyle: set_text_style(v: Option<TextProps>)
        => boxed(v, |p| Some(&mut p.text)),
        mut = Mut::TEXT;
    /// Replace the node's states when its value differs.
    States: set_states(v: StateStyles)
        => custom(write_states),
        mut = Mut::STATES;
    /// Replace the node's disabled when its value differs.
    Disabled: set_disabled(v: bool)
        => field(Some(v), |p| Some(&mut p.input.disabled)),
        mut = Mut::SUBTREE, then = after_disabled;
    /// Replace the node's hidden when its value differs.
    Hidden: set_hidden(v: bool)
        => field(Some(v), |p| Some(&mut p.input.hidden)),
        mut = Mut::STYLE.and(Mut::RESTACK);
    /// Replace whether the pointer hits the node and its subtree.
    HitTestable: set_hit_testable(v: bool)
        => field(Some(v), |p| Some(&mut p.input.hit_testable)),
        mut = Mut::NONE;
    /// Replace the node's focusable when its value differs.
    Focusable: set_focusable(v: bool)
        => field(Some(v), |p| Some(&mut p.input.focusable)),
        mut = Mut::NONE;
    /// Replace the node's listen pointer when its value differs.
    ListenPointer: set_listen_pointer(v: bool)
        => field(Some(v), |p| Some(&mut p.input.listen_pointer)),
        mut = Mut::NONE;
    /// Replace the node's focus on press when its value differs.
    FocusOnPress: set_focus_on_press(v: Option<FocusTarget>)
        => field(v, |p| Some(&mut p.input.focus_on_press)),
        mut = Mut::NONE;
    /// Replace the node's listen wheel when its value differs.
    ListenWheel: set_listen_wheel(v: bool)
        => field(Some(v), |p| Some(&mut p.input.listen_wheel)),
        mut = Mut::NONE;
    /// Replace the node's listen file when its value differs.
    ListenFile: set_listen_file(v: bool)
        => field(Some(v), |p| Some(&mut p.input.listen_file)),
        mut = Mut::NONE;
    /// Replace the node's role when its value differs.
    Role: set_role(v: Role)
        => field(Some(v), |p| Some(&mut semantics(p).role)),
        mut = Mut::NONE;
    /// Replace the node's name when its value differs.
    Name: set_name(v: String)
        => field(Some(v), |p| Some(&mut semantics(p).name)),
        mut = Mut::NONE;
    /// Replace the node's description when its value differs.
    Description: set_description(v: String)
        => field(Some(v), |p| Some(&mut semantics(p).description)),
        mut = Mut::NONE;
    /// Replace the node's value when its value differs.
    Value: set_value(v: String)
        => field(Some(v), |p| Some(&mut semantics(p).value)),
        mut = Mut::NONE;
    /// Replace the node's selected when its value differs.
    Selected: set_selected(v: bool)
        => field(Some(v), |p| Some(&mut semantics(p).selected)),
        mut = Mut::NONE;
    /// Replace the node's checked when its value differs.
    Checked: set_checked(v: Option<CheckedState>)
        => field(v, |p| Some(&mut semantics(p).checked)),
        mut = Mut::NONE;
    /// Replace the node's expanded when its value differs.
    Expanded: set_expanded(v: bool)
        => field(Some(v), |p| Some(&mut semantics(p).expanded)),
        mut = Mut::NONE;
    /// Replace the node's invalid when its value differs.
    Invalid: set_invalid(v: bool)
        => field(Some(v), |p| Some(&mut semantics(p).invalid)),
        mut = Mut::NONE;
    /// Replace the node's accessibility when its value differs.
    Accessibility: set_accessibility(v: SemanticsProperties)
        => boxed(Some(v), |p| Some(&mut semantics(p).accessibility)),
        mut = Mut::NONE;
    /// Replace the node's actions when its value differs.
    Actions: set_actions(v: Vec<A11yAction>)
        => field(Some(v), |p| Some(&mut semantics(p).actions)),
        mut = Mut::NONE;
    /// Replace the node's test id when its value differs.
    TestId: set_test_id(v: String)
        => field(Some(v), |p| Some(&mut semantics(p).test_id)),
        mut = Mut::NONE;
    /// Replace the node's content when its value differs.
    Content: set_content(v: String)
        => field(Some(v), |p| Some(&mut text(p)?.content)),
        mut = Mut::MEASURE.and(Mut::LOCAL_TEXT);
    /// Replace the node's text spans when its value differs.
    TextSpans: set_text_spans(v: Vec<TextSpan>)
        => field(Some(v), |p| Some(&mut text(p)?.spans)),
        mut = Mut::MEASURE;
    /// Replace the node's editing data when its value differs: it gains,
    /// keeps or loses its native editing state with it.
    Editing: set_editing(v: Option<EditingData>)
        => boxed(v, |p| Some(&mut p.editing)),
        mut = Mut::MEASURE
            .and(Mut::RESTACK)
            .and(Mut::READ_ONLY.when(read_only(v.as_ref()))),
        then = after_editing;
    /// Update the controlled text input value, respecting an active
    /// composition. The same `Rc` given again at the text's revision it
    /// was given at changes nothing without comparing texts.
    InputValue: set_input_value(v: std::rc::Rc<str>)
        => custom(write_input_value),
        mut = Mut::NONE;
    /// Replace the colors of the editor's lines when they are another
    /// function.
    LineColors: set_line_colors(v: kivact_editing::LineColors)
        => field(Some(v), |p| Some(&mut editing(p)?.colors)),
        mut = Mut::NONE;
    /// Replace the node's password when its value differs.
    Password: set_password(v: bool)
        => field(Some(v), |p| Some(&mut editing(p)?.password)),
        mut = Mut::MEASURE.and(Mut::RESTACK);
    /// Replace the node's read only when its value differs.
    ReadOnly: set_read_only(v: bool)
        => field(Some(v), |p| Some(&mut editing(p)?.read_only)),
        mut = Mut::READ_ONLY.when(v);
    /// Replace the node's max length (`None`: no limit) when its value differs.
    MaxLength: set_max_length(v: Option<usize>)
        => field(v, |p| Some(&mut editing(p)?.max_length)),
        mut = Mut::NONE;
    /// Replace the node's placeholder when its value differs.
    Placeholder: set_placeholder(v: String)
        => field(Some(v), |p| Some(&mut editing(p)?.placeholder)),
        mut = Mut::MEASURE;
    /// Control keys owned by the client's handlers instead of native editing.
    ForwardKeys: set_forward_keys(v: Vec<Key>)
        => field(Some(v), |p| Some(&mut editing(p)?.forward_keys)),
        mut = Mut::NONE;
    /// Report the editor's whole text with each `TextChanged` (a copy per
    /// edit) when its value differs.
    ReportText: set_report_text(v: bool)
        => field(Some(v), |p| Some(&mut editing(p)?.report_text)),
        mut = Mut::NONE;
    /// Report the editor's session with `SelectionChanged` whenever it
    /// changes, when its value differs.
    ReportSession: set_report_session(v: bool)
        => field(Some(v), |p| Some(&mut editing(p)?.report_session)),
        mut = Mut::NONE;
    /// Replace the node's IME session (a focusable node that edits its own
    /// text, not a native editor) when its value differs.
    ImeSession: set_ime_session(v: ImeSession)
        => boxed(Some(v), |p| Some(&mut p.ime)),
        mut = Mut::NONE;
    /// Replace the node's multiline when its value differs.
    Multiline: set_multiline(v: bool)
        => field(Some(v), |p| Some(&mut editing(p)?.multiline)),
        mut = Mut::MEASURE.and(Mut::RESTACK);
    /// Replace the node's image when its value differs.
    Image: set_image(v: Asset)
        => field(Some(v), |p| Some(&mut image(p)?.image)),
        mut = Mut::MEASURE;
    /// Replace the node's image fit when its value differs.
    ImageFit: set_image_fit(v: ImageFit)
        => field(Some(v), |p| Some(&mut image(p)?.fit)),
        mut = Mut::NONE;
    /// Replace the node's monochrome when its value differs.
    Monochrome: set_monochrome(v: bool)
        => field(Some(v), |p| Some(&mut image(p)?.monochrome)),
        mut = Mut::NONE;
    /// Replace the node's overlay when its value differs (`STYLE`: a
    /// layout pass applies `OverlayAt::Anchor::at_least_width`).
    Overlay: set_overlay(v: OverlayProps)
        => field(Some(v), |p| match &mut p.kind {
            NodeKind::Overlay(data) => Some(data),
            _ => None,
        }),
        mut = Mut::STYLE.and(Mut::PLACED).and(Mut::RESTACK);
    /// Replace the node's intrinsic when its value differs.
    Intrinsic: set_intrinsic(v: IntrinsicSize)
        => field(Some(v), |p| match &mut p.kind {
            NodeKind::Box(data) => Some(&mut data.intrinsic),
            NodeKind::Image(data) => Some(&mut data.intrinsic),
            _ => None,
        }),
        mut = Mut::MEASURE.and(Mut::RESTACK);
    /// Replace the node's measure when its value differs.
    Measure: set_measure(v: LeafMeasure)
        => field(Some(v), |p| match &mut p.kind {
            NodeKind::Box(data) => Some(&mut data.measure),
            _ => None,
        }),
        mut = Mut::MEASURE;
    @owned {
        /// The native editing state, written by [`Writer::set_edit_state`].
        EditState(kivact_editing::EditState) => set_edit_state,
        /// The native editing session, written by [`Writer::set_edit_session`].
        EditSession(kivact_editing::TextSession) => set_edit_session,
    }
}

impl Writer {
    /// Replace the native editing state of the text input `id` (selection,
    /// composition and value) with kivact's: by the changes it recorded
    /// when it was read from this editor's current text, else as a whole.
    pub fn set_edit_state(&self, id: NodeId, state: kivact_editing::EditState) -> bool {
        self.record(id, |model| model.replace_edit_state(id, state))
    }

    /// Replace the editing session of the text input `id` (selection and
    /// composition) with kivact's, clamped into its text, which is neither
    /// read nor copied.
    pub fn set_edit_session(&self, id: NodeId, session: kivact_editing::TextSession) -> bool {
        self.record(id, |model| model.replace_edit_session(id, session))
    }

    /// Apply one write to the model and count it when it changed something.
    fn record(&self, id: NodeId, write: impl FnOnce(&mut Model) -> bool) -> bool {
        let mut model = self.model.borrow_mut();
        let changed = write(&mut model);
        if changed {
            model.writes.field(id);
        }
        changed
    }

    /// A node got disabled: a capture it holds ends.
    fn after_disabled(&self, id: NodeId) {
        let mut model = self.model.borrow_mut();
        if model.node(id).is_some_and(|n| n.disabled()) {
            model.cancel_disabled_capture();
        }
    }

    fn after_editing(&self, id: NodeId) {
        // Cached line state may own user destructors. Complete the model
        // update first, then destroy the old editor before this setter returns.
        let previous = { self.model.borrow_mut().editing_changed(id) };
        drop(previous);
    }

    fn write_states(&self, id: NodeId, states: StateStyles, mutation: Mut) -> bool {
        self.record(id, |model| model.set_states(id, states, mutation))
    }

    fn write_input_value(&self, id: NodeId, value: std::rc::Rc<str>, _: Mut) -> bool {
        self.record(id, |model| model.controlled_value(id, value))
    }
}
