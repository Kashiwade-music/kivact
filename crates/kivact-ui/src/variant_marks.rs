//! Role: Generic variant marks any theme may honour: slanted and striped components.

use kivact_core::view::Variant;
use kivact_macros::Prop;

/// Draw a standard component slanted: `button("Go").variant(Slant)`.
/// A theme without a slanted look draws the plain component.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Prop)]
pub struct Slant;

impl Variant for Slant {}

/// Draw diagonal stripes on a standard component's fill:
/// `progress(value).variant(Striped)`. A theme without stripes draws the
/// plain component.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Prop)]
pub struct Striped;

impl Variant for Striped {}
