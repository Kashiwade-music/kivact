//! Role: Primitive contracts support custom components and measurement.
//!
//! Component authors share [`Owner`] resources and [`NodeRefExt`] queries
//! with the built-in interaction layer. These contracts keep runtime
//! internals and the retained tree private.
//!
//! The official interaction layer and external components use one public
//! contract (implemented here, re-exported by the facade as
//! `kivact::primitive`):
//! - [`Owner`] holds only the identity of an App and an owner, and
//!   [`Owned<T>`] is a Copy handle to an `Rc<T>` released with its owner;
//!   neither retains the runtime. An owner stores objects, registers
//!   cleanups, provides and reads contexts and reports through its error
//!   boundary; it does not create signals or run setup (create what a
//!   component needs during its setup instead). `Owner::root` with
//!   `context` and `provide` builds App-wide services from handlers and
//!   tasks too. Equal roots tell that two owners belong to the same App, and
//!   an `Owned<()>` stored at setup tells, through [`Owned::try_get`],
//!   whether the component is still shown. [`require_setup`] and
//!   [`require_effectful`] assert where a component API may be called;
//!   diagnostics ([`raise`]) and the monotonic clock ([`clock_now`]) are on
//!   the same surface.
//! - [`NodeToken`] identifies a mount within its App, also as a focus
//!   snapshot after removal and for restore requests; neither the tree nor
//!   numeric ids are public. [`NodeRefExt`] answers a limited set of mount,
//!   focus and modal questions: `tracked_node(at)` keeps the author's call
//!   site for deferred reads, and `is_visible` follows the binding and the
//!   ancestors' visibility even while unmounted, and tracks notifications
//!   of an equal value too. Wrap queries made during an event in `untrack`.
//!   A restore request does not beat an explicit focus of the same turn.
//! - [`view_is_empty`] tells an explicitly empty view;
//!   `NodeExt::accessible_name` shares the stored input closure, and
//!   `has_description` sees an explicitly empty description too.
//! - The runtime, the reactive owners and `InputProp`s, the wire's `NodeId`
//!   and borrows of the tree are not public.
//!
//! **Experimental.** New primitives: elements at the level of the core's
//! own `node`, `text` and `image` that an author outside this crate
//! writes against the public API alone, without forking Kivact.
//!
//! A [`Primitive`] is a leaf whose size its author computes. Everything
//! else it needs is the machinery every element already has, on the same
//! node:
//!
//! | need | how |
//! |---|---|
//! | measure and layout | [`Primitive::measure`]: a synchronous function of a content value, called by the layout (like a text's shaping); it may return a [`Measurement`] with a first baseline for `Align::Baseline` |
//! | draw | `.draw(..)` / `.draw_over(..)` ([`NodeBuilder`]): the [`Draw`] vocabulary at the laid-out size, run again when the size or a signal it reads changes |
//! | input | `.on_pointer(..)`, `.on_key(..)`, `.on_wheel(..)`, `.on_a11y(..)`; `Draw::region` for hit areas |
//! | focus | `.focusable(true)`, the `focus` state style |
//! | accessibility | `.role(..)` and `.value_text(..)` ([`NodeExt`]), `.name(..)`, [`Primitive::checked`], [`Primitive::expanded`], `.on_a11y(..)`; `Draw::region` for parts |
//! | test identification | `.test_id(..)` |
//! | resource lifetime | the primitive's owner: its effects, handlers and measure function are dropped when it unmounts; `on_cleanup` for the author's own resources |
//!
//! [`TextEditing`] attaches native editing to a node through a Part, with
//! a key policy ([`TextEditing::behavior`]), line colors
//! ([`TextEditing::highlight`]) and range edits ([`TextEditEvent`]).
//! [`NodeRefExt::edit_session`] reads its selection without the text,
//! [`NodeRefExt::edit_state`] copies its text and selection;
//! [`NodeRefExt::text_layout`] reads the shaped lines asked for and their
//! carets. Native editors are leaves and do not accept custom drawing or
//! children.
//!
//! Not supported yet: children on measured primitives, editing canvas regions,
//! hit testing beyond the node's box and its `Draw::region`s, and embedding
//! native widgets; also a shaping interface that edits glyph arrays, custom
//! drawing or child nodes inside a native editor, primitives that lay out
//! children of their own, and reading the theme or the scale inside
//! `measure` (put them in the content). Plain text is `Constraints::
//! measure_text` and `Draw::text`; native editing is [`TextEditing`].
//!
//! How a primitive meets the engine:
//! - It makes no new node kind: the engine's Box leaf gets a
//!   `BoxData::measure`. The public types are Kivact's `Size`,
//!   [`Constraints`] and `AvailableSpace`; no engine, Arvane or taffy type
//!   shows. The engine knows nothing of `kivact-reactive` and takes a plain
//!   closure, `Fn(MeasureRequest, LeafTextMeasurer) -> LeafSize` (a `Size`
//!   and an optional `Baseline`).
//! - One closure is made per content, and the engine compares
//!   `LeafMeasure`s by `Rc` identity. A constant content goes with the
//!   create; a dynamic one is written to the Box payload by a render effect
//!   only when it changes (`PartialEq`). A new measure function counts as a
//!   content change: the engine measures it again for every request it
//!   recorded and skips taffy when all the sizes are the same, else marks
//!   the leaf dirty. Otherwise it measures again only when an edit clears
//!   the layout record, as for any leaf, never per frame. Registering a
//!   font measures again every leaf that may measure text.
//! - `Constraints::measure_text` answers what `Draw::measure_text` does
//!   (a style without a family takes the leaf's inherited one): the engine
//!   lends the measuring Arvane engine to [`Constraints`] for the call, so
//!   neither the `Reader` nor the tree is borrowed again, and `Constraints`
//!   stays `Copy`.
//! - The effects, handlers, measure function and content belong to the
//!   primitive's owner. The engine's `LeafMeasure` holds them weakly, so
//!   the engine dropping its closure while its tree is borrowed (a new
//!   content, an unmount) drops no user value. A content is dropped by the
//!   render effect writing the next one or by the owner's cleanup, both
//!   inside a turn. The layout record keeps only sizes.
//! - A panic in the function is caught and sizes the leaf 0 × 0; it is
//!   reported once per content, at the start of the next turn
//!   (`Runtime::report_later`), to the nearest `error_boundary`, whose
//!   `on_error` and fallback may change the tree like any handler. Only a
//!   finite, non-negative size and a finite baseline are taken. Without a
//!   measure the size comes from the style and `intrinsic_size`, and with
//!   both the measure wins.
//! - Drawing is the usual draw layers: a render effect that depends on the
//!   node's size signal (`LayoutChanged`) and on what it reads writes the
//!   scene, and runs only when one of them changes. Input, focus,
//!   accessibility and test ids are the Box props and handlers, with no
//!   other path.
//! - The engine reads `measure` straight from the Box leaf's payload;
//!   `Measure` is a content prop that does not restack, so the paint list is
//!   not rebuilt. Debug builds check that only a Box carries `Measure`, and
//!   that a node with it has no children.
//!
//! An example is `page_dots` in `kivact-external-component` (measured by
//! its page count, drawing a hit and accessibility region per page); the
//! E2E test is `dev/tests/kivact-public-api/tests/e2e/primitive.rs`.
//!
//! ```no_run
//! # use kivact_core as kivact;
//! use kivact::prelude::*;
//! use kivact::draw::{Rect, Size};
//! use kivact::primitive::{Constraints, NodeExt, primitive};
//!
//! /// A row of `n` dots, as wide as it has dots.
//! fn dots(n: Signal<u32>) -> impl IntoView {
//!     primitive()
//!         .measure(n, |n: &u32, _: Constraints| Size::new(*n as f32 * 12.0, 12.0))
//!         .draw(move |d, size| {
//!             for i in 0..n.get() {
//!                 let x = i as f32 * 12.0;
//!                 d.rounded_rect(Rect::new(x + 2.0, 2.0, 8.0, size.height - 4.0), 4.0, ColorToken::Accent);
//!             }
//!         })
//!         .role(Role::Status)
//!         .name(move || format!("{} dots", n.get()))
//!         .test_id("dots")
//! }
//! ```
//!
//! [`NodeBuilder`]: crate::control::NodeBuilder
//! [`Draw`]: crate::draw::Draw

use std::cell::RefCell;
use std::panic::Location;
use std::rc::Rc;

use kivact_engine::bridge::PropValue;
use kivact_reactive::EffectKind;
use kivact_reactive::Prop as InputProp;
use kivact_wire::{self as wire, LeafMeasure, NodeId};

use crate::control::node;
use crate::view::descriptor::Mcx;
use crate::view::descriptor::{mount, view_fn};
pub use crate::view::ime_session::{ImeEvent, ImeSession};
use crate::view::input::{KeyEvent, KeyHandlerResult, PointerEvent, WheelEvent};
use crate::view::node::{Node, NodeBuilder};
use crate::view::state::ViewState;
use crate::view::sync_hook::{Current, SyncHook};
use crate::view::{
    A11yAction, A11yEvent, HandlerResult, IntoText, LiveRegion, NodeRef, Orientation, Role,
    SortDirection,
};
use crate::view::{IntoValue, IntoView, View};
use kivact_style::Style;
pub use kivact_style::draw::TextParagraph;
use kivact_style::draw::{Draw, Size, TextMetrics, TextRun, TextStyle};
pub use kivact_wire::{SemanticsText, SemanticsTextRun, TextCaret, TextRunId};

mod node;
pub use crate::view::owner::{Owner, app_state};
pub use crate::view::part::{Part, PartState};
pub use crate::view::part_data::PartMount;
pub use kivact_reactive::{Owned, clock_now, raise, require_effectful, require_setup};
/// Whether the view is explicitly empty, including `()`.
#[track_caller]
pub fn view_is_empty(view: &crate::view::View) -> bool {
    view.is_empty()
}
/// The `!Send + !Sync` marker of Kivact's handles, for a component's own
/// Copy handle type (`PhantomData<UiThreadOnly>`).
pub use kivact_reactive::UiThreadOnly;
pub use node::{NodeRefExt, NodeToken, focused_node};

/// Shape text outside drawing or measurement, using this App's text engine.
///
/// Callable in setup, handlers, effects and value closures. The current theme's
/// family and line height fill unspecified style fields; node-specific inherited
/// overrides require the same explicit fields here. Equal effective text, style
/// and width share the shaping cache with drawing and measurement. No engine
/// borrow survives this call and no reactive engine state is allocated.
#[track_caller]
pub fn shape_paragraph(text: &str, style: &TextStyle, max_width: Option<f32>) -> TextParagraph {
    let rt = kivact_reactive::Owner::ambient("shape_paragraph", Location::caller()).runtime();
    let theme = kivact_style::current_theme();
    let inherited = wire::ComputedText {
        family: theme.text().family.clone(),
        line_height: Some(theme.text().line_height),
        ..Default::default()
    };
    let measurer = ViewState::tree(&rt)
        .reader()
        .map(|reader| reader.paragraph_measurer(inherited));
    let run = TextRun::from_style(
        text,
        wire::Point::default(),
        style,
        1.0,
        wire::Rgba::TRANSPARENT,
        None,
    );
    let shaped = measurer
        .and_then(|measurer| measurer.shape(run.as_wire(), max_width))
        .unwrap_or_default();
    TextParagraph::from_shaped(text, shaped, max_width)
}

/// Create a setup-owned semantic and input attachment for one target.
///
/// Read [`Part::state`](crate::primitive::Part::state) before consuming the attachment
/// with an element's `with` modifier or a canvas node's `region` declaration.
/// The closure receives the exact target's input, focus and availability state.
#[track_caller]
pub fn part<S: Clone + PartialEq + 'static>(
    state: impl Fn(crate::primitive::PartState) -> S + 'static,
) -> crate::primitive::Part<S> {
    crate::view::part::create(state)
}

/// Space available to a primitive along one axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AvailableSpace {
    /// A definite number of logical pixels.
    Definite(f32),
    /// As little as the content allows (the primitive's minimum size).
    MinContent,
    /// As much as the content wants (the primitive's preferred size).
    MaxContent,
}

/// What a primitive is measured under, in logical pixels. A dimension the
/// layout already fixed (`known_*`, from the style or a stretch) wins
/// over the measured one. It lends the layout's text measurement
/// ([`Constraints::measure_text`]) for the call.
#[derive(Clone, Copy)]
pub struct Constraints<'a> {
    /// The width the layout already fixed.
    pub known_width: Option<f32>,
    /// The height the layout already fixed.
    pub known_height: Option<f32>,
    /// The width available.
    pub available_width: AvailableSpace,
    /// The height available.
    pub available_height: AvailableSpace,
    text: wire::LeafTextMeasurer<'a>,
}

impl Constraints<'_> {
    /// The widest the primitive can be: the known width, else a definite
    /// available width (`None`: unbounded, `Some(0.0)`: as narrow as it
    /// can).
    pub fn max_width(&self) -> Option<f32> {
        match (self.known_width, self.available_width) {
            (Some(w), _) | (None, AvailableSpace::Definite(w)) => Some(w),
            (None, AvailableSpace::MinContent) => Some(0.0),
            (None, AvailableSpace::MaxContent) => None,
        }
    }

    /// The extent `Draw::text` draws `text` with in `style` in this
    /// primitive (one line; a style without a family takes the
    /// primitive's inherited one): size a chip by its label. It shapes
    /// with the layout's own text engine, so it agrees with drawing. A
    /// new font registered later measures the primitive again.
    ///
    /// ```no_run
    /// # use kivact_core as kivact;
    /// use kivact::prelude::*;
    /// use kivact::draw::{Size};
    /// use kivact::draw::TextStyle;
    /// use kivact::primitive::{Constraints, Measurement, primitive};
    ///
    /// fn chip(label: Signal<String>) -> impl IntoView {
    ///     primitive().measure(label, |label: &String, c: Constraints| {
    ///         let m = c.measure_text(label, &TextStyle::default());
    ///         // The label sits 4 px below the top.
    ///         Measurement::new(Size::new(m.width + 16.0, m.height + 8.0)).baseline(m.baseline + 4.0)
    ///     })
    /// }
    /// # let _ = chip;
    /// ```
    pub fn measure_text(&self, text: &str, style: &TextStyle) -> TextMetrics {
        let origin = wire::Point { x: 0.0, y: 0.0 };
        let run = TextRun::from_style(text, origin, style, 1.0, wire::Rgba::TRANSPARENT, None);
        self.text.measure(run.as_wire())
    }

    /// `text` shaped in `style` and wrapped at `max_width` (`None`: at
    /// hard line breaks only) as a [`TextParagraph`]: its size, lines and
    /// carets for a caret, a hit test or a selection, in the primitive's
    /// own pixels. It is the paragraph [`Draw::layout_paragraph`] makes of
    /// the same inputs in this primitive's draw closure, shaped once by the
    /// layout's own text engine (a style without a family takes the
    /// primitive's inherited one). A new font registered later measures the
    /// primitive again.
    ///
    /// ```no_run
    /// # use kivact_core as kivact;
    /// use kivact::prelude::*;
    /// use kivact::draw::{Size, TextStyle};
    /// use kivact::primitive::{Constraints, Measurement, primitive};
    ///
    /// fn line(text: Signal<String>) -> impl IntoView {
    ///     primitive().measure(text, |text: &String, c: Constraints| {
    ///         let paragraph = c.measure_paragraph(text, &TextStyle::default(), c.max_width());
    ///         Measurement::new(paragraph.size()).baseline(paragraph.baseline())
    ///     })
    /// }
    /// # let _ = line;
    /// ```
    pub fn measure_paragraph(
        &self,
        text: &str,
        style: &TextStyle,
        max_width: Option<f32>,
    ) -> TextParagraph {
        let origin = wire::Point { x: 0.0, y: 0.0 };
        let run = TextRun::from_style(text, origin, style, 1.0, wire::Rgba::TRANSPARENT, None);
        let shaped = self
            .text
            .shape(run.as_wire(), max_width)
            .unwrap_or_default();
        TextParagraph::from_shaped(text, shaped, max_width)
    }
}

impl PartialEq for Constraints<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.known_width == other.known_width
            && self.known_height == other.known_height
            && self.available_width == other.available_width
            && self.available_height == other.available_height
    }
}

impl std::fmt::Debug for Constraints<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Constraints")
            .field("known_width", &self.known_width)
            .field("known_height", &self.known_height)
            .field("available_width", &self.available_width)
            .field("available_height", &self.available_height)
            .finish_non_exhaustive()
    }
}

fn space(s: wire::AvailableSpace) -> AvailableSpace {
    match s {
        wire::AvailableSpace::Definite(v) => AvailableSpace::Definite(v),
        wire::AvailableSpace::MinContent => AvailableSpace::MinContent,
        wire::AvailableSpace::MaxContent => AvailableSpace::MaxContent,
    }
}

fn constraints(r: wire::MeasureRequest, text: wire::LeafTextMeasurer<'_>) -> Constraints<'_> {
    Constraints {
        known_width: r.known_width,
        known_height: r.known_height,
        available_width: space(r.available_width),
        available_height: space(r.available_height),
        text,
    }
}

/// **Experimental.** What [`Primitive::measure`]'s function returns: the
/// primitive's size and, optionally, where its first baseline lies. A
/// plain [`Size`] converts to a measurement without a baseline.
///
/// A flex container that aligns the primitive by baseline
/// (`align(Align::Baseline)`) lines this baseline up with its siblings'
/// (a text's, a text field's). Without one, the primitive's baseline is
/// the bottom of its box.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Measurement {
    /// The primitive's size.
    pub size: Size,
    /// The first baseline, in logical pixels below the top of the
    /// primitive's content box (inside its padding and border); `None`:
    /// the bottom of the box.
    pub baseline: Option<f32>,
}

impl Measurement {
    /// A measurement of `size` with no baseline.
    pub fn new(size: Size) -> Self {
        Measurement {
            size,
            baseline: None,
        }
    }

    /// This measurement with its first baseline `y` below the top of the
    /// content box (for example a label's [`TextMetrics::baseline`] plus
    /// the padding above it).
    #[must_use]
    pub fn baseline(mut self, y: f32) -> Self {
        self.baseline = Some(y);
        self
    }
}

impl From<Size> for Measurement {
    fn from(size: Size) -> Self {
        Measurement::new(size)
    }
}

/// A size the layout can use: finite and not negative; a baseline it can
/// use: finite.
fn sanitize(m: Measurement) -> wire::LeafSize {
    let clean = |v: f32| if v.is_finite() { v.max(0.0) } else { 0.0 };
    wire::LeafSize {
        size: wire::Size {
            width: clean(m.size.width),
            height: clean(m.size.height),
        },
        baseline: m
            .baseline
            .filter(|y| y.is_finite())
            .map(wire::Baseline::FromTop),
    }
}

/// Modifiers for component authors: what the component is (`role`,
/// `checked`, `expanded`, `selected`, `value_text`), attaching a part, extra
/// semantics, handler layering, and the overlay draw layer.
///
/// Implemented for every [`NodeBuilder`] ([`Node`] and each component holding
/// a root `Node`); bring it into scope with `use kivact::primitive::NodeExt;`.
/// A component sets its own role and states on its root with these, so an
/// app that imports only the prelude cannot set a `checked` that contradicts
/// the binding the component edits: `checkbox("x", on).checked(false)` does
/// not compile. [`Node`] has the same methods inherent, so `node().role(..)`
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
            .map(kivact_reactive::Prop::into_fn)
    }

    /// Read the preserved accessible-name input without allocating a value.
    #[track_caller]
    fn accessible_name(&mut self) -> Option<Rc<dyn Fn() -> String>> {
        self.node_mut()
            .name_source()
            .map(kivact_reactive::Prop::into_fn)
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
            .map(kivact_reactive::Prop::into_fn)
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
            .map(kivact_reactive::Prop::into_fn)
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
    /// click clearing it, say). `kivact::test`'s
    /// `Element::is_indeterminate` reads what the host published.
    fn indeterminate(mut self, value: impl IntoValue<bool>) -> Self {
        self.node_mut().common.semantics_mut().indeterminate = Some(value.into_prop());
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

    /// Offer an action while preserving the requested drawn region's
    /// identity and optional value. The root node reports `region: None`;
    /// a region reports the `RegionId` its `Draw::region` declared.
    /// `Activate` is supported here for canvas regions; use `on_press` for
    /// ordinary activation. The later handler for an action wins.
    /// A `SetValue` hands the requested text over as it came: parsing it,
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
    /// native editor ([`TextEditing`]). The IME's composition, commits and
    /// reconversion ranges arrive at [`on_ime`](Self::on_ime). A later
    /// session replaces an earlier one.
    ///
    /// The session is published after the node's draw layers run in the
    /// same turn, so a value closure that reads what the drawing just made
    /// (the caret of the [`TextParagraph`] `Draw::layout_paragraph` laid
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
        F: Fn(ImeEvent) -> R + 'static,
        R: KeyHandlerResult,
    {
        self.node_mut().common.prepend_ime(f);
        self
    }

    /// Publish this node's own text for assistive technology as text runs
    /// with a selection and a caret, in the node's own coordinates, so a
    /// screen reader can read, navigate and locate the text of a node that
    /// draws it (see [`TextParagraph::semantics_text`]). Constant or
    /// reactive. A native editor ([`TextEditing`]) publishes its own and
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

type BindMeasure = Box<dyn FnOnce(&Mcx<'_>, NodeId, &'static Location<'static>)>;

/// **Experimental.** A new primitive element: a leaf box measured by its
/// author. Built by [`primitive`]; the standard modifiers (`draw`,
/// `on_pointer`, `role`, `test_id`, `style`, ...) come from
/// [`NodeBuilder`] and apply to its node. See the
/// [module](self) for what it supports.
pub struct Primitive {
    root: Node,
    measure: Option<BindMeasure>,
    at: &'static Location<'static>,
}

/// **Experimental.** Start a new primitive, located at the call (snapshots
/// and diagnostics name it by its location). Without
/// [`measure`](Primitive::measure) it is sized by its style and
/// `intrinsic_size` like an empty `node()`.
#[track_caller]
pub fn primitive() -> Primitive {
    Primitive {
        root: node(),
        measure: None,
        at: Location::caller(),
    }
}

impl Primitive {
    /// Size the primitive by `f(content, constraints)` (a [`Size`], or a
    /// [`Measurement`] that adds a first baseline), which the layout
    /// calls synchronously, as it shapes a text. `content` (a constant, a
    /// `Signal`, a `Value` or a closure) holds everything the size depends
    /// on: the layout measures again only when a new content differs
    /// (`PartialEq`) from the last, and then only if the new sizes differ
    /// under the requests it measured before; otherwise it keeps every
    /// size it measured (it never measures every frame). The size is
    /// given in logical pixels; a known dimension of the constraints wins
    /// over it.
    ///
    /// `f` runs inside the layout: it must be a plain function of its
    /// arguments (nothing that waits). It runs pure and untracked: a signal
    /// write in it is rejected (`WriteInPureContext`, reported like a
    /// panic) and a signal read subscribes nothing. A panic
    /// in it sizes the primitive 0 × 0 and is reported once per content
    /// to the nearest `error_boundary` (else the App's default sink), on
    /// the turn after the layout, so the boundary's handlers may do
    /// anything a handler does. A later call replaces an earlier one.
    pub fn measure<T, F, R>(mut self, content: impl IntoValue<T>, f: F) -> Self
    where
        T: Clone + PartialEq + 'static,
        R: Into<Measurement>,
        F: Fn(&T, Constraints<'_>) -> R + 'static,
    {
        let src = content.into_prop();
        let f = Rc::new(f);
        self.measure = Some(Box::new(move |cx, id, at| {
            bind_measure(cx, id, src, f, at);
        }));
        self
    }

    /// Toggle state for a check box or switch primitive (semantics and
    /// the `checked` reading of assistive technology).
    pub fn checked(mut self, v: impl IntoValue<bool>) -> Self {
        self.root = self.root.checked(v);
        self
    }

    /// Expanded state for a disclosure, combo box or menu primitive.
    pub fn expanded(mut self, v: impl IntoValue<bool>) -> Self {
        self.root = self.root.expanded(v);
        self
    }
}

/// Write the leaf's measure function for every new content of `src`: a
/// constant once (in the node's creation), anything else from one render
/// effect that skips a content equal to the last.
///
/// The engine calls the function inside its layout, with its tree
/// borrowed: a panic there is caught and reported on the next turn
/// ([`Runtime::report_later`]), where the error's handlers may read and
/// write the tree. The current content lives in the owner (until a new
/// content or its cleanup) and is always the one the engine holds when it
/// lays out: a new content is written in the turn that replaced it, and
/// the engine lays out only between turns.
///
/// [`Runtime::report_later`]: kivact_reactive::Runtime::report_later
fn bind_measure<T, F, R>(
    cx: &Mcx<'_>,
    id: NodeId,
    src: InputProp<T>,
    f: Rc<F>,
    at: &'static Location<'static>,
) where
    T: Clone + PartialEq + 'static,
    R: Into<Measurement>,
    F: Fn(&T, Constraints<'_>) -> R + 'static,
{
    let owner = cx.owner;
    let current: Current<T, Rc<F>> = Rc::new(RefCell::new(None));
    let held = current.clone();
    owner.on_cleanup(move || drop(held.borrow_mut().take()));
    let weak_rt = cx.weak();
    let now = current.clone();
    let leaf = move |content: T| {
        let (hook, hooked) = SyncHook::new(weak_rt.clone(), owner, at, content, f.clone());
        *current.borrow_mut() = Some(hooked);
        LeafMeasure::new(move |request, text| {
            hook.call(|content, f| f(content, constraints(request, text)))
                .map_or_else(wire::LeafSize::default, |measured| {
                    sanitize(measured.into())
                })
        })
    };
    if let Some(v) = src.fixed() {
        let measure = leaf(v);
        cx.update(id, PropValue::Measure(measure));
        return;
    }
    let weak = cx.weak();
    cx.effect(EffectKind::Render, at, move || {
        let v = src.get();
        // Compared with the content the engine measures (no second copy).
        let same = now.borrow().as_ref().is_some_and(|m| m.content == v);
        if same {
            return;
        }
        // The last content is dropped here, before the view tree is
        // borrowed: its `Drop` may read the tree.
        let prop = leaf(v);
        if let Some(rt) = weak.upgrade() {
            ViewState::tree_mut(&rt).update(id, PropValue::Measure(prop), rt.running_effect());
        }
    });
}

impl NodeBuilder for Primitive {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Primitive {
    fn into_view(self) -> View {
        view_fn(move |cx| {
            let Primitive { root, measure, at } = self;
            let id = mount(root.into_view(), cx);
            if let Some(bind) = measure {
                bind(cx, id, at);
            }
            id
        })
    }
}

mod text_editing;
pub use crate::view::editing_data::{
    TextChange, TextEditEvent, TextEditing, TextHighlight, TextSession,
};
pub use kivact_editing::{
    EditCaret, EditContext, EditLine, EditOutcome, EditRequest, EditState, TextLayout,
};
