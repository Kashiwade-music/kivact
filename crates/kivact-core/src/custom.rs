//! Role: Primitive contracts support custom components and measurement.
//!
//! Component authors share [`Owner`] resources and [`NodeRefExt`] queries
//! with the built-in interaction layer. These contracts keep runtime
//! internals and the retained tree private.
//!
//! The official interaction layer and external components use one public
//! contract (implemented here, re-exported by the facade as
//! `kivact::custom`):
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
//!   `Node::accessible_name` shares the stored input closure, and
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
//! | input | `.on_pointer(..)`, `.on_key(..)`, `.on_wheel(..)`, `.a11y_action(..)`; `Draw::region` for hit areas |
//! | focus | `.focusable(true)`, the `focus` state style |
//! | accessibility | `.role(..)` and `.value_text(..)` ([`Node`]), `.name(..)`, [`Primitive::checked`], [`Primitive::expanded`], `.a11y_action(..)`; `Draw::region` for parts |
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
//!   shows. The engine knows nothing of `kivact-runtime` and takes a plain
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
//! - A panic in the function propagates through the layout and ends the
//!   App. Only a finite, non-negative size and a finite baseline are taken. Without a
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
//! The E2E test is `dev/tests/kivact-public-api/tests/e2e/primitive.rs`.
//!
//! ```no_run
//! # use kivact_core as kivact;
//! use kivact::state::{Signal};
//! use kivact::style::{ColorToken};
//! use kivact::view::{IntoView, NodeBuilder, Role, row};
//! use kivact::geometry::{Rect, Size};
//! use kivact::custom::{Constraints, primitive};
//!
//! /// A row of `n` dots, as wide as it has dots.
//! fn dots(n: Signal<u32>) -> impl IntoView {
//!     primitive()
//!         .measure(n, |n: &u32, _: Constraints| Size::new(*n as f64 * 12.0, 12.0))
//!         .draw(move |d, size| {
//!             for i in 0..n.get() {
//!                 let x = i as f64 * 12.0;
//!                 d.rounded_rect(Rect::new(x + 2.0, 2.0, 8.0, size.height - 4.0), 4.0, ColorToken::Accent);
//!             }
//!         })
//!         .with_node(|node| node.role(Role::Status))
//!         .name(move || format!("{} dots", n.get()))
//!         .test_id("dots")
//! }
//! ```
//!
//! [`NodeBuilder`]: crate::view::NodeBuilder
//! [`Draw`]: crate::draw::Draw

pub use crate::view::descriptor::view_is_empty;
pub use crate::view::ime_session::{ImeInput, ImeSession};
pub use kivact_types::cell::{Generation, Sent};
pub use kivact_types::{
    EditLineLayout, SemanticsText, SemanticsTextRun, SemanticsTextRuns, TextCaret, TextRunId,
};

mod measured_leaf;
mod measurement;
mod node;
mod text_shaping;
pub use crate::view::owner::Owner;
pub use crate::view::part::{Part, PartState, part};
pub use crate::view::part_data::PartMount;
/// The `!Send + !Sync` marker of Kivact's handles, for a component's own
/// Copy handle type (`PhantomData<UiThreadOnly>`).
pub use kivact_runtime::UiThreadOnly;
pub use kivact_runtime::{Owned, clock_now, require_effectful, require_setup};
pub use measured_leaf::{Primitive, primitive};
pub use measurement::{AvailableSpace, Constraints, Measurement};
pub use node::{NodeRefExt, NodeToken, focused_node};
pub use text_shaping::{shape_inline_paragraph, shape_paragraph};

mod text_editing;
pub use crate::view::editing_data::{
    Decoration, DecorationKind, TextChange, TextEditEvent, TextEditing, TextSession,
};
pub use kivact_editing::{
    EditCaret, EditContext, EditLine, EditOutcome, EditRequest, EditState, TextLayout,
};

mod inline_view;
pub use inline_view::inline_view;

#[doc(inline)]
pub use kivact_runtime::Reactive;

mod misuse;
pub use misuse::raise;
