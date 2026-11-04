//! Role: The element type and the modifier contract of every element builder.
//!
//! `Node` and `NodeBuilder` are defined here, below the element builders in
//! `view` (`overlay`, `virtual_list`) that hold a `Node` and implement
//! `NodeBuilder`; `kivact::control` re-exports them and holds the primitives
//! that construct a `Node`.
//!
//! Invariant: what a node is (a box, a text, an image) is fixed by the
//! function that made it. A setting that only one kind of node can show
//! (`child`, `draw`, `fit`, ...) given to another kind is reported once at
//! mount as `ErrorKind::Unsupported`, located at the node, and dropped; it
//! never reaches the host.

use std::panic::Location;
use std::rc::Rc;

use kivact_engine::contract::NodeType;

use crate::view::View;
use crate::view::common::Common;
use crate::view::descriptor::Mcx;
use crate::view::ime_session::{ImeInput, ImeSession};
use crate::view::input::{FileEvent, KeyEvent, PointerEvent, WheelEvent};
use crate::view::node_ref::NodeRef;
use crate::view::part::Part;
use crate::view::part_data::PartMount;
use crate::view::span::Span;
use crate::view::{A11yAction, A11yEvent, LiveRegion, Orientation, Role, SortDirection};
use crate::view::{HandlerResult, IntoText, IntoValue};
use kivact_runtime::Prop as InputProp;
use kivact_runtime::{Callback, ErrorKind, UiError};
use kivact_style::Style;
use kivact_types::SemanticsText;

use super::exit_layout::ExitLayout;
use kivact_style::asset::Asset;
use kivact_style::draw::{Draw, IntrinsicSize, Size};

/// What a node shows.
pub(crate) enum Content {
    /// A flex container of its children.
    Box,
    /// A run of text.
    Plain(InputProp<String>),
    /// A paragraph of styled spans.
    Rich(InputProp<Vec<Span>>),
    /// An image.
    Image(InputProp<Asset>),
}

impl Content {
    pub(crate) fn kind(&self) -> NodeType {
        match self {
            Content::Box => NodeType::Box,
            Content::Plain(_) | Content::Rich(_) => NodeType::Text,
            Content::Image(_) => NodeType::Image,
        }
    }
}

/// An element: a box (`node()`, `column`, `row`, `stack`, `grid`,
/// `scroll`, `spacer`, `canvas`), a text (`text`, `rich_text`) or an image
/// (`image`). Its everyday modifiers (`style`, `name`, `on_press`, `draw`,
/// ...) are the methods of [`NodeBuilder`] (in the prelude), which a
/// component holding a root `Node` implements to get them. The semantics a
/// node of your own declares ([`role`](Self::role), [`checked`](Self::checked),
/// [`expanded`](Self::expanded), [`selected`](Self::selected),
/// [`value_text`](Self::value_text)) and [`part`](Self::part) are its own
/// methods too, so `node().role(..)` needs no import; a component's builder
/// gets them, for its root, from [`Node`](crate::view::Node). The
/// settings that mean something only on one kind of node are its own methods
/// ([`child`](Self::child) for a box, [`fit`](Self::fit) and
/// [`monochrome`](Self::monochrome) for an image). Text properties (font, wrap, line clamp, alignment, overflow)
/// are style properties, inherited by the texts inside a node.
pub struct Node {
    pub(crate) content: Content,
    pub(crate) common: Common,
    pub(crate) children: Vec<View>,
    pub(crate) modifiers: Vec<Modifier>,
    /// Where the node was created; `None` for a plain string child, which
    /// diagnostics show with the location of its nearest located ancestor.
    pub(crate) at: Option<&'static Location<'static>>,
}

impl Node {
    /// A node showing `content`, located at `at`.
    pub(crate) fn new(content: Content, at: Option<&'static Location<'static>>) -> Self {
        Node {
            content,
            common: Common::default(),
            children: Vec::new(),
            modifiers: Vec::new(),
            at,
        }
    }
}

impl Node {
    /// The modifiers of an element builder that mounts its own host node
    /// (`overlay`, `virtual_list`) and keeps them in a `Node` it holds for
    /// `NodeBuilder`. The node's content and children are unused. A draw
    /// layer or size hint that such a builder cannot show is reported once,
    /// located at the builder, and dropped.
    pub(crate) fn into_modifiers(
        self,
        cx: &Mcx<'_>,
        what: &str,
        at: &'static Location<'static>,
    ) -> Common {
        let Node {
            mut common,
            children,
            ..
        } = self.resolve_modifiers();
        debug_assert!(
            children.is_empty(),
            "{what} keeps its modifiers in a Node and takes no children there"
        );
        let mut misused: Vec<&str> = Vec::new();
        if common.draw.take().is_some() {
            misused.push("draw");
        }
        if common.draw_over.take().is_some() {
            misused.push("draw_over");
        }
        if common.intrinsic.take().is_some() {
            misused.push("intrinsic_size");
        }
        if !misused.is_empty() {
            let list = misused
                .iter()
                .map(|m| format!("`.{m}(..)`"))
                .collect::<Vec<_>>()
                .join(", ");
            cx.rt.report(
                Some(cx.owner),
                UiError::new(
                    ErrorKind::Unsupported,
                    format!("{list} on {what}, which cannot show it; dropped"),
                    Some(at),
                ),
            );
        }
        common
    }
}

/// The everyday modifiers of every element: implemented by [`Node`] and by
/// every component builder that holds a root `Node`. A component implements
/// [`node_mut`](Self::node_mut) (one line), and `style`, `name`, `disabled`,
/// `test_id`, `on_key`, `on_press`, `draw`, ... all apply to its root. The
/// semantics that belong to what the component is (`role`, `checked`,
/// `expanded`, `selected`, `value_text`), attaching a part, the extra
/// semantics, handler layering and `draw_over` are the methods of
/// [`Node`](crate::view::Node): a component decides what its own
/// `checked` or `role` is, so a caller does not set them from outside.
///
/// ```
/// # use kivact_core as kivact;
/// use kivact::prelude::*;
/// use kivact::view::View;
///
/// pub struct Card { root: Node }
///
/// impl NodeBuilder for Card {
///     fn node_mut(&mut self) -> &mut Node { &mut self.root }
/// }
/// impl IntoView for Card {
///     fn into_view(self) -> View { self.root.into_view() }
/// }
///
/// fn card() -> Card { Card { root: node() } }
///
/// let _ = card().name("Profile").style(style().margin(4));
/// ```
///
/// `Node`, `Overlay` and `VirtualList` implement it too. A modifier written
/// after the component's own settings wins. `draw`
/// and `intrinsic_size` mean something on a box (and
/// `intrinsic_size` on an image too); on another node they are reported
/// once at mount (`ErrorKind::Unsupported`). `Node::{child, region,
/// fit, monochrome}` are not part of it: a component exposes
/// what its children and states mean through its own methods.
///
/// Every `on_*` handler runs untracked: a `get` in it reads the current
/// value and subscribes nothing, so it needs no `get_untracked`.
pub trait NodeBuilder: Sized {
    /// The root node the modifiers apply to.
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node;

    /// Transform the root node while retaining this component builder.
    fn with_node(mut self, f: impl FnOnce(Node) -> Node) -> Self {
        let root = self.node_mut();
        let placeholder = Node::new(Content::Box, None);
        *root = f(std::mem::replace(root, placeholder));
        self
    }

    /// Test identifier for the `*_by_test_id` queries. Takes any
    /// string (`"save"`, `format!("row-{id}")`). It is not scoped:
    /// a query matching several nodes fails and lists them all;
    /// narrow it with `Screen::within`.
    fn test_id(mut self, id: impl AsRef<str>) -> Self {
        self.node_mut().common.test_id = Some(Rc::from(id.as_ref()));
        self
    }

    /// Bind `r` to this node once it is mounted. Several refs can be
    /// bound to one node (a component's own and the user's): each
    /// `.node_ref(..)` adds one, none replaces another.
    fn node_ref(mut self, r: NodeRef) -> Self {
        self.node_mut().common.node_ref.push(r);
        self
    }

    /// Add a style layer, constant or reactive. Layers merge in
    /// call order: a property set by a later layer wins, the
    /// properties it leaves unset keep their earlier values. A
    /// builder sets its own base style first, so `.style(..)`
    /// written after it overrides only what it sets. A closure that
    /// reads [`current_theme`](crate::style::current_theme) is
    /// recomputed when the theme visible at the node changes:
    /// `.style(move || { let t = &current_theme(); style().padding(t.space().md) })`.
    fn style(mut self, s: impl IntoValue<Style>) -> Self {
        self.node_mut().common.push_style(s.into_prop());
        self
    }

    /// Accessible name.
    fn name(mut self, v: impl IntoText) -> Self {
        self.node_mut().common.name = Some(v.into_text());
        self
    }

    /// Accessible description.
    fn description(mut self, v: impl IntoText) -> Self {
        self.node_mut().common.description = Some(v.into_text());
        self
    }

    /// Disabled state (semantics and the `disabled` style). A component
    /// reads it with
    /// [`accessible_disabled`](crate::view::Node::accessible_disabled)
    /// and gives it to its behavior, so this one definition disables the
    /// component's own input too.
    /// Suppresses activation, key and wheel callbacks, and pointer
    /// Down/Move/Up on this node; enabled descendants remain interactive.
    /// Pointer Enter/Leave remain available for explanations, and Cancel
    /// ends an existing drag when the capturing node becomes disabled.
    /// Unhandled wheels continue to enabled ancestors and native scrolling.
    fn disabled(mut self, v: impl IntoValue<bool>) -> Self {
        self.node_mut().common.disabled = Some(v.into_prop());
        self
    }

    /// Report an invalid value to assistive technology. A component that
    /// draws an invalid look reads it with
    /// [`accessible_invalid`](crate::view::Node::accessible_invalid).
    fn invalid(mut self, value: impl IntoValue<bool>) -> Self {
        self.node_mut().common.invalid = Some(value.into_prop());
        self
    }

    /// Set how this node lays out when it is the root of a removed subtree.
    /// Absolute, fixed and overlay nodes already occupy no slot in normal flow.
    fn exit_layout(mut self, layout: ExitLayout) -> Self {
        self.node_mut().common.exit_layout = layout;
        self
    }

    /// Run after this node, when directly removed, is no longer drawn.
    /// The callback runs once even when the removal has no animation or an
    /// ancestor cuts the animation short. Create it outside the removed
    /// branch so its owner can still be alive when the exit finishes.
    fn on_exit_complete(mut self, callback: Callback<()>) -> Self {
        self.node_mut().common.exit_complete = Some(kivact_runtime::Handler::from_fn(move |()| {
            callback.call_if_alive(())
        }));
        self
    }

    /// Hide the node and its subtree.
    fn hidden(mut self, v: impl IntoValue<bool>) -> Self {
        self.node_mut().common.hidden = Some(v.into_prop());
        self
    }

    /// Whether the pointer hits the node and its subtree (default `true`).
    /// With `false` the pointer passes through them to what lies behind,
    /// as CSS `pointer-events: none`: no hover, press or wheel reaches
    /// them, which suits a floating card that follows the pointer.
    /// Keyboard focus and assistive technology are unaffected.
    fn hit_testable(mut self, on: bool) -> Self {
        self.node_mut().common.hit_testable = Some(on);
        self
    }

    /// Whether the node takes keyboard focus, overriding the builder's default.
    /// Reactive values support a roving tab stop without replacing nodes:
    /// only the selected tab or segment is a Tab stop, and the stop moves
    /// while the nodes stay.
    fn focusable(mut self, on: impl IntoValue<bool>) -> Self {
        self.node_mut().common.focusable = Some(on.into_prop());
        self
    }

    /// Keep an otherwise focusable control in keyboard focus order while it
    /// is disabled, so users can reach its label and any attached tooltip.
    /// Activation and editing remain disabled.
    fn focusable_when_disabled(mut self, on: bool) -> Self {
        self.node_mut().common.focusable_when_disabled = on;
        self
    }

    /// Give this group one Tab stop and move between its eligible controls
    /// with unhandled arrows along `orientation`, Home and End. A text field
    /// keeps arrows that still move its caret or selection.
    #[track_caller]
    fn roving_focus(mut self, orientation: crate::view::Orientation) -> Self {
        use crate::view::node_ref::RovingMove;
        use crate::view::{Handled, Key, Modifiers, Orientation, ReadingDirection};
        let rtl = crate::view::current_direction() == ReadingDirection::Rtl;
        let reference = crate::view::node_ref();
        let common = &mut self.node_mut().common;
        common.roving_focus = Some(kivact_engine::contract::RovingFocus { orientation, rtl });
        common.node_ref.push(reference);
        self.on_key(move |event| {
            if event.modifiers() != Modifiers::default() {
                return Handled::No;
            }
            let movement = match (orientation, event.key()) {
                (Orientation::Horizontal, Key::ArrowLeft) if !rtl => RovingMove::Previous,
                (Orientation::Horizontal, Key::ArrowRight) if rtl => RovingMove::Previous,
                (Orientation::Horizontal, Key::ArrowLeft | Key::ArrowRight) => RovingMove::Next,
                (Orientation::Vertical, Key::ArrowUp) => RovingMove::Previous,
                (Orientation::Vertical, Key::ArrowDown) => RovingMove::Next,
                (_, Key::Home) => RovingMove::First,
                (_, Key::End) => RovingMove::Last,
                _ => return Handled::No,
            };
            if reference.move_roving_focus(movement) {
                Handled::Yes
            } else {
                Handled::No
            }
        })
    }

    /// Handle keys reported by the host before existing key handlers,
    /// default activation and Escape dismissal. IME, Tab and editing keys
    /// consumed by the host do not reach this callback. A key starts at
    /// the focused node (inside the topmost open modal, else at that
    /// modal) and bubbles to its ancestors, stopping at an open modal;
    /// while nothing has the focus and no modal is open it starts at
    /// the App's view (the view passed to `App::run` or
    /// `kivact::test::render`, its first node when that is a
    /// fragment), so window-level shortcuts are handlers on that view.
    /// An Enter or Space that reached the App's view that way does not
    /// activate it.
    /// `No` continues, `Yes` consumes, `PreventDefault` continues without
    /// the default activation or dismissal, and an error stops and is
    /// reported once.
    /// Repeated customizations run most recently registered first.
    fn on_key<F, R>(mut self, f: F) -> Self
    where
        F: Fn(KeyEvent) -> R + 'static,
        R: HandlerResult,
    {
        self.node_mut().common.prepend_key(f);
        self
    }

    /// Key release handler; releases do not edit or activate controls.
    fn on_key_up<F, R>(mut self, f: F) -> Self
    where
        F: Fn(KeyEvent) -> R + 'static,
        R: HandlerResult,
    {
        self.node_mut().common.prepend_key_up(f);
        self
    }

    /// Run before existing pointer behavior, in node-local coordinates.
    /// `No` and `PreventDefault` continue, `Yes` consumes, and an error
    /// stops the chain.
    /// This controls the callback chain; host focus/capture/caret changes
    /// and its separate Activate action retain their native behavior.
    fn on_pointer<F, R>(mut self, f: F) -> Self
    where
        F: Fn(PointerEvent) -> R + 'static,
        R: HandlerResult,
    {
        self.node_mut().common.prepend_pointer(f);
        self
    }

    /// Run before existing wheel behavior and native scrolling.
    /// `No` continues, `Yes` consumes, `PreventDefault` continues without
    /// native scrolling, and an error stops the chain.
    fn on_wheel<F, R>(mut self, f: F) -> Self
    where
        F: Fn(WheelEvent) -> R + 'static,
        R: HandlerResult,
    {
        self.node_mut().common.prepend_wheel(f);
        self
    }

    /// Set the shared pointer, Enter/Space and accessibility Activate action.
    /// This replaces the existing caller action.
    /// Attached part defaults continue to run after the caller action.
    fn on_press<F, R>(mut self, f: F) -> Self
    where
        F: Fn() -> R + 'static,
        R: HandlerResult,
    {
        self.node_mut().common.set_press(f);
        self
    }

    /// OS file drag handler, with node-local and window positions.
    fn on_file<F, R>(mut self, f: F) -> Self
    where
        F: Fn(FileEvent) -> R + 'static,
        R: HandlerResult,
    {
        self.node_mut()
            .common
            .handlers
            .file
            .replace_caller(move |event| f(event).into_outcome());
        self
    }

    /// Local draw layer behind the children (node-local coordinates).
    /// Callers use this layer; recipes and themes use `Style::paint`
    /// so style merging retains their drawing.
    fn draw(mut self, f: impl Fn(&mut Draw, Size) + 'static) -> Self {
        self.node_mut().common.draw = Some(Rc::new(f));
        self
    }

    /// Size hints used by layout when the node has no children to measure.
    fn intrinsic_size(mut self, s: IntrinsicSize) -> Self {
        self.node_mut().common.intrinsic = Some(s);
        self
    }
}

impl NodeBuilder for Node {
    fn node_mut(&mut self) -> &mut Node {
        self
    }
}

impl Node {
    /// Test identifier for the `*_by_test_id` queries. Takes any
    /// string (`"save"`, `format!("row-{id}")`). It is not scoped:
    /// a query matching several nodes fails and lists them all;
    /// narrow it with `Screen::within`.
    pub fn test_id(mut self, id: impl AsRef<str>) -> Self {
        self.common.test_id = Some(Rc::from(id.as_ref()));
        self
    }

    /// Bind `r` to this node once it is mounted. Several refs can be
    /// bound to one node (a component's own and the user's): each
    /// `.node_ref(..)` adds one, none replaces another.
    pub fn node_ref(mut self, r: NodeRef) -> Self {
        self.common.node_ref.push(r);
        self
    }

    /// Add a style layer, constant or reactive. Layers merge in
    /// call order: a property set by a later layer wins, the
    /// properties it leaves unset keep their earlier values. A
    /// builder sets its own base style first, so `.style(..)`
    /// written after it overrides only what it sets. A closure that
    /// reads [`current_theme`](crate::style::current_theme) is
    /// recomputed when the theme visible at the node changes:
    /// `.style(move || { let t = &current_theme(); style().padding(t.space().md) })`.
    pub fn style(mut self, s: impl IntoValue<Style>) -> Self {
        self.common.push_style(s.into_prop());
        self
    }

    /// Accessible name.
    pub fn name(mut self, v: impl IntoText) -> Self {
        self.common.name = Some(v.into_text());
        self
    }

    /// Accessible description.
    pub fn description(mut self, v: impl IntoText) -> Self {
        self.common.description = Some(v.into_text());
        self
    }

    /// Disabled state (semantics and the `disabled` style). A component
    /// reads it with
    /// [`accessible_disabled`](crate::view::Node::accessible_disabled)
    /// and gives it to its behavior, so this one definition disables the
    /// component's own input too.
    /// Suppresses activation, key and wheel callbacks, and pointer
    /// Down/Move/Up on this node; enabled descendants remain interactive.
    /// Pointer Enter/Leave remain available for explanations, and Cancel
    /// ends an existing drag when the capturing node becomes disabled.
    /// Unhandled wheels continue to enabled ancestors and native scrolling.
    pub fn disabled(mut self, v: impl IntoValue<bool>) -> Self {
        self.common.disabled = Some(v.into_prop());
        self
    }

    /// Report an invalid value to assistive technology. A component that
    /// draws an invalid look reads it with
    /// [`accessible_invalid`](crate::view::Node::accessible_invalid).
    pub fn invalid(mut self, value: impl IntoValue<bool>) -> Self {
        self.common.invalid = Some(value.into_prop());
        self
    }

    /// Set how this node lays out when it is the root of a removed subtree.
    /// Absolute, fixed and overlay nodes already occupy no slot in normal flow.
    pub fn exit_layout(mut self, layout: ExitLayout) -> Self {
        self.common.exit_layout = layout;
        self
    }

    /// Run after this node, when directly removed, is no longer drawn.
    /// The callback runs once even when the removal has no animation or an
    /// ancestor cuts the animation short. Create it outside the removed
    /// branch so its owner can still be alive when the exit finishes.
    pub fn on_exit_complete(mut self, callback: Callback<()>) -> Self {
        self.common.exit_complete = Some(kivact_runtime::Handler::from_fn(move |()| {
            callback.call_if_alive(())
        }));
        self
    }

    /// Hide the node and its subtree.
    pub fn hidden(mut self, v: impl IntoValue<bool>) -> Self {
        self.common.hidden = Some(v.into_prop());
        self
    }

    /// Whether the pointer hits the node and its subtree (default `true`).
    /// With `false` the pointer passes through them to what lies behind,
    /// as CSS `pointer-events: none`: no hover, press or wheel reaches
    /// them, which suits a floating card that follows the pointer.
    /// Keyboard focus and assistive technology are unaffected.
    pub fn hit_testable(mut self, on: bool) -> Self {
        self.common.hit_testable = Some(on);
        self
    }

    /// Whether the node takes keyboard focus, overriding the builder's default.
    /// Reactive values support a roving tab stop without replacing nodes:
    /// only the selected tab or segment is a Tab stop, and the stop moves
    /// while the nodes stay.
    pub fn focusable(mut self, on: impl IntoValue<bool>) -> Self {
        self.common.focusable = Some(on.into_prop());
        self
    }

    /// Keep an otherwise focusable control in keyboard focus order while it
    /// is disabled, so users can reach its label and any attached tooltip.
    /// Activation and editing remain disabled.
    pub fn focusable_when_disabled(mut self, on: bool) -> Self {
        self.common.focusable_when_disabled = on;
        self
    }

    /// Give this group one Tab stop and move between its eligible controls
    /// with unhandled arrows along `orientation`, Home and End. A text field
    /// keeps arrows that still move its caret or selection.
    #[track_caller]
    pub fn roving_focus(mut self, orientation: crate::view::Orientation) -> Self {
        use crate::view::node_ref::RovingMove;
        use crate::view::{Handled, Key, Modifiers, Orientation, ReadingDirection};
        let rtl = crate::view::current_direction() == ReadingDirection::Rtl;
        let reference = crate::view::node_ref();
        let common = &mut self.common;
        common.roving_focus = Some(kivact_engine::contract::RovingFocus { orientation, rtl });
        common.node_ref.push(reference);
        self.on_key(move |event| {
            if event.modifiers() != Modifiers::default() {
                return Handled::No;
            }
            let movement = match (orientation, event.key()) {
                (Orientation::Horizontal, Key::ArrowLeft) if !rtl => RovingMove::Previous,
                (Orientation::Horizontal, Key::ArrowRight) if rtl => RovingMove::Previous,
                (Orientation::Horizontal, Key::ArrowLeft | Key::ArrowRight) => RovingMove::Next,
                (Orientation::Vertical, Key::ArrowUp) => RovingMove::Previous,
                (Orientation::Vertical, Key::ArrowDown) => RovingMove::Next,
                (_, Key::Home) => RovingMove::First,
                (_, Key::End) => RovingMove::Last,
                _ => return Handled::No,
            };
            if reference.move_roving_focus(movement) {
                Handled::Yes
            } else {
                Handled::No
            }
        })
    }

    /// Handle keys reported by the host before existing key handlers,
    /// default activation and Escape dismissal. IME, Tab and editing keys
    /// consumed by the host do not reach this callback. A key starts at
    /// the focused node (inside the topmost open modal, else at that
    /// modal) and bubbles to its ancestors, stopping at an open modal;
    /// while nothing has the focus and no modal is open it starts at
    /// the App's view (the view passed to `App::run` or
    /// `kivact::test::render`, its first node when that is a
    /// fragment), so window-level shortcuts are handlers on that view.
    /// An Enter or Space that reached the App's view that way does not
    /// activate it.
    /// `No` continues, `Yes` consumes, `PreventDefault` continues without
    /// the default activation or dismissal, and an error stops and is
    /// reported once.
    /// Repeated customizations run most recently registered first.
    pub fn on_key<F, R>(mut self, f: F) -> Self
    where
        F: Fn(KeyEvent) -> R + 'static,
        R: HandlerResult,
    {
        self.common.prepend_key(f);
        self
    }

    /// Key release handler; releases do not edit or activate controls.
    pub fn on_key_up<F, R>(mut self, f: F) -> Self
    where
        F: Fn(KeyEvent) -> R + 'static,
        R: HandlerResult,
    {
        self.common.prepend_key_up(f);
        self
    }

    /// Run before existing pointer behavior, in node-local coordinates.
    /// `No` and `PreventDefault` continue, `Yes` consumes, and an error
    /// stops the chain.
    /// This controls the callback chain; host focus/capture/caret changes
    /// and its separate Activate action retain their native behavior.
    pub fn on_pointer<F, R>(mut self, f: F) -> Self
    where
        F: Fn(PointerEvent) -> R + 'static,
        R: HandlerResult,
    {
        self.common.prepend_pointer(f);
        self
    }

    /// Run before existing wheel behavior and native scrolling.
    /// `No` continues, `Yes` consumes, `PreventDefault` continues without
    /// native scrolling, and an error stops the chain.
    pub fn on_wheel<F, R>(mut self, f: F) -> Self
    where
        F: Fn(WheelEvent) -> R + 'static,
        R: HandlerResult,
    {
        self.common.prepend_wheel(f);
        self
    }

    /// Set the shared pointer, Enter/Space and accessibility Activate action.
    /// This replaces the existing caller action.
    /// Attached part defaults continue to run after the caller action.
    pub fn on_press<F, R>(mut self, f: F) -> Self
    where
        F: Fn() -> R + 'static,
        R: HandlerResult,
    {
        self.common.set_press(f);
        self
    }

    /// OS file drag handler, with node-local and window positions.
    pub fn on_file<F, R>(mut self, f: F) -> Self
    where
        F: Fn(FileEvent) -> R + 'static,
        R: HandlerResult,
    {
        self.common
            .handlers
            .file
            .replace_caller(move |event| f(event).into_outcome());
        self
    }

    /// Local draw layer behind the children (node-local coordinates).
    /// Callers use this layer; recipes and themes use `Style::paint`
    /// so style merging retains their drawing.
    pub fn draw(mut self, f: impl Fn(&mut Draw, Size) + 'static) -> Self {
        self.common.draw = Some(Rc::new(f));
        self
    }

    /// Size hints used by layout when the node has no children to measure.
    pub fn intrinsic_size(mut self, s: IntrinsicSize) -> Self {
        self.common.intrinsic = Some(s);
        self
    }
}

impl Node {
    /// Attach behavior, semantic relationships and input defaults to this element.
    ///
    /// Later semantic modifiers override values supplied by the part.
    /// Disabled and hidden requirements combine with OR; focus constraints
    /// combine with AND. Caller input customizations run before part defaults,
    /// regardless of whether they are registered before or after attachment.
    /// References, observations and cleanup from earlier parts are retained.
    pub fn part<S: Clone + PartialEq + 'static>(mut self, part: Part<S>) -> Self {
        self.record_part(part);
        self
    }

    /// Accessibility role, overriding the builder's default role.
    pub fn role(mut self, r: Role) -> Self {
        self.common.role = Some(r);
        self
    }

    /// Accessible value text, as a string (`aria-valuetext`) of a slider or
    /// progress bar. A text input's value is its own, not this one.
    pub fn value_text(mut self, v: impl IntoText) -> Self {
        self.common.value = Some(v.into_text());
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
    pub fn base_style(mut self, s: impl IntoValue<Style>) -> Self {
        self.common.prepend_style(s.into_prop());
        self
    }

    /// Selected state (semantics and the `selected` style).
    pub fn selected(mut self, v: impl IntoValue<bool>) -> Self {
        self.common.selected = Some(v.into_prop());
        self
    }

    /// Binary checked state for a check box or switch.
    pub fn checked(mut self, value: impl IntoValue<bool>) -> Self {
        self.common.checked = Some(value.into_prop());
        self
    }

    /// Expanded state of a disclosure, menu or combo box.
    pub fn expanded(mut self, value: impl IntoValue<bool>) -> Self {
        self.common.expanded = Some(value.into_prop());
        self
    }

    /// Read the preserved disabled-state input (`.disabled(..)`, set by the
    /// caller) without allocating a value; `None` when nothing publishes
    /// one. A component gives it to its behavior once its root is complete,
    /// so one `disabled` modifier disables the node and the component's own
    /// input, with no method of the component's to forget.
    #[track_caller]
    pub fn accessible_disabled(&mut self) -> Option<Rc<dyn Fn() -> bool>> {
        self.disabled_source().map(kivact_runtime::Prop::into_fn)
    }

    /// Read the preserved accessible-name input without allocating a value.
    #[track_caller]
    pub fn accessible_name(&mut self) -> Option<Rc<dyn Fn() -> String>> {
        self.name_source().map(kivact_runtime::Prop::into_fn)
    }

    /// Copy the preserved label references, in caller/part order, without
    /// resolving their mounted targets. A composite forwards these to its
    /// primary control with [`Self::labelled_by`], retaining the editor's
    /// own relations. Unbound/disposed targets follow the normal relation
    /// lifecycle; reading this source does not consume it.
    #[track_caller]
    pub fn accessible_labelled_by(&mut self) -> Vec<NodeRef> {
        self.labelled_by_sources()
    }

    /// Copy the preserved description references, in caller/part order.
    /// A composite forwards them with [`Self::described_by`] to its primary
    /// control so a field's hint and error describe the actual editor too.
    /// This does not consume sources or resolve targets; the normal relation
    /// lifecycle still tracks mounts and disposal.
    #[track_caller]
    pub fn accessible_described_by(&mut self) -> Vec<NodeRef> {
        self.described_by_sources()
    }

    /// The role set so far (by the caller, a builder's default or an
    /// attached part), if any: a component that attaches a part with a role
    /// of its own to the caller's node keeps the caller's role with it.
    pub fn accessible_role(&mut self) -> Option<crate::view::Role> {
        self.role_source()
    }

    /// Whether a description was supplied, including an empty description.
    #[track_caller]
    pub fn has_description(&mut self) -> bool {
        self.description_source().is_some()
    }

    /// Read the preserved invalid-state input (`.invalid(..)`, set by the
    /// caller or by a form field around the control) without allocating a
    /// value; `None` when nothing publishes one. A component that draws an
    /// invalid look reads it once its root is complete.
    #[track_caller]
    pub fn accessible_invalid(&mut self) -> Option<Rc<dyn Fn() -> bool>> {
        self.invalid_source().map(kivact_runtime::Prop::into_fn)
    }

    /// Read the preserved expanded-state input (`.expanded(..)`, set by the
    /// caller or by an attached part such as a popup's or a disclosure's
    /// trigger) without allocating a value; `None` when nothing publishes
    /// an expanded state. A component reads it once its root is complete,
    /// to draw itself as the trigger of what it opens.
    #[track_caller]
    pub fn accessible_expanded(&mut self) -> Option<Rc<dyn Fn() -> bool>> {
        self.expanded_source().map(kivact_runtime::Prop::into_fn)
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
    #[track_caller]
    pub fn at(mut self) -> Self {
        let at = Location::caller();
        self.common.at = Some(at);
        self
    }

    /// Mixed checked state, like an HTML checkbox's `indeterminate`.
    /// Assistive technology sees a partially checked control while
    /// this is true (the mixed state wins over `.checked(..)`); the
    /// binary `.checked(..)` value of the App is preserved. The
    /// component decides how to draw and clear its mixed state (a
    /// click clearing it, say). `kivact::test`'s `Element::checked` reads
    /// `Some(CheckedState::Indeterminate)`.
    pub fn indeterminate(mut self, value: impl IntoValue<bool>) -> Self {
        self.common.indeterminate = Some(value.into_prop());
        self
    }

    /// Set value_min semantics, constant or reactive.
    pub fn value_min(mut self, value: impl IntoValue<f64>) -> Self {
        self.common.semantics_mut().value_min = Some(value.into_prop());
        self
    }

    /// Set value_max semantics, constant or reactive.
    pub fn value_max(mut self, value: impl IntoValue<f64>) -> Self {
        self.common.semantics_mut().value_max = Some(value.into_prop());
        self
    }

    /// Set value_now semantics, constant or reactive.
    pub fn value_now(mut self, value: impl IntoValue<f64>) -> Self {
        self.common.semantics_mut().value_now = Some(value.into_prop());
        self
    }

    /// Set live_region semantics, constant or reactive.
    pub fn live_region(mut self, value: impl IntoValue<LiveRegion>) -> Self {
        self.common.semantics_mut().live_region = Some(value.into_prop());
        self
    }

    /// Set sort semantics, constant or reactive.
    pub fn sort(mut self, value: impl IntoValue<SortDirection>) -> Self {
        self.common.semantics_mut().sort = Some(value.into_prop());
        self
    }

    /// Set orientation semantics, constant or reactive.
    pub fn orientation(mut self, value: impl IntoValue<Orientation>) -> Self {
        self.common.semantics_mut().orientation = Some(value.into_prop());
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
    pub fn active_descendant(mut self, descendant: impl IntoValue<Option<NodeRef>>) -> Self {
        self.common.semantics_mut().active_descendant = Some(descendant.into_prop());
        self
    }

    /// Set the depth of this node in a hierarchy (`aria-level`), counted
    /// from 1: a heading's level, or a tree item's depth. Constant or
    /// reactive. `0` means "not set" and is not published.
    /// `kivact::test`'s `Element::level` reads what the host published
    /// (`set_size`, `pos_in_set` and `is_multiselectable` likewise).
    pub fn level(mut self, value: impl IntoValue<u32>) -> Self {
        self.common.semantics_mut().level = Some(value.into_prop());
        self
    }

    /// Set how many items the set this node belongs to or holds has
    /// (`aria-setsize`): a list's item count when only part of it is
    /// mounted, or the siblings of a tree item. Constant or reactive.
    /// `0` is published as an empty set.
    pub fn set_size(mut self, value: impl IntoValue<u32>) -> Self {
        self.common.semantics_mut().set_size = Some(value.into_prop());
        self
    }

    /// Set the position of this node in its set (`aria-posinset`),
    /// counted from 1 and not above `.set_size(..)`. Constant or
    /// reactive. `0` means "not set" and is not published; a position
    /// above the set size is not published either (the set size is).
    pub fn pos_in_set(mut self, value: impl IntoValue<u32>) -> Self {
        self.common.semantics_mut().pos_in_set = Some(value.into_prop());
        self
    }

    /// State that this list, tree or grid allows more than one selected
    /// item (`aria-multiselectable`). Constant or reactive.
    pub fn multiselectable(mut self, value: impl IntoValue<bool>) -> Self {
        self.common.semantics_mut().multiselectable = Some(value.into_prop());
        self
    }

    /// Whether assistive technology should wait for updates to finish.
    pub fn busy(mut self, value: impl IntoValue<bool>) -> Self {
        self.common.semantics_mut().busy = Some(value.into_prop());
        self
    }

    /// Whether a value is required.
    pub fn required(mut self, value: impl IntoValue<bool>) -> Self {
        self.common.semantics_mut().required = Some(value.into_prop());
        self
    }

    /// Whether assistive technology should report the value as read-only.
    /// This changes semantics only; configure editing behavior on the editor.
    pub fn semantics_read_only(mut self, value: impl IntoValue<bool>) -> Self {
        self.common.semantics_mut().read_only = Some(value.into_prop());
        self
    }

    /// The type of popup this control opens.
    /// Pass `None` to leave it unstated.
    pub fn has_popup(mut self, value: impl IntoValue<Option<crate::view::PopupKind>>) -> Self {
        self.common.semantics_mut().has_popup = Some(value.into_prop());
        self
    }

    /// The meaning of the current item in a related set.
    /// Pass `None` to leave it unstated.
    pub fn current_item(mut self, value: impl IntoValue<Option<crate::view::CurrentItem>>) -> Self {
        self.common.semantics_mut().current_item = Some(value.into_prop());
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
    pub fn described_by(mut self, target: NodeRef) -> Self {
        self.common.semantics_mut().described_by.push(target);
        self
    }

    /// Name a node this one controls (`aria-controls`), such as the
    /// panel a button expands. Call it again to add more. A reference
    /// that is not bound to a mounted node is left out.
    pub fn controls(mut self, target: NodeRef) -> Self {
        self.common.semantics_mut().controls.push(target);
        self
    }

    /// Name a node that labels this one (`aria-labelledby`), for a
    /// visible heading or caption that should be the accessible name.
    /// Call it again to add more. A reference that is not bound to a
    /// mounted node is left out.
    pub fn labelled_by(mut self, target: NodeRef) -> Self {
        self.common.semantics_mut().labelled_by.push(target);
        self
    }

    /// Append component key behavior after existing customizations.
    pub fn on_key_default<F, R>(mut self, f: F) -> Self
    where
        F: Fn(KeyEvent) -> R + 'static,
        R: HandlerResult,
    {
        self.common
            .handlers
            .key
            .node_default(move |e| f(e).into_outcome());
        self
    }

    /// Append component pointer behavior after existing customizations.
    pub fn on_pointer_default<F, R>(mut self, f: F) -> Self
    where
        F: Fn(PointerEvent) -> R + 'static,
        R: HandlerResult,
    {
        self.common
            .handlers
            .pointer
            .node_default(move |e| f(e).into_outcome());
        self
    }

    /// Append component wheel behavior after existing customizations.
    pub fn on_wheel_default<F, R>(mut self, f: F) -> Self
    where
        F: Fn(WheelEvent) -> R + 'static,
        R: HandlerResult,
    {
        self.common
            .handlers
            .wheel
            .node_default(move |e| f(e).into_outcome());
        self
    }

    /// Offer an assistive-technology action. A node has one handler per
    /// action: the later registration replaces the earlier one.
    /// `Activate` is reported and ignored; use `on_press` for activation.
    #[track_caller]
    pub fn on_a11y<F, R>(mut self, action: A11yAction, f: F) -> Self
    where
        F: Fn() -> R + 'static,
        R: HandlerResult,
    {
        self.common.offer_a11y(action, f);
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
    pub fn on_a11y_event<F, R>(mut self, action: A11yAction, f: F) -> Self
    where
        F: Fn(A11yEvent) -> R + 'static,
        R: HandlerResult,
    {
        self.common.offer_a11y_event(action, f);
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
    pub fn ime_session(mut self, session: ImeSession) -> Self {
        self.common.ime = Some(session);
        self
    }

    /// Handle the input method's events for this node's
    /// [`ime_session`](Self::ime_session): a composition (`Preedit`), a
    /// `Commit`, a reconversion `Selection` and the `End` of the session.
    /// Like `on_key`, the most recently registered handler runs first and
    /// `Yes` stops the rest.
    pub fn on_ime<F, R>(mut self, f: F) -> Self
    where
        F: Fn(ImeInput) -> R + 'static,
        R: HandlerResult,
    {
        self.common.prepend_ime(f);
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
    pub fn semantics_text(mut self, text: impl IntoValue<SemanticsText>) -> Self {
        self.common.semantics_mut().text = Some(text.into_prop());
        self
    }

    /// Local draw layer over the children, after both `draw` and `Style::paint`.
    pub fn draw_over(mut self, f: impl Fn(&mut Draw, Size) + 'static) -> Self {
        self.common.draw_over = Some(Rc::new(f));
        self
    }
}

/// Deferred part transformations interleaved with already declared caller properties.
pub(crate) enum Modifier {
    Caller(Common),
    Part(Box<dyn FnOnce(Node) -> Node>),
}
impl Node {
    fn record_part<S: kivact_runtime::PropValue>(&mut self, part: Part<S>) {
        self.modifiers
            .push(Modifier::Caller(std::mem::take(&mut self.common)));
        self.modifiers.push(Modifier::Part(part.into_apply()));
    }
    pub(crate) fn resolve_modifiers(mut self) -> Self {
        let modifiers = std::mem::take(&mut self.modifiers);
        if modifiers.is_empty() {
            return self;
        }
        let final_caller = std::mem::take(&mut self.common);
        for modifier in modifiers {
            match modifier {
                Modifier::Caller(caller) => self.common.apply_caller(caller),
                Modifier::Part(apply) => {
                    self = apply(self).resolve_modifiers();
                }
            }
        }
        self.common.apply_caller(final_caller);
        self
    }
    /// Run after references are bound, under the mounted element's setup owner.
    pub fn on_attach<R: HandlerResult>(mut self, f: impl Fn(PartMount) -> R + 'static) -> Self {
        let reference = crate::view::node_ref();
        self.common.node_ref.push(reference);
        self.common
            .part_bindings
            .push(crate::view::part_data::PartBinding {
                node: reference,
                hooks: vec![Rc::new(move |mount| f(mount).into_outcome().map(|_| ()))],
            });
        self
    }
}

impl Node {
    /// Require modal input confinement when this node configures an overlay attachment.
    pub fn modal(mut self) -> Self {
        self.common.part_modal = true;
        self
    }
    /// Receive the unhandled input of this canvas's regions.
    pub fn region_events(mut self) -> Self {
        self.common.part_region_events = true;
        self
    }
    /// Direct pointer presses to this focus target.
    pub fn focus_on_press(mut self, destination: NodeRef) -> Self {
        self.common.focus_on_press = Some(destination);
        self
    }
}
