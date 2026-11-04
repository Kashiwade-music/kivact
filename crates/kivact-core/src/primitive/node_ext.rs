//! Role: Semantic, handler and drawing extensions that every node builder gets.

use crate::view::IntoValue;
use crate::view::ime_session::{ImeInput, ImeSession};
use crate::view::input::{KeyEvent, KeyHandlerResult, PointerEvent, WheelEvent};
use crate::view::node::NodeBuilder;
use crate::view::part::Part;
use crate::view::{
    A11yAction, A11yEvent, HandlerResult, IntoText, LiveRegion, NodeRef, Orientation, Role,
    SortDirection,
};
use kivact_style::Style;
use kivact_style::draw::{Draw, Size};
use kivact_types::SemanticsText;
use std::panic::Location;
use std::rc::Rc;

/// Modifiers for component authors: what the component is (`role`,
/// `checked`, `expanded`, `selected`, `value_text`), attaching a part, extra
/// semantics, handler layering, and the overlay draw layer.
///
/// Implemented for every [`NodeBuilder`] ([`Node`](crate::control::Node) and each component holding
/// a root `Node`); bring it into scope with `use kivact::primitive::NodeExt;`.
/// A component sets its own role and states on its root with these, so an
/// app that imports only the prelude cannot set a `checked` that contradicts
/// the binding the component edits: `checkbox("x", on).checked(false)` does
/// not compile. [`Node`](crate::control::Node) has the same methods inherent, so `node().role(..)`
/// needs no import. `at` relocates where the element is reported, the
/// semantics modifiers (`value_min`, `level`, `controls`, ...) publish more
/// than the everyday `name` / `description`, and the `*_default` handlers run
/// a component's own behavior after the caller's customizations.
pub trait NodeExt: NodeBuilder {
    /// Attach behavior, semantic relationships and input defaults to this element.
    ///
    /// Later semantic modifiers override values supplied by the part.
    /// Disabled and hidden requirements combine with OR; focus constraints
    /// combine with AND. Caller input customizations run before part defaults,
    /// regardless of whether they are registered before or after attachment.
    /// References, observations and cleanup from earlier parts are retained.
    fn part<S: Clone + PartialEq + 'static>(mut self, part: Part<S>) -> Self {
        part.attach(&mut self.node_mut().common);
        self
    }

    /// Accessibility role, overriding the builder's default role.
    fn role(mut self, r: Role) -> Self {
        self.node_mut().common.role = Some(r);
        self
    }

    /// Accessible value text, as a string (`aria-valuetext`) of a slider or
    /// progress bar. A text input's value is its own, not this one.
    fn value_text(mut self, v: impl IntoText) -> Self {
        self.node_mut().common.value = Some(v.into_text());
        self
    }

    /// Add the component's own base style, constant or reactive: a layer
    /// below every `.style(..)` of the caller, whatever the order of the
    /// calls, so what the caller sets always wins.
    ///
    /// A builder that learns its settings after its root exists (`.size(..)`,
    /// `.tone(..)`, a reactive flag) calls this in `into_view`, when the
    /// settings are final, instead of installing a style at creation that
    /// reads them through a shared cell. Several base styles merge in the
    /// order they were added, below the style layers the root already has,
    /// including a recipe's.
    ///
    /// ```
    /// use kivact_core::control::{NodeBuilder, node};
    /// use kivact_core::primitive::NodeExt;
    /// use kivact_core::style::style;
    ///
    /// // The caller's height wins although the component adds its own later.
    /// let root = node().style(style().height(30)).base_style(style().height(22).width(80));
    /// # let _ = root;
    /// ```
    fn base_style(mut self, s: impl IntoValue<Style>) -> Self {
        self.node_mut().common.prepend_style(s.into_prop());
        self
    }

    /// Selected state (semantics and the `selected` style).
    fn selected(mut self, v: impl IntoValue<bool>) -> Self {
        self.node_mut().common.selected = Some(v.into_prop());
        self
    }

    /// Binary checked state for a check box or switch.
    fn checked(mut self, value: impl IntoValue<bool>) -> Self {
        self.node_mut().common.checked = Some(value.into_prop());
        self
    }

    /// Expanded state of a disclosure, menu or combo box.
    fn expanded(mut self, value: impl IntoValue<bool>) -> Self {
        self.node_mut().common.expanded = Some(value.into_prop());
        self
    }

    /// Read the preserved disabled-state input (`.disabled(..)`, set by the
    /// caller) without allocating a value; `None` when nothing publishes
    /// one. A component gives it to its behavior once its root is complete,
    /// so one `disabled` modifier disables the node and the component's own
    /// input, with no method of the component's to forget.
    #[track_caller]
    fn accessible_disabled(&mut self) -> Option<Rc<dyn Fn() -> bool>> {
        self.node_mut()
            .disabled_source()
            .map(kivact_runtime::Prop::into_fn)
    }

    /// Read the preserved accessible-name input without allocating a value.
    #[track_caller]
    fn accessible_name(&mut self) -> Option<Rc<dyn Fn() -> String>> {
        self.node_mut()
            .name_source()
            .map(kivact_runtime::Prop::into_fn)
    }

    /// Copy the preserved label references, in caller/part order, without
    /// resolving their mounted targets. A composite forwards these to its
    /// primary control with [`Self::labelled_by`], retaining the editor's
    /// own relations. Unbound/disposed targets follow the normal relation
    /// lifecycle; reading this source does not consume it.
    #[track_caller]
    fn accessible_labelled_by(&mut self) -> Vec<NodeRef> {
        self.node_mut().labelled_by_sources()
    }

    /// Copy the preserved description references, in caller/part order.
    /// A composite forwards them with [`Self::described_by`] to its primary
    /// control so a field's hint and error describe the actual editor too.
    /// This does not consume sources or resolve targets; the normal relation
    /// lifecycle still tracks mounts and disposal.
    #[track_caller]
    fn accessible_described_by(&mut self) -> Vec<NodeRef> {
        self.node_mut().described_by_sources()
    }

    /// The role set so far (by the caller, a builder's default or an
    /// attached part), if any: a component that attaches a part with a role
    /// of its own to the caller's node keeps the caller's role with it.
    fn accessible_role(&mut self) -> Option<crate::view::Role> {
        self.node_mut().role_source()
    }

    /// Whether a description was supplied, including an empty description.
    #[track_caller]
    fn has_description(&mut self) -> bool {
        self.node_mut().description_source().is_some()
    }

    /// Read the preserved invalid-state input (`.invalid(..)`, set by the
    /// caller or by a form field around the control) without allocating a
    /// value; `None` when nothing publishes one. A component that draws an
    /// invalid look reads it once its root is complete.
    #[track_caller]
    fn accessible_invalid(&mut self) -> Option<Rc<dyn Fn() -> bool>> {
        self.node_mut()
            .invalid_source()
            .map(kivact_runtime::Prop::into_fn)
    }

    /// Read the preserved expanded-state input (`.expanded(..)`, set by the
    /// caller or by an attached part such as a popup's or a disclosure's
    /// trigger) without allocating a value; `None` when nothing publishes
    /// an expanded state. A component reads it once its root is complete,
    /// to draw itself as the trigger of what it opens.
    #[track_caller]
    fn accessible_expanded(&mut self) -> Option<Rc<dyn Fn() -> bool>> {
        self.node_mut()
            .expanded_source()
            .map(kivact_runtime::Prop::into_fn)
    }

    /// Record `at` as where this element was created, instead of
    /// the call of the builder function: snapshots and diagnostics
    /// name the nodes it creates by `at`. On a component that holds
    /// a root `Node` it relocates the root only; the nodes its
    /// constructor already created keep their location. A component
    /// that creates nodes later (in `into_view`) captures
    /// `Location::caller()` in its `#[track_caller]` constructor and
    /// passes it to `.at(..)` of those nodes, so each instance is
    /// named by its own call site rather than by a line inside the
    /// component.
    fn at(mut self, at: &'static Location<'static>) -> Self {
        self.node_mut().common.at = Some(at);
        self
    }

    /// Mixed checked state, like an HTML checkbox's `indeterminate`.
    /// Assistive technology sees a partially checked control while
    /// this is true (the mixed state wins over `.checked(..)`); the
    /// binary `.checked(..)` value of the App is preserved. The
    /// component decides how to draw and clear its mixed state (a
    /// click clearing it, say). `kivact::test`'s `Element::checked` reads
    /// `Some(CheckedState::Indeterminate)`.
    fn indeterminate(mut self, value: impl IntoValue<bool>) -> Self {
        self.node_mut().common.indeterminate = Some(value.into_prop());
        self
    }

    /// Set value_min semantics, constant or reactive.
    fn value_min(mut self, value: impl IntoValue<f64>) -> Self {
        self.node_mut().common.semantics_mut().value_min = Some(value.into_prop());
        self
    }

    /// Set value_max semantics, constant or reactive.
    fn value_max(mut self, value: impl IntoValue<f64>) -> Self {
        self.node_mut().common.semantics_mut().value_max = Some(value.into_prop());
        self
    }

    /// Set value_now semantics, constant or reactive.
    fn value_now(mut self, value: impl IntoValue<f64>) -> Self {
        self.node_mut().common.semantics_mut().value_now = Some(value.into_prop());
        self
    }

    /// Set live_region semantics, constant or reactive.
    fn live_region(mut self, value: impl IntoValue<LiveRegion>) -> Self {
        self.node_mut().common.semantics_mut().live_region = Some(value.into_prop());
        self
    }

    /// Set sort semantics, constant or reactive.
    fn sort(mut self, value: impl IntoValue<SortDirection>) -> Self {
        self.node_mut().common.semantics_mut().sort = Some(value.into_prop());
        self
    }

    /// Set orientation semantics, constant or reactive.
    fn orientation(mut self, value: impl IntoValue<Orientation>) -> Self {
        self.node_mut().common.semantics_mut().orientation = Some(value.into_prop());
        self
    }

    /// Identify the active option while focus stays on this control,
    /// constant or reactive. `None`, a reference that is not bound to a
    /// mounted node, and a node that is gone name no active option; the
    /// relation follows the value and the node being bound and unmounted.
    /// The option must be a descendant of this node or lie in a node it
    /// `controls` (that node included), as in the ARIA combobox pattern
    /// of a search field and its sibling list; the engine drops any
    /// other.
    fn active_descendant(mut self, descendant: impl IntoValue<Option<NodeRef>>) -> Self {
        self.node_mut().common.semantics_mut().active_descendant = Some(descendant.into_prop());
        self
    }

    /// Set the depth of this node in a hierarchy (`aria-level`), counted
    /// from 1: a heading's level, or a tree item's depth. Constant or
    /// reactive. `0` means "not set" and is not published.
    /// `kivact::test`'s `Element::level` reads what the host published
    /// (`set_size`, `pos_in_set` and `is_multiselectable` likewise).
    fn level(mut self, value: impl IntoValue<u32>) -> Self {
        self.node_mut().common.semantics_mut().level = Some(value.into_prop());
        self
    }

    /// Set how many items the set this node belongs to or holds has
    /// (`aria-setsize`): a list's item count when only part of it is
    /// mounted, or the siblings of a tree item. Constant or reactive.
    /// `0` is published as an empty set.
    fn set_size(mut self, value: impl IntoValue<u32>) -> Self {
        self.node_mut().common.semantics_mut().set_size = Some(value.into_prop());
        self
    }

    /// Set the position of this node in its set (`aria-posinset`),
    /// counted from 1 and not above `.set_size(..)`. Constant or
    /// reactive. `0` means "not set" and is not published; a position
    /// above the set size is not published either (the set size is).
    fn pos_in_set(mut self, value: impl IntoValue<u32>) -> Self {
        self.node_mut().common.semantics_mut().pos_in_set = Some(value.into_prop());
        self
    }

    /// State that this list, tree or grid allows more than one selected
    /// item (`aria-multiselectable`). Constant or reactive.
    fn multiselectable(mut self, value: impl IntoValue<bool>) -> Self {
        self.node_mut().common.semantics_mut().multiselectable = Some(value.into_prop());
        self
    }

    /// Whether assistive technology should wait for updates to finish.
    fn busy(mut self, value: impl IntoValue<bool>) -> Self {
        self.node_mut().common.semantics_mut().busy = Some(value.into_prop());
        self
    }

    /// Whether a value is required.
    fn required(mut self, value: impl IntoValue<bool>) -> Self {
        self.node_mut().common.semantics_mut().required = Some(value.into_prop());
        self
    }

    /// Whether assistive technology should report the value as read-only.
    /// This changes semantics only; configure editing behavior on the editor.
    fn semantics_read_only(mut self, value: impl IntoValue<bool>) -> Self {
        self.node_mut().common.semantics_mut().read_only = Some(value.into_prop());
        self
    }

    /// The type of popup this control opens.
    /// Pass `None` to leave it unstated.
    fn has_popup(mut self, value: impl IntoValue<Option<crate::view::PopupKind>>) -> Self {
        self.node_mut().common.semantics_mut().has_popup = Some(value.into_prop());
        self
    }

    /// The meaning of the current item in a related set.
    /// Pass `None` to leave it unstated.
    fn current_item(mut self, value: impl IntoValue<Option<crate::view::CurrentItem>>) -> Self {
        self.node_mut().common.semantics_mut().current_item = Some(value.into_prop());
        self
    }

    /// Name a node that describes this one (`aria-describedby`), for
    /// text such as a dialog's message or a field's hint. Call it again
    /// to add more; they keep the call order. A reference that is not
    /// bound to a mounted node is left out, and returns when it is.
    /// Core resolves the references when the node mounts and publishes
    /// the relation again whenever a target mounts or is disposed, so a
    /// removed, moved or re-roled target leaves no stale relation (an
    /// unbound or disposed one is no error). `kivact::test`'s
    /// `Element::described_by` (and `controls`, `labelled_by`) returns
    /// the elements the host published.
    fn described_by(mut self, target: NodeRef) -> Self {
        self.node_mut()
            .common
            .semantics_mut()
            .described_by
            .push(target);
        self
    }

    /// Name a node this one controls (`aria-controls`), such as the
    /// panel a button expands. Call it again to add more. A reference
    /// that is not bound to a mounted node is left out.
    fn controls(mut self, target: NodeRef) -> Self {
        self.node_mut().common.semantics_mut().controls.push(target);
        self
    }

    /// Name a node that labels this one (`aria-labelledby`), for a
    /// visible heading or caption that should be the accessible name.
    /// Call it again to add more. A reference that is not bound to a
    /// mounted node is left out.
    fn labelled_by(mut self, target: NodeRef) -> Self {
        self.node_mut()
            .common
            .semantics_mut()
            .labelled_by
            .push(target);
        self
    }

    /// Append component key behavior after existing customizations.
    fn on_key_default<F, R>(mut self, f: F) -> Self
    where
        F: Fn(KeyEvent) -> R + 'static,
        R: KeyHandlerResult,
    {
        self.node_mut()
            .common
            .handlers
            .key
            .node_default(move |e| f(e).into_handled());
        self
    }

    /// Append component pointer behavior after existing customizations.
    fn on_pointer_default<F, R>(mut self, f: F) -> Self
    where
        F: Fn(PointerEvent) -> R + 'static,
        R: KeyHandlerResult,
    {
        self.node_mut()
            .common
            .handlers
            .pointer
            .node_default(move |e| f(e).into_handled());
        self
    }

    /// Append component wheel behavior after existing customizations.
    fn on_wheel_default<F, R>(mut self, f: F) -> Self
    where
        F: Fn(WheelEvent) -> R + 'static,
        R: KeyHandlerResult,
    {
        self.node_mut()
            .common
            .handlers
            .wheel
            .node_default(move |e| f(e).into_handled());
        self
    }

    /// Offer an assistive-technology action. A node has one handler per
    /// action: the later registration replaces the earlier one.
    /// `Activate` is reported and ignored; use `on_press` for activation.
    #[track_caller]
    fn on_a11y<F, R>(mut self, action: A11yAction, f: F) -> Self
    where
        F: Fn() -> R + 'static,
        R: HandlerResult,
    {
        self.node_mut().common.offer_a11y(action, f);
        self
    }

    /// Offer an action and receive its [`A11yEvent`]: the region the request
    /// names (`None` for the node itself, else the `RegionId` its
    /// `Draw::region` declared) and the request (a `SetValue`'s text, a
    /// `Dismiss`'s reason). `Activate` is supported here for canvas regions;
    /// use `on_press` for ordinary activation. The later handler for an
    /// action wins. A `SetValue` hands the requested text over as it came: parsing it,
    /// checking it is finite and clamping it to the range are the
    /// component's.
    fn on_a11y_event<F, R>(mut self, action: A11yAction, f: F) -> Self
    where
        F: Fn(A11yEvent) -> R + 'static,
        R: HandlerResult,
    {
        self.node_mut().common.offer_a11y_event(action, f);
        self
    }

    /// Give the platform's input method (IME) this node's text, selection and
    /// caret while it has the focus, so a node that draws and edits its own
    /// text composes and reconverts like a native editor (see
    /// [`ImeSession`]). The node must be focusable and must not be a
    /// native editor ([`TextEditing`](super::TextEditing)). The IME's composition, commits and
    /// reconversion ranges arrive at [`on_ime`](Self::on_ime). A later
    /// session replaces an earlier one.
    ///
    /// The session is published after the node's draw layers run in the
    /// same turn, so a value closure that reads what the drawing just made
    /// (the caret of the [`TextParagraph`](super::TextParagraph) `Draw::layout_paragraph` laid
    /// out, kept in a cell) sees this turn's paragraph: a draw closure
    /// cannot write a signal, so that is how geometry reaches the session.
    fn ime_session(mut self, session: ImeSession) -> Self {
        self.node_mut().common.ime = Some(session);
        self
    }

    /// Handle the input method's events for this node's
    /// [`ime_session`](Self::ime_session): a composition (`Preedit`), a
    /// `Commit`, a reconversion `Selection` and the `End` of the session.
    /// Like `on_key`, the most recently registered handler runs first and
    /// `Yes` stops the rest.
    fn on_ime<F, R>(mut self, f: F) -> Self
    where
        F: Fn(ImeInput) -> R + 'static,
        R: KeyHandlerResult,
    {
        self.node_mut().common.prepend_ime(f);
        self
    }

    /// Publish this node's own text for assistive technology as text runs
    /// with a selection and a caret, in the node's own coordinates, so a
    /// screen reader can read, navigate and locate the text of a node that
    /// draws it (see [`TextParagraph::semantics_text`](super::TextParagraph::semantics_text)). Constant or
    /// reactive. A native editor ([`TextEditing`](super::TextEditing)) publishes its own and
    /// ignores this; a value that cannot be published (no run, a selection
    /// outside the runs, a rectangle that is not finite) is reported as
    /// `ErrorKind::Unsupported` and not published. Give the node a role
    /// that reads text (`Role::TextBox`). Like [`ime_session`](Self::ime_session)
    /// it is published after the node's draw layers.
    fn semantics_text(mut self, text: impl IntoValue<SemanticsText>) -> Self {
        self.node_mut().common.semantics_mut().text = Some(text.into_prop());
        self
    }

    /// Local draw layer over the children, after both `draw` and `Style::paint`.
    fn draw_over(mut self, f: impl Fn(&mut Draw, Size) + 'static) -> Self {
        self.node_mut().common.draw_over = Some(Rc::new(f));
        self
    }
}

impl<T: NodeBuilder> NodeExt for T {}
