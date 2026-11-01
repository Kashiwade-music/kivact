//! Role: Primitive elements and layout containers.
//!
//! Primitive elements and layout containers, all of the one element type
//! `Node`: `node`, `canvas`, `image` (with `ImageFit`), `text` and
//! `rich_text` (with `span`), `column`/`row`/`grid`/`stack`/`scroll`/`spacer`;
//! the text inputs; and `NodeBuilder`, the standard modifiers of `Node` and
//! of a component builder holding a root `Node`. Host node kinds are limited to Root, Box, Text,
//! Image and Overlay. The standard components (`button`,
//! `checkbox`, ...) are `kivact::ui`, and the components of a UI kit are in
//! the kit's crate; both are built from these through the public API. A
//! new primitive measured by its author is `kivact::primitive` (experimental).
//!
//! The host node kinds stay these five: native editing is a payload any of
//! them can carry, and a new primitive is a Box leaf that Kivact measures
//! (`BoxData::measure`), not a new kind. Only primitives live in `control`;
//! the few components with a look that make a first screen without a kit
//! are `kivact::ui`, and domain components are in the optional official
//! kits.

// Invariant: only primitives live here. Components built from them live in
// `kivact::ui` and in UI kits, which use the public API only.

mod container;
mod node;
mod text;
mod text_binding;

#[doc(inline)]
pub use crate::view::node::{Node, NodeBuilder};
#[doc(inline)]
pub use crate::view::span::{Span, span};
pub use container::{Scroll, column, grid, row, scroll, spacer, stack};
pub use node::{canvas, icon, image, node};
pub use text::{rich_text, text};

#[doc(inline)]
pub use kivact_style::asset::ImageFit;
