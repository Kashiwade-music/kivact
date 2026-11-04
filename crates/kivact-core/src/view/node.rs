//! Role: The element type and the modifier contract of every element builder.
//!
//! `Node` and `NodeBuilder` are defined here, below the element builders in
//! `view` (`overlay`, `virtual_list`) that hold a `Node` and implement
//! `NodeBuilder`; `kivact::view` re-exports them and holds the primitives
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

use crate::state::{IntoText, IntoValue};
use crate::view::HandlerResult;
use crate::view::View;
use crate::view::common::Common;
use crate::view::descriptor::Mcx;
use crate::view::input::{FileEvent, KeyEvent, PointerEvent, WheelEvent};
use crate::view::node_ref::NodeRef;
use crate::view::semantics_data::Modifier;
use crate::view::span::Span;
use kivact_runtime::Prop as InputProp;
use kivact_runtime::{Callback, ErrorKind, UiError};
use kivact_style::Style;

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
    /// Later caller patches visible to a part while its transformation runs.
    pub(crate) semantic_reads: Vec<crate::view::semantics_data::SemanticsPatch>,
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
            semantic_reads: Vec::new(),
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
/// use kivact::style::{style};
/// use kivact::view::{IntoView, Node, NodeBuilder, node};
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
    #[track_caller]
    fn test_id(mut self, id: impl Into<String>) -> Self {
        self.node_mut()
            .common
            .semantic(kivact_runtime::Prop::constant(id.into()), |s, value| {
                s.test_id = Some(value)
            });
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
    /// reads [`use_theme`](crate::style::use_theme) is
    /// recomputed when the theme visible at the node changes:
    /// `.style(move || { let t = &use_theme(); style().padding(t.space().md) })`.
    fn style(mut self, s: impl IntoValue<Style>) -> Self {
        self.node_mut().common.push_style(s.into_prop());
        self
    }

    /// Accessible name.
    #[track_caller]
    fn name(mut self, v: impl IntoText) -> Self {
        self.node_mut().common.set_name(v.into_text());
        self
    }

    /// Accessible description.
    #[track_caller]
    fn description(mut self, v: impl IntoText) -> Self {
        self.node_mut().common.set_description(v.into_text());
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
    #[track_caller]
    fn disabled(mut self, v: impl IntoValue<bool>) -> Self {
        self.node_mut().common.set_disabled(v.into_prop());
        self
    }

    /// Report an invalid value to assistive technology. A component that
    /// draws an invalid look reads it with
    /// [`accessible_invalid`](crate::view::Node::accessible_invalid).
    #[track_caller]
    fn invalid(mut self, value: impl IntoValue<bool>) -> Self {
        self.node_mut().common.set_invalid(value.into_prop());
        self
    }

    /// Set how this node lays out when it is the root of a removed subtree.
    /// Absolute, fixed and overlay nodes already occupy no slot in normal flow.
    fn exit_layout(mut self, layout: ExitLayout) -> Self {
        self.node_mut().common.exit_layout = Some(layout);
        self
    }

    /// Run after this node, when directly removed, is no longer drawn.
    /// The callback runs once even when the removal has no animation or an
    /// ancestor cuts the animation short. Create it outside the removed
    /// branch so its owner can still be alive when the exit finishes.
    #[track_caller]
    fn on_exit_complete(mut self, callback: impl Into<Callback<()>>) -> Self {
        let callback = callback.into();
        self.node_mut().common.exit_complete = Some(kivact_runtime::Handler::from_fn(move |()| {
            match callback.try_call(()) {
                Err(error)
                    if matches!(error.kind(), kivact_runtime::ErrorKind::Disposed { .. }) =>
                {
                    Ok(())
                }
                result => result,
            }
        }));
        self
    }

    /// Hide the node and its subtree.
    #[track_caller]
    fn hidden(mut self, v: impl IntoValue<bool>) -> Self {
        self.node_mut().common.set_hidden(v.into_prop());
        self
    }

    /// Whether the pointer hits the node and its subtree (default `true`).
    /// With `false` the pointer passes through them to what lies behind,
    /// as CSS `pointer-events: none`: no hover, press or wheel reaches
    /// them, which suits a floating card that follows the pointer.
    /// Keyboard focus and assistive technology are unaffected.
    fn hit_testable(mut self, on: impl IntoValue<bool>) -> Self {
        self.node_mut().common.hit_testable = Some(on.into_prop());
        self
    }

    /// Whether the node takes keyboard focus, overriding the builder's default.
    /// Reactive values support a roving tab stop without replacing nodes:
    /// only the selected tab or segment is a Tab stop, and the stop moves
    /// while the nodes stay.
    #[track_caller]
    fn focusable(mut self, on: impl IntoValue<bool>) -> Self {
        self.node_mut().common.set_focusable(on.into_prop());
        self
    }

    /// Keep an otherwise focusable control in keyboard focus order while it
    /// is disabled, so users can reach its label and any attached tooltip.
    /// Activation and editing remain disabled.
    fn focusable_when_disabled(mut self, on: impl IntoValue<bool>) -> Self {
        self.node_mut().common.focusable_when_disabled = Some(on.into_prop());
        self
    }

    /// Give this group one Tab stop and move between its eligible controls
    /// with unhandled arrows along `orientation`, Home and End. A text field
    /// keeps arrows that still move its caret or selection.
    #[track_caller]
    fn roving_focus(mut self, orientation: crate::view::Orientation) -> Self {
        use crate::view::node_ref::RovingMove;
        use crate::view::{Handled, Key, Modifiers, Orientation, ReadingDirection};
        let rtl = crate::view::use_direction().get() == ReadingDirection::Rtl;
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
    #[track_caller]
    fn on_press(mut self, f: impl Into<Callback<()>>) -> Self {
        let f = f.into();
        self.node_mut().common.set_press(move || f.call(()));
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
