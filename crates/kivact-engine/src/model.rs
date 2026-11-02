//! Role: The engine's one model: the node tree, its state groups and the event queue.
//!
//! One Model owns the retained node tree, the ordered event queue, the
//! pending Changes and one state group per concern. Each group's module
//! declares its state with private fields together with the methods that
//! change it; the semantic algorithms extend this same Model:
//!
//! - `crate::tree`: the node store, links and passive typed properties.
//!   Model owns derived layout/measurement/paint columns, editor payloads
//!   and resources; `interact` processes them without changing tree ownership.
//! - `changes`: what writes, input and layout outdated, and the typed
//!   [`crate::mutation::Mutation`] flags; `sync` alone dispatches it into layout,
//!   paint and accessibility work. `apply` applies the structural writes.
//! - `focus`: the focus target, Tab order and the reveals after layout;
//!   `modal` and `dismiss` confine and close overlays.
//! - `pointer`: the pointer state machine and the shell's pointer input;
//!   `keys` routes keys, `edit` keeps the text inputs' editing state and
//!   the IME's view of it.
//! - `states`, `state_motion`: state styles and their transitions;
//!   `transform`: element transforms; `scrollbar`: scrollbar geometry.
//! - `frame`: clocks, frame requests, smooth scrolls and drawability.
//! - `semantics`: the semantics policy; `events`: the event queue.
//!
//! Paint and accessibility keep their groups in their own modules
//! (`paint::state`, `a11y::state`).
//! Events always pass through the single ordered event queue.
//!
//! The model is the meaning of the tree, which knows no window, drawing or
//! shaping: focus, Tab order, modals and outside dismissal, the pointer,
//! the native editing state and the interpretation of keys, state styles,
//! the cursor and the semantics policy are decided here. `paint`, `a11y`
//! and `host` map that state onto Arvane and decide none of it. Besides
//! the groups, the model holds the window size (`viewport`, which the
//! layout fills) and, apart from it, the window title and minimum size to
//! set after the turn (`window_request`) and the imports and drops of
//! shared textures to carry out after it (`external_writes`, feature
//! `gpu-interop`).
//!
//! A group's state lives in a leaf `state` module under the module that
//! changes it: `focus::state::Focus` (the focus with its indicator and pending
//! reveal, the reveals after layout), `pointer::state::Pointer` (the pointer
//! state machine's tracks, each with its gesture, the file drop target and
//! the shown cursor), `edit::state::Edits` (the text inputs with an editing state and
//! the text last given to the IME), `state_motion::state::StateMotion` (the
//! moving state style transitions), `presence::state::Presences` (nodes
//! awaiting their entrance, leaving subtrees), `frame::state::Frame`
//! (clocks, frame requests, smooth scrolls, the surface) and
//! `check::lint::state::Lint` (debug builds) and `modal::state::Admission`
//! (the pointer drags admitted before a modal opened).
//!
//! Invariants:
//! - A group's fields are `pub(super)`: only its own module writes them,
//!   and everything else goes through its methods.
//! - A leaf `state` module never refers to the modules that process it
//!   (those with `impl Model`), so the Model holding every group makes no
//!   cycle of module references (xtask checks it).
//! - The Model is never cloned; work borrows the field groups it needs one
//!   by one.

mod aggregate;
pub(crate) mod apply;
pub(crate) mod changes;
pub(crate) mod dismiss;
pub(crate) mod edit;
pub(crate) mod events;
pub(crate) mod focus;
pub(crate) mod frame;
mod geometry;
pub(crate) mod interact;
pub(crate) mod keys;
mod lifecycle;
pub(crate) mod modal;
pub(crate) mod pointer;
pub(crate) mod presence;
pub(crate) mod scrollbar;
pub(crate) mod semantics;
pub(crate) mod state_motion;
pub(crate) mod states;
pub(crate) mod text;
pub(crate) mod transform;

#[cfg(feature = "gpu-interop")]
pub(crate) use aggregate::ExternalWrite;
pub(crate) use aggregate::Model;
